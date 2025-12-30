#![windows_subsystem = "windows"]

mod window;
mod system;
mod config;

use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;
use wry::{
    application::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
        window::{WindowBuilder, WindowId, Icon},
        dpi::{LogicalSize, LogicalPosition},
    },
    webview::{WebViewBuilder, WebContext, WebView},
    http::{Response, header},
};
use image::imageops::FilterType;
use notify::{Watcher, RecursiveMode, EventKind};
use std::time::{Duration, Instant};
use native_dialog::{MessageDialog, MessageType};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

// --- GLOBAL BROWSER LOCK FOR DEDUPLICATION ---
// Prevents multiple threads from simultaneously opening browser windows for the same URL.
// Stores: (last_opened_url_base, timestamp_of_open)
// Used to deduplicate redirect chains and concurrent handler fires
lazy_static::lazy_static! {
    static ref BROWSER_LOCK: Mutex<(String, Instant)> = Mutex::new((String::new(), Instant::now()));
}

struct AppState {
    webviews: HashMap<WindowId, WebView>,
    persistence: HashMap<WindowId, PersistenceConfig>,
    system: Arc<Mutex<system::SystemState>>,
    main_proxy: EventLoopProxy<FrontierEvent>,
    debounce: HashMap<PathBuf, Instant>,
}

struct PersistenceConfig {
    should_save: bool,
    save_file: PathBuf,
}

enum FrontierEvent {
    RunCommand(WindowId, String),
    BackendReply(WindowId, String),
    OpenWindow(String), 
    FileChanged(PathBuf),
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum UrlCategory { Frontier, Internal, Browser, Blocked }

// --- MAIN ---

fn main() {
    if let Err(e) = run_application() {
        let _ = MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Frontier Runtime Error")
            .set_text(&format!("{}", e))
            .show_alert();
    }
}

fn run_application() -> Result<(), Box<dyn std::error::Error>> {
    let is_dev = std::env::var("FRONTIER_DEV").is_ok();

    if is_dev {
        #[cfg(target_os = "windows")]
        unsafe {
            use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }

    let (base_dir, data_dir, dev_cache) = setup_paths(is_dev)?;
    let (commands, _modules_map) = scan_environment(&base_dir, &dev_cache, is_dev);
    let security_global = config::load_security_config(&base_dir.join("frontier.toml"));

    let system = Arc::new(Mutex::new(system::SystemState {
        commands,
        #[cfg(debug_assertions)]
        modules_map: _modules_map,
        base_dir: base_dir.clone(),
        data_dir: data_dir.clone(),
        #[cfg(debug_assertions)]
        dev_cache,
        allowed_internal: security_global.allowed_internal,
        allowed_browser: security_global.allowed_browser,
        is_dev,
        window_icon: load_application_icon(&base_dir),
    }));

    let event_loop = EventLoop::<FrontierEvent>::with_user_event();
    let main_proxy = event_loop.create_proxy();
    let mut web_context = WebContext::new(Some(data_dir));

    let mut app_state = AppState {
        webviews: HashMap::new(),
        persistence: HashMap::new(),
        system: system.clone(),
        main_proxy: main_proxy.clone(),
        debounce: HashMap::new(),
    };

    let mut _watcher = None;
    if is_dev {
        let watch_proxy = main_proxy.clone();
        let mut w = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    for path in event.paths { 
                        let _ = watch_proxy.send_event(FrontierEvent::FileChanged(path)); 
                    }
                }
            }
        })?;
        let _ = w.watch(&base_dir.join("app"), RecursiveMode::Recursive);
        _watcher = Some(w);
    }

    create_new_window(&event_loop, &mut app_state, &mut web_context, "index.html", main_proxy.clone())?;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(FrontierEvent::FileChanged(path)) => {
                if app_state.debounce.get(&path).map_or(false, |t| t.elapsed() < Duration::from_millis(500)) { return; }
                app_state.debounce.insert(path.clone(), Instant::now());
                for webview in app_state.webviews.values() { let _ = webview.evaluate_script("location.reload();"); }
            }
            Event::UserEvent(FrontierEvent::RunCommand(wid, cmd_str)) => {
                let sys = app_state.system.clone();
                let proxy = app_state.main_proxy.clone();
                thread::spawn(move || {
                    let mut parts = cmd_str.splitn(2, '|');
                    let trigger = parts.next().unwrap_or("");
                    let args = parts.next().unwrap_or("");
                    let res = system::execute_backend(&sys.lock().unwrap(), trigger, args);
                    let _ = proxy.send_event(FrontierEvent::BackendReply(wid, res));
                });
            }
            Event::UserEvent(FrontierEvent::BackendReply(wid, msg)) => {
                if let Some(webview) = app_state.webviews.get(&wid) {
                    let safe = msg.replace('\\', "\\\\").replace('`', "\\`").replace('\'', "\\'");
                    let js = format!("if(window.Frontier) window.Frontier.dispatch('log', `{}`)", safe);
                    let _ = webview.evaluate_script(&js);
                }
            }
            Event::UserEvent(FrontierEvent::OpenWindow(req)) => {
                let proxy = main_proxy.clone(); 
                let _ = create_new_window(event_loop, &mut app_state, &mut web_context, &req, proxy);
            }
            Event::WindowEvent { event, window_id, .. } => match event {
                WindowEvent::CloseRequested => {
                    save_window_state(&window_id, &app_state);
                    app_state.webviews.remove(&window_id);
                    app_state.persistence.remove(&window_id);
                    if app_state.webviews.is_empty() { *control_flow = ControlFlow::Exit; }
                }
                _ => {}
            },
            _ => {}
        }
    });
}

fn create_new_window(
    event_loop: &EventLoopWindowTarget<FrontierEvent>,
    app_state: &mut AppState,
    context: &mut WebContext,
    request: &str,
    proxy: EventLoopProxy<FrontierEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sys = app_state.system.lock().unwrap();
    let wl_internal_global = sys.allowed_internal.clone();
    let wl_browser_global = sys.allowed_browser.clone();
    let sys_is_dev = sys.is_dev;
    let sys_base = sys.base_dir.clone();
    let sys_icon = sys.window_icon.clone();

    let (target_url, config) = if request.starts_with("spawn://") {
        let raw = request.replace("spawn://", "");
        let mut parts = raw.splitn(2, '?');
        let url = parts.next().unwrap_or("").to_string();
        let config_raw = parts.next().unwrap_or("");
        let manual_cfg = window::create_manual_config(&url, config_raw);
        if sys_is_dev { eprintln!("📦 [SPAWN] {}", url); }
        (url, manual_cfg)
    } else {
        if sys_is_dev { eprintln!("📄 [WINDOW] {}", request); }
        let html = if sys_is_dev {
            fs::read_to_string(sys_base.join("app/frontend").join(request))?
        } else {
            Assets::get(&format!("frontend/{}", request))
                .map(|f| String::from_utf8_lossy(f.data.as_ref()).to_string())
                .ok_or("404")?
        };
        // Use frontier://app/filename.html format (app is a fake host)
        let url = format!("frontier://app/{}", request);
        (url, window::parse_html_config(&html, request))
    };

    let (mut combined_internal, mut combined_browser) = if config.ignore_global_security {
        (Vec::new(), Vec::new())
    } else {
        (wl_internal_global, wl_browser_global)
    };
    combined_internal.extend(config.allowed_internal.clone());
    combined_browser.extend(config.allowed_browser.clone());

    let save_file = sys.data_dir.join(format!("state_{}.json", config.id));
    let mut win_w = config.width;
    let mut win_h = config.height;
    let mut win_is_max = config.maximized;
    let mut win_x = None;
    let mut win_y = None;

    if config.persistent {
        if let Ok(json) = fs::read_to_string(&save_file) {
            if let Ok(saved) = serde_json::from_str::<window::WindowState>(&json) {
                win_w = saved.width; win_h = saved.height;
                win_is_max = saved.maximized;
                win_x = Some(saved.x); win_y = Some(saved.y);
            }
        }
    }

    let mut current_icon = sys_icon;
    if let Some(ipath) = &config.icon_path {
        let full_ipath = if sys_is_dev { sys_base.join("app/frontend").join(ipath) } else { sys_base.join("frontend").join(ipath) };
        if let Some(loaded) = load_icon_from_disk(&full_ipath) { current_icon = Some(loaded); }
    }

    let mut builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(LogicalSize::new(win_w, win_h))
        .with_resizable(config.resizable)
        .with_minimizable(config.minimizable)
        .with_maximizable(config.maximizable)
        .with_maximized(win_is_max)
        .with_window_icon(current_icon);

    // Apply minimum window size constraints if specified
    if let (Some(w), Some(h)) = (config.min_width, config.min_height) {
        builder = builder.with_min_inner_size(LogicalSize::new(w, h));
    }
    
    // Apply maximum window size constraints if specified
    if let (Some(w), Some(h)) = (config.max_width, config.max_height) {
        builder = builder.with_max_inner_size(LogicalSize::new(w, h));
    }

    if !win_is_max {
        if let (Some(x), Some(y)) = (win_x, win_y) {
            builder = builder.with_position(LogicalPosition::new(x, y));
        } else if let (Some(fx), Some(fy)) = (config.x.clone(), config.y.clone()) {
            if let Some(mon) = event_loop.primary_monitor() {
                let s = mon.size().to_logical::<f64>(mon.scale_factor());
                let px = window::evaluate_math_expression(&fx, s.width, s.height, win_w, win_h);
                let py = window::evaluate_math_expression(&fy, s.width, s.height, win_w, win_h);
                builder = builder.with_position(LogicalPosition::new(px, py));
            }
        }
    }

    let window = builder.build(event_loop)?;
    let wid = window.id();
    
    // --- ROUTING LOGIC WITH DEDUPLICATION ---
    // This system prevents duplicate window opens by routing external URLs through a single handler
    // and using atomic locks to prevent race conditions between navigation_handler and new_window_req_handler
    let w_int_nav = combined_internal.clone();
    let w_bro_nav = combined_browser.clone();
    let w_int_req = combined_internal.clone();
    let w_bro_req = combined_browser.clone();
    let initial_url = target_url.clone();
    let nav_proxy = proxy.clone();
    let ipc_proxy = proxy.clone();

    let webview = WebViewBuilder::new(window)?
        .with_web_context(context)
        .with_navigation_handler(move |url| {
            // Rule 1: Always allow initial URL load to prevent blocking the first page
            if url == initial_url { return true; }

            let cat = get_url_category(&url, &w_int_nav, &w_bro_nav);
            match cat {
                // Frontier and internally-whitelisted URLs load within the window
                UrlCategory::Frontier | UrlCategory::Internal => true,
                // External browser URLs are routed to the system browser with deduplication
                UrlCategory::Browser => {
                    route_to_browser(&url, sys_is_dev);
                    false // Block window load to prevent internal opening
                },
                // Security-blocked URLs are rejected
                UrlCategory::Blocked => {
                    if sys_is_dev { eprintln!("🚫 [SECURITY] Blocked access to: {}", url); }
                    false
                }
            }
        })
        .with_new_window_req_handler(move |url| {
            // Handles new window requests (e.g., target="_blank" links, window.open() calls)
            // Routes based on URL category without duplicating browser opens
            let cat = get_url_category(&url, &w_int_req, &w_bro_req);
            match cat {
                // Frontier protocol URLs spawn a new Frontier window
                UrlCategory::Frontier => {
                    let path = url.replace("https://frontier.", "").replace("frontier://", "");
                    let _ = nav_proxy.send_event(FrontierEvent::OpenWindow(path));
                    false
                },
                // Internal URLs open as browser popups within the Edge WebView
                UrlCategory::Internal => true,
                // Browser URLs are NOT opened here - the navigation_handler already handles them
                // This prevents duplicate opens when redirect chains occur (e.g., GitHub's locale redirect)
                UrlCategory::Browser => false,
                // Security-blocked URLs are rejected
                UrlCategory::Blocked => false
            }
        })
        .with_custom_protocol("frontier".into(), move |req| {
            // frontier://app/filename.html -> extract /filename.html
            let path = req.uri().path();
            let clean_path = percent_encoding::percent_decode_str(path).decode_utf8_lossy().to_string();
            let mut resource = clean_path.trim_start_matches('/').to_string();
            if resource.is_empty() { resource = "index.html".to_string(); }
            
            // Ignore favicon requests (browsers automatically request this)
            if resource == "favicon.ico" {
                return Response::builder().status(404).body(Cow::Owned(b"404".to_vec())).map_err(|_| wry::Error::InitScriptError);
            }
            
            let fp = if sys_is_dev { sys_base.join("app/frontend").join(&resource) } else { sys_base.join("frontend").join(&resource) };
            let mime = mime_guess::from_path(&fp).first_or_octet_stream().to_string();
            match fs::read(&fp) {
                Ok(b) => {
                    if sys_is_dev { eprintln!("📦 [ASSET] {} ({})", resource, mime); }
                    Response::builder().header(header::CONTENT_TYPE, mime).header("Access-Control-Allow-Origin", "*").body(Cow::Owned(b)).map_err(|_| wry::Error::InitScriptError)
                },
                Err(_) => {
                    if sys_is_dev { eprintln!("❌ [ASSET] Not found: {}", resource); }
                    Response::builder().status(404).body(Cow::Owned(b"404".to_vec())).map_err(|_| wry::Error::InitScriptError)
                }
            }
        })
        .with_url(&target_url)?
        .with_ipc_handler(move |_, req| {
            let mut parts = req.splitn(3, '|');
            let cmd = parts.next().unwrap_or("");
            match cmd {
                "open" => { 
                    let file = parts.next().unwrap_or("").to_string();
                    if sys_is_dev { eprintln!("💬 [IPC] open: {}", file); }
                    let _ = ipc_proxy.send_event(FrontierEvent::OpenWindow(file)); 
                },
                "spawn" => {
                    let u = parts.next().unwrap_or("").to_string();
                    let c = parts.next().unwrap_or("").to_string();
                    if sys_is_dev { eprintln!("💬 [IPC] spawn: {}", u); }
                    let _ = ipc_proxy.send_event(FrontierEvent::OpenWindow(format!("spawn://{}?{}", u, c)));
                },
                _ => {
                    let arg = parts.next().unwrap_or("").to_string();
                    if sys_is_dev { eprintln!("💬 [IPC] exec: {} {}", cmd, if arg.is_empty() { "(no args)" } else { &arg }); }
                    let _ = ipc_proxy.send_event(FrontierEvent::RunCommand(wid, format!("{}|{}", cmd, arg)));
                }
            }
        })
        .build()?;

    app_state.webviews.insert(wid, webview);
    app_state.persistence.insert(wid, PersistenceConfig { should_save: config.persistent, save_file });
    Ok(())
}

// --- HELPERS ---

/// Routes URLs to the system browser with atomic deduplication to prevent duplicate opens
/// 
/// This function prevents the same URL from being opened multiple times within a short timeframe,
/// which can occur when redirect chains happen (e.g., GitHub's automatic locale redirect from
/// https://github.com/ to https://github.com/?locale=pt-BR). 
/// 
/// The deduplication works by:
/// 1. Normalizing URLs (removing query params and fragments)
/// 2. Comparing base URLs only (ignoring locale, tracking, and other query parameters)
/// 3. Using an atomic lock to ensure only one thread can open a URL at a time
/// 4. Ignoring rapid subsequent opens of the same base URL (2-second window)
/// 
/// # Arguments
/// * `url` - The full URL to open in the system browser
fn route_to_browser(url: &str, is_dev: bool) {
    let mut lock = BROWSER_LOCK.lock().unwrap();
    let now = Instant::now();
    
    // Extract only the domain + path, removing query params and fragments
    let base_url = url
        .split('?')
        .next()
        .unwrap_or(url)
        .split('#')
        .next()
        .unwrap_or(url)
        .trim_end_matches('/');

    // If the same base URL was opened within the last 2 seconds, ignore this request
    // This prevents duplicate tabs when redirect chains or multiple handlers fire for the same URL
    if lock.0 == base_url && now.duration_since(lock.1) < Duration::from_millis(2000) {
        if is_dev { eprintln!("⏱️ [BROWSER] Deduped (within 2s): {}", base_url); }
        return;
    }
    
    // Update state BEFORE opening to atomically block any parallel threads
    lock.0 = base_url.to_string();
    lock.1 = now;
    
    if is_dev { eprintln!("🌐 [BROWSER] Opening: {}", url); }
    let _ = webbrowser::open(url);
}

fn get_url_category(url: &str, internal: &[String], browser: &[String]) -> UrlCategory {
    if url.starts_with("frontier://") || url.starts_with("https://frontier.") || url == "about:blank" {
        eprintln!("📍 [ROUTING] Frontier: {}", url);
        return UrlCategory::Frontier;
    }
    if is_url_allowed(url, internal) { 
        eprintln!("📍 [ROUTING] Internal (whitelisted): {}", url);
        return UrlCategory::Internal; 
    }
    if is_url_allowed(url, browser) { 
        eprintln!("📍 [ROUTING] Browser (whitelisted): {}", url);
        return UrlCategory::Browser; 
    }
    eprintln!("📍 [ROUTING] Blocked: {}", url);
    UrlCategory::Blocked
}

fn is_url_allowed(url: &str, whitelist: &[String]) -> bool {
    let base_url = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);
    let clean_url = base_url.trim_end_matches('/');
    
    for pattern in whitelist {
        let has_wildcard = pattern.ends_with('*');
        let base_pattern = pattern.trim_end_matches('*').trim_end_matches('/');
        let regex_pattern = base_pattern.replace(".", "\\.").replace("/", "\\/");
        
        // If pattern has wildcard: allow base path and any subpaths (e.g., https://kaiohsg.dev/*)
        // If pattern has no wildcard: allow only exact URL (e.g., https://kaiohsg.dev)
        let final_regex = if has_wildcard {
            format!(r"^{}(/.*)?\/?$", regex_pattern)
        } else {
            format!(r"^{}\/?$", regex_pattern)
        };
        
        if let Ok(re) = regex::Regex::new(&final_regex) {
            if re.is_match(base_url) || re.is_match(clean_url) { return true; }
        }
    }
    false
}

fn setup_paths(is_dev: bool) -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let root = std::env::current_dir()?;
    if is_dev {
        let data = root.join(".frontier").join("target").join("dev_profile");
        let cache = root.join(".frontier").join("target").join("dev_cache");
        let _ = fs::create_dir_all(&data);
        let _ = fs::create_dir_all(&cache);
        Ok((root, data, cache))
    } else {
        let base = std::env::temp_dir().join("frontier_rt_v1");
        let _ = fs::create_dir_all(&base);
        let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
        let data = Path::new(&local).join("FrontierData").join("App");
        let _ = fs::create_dir_all(&data);
        for file in Assets::iter() {
            let dest = base.join(file.as_ref());
            if let Some(p) = dest.parent() { let _ = fs::create_dir_all(p); }
            if let Some(c) = Assets::get(file.as_ref()) { let _ = fs::write(&dest, c.data.as_ref()); }
        }
        Ok((base, data, PathBuf::new()))
    }
}

fn scan_environment(base: &Path, _cache: &Path, is_dev: bool) -> (HashMap<String, system::RuntimeMeta>, HashMap<String, system::ModuleManifest>) {
    let mut cmds = HashMap::new();
    let mut mods = HashMap::new();
    if is_dev {
        let m_dir = base.join("modules");
        if m_dir.exists() {
            for entry in WalkDir::new(m_dir).min_depth(2).max_depth(2) {
                if let Ok(e) = entry {
                    if e.file_name() == "manifest.toml" {
                        if let Ok(c) = fs::read_to_string(e.path()) {
                            if let Ok(m) = toml::from_str::<system::ModuleManifest>(&c) {
                                mods.insert(m.extension.clone(), m);
                            }
                        }
                    }
                }
            }
        }
        let b_dir = base.join("app").join("backend");
        if b_dir.exists() {
            if let Ok(entries) = fs::read_dir(b_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let stem = p.file_stem().unwrap().to_str().unwrap().to_string();
                    if let Some(m) = mods.get(ext) {
                        let trigger_key = stem.clone();
                        cmds.insert(trigger_key, system::RuntimeMeta { 
                            trigger: stem, 
                            filename: p.to_string_lossy().to_string(), 
                            interpreter: m.interpreter.clone(), 
                            suppress_window: m.suppress_window 
                        });
                    }
                }
            }
        }
    } else {
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                if entry.path().to_string_lossy().ends_with(".meta.json") {
                    if let Ok(c) = fs::read_to_string(entry.path()) {
                        if let Ok(m) = serde_json::from_str::<system::RuntimeMeta>(&c) {
                            cmds.insert(m.trigger.clone(), m);
                        }
                    }
                }
            }
        }
    }
    (cmds, mods)
}

fn save_window_state(wid: &WindowId, app: &AppState) {
    if let (Some(p), Some(wv)) = (app.persistence.get(wid), app.webviews.get(wid)) {
        if !p.should_save { return; }
        let win = wv.window();
        let scale = win.scale_factor();
        let is_max = win.is_maximized();

        let mut final_x = 0.0; let mut final_y = 0.0;
        let mut final_w = 800.0; let mut final_h = 600.0;

        if is_max {
            if let Ok(old_json) = fs::read_to_string(&p.save_file) {
                if let Ok(old) = serde_json::from_str::<window::WindowState>(&old_json) {
                    final_x = old.x; final_y = old.y; final_w = old.width; final_h = old.height;
                }
            }
        } else {
            let pos = win.outer_position().unwrap_or_default().to_logical::<f64>(scale);
            let size = win.inner_size().to_logical::<f64>(scale);
            final_x = pos.x; final_y = pos.y; final_w = size.width; final_h = size.height;
        }

        let state = window::WindowState { x: final_x, y: final_y, width: final_w, height: final_h, maximized: is_max };
        if let Ok(j) = serde_json::to_string(&state) { let _ = fs::write(&p.save_file, j); }
    }
}

fn load_application_icon(base: &Path) -> Option<Icon> {
    let p = base.join("assets").join("app_icon.png");
    if p.exists() { load_icon_from_disk(&p) } else { None }
}

fn load_icon_from_disk(path: &Path) -> Option<Icon> {
    image::open(path).ok().and_then(|img| {
        let rgba = img.resize(32, 32, FilterType::Lanczos3).into_rgba8().into_raw();
        Icon::from_rgba(rgba, 32, 32).ok()
    })
}