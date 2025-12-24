#![windows_subsystem = "windows"]

/// Frontier Core - Runtime Engine
/// 
/// The Core is the runtime for the final executable. It is responsible for:
/// - Rendering the native WebView
/// - Managing the virtual filesystem protocol (frontier://)
/// - Handling Inter-Process Communication (IPC) with the frontend
/// - Managing window state and persistence
/// - Executing backend commands
///
/// The Core no longer handles build logic - that's the Manager's job.

mod window;
mod system;

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
use system::ModuleManifest;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

// --- STRUCTURES ---

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
    #[cfg(debug_assertions)]
    NotifyFrontend(String, bool),
}

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

    let system = Arc::new(Mutex::new(system::SystemState {
        commands,
        #[cfg(debug_assertions)]
        modules_map: _modules_map,
        base_dir: base_dir.clone(),
        data_dir: data_dir.clone(),
        #[cfg(debug_assertions)]
        dev_cache,
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

    create_new_window(
        &event_loop,
        &mut app_state,
        &mut web_context,
        "index.html",
        main_proxy.clone(),
    )?;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            #[cfg(debug_assertions)]
            Event::UserEvent(FrontierEvent::NotifyFrontend(msg, success)) => {
                let js = format!(
                    "console.log('%c[Frontier] {}', 'color: {}; font-weight: bold')",
                    msg.replace('\'', "\\'"),
                    if success { "#0f0" } else { "#f55" }
                );
                for webview in app_state.webviews.values() {
                    let _ = webview.evaluate_script(&js);
                }
            }
            Event::UserEvent(FrontierEvent::FileChanged(path)) => {
                if app_state
                    .debounce
                    .get(&path)
                    .map_or(false, |t| t.elapsed() < Duration::from_millis(500))
                {
                    return;
                }
                app_state.debounce.insert(path.clone(), Instant::now());

                #[cfg(debug_assertions)]
                if path.to_string_lossy().contains("backend") {
                    let sys = app_state.system.clone();
                    let proxy = app_state.main_proxy.clone();
                    thread::spawn(move || {
                        let (msg, success) = recompile_backend(&sys, &path);
                        let _ = proxy.send_event(FrontierEvent::NotifyFrontend(msg, success));
                    });
                    return;
                }

                for webview in app_state.webviews.values() {
                    let _ = webview.evaluate_script("location.reload();");
                }
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
                    let safe = msg
                        .replace('\\', "\\\\")
                        .replace('`', "\\`")
                        .replace('\'', "\\'");
                    let _ = webview.evaluate_script(&format!(
                        "if(window.Frontier) window.Frontier.dispatch('log', `{}`)",
                        safe
                    ));
                }
            }
            Event::UserEvent(FrontierEvent::OpenWindow(path)) => {
                let proxy_clone = app_state.main_proxy.clone();
                let _ = create_new_window(event_loop, &mut app_state, &mut web_context, &path, proxy_clone);
            }
            Event::WindowEvent { event, window_id, .. } => match event {
                WindowEvent::CloseRequested => {
                    save_window_state(&window_id, &app_state);
                    app_state.webviews.remove(&window_id);
                    app_state.persistence.remove(&window_id);
                    if app_state.webviews.is_empty() {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });
}

// --- BACKEND RECOMPILATION (DEBUG ONLY) ---

#[cfg(debug_assertions)]
fn recompile_backend(_sys_arc: &Arc<Mutex<system::SystemState>>, _file_path: &Path) -> (String, bool) {
    (
        "Hot reload not implemented yet".to_string(),
        false,
    )
}

// --- PERSISTENCE ---

fn save_window_state(wid: &WindowId, app: &AppState) {
    if let (Some(p), Some(wv)) = (app.persistence.get(wid), app.webviews.get(wid)) {
        if !p.should_save {
            return;
        }

        let win = wv.window();
        let is_max = win.is_maximized();
        let scale = win.scale_factor();

        let pos = win.outer_position().unwrap_or_default().to_logical::<f64>(scale);
        let size = win.inner_size().to_logical::<f64>(scale);

        let mut final_x = pos.x;
        let mut final_y = pos.y;
        let mut final_w = size.width;
        let mut final_h = size.height;

        if is_max && p.save_file.exists() {
            if let Ok(old_json) = fs::read_to_string(&p.save_file) {
                if let Ok(old) = serde_json::from_str::<window::WindowState>(&old_json) {
                    final_x = old.x;
                    final_y = old.y;
                    final_w = old.width;
                    final_h = old.height;
                }
            }
        }

        let state = window::WindowState {
            x: final_x,
            y: final_y,
            width: final_w,
            height: final_h,
            maximized: is_max,
        };

        if let Ok(j) = serde_json::to_string(&state) {
            let _ = fs::write(&p.save_file, j);
        }
    }
}

// --- UTILITIES ---

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

        // Extract embedded assets
        for file in Assets::iter() {
            let dest = base.join(file.as_ref());
            if let Some(p) = dest.parent() {
                let _ = fs::create_dir_all(p);
            }
            if let Some(c) = Assets::get(file.as_ref()) {
                let _ = fs::write(&dest, c.data.as_ref());
            }
        }

        Ok((base, data, PathBuf::new()))
    }
}

fn scan_environment(
    base: &Path,
    _cache: &Path,
    is_dev: bool,
) -> (HashMap<String, system::RuntimeMeta>, HashMap<String, ModuleManifest>) {
    let mut cmds = HashMap::new();
    let mut mods = HashMap::new();

    if is_dev {
        let m_dir = base.join("modules");
        if m_dir.exists() {
            for entry in WalkDir::new(m_dir).min_depth(2).max_depth(2) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if entry.file_name() == "manifest.toml" {
                    if let Ok(c) = fs::read_to_string(entry.path()) {
                        if let Ok(m) = toml::from_str::<ModuleManifest>(&c) {
                            mods.insert(m.extension.clone(), m);
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
                        let fname = p.to_string_lossy().to_string();
                        cmds.insert(
                            stem,
                            system::RuntimeMeta {
                                trigger: "".into(),
                                filename: fname,
                                interpreter: m.interpreter.clone(),
                                suppress_window: m.suppress_window,
                            },
                        );
                    }
                }
            }
        }
    } else {
        // Production mode - load from embedded metadata files
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                if entry
                    .path()
                    .to_string_lossy()
                    .ends_with(".meta.json")
                {
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

fn create_new_window(
    event_loop: &EventLoopWindowTarget<FrontierEvent>,
    app_state: &mut AppState,
    context: &mut WebContext,
    file_name: &str,
    proxy: EventLoopProxy<FrontierEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sys = app_state.system.lock().unwrap();

    // Load HTML content
    let html = if sys.is_dev {
        fs::read_to_string(sys.base_dir.join("app/frontend").join(file_name))?
    } else {
        Assets::get(&format!("frontend/{}", file_name))
            .map(|f| String::from_utf8_lossy(f.data.as_ref()).to_string())
            .ok_or("404")?
    };

    let mut config = window::parse_html_config(&html, file_name);
    let save_file = sys.data_dir.join(format!("state_{}.json", config.id));
    let mut is_max = config.maximized;

    // Load saved window state if persistence is enabled
    if config.persistent {
        if let Ok(json) = fs::read_to_string(&save_file) {
            if let Ok(saved) = serde_json::from_str::<window::WindowState>(&json) {
                config.width = saved.width;
                config.height = saved.height;
                config.x = Some(saved.x.to_string());
                config.y = Some(saved.y.to_string());
                is_max = saved.maximized;
            }
        }
    }

    // Build window with configuration
    let mut builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .with_resizable(config.resizable)
        .with_minimizable(config.minimizable)
        .with_maximizable(config.maximizable)
        .with_maximized(is_max);

    if let (Some(w), Some(h)) = (config.min_width, config.min_height) {
        builder = builder.with_min_inner_size(LogicalSize::new(w, h));
    }

    // Set window icon
    let mut win_icon = None;
    if let Some(ipath) = &config.icon_path {
        let full_ipath = if sys.is_dev {
            sys.base_dir.join("app/frontend").join(ipath)
        } else {
            sys.base_dir.join("frontend").join(ipath)
        };
        win_icon = load_icon_from_disk(&full_ipath);
    }
    builder = builder.with_window_icon(win_icon.or_else(|| sys.window_icon.clone()));

    // Set window position
    if !is_max {
        if let (Some(fx), Some(fy)) = (config.x, config.y) {
            if let Some(mon) = event_loop.primary_monitor() {
                let s = mon.size().to_logical::<f64>(mon.scale_factor());
                let x = window::evaluate_math_expression(&fx, s.width, s.height, config.width, config.height);
                let y = window::evaluate_math_expression(&fy, s.width, s.height, config.width, config.height);
                builder = builder.with_position(LogicalPosition::new(x, y));
            }
        }
    }

    let window = builder.build(event_loop)?;
    let wid = window.id();
    let sys_base = sys.base_dir.clone();
    let sys_is_dev = sys.is_dev;
    let ipc_proxy = proxy.clone();

    // Create webview
    let webview = WebViewBuilder::new(window)?
        .with_web_context(context)
        .with_custom_protocol("frontier".into(), move |req| {
            let uri = req.uri().to_string();
            let mut clean = uri
                .strip_prefix("frontier://")
                .unwrap_or(&uri)
                .split('?')
                .next()
                .unwrap_or("")
                .trim_matches('/')
                .to_string();

            if clean.is_empty() {
                clean = "index.html".to_string();
            }

            let fp = if sys_is_dev {
                sys_base.join("app/frontend").join(&clean)
            } else {
                sys_base.join("frontend").join(&clean)
            };

            let mime = mime_guess::from_path(&fp)
                .first_or_octet_stream()
                .to_string();

            match fs::read(&fp) {
                Ok(b) => Response::builder()
                    .header(header::CONTENT_TYPE, mime)
                    .body(Cow::Owned(b))
                    .map_err(|_| wry::Error::InitScriptError),
                Err(_) => Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Cow::Owned(b"404".to_vec()))
                    .map_err(|_| wry::Error::InitScriptError),
            }
        })
        .with_url(&format!("frontier://{}", file_name))?
        .with_ipc_handler(move |_, req| {
            let mut parts = req.splitn(2, '|');
            let cmd = parts.next().unwrap_or("");
            let arg = parts.next().unwrap_or("").to_string();

            if cmd == "open" {
                let _ = ipc_proxy.send_event(FrontierEvent::OpenWindow(arg));
            } else {
                let _ = ipc_proxy.send_event(FrontierEvent::RunCommand(wid, format!("{}|{}", cmd, arg)));
            }
        })
        .build()?;

    app_state.webviews.insert(wid, webview);
    app_state.persistence.insert(
        wid,
        PersistenceConfig {
            should_save: config.persistent,
            save_file,
        },
    );

    Ok(())
}

fn load_application_icon(base: &Path) -> Option<Icon> {
    let p = base.join("assets").join("app_icon.png");
    if p.exists() {
        load_icon_from_disk(&p)
    } else {
        None
    }
}

fn load_icon_from_disk(path: &Path) -> Option<Icon> {
    image::open(path)
        .ok()
        .and_then(|img| {
            let rgba = img
                .resize(32, 32, FilterType::Lanczos3)
                .into_rgba8()
                .into_raw();
            Icon::from_rgba(rgba, 32, 32).ok()
        })
}