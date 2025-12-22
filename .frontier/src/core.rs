#![windows_subsystem = "windows"]

use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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
use image::GenericImageView;
use image::imageops::FilterType;
use regex::Regex;
use evalexpr::*; 
use notify::{Watcher, RecursiveMode, EventKind};
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

// --- ESTRUTURAS ---
struct AppState {
    webviews: HashMap<WindowId, WebView>,
    persistence: HashMap<WindowId, PersistenceConfig>,
    system: Arc<Mutex<SystemState>>,
    main_proxy: EventLoopProxy<FrontierEvent>,
    debounce: HashMap<PathBuf, Instant>,
}

struct PersistenceConfig {
    should_save: bool,
    save_file: PathBuf,
}

struct SystemState {
    commands: HashMap<String, RuntimeMeta>,
    modules_map: HashMap<String, ModuleManifest>,
    base_dir: PathBuf,
    data_dir: PathBuf,
    dev_cache: PathBuf,
    is_dev: bool,
    window_icon: Option<Icon>,
}

#[derive(Serialize, Deserialize)]
struct WindowState {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    maximized: bool,
}

#[derive(Deserialize, Clone, Debug)]
struct RuntimeMeta {
    trigger: String,
    filename: String,
    interpreter: Option<String>,
    #[serde(default = "default_true")]
    suppress_window: bool,
}

fn default_true() -> bool { true }

struct PageConfig {
    title: String,
    width: f64,
    height: f64,
    x: Option<String>,
    y: Option<String>,
    resizable: bool,
    maximized: bool,
    persistent: bool,
    id: String,
    icon_path: Option<String>,
    min_width: Option<f64>,
    min_height: Option<f64>,
    minimizable: bool,
    maximizable: bool,
}

#[derive(Deserialize, Clone)]
struct ModuleManifest {
    extension: String,
    interpreter: Option<String>,
    #[serde(default = "default_true")]
    suppress_window: bool,
    build: Option<BuildRule>,
    dev: Option<DevConfig>,
}

#[derive(Deserialize, Clone)]
struct BuildRule {
    command: String,
}

#[derive(Deserialize, Clone)]
struct DevConfig {
    strategy: String,
}

enum FrontierEvent {
    RunCommand(WindowId, String),
    BackendReply(WindowId, String),
    OpenWindow(String),
    FileChanged(PathBuf),
}

// --- MAIN ---
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("FRONTIER_DEV").is_ok() {
        #[cfg(target_os = "windows")]
        unsafe {
            use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }

    let is_dev = std::env::var("FRONTIER_DEV").is_ok();
    
    let base_dir;
    let data_dir;
    let dev_cache;
    
    if is_dev {
        let root = std::env::current_dir()?;
        base_dir = root.clone();
        data_dir = root.join(".frontier/target/dev_profile");
        dev_cache = root.join(".frontier/target/dev_cache");
        fs::create_dir_all(&dev_cache)?;
        fs::create_dir_all(&data_dir)?;
    } else {
        let temp = std::env::temp_dir().join("frontier_runtime_v115");
        if !temp.exists() { fs::create_dir_all(&temp)?; }
        base_dir = temp.clone();
        dev_cache = PathBuf::new();
        
        let local = std::env::var("LOCALAPPDATA").unwrap_or(".".into());
        let exe_name = std::env::current_exe()?.file_stem().unwrap_or_default().to_string_lossy().to_string();
        data_dir = Path::new(&local).join("FrontierData").join(exe_name);
        fs::create_dir_all(&data_dir)?;

        for file in Assets::iter() {
            let file_str = file.as_ref();
            let dest_path = base_dir.join(file_str);
            
            if let Some(parent) = dest_path.parent() { 
                fs::create_dir_all(parent)?; 
            }
            
            if let Some(content) = Assets::get(file_str) {
                fs::write(&dest_path, content.data.as_ref())?;
            }
        }
    }

    let (commands, modules_map) = scan_environment(&base_dir, &dev_cache, is_dev);

    let mut icon = None;
    let prod_icon = base_dir.join("assets").join("app_icon.png"); 
    let dev_icon = base_dir.join(".frontier").join("assets").join("app_icon.png");
    let icon_to_load = if prod_icon.exists() { Some(prod_icon) } else if dev_icon.exists() { Some(dev_icon) } else { None };
    if let Some(p) = icon_to_load {
        if let Ok(img) = image::open(p) {
            let resized = img.resize(32, 32, FilterType::Lanczos3);
            let (w, h) = resized.dimensions();
            icon = Some(Icon::from_rgba(resized.into_rgba8().into_raw(), w, h).unwrap());
        }
    }

    let system = Arc::new(Mutex::new(SystemState {
        commands,
        modules_map,
        base_dir: base_dir.clone(),
        data_dir: data_dir.clone(),
        dev_cache: dev_cache.clone(),
        is_dev,
        window_icon: icon,
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
        let watch_dir = base_dir.join("app");
        let mut w = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        for path in event.paths {
                            let _ = watch_proxy.send_event(FrontierEvent::FileChanged(path));
                        }
                    },
                    _ => {}
                }
            }
        }).unwrap();
        w.watch(&watch_dir, RecursiveMode::Recursive).unwrap();
        _watcher = Some(w);
    }

    create_new_window(&event_loop, &mut app_state, &mut web_context, "index.html", main_proxy.clone())?;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(FrontierEvent::FileChanged(path)) => {
                if let Some(last_time) = app_state.debounce.get(&path) {
                    if last_time.elapsed() < Duration::from_millis(500) { return; }
                }
                app_state.debounce.insert(path.clone(), Instant::now());
                let path_str = path.to_string_lossy();
                
                if path_str.contains("frontend") && (path_str.ends_with(".html") || path_str.ends_with(".css") || path_str.ends_with(".js")) {
                    println!("🔄  Frontend alterado: {:?}", path.file_name().unwrap());
                    for webview in app_state.webviews.values() {
                        let _ = webview.evaluate_script("location.reload();");
                    }
                }
                if path_str.contains("backend") {
                    println!("🛠️  Backend alterado: {:?}", path.file_name().unwrap());
                    let sys = app_state.system.clone();
                    let (msg, success) = recompile_and_notify(&sys, &path);
                    if success { println!("✅ {}", msg); } else { println!("❌ {}", msg); }
                    let js = format!("console.log('%c[Frontier] {}', 'color: {}; font-weight: bold')", msg, if success { "#0f0" } else { "#f55" });
                    for webview in app_state.webviews.values() { let _ = webview.evaluate_script(&js); }
                }
            },
            Event::UserEvent(FrontierEvent::RunCommand(wid, cmd_str)) => {
                let sys = app_state.system.clone();
                let proxy = app_state.main_proxy.clone();
                thread::spawn(move || {
                    let mut parts = cmd_str.splitn(2, '|');
                    let res = execute_backend(&sys, parts.next().unwrap_or(""), parts.next().unwrap_or(""));
                    let _ = proxy.send_event(FrontierEvent::BackendReply(wid, res));
                });
            },
            Event::UserEvent(FrontierEvent::BackendReply(wid, msg)) => {
                if let Some(webview) = app_state.webviews.get(&wid) {
                    let safe_msg = msg.replace('\\', "\\\\").replace('`', "\\`").replace('\'', "\\'");
                    let js = format!("window.Frontier.dispatch('log', `{}`)", safe_msg);
                    let _ = webview.evaluate_script(&js);
                }
            },
            Event::UserEvent(FrontierEvent::OpenWindow(path)) => {
                let proxy_clone = app_state.main_proxy.clone();
                let _ = create_new_window(event_loop, &mut app_state, &mut web_context, &path, proxy_clone);
            },
            Event::WindowEvent { event, window_id, .. } => match event {
                WindowEvent::CloseRequested => {
                    if let Some(persist) = app_state.persistence.get(&window_id) {
                        if persist.should_save {
                            if let Some(webview) = app_state.webviews.get(&window_id) {
                                let window = webview.window();
                                let is_max = window.is_maximized();
                                if let Ok(pos) = window.outer_position() {
                                    let size = window.inner_size();
                                    let scale = window.scale_factor();
                                    let mut final_x = pos.to_logical::<f64>(scale).x;
                                    let mut final_y = pos.to_logical::<f64>(scale).y;
                                    let mut final_w = size.to_logical::<f64>(scale).width;
                                    let mut final_h = size.to_logical::<f64>(scale).height;

                                    if is_max && persist.save_file.exists() {
                                        if let Ok(old_json) = fs::read_to_string(&persist.save_file) {
                                            if let Ok(old) = serde_json::from_str::<WindowState>(&old_json) {
                                                final_x = old.x; final_y = old.y; final_w = old.width; final_h = old.height;
                                            }
                                        }
                                    }
                                    let state = WindowState { x: final_x, y: final_y, width: final_w, height: final_h, maximized: is_max };
                                    let _ = serde_json::to_string(&state).map(|j| fs::write(&persist.save_file, j));
                                }
                            }
                        }
                    }
                    app_state.webviews.remove(&window_id);
                    app_state.persistence.remove(&window_id);
                    if app_state.webviews.is_empty() { *control_flow = ControlFlow::Exit; }
                },
                _ => ()
            },
            _ => ()
        }
    });
}

fn create_new_window(
    event_loop: &EventLoopWindowTarget<FrontierEvent>,
    app_state: &mut AppState,
    context: &mut WebContext,
    file_name: &str,
    proxy: EventLoopProxy<FrontierEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sys = app_state.system.lock().unwrap();
    
    let (html_content, _html_path) = if sys.is_dev {
        let p = sys.base_dir.join("app/frontend").join(file_name);
        (fs::read_to_string(&p).unwrap_or_else(|e| format!("<h1>Erro Dev</h1><p>{}</p>", e)), p)
    } else {
        let embed_path = format!("frontend/{}", file_name);
        if let Some(f) = Assets::get(&embed_path) {
            (std::str::from_utf8(f.data.as_ref()).unwrap_or("Encoding error").to_string(), PathBuf::from(&embed_path))
        } else {
            (format!("<h1>404 Embed</h1><p>{}</p>", embed_path), PathBuf::from("404"))
        }
    };

    let mut config = parse_html_config(&html_content, file_name);
    let save_file_path = sys.data_dir.join(format!("state_{}.json", config.id));
    let mut is_maximized = config.maximized;

    if config.persistent && save_file_path.exists() {
        if let Ok(json) = fs::read_to_string(&save_file_path) {
            if let Ok(saved) = serde_json::from_str::<WindowState>(&json) {
                config.width = saved.width; config.height = saved.height;
                config.x = Some(saved.x.to_string()); config.y = Some(saved.y.to_string());
                is_maximized = saved.maximized;
            }
        }
    }

    let mut builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .with_resizable(config.resizable)
        .with_minimizable(config.minimizable)
        .with_maximizable(config.maximizable)
        .with_maximized(is_maximized)
        .with_visible(true);
    
    if let (Some(w), Some(h)) = (config.min_width, config.min_height) { builder = builder.with_min_inner_size(LogicalSize::new(w, h)); }
    
    let mut win_icon = None;
    if let Some(path_str) = &config.icon_path {
        if sys.is_dev {
            let p = sys.base_dir.join("app/frontend").join(path_str);
            if p.exists() { win_icon = load_icon_from_disk(&p); }
        } else {
            let embed_path = format!("frontend/{}", path_str);
            if let Some(d) = Assets::get(&embed_path) {
                if let Ok(img) = image::load_from_memory(&d.data) {
                    let resized = img.resize(32, 32, FilterType::Lanczos3);
                    let (w, h) = resized.dimensions();
                    win_icon = Some(Icon::from_rgba(resized.into_rgba8().into_raw(), w, h).unwrap());
                }
            }
        }
    }
    if let Some(icon) = win_icon { builder = builder.with_window_icon(Some(icon)); } 
    else if let Some(icon) = &sys.window_icon { builder = builder.with_window_icon(Some(icon.clone())); }

    if !is_maximized {
        if let (Some(fx), Some(fy)) = (config.x, config.y) {
            if let Some(monitor) = event_loop.primary_monitor() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let sw = size.width as f64 / scale;
                let sh = size.height as f64 / scale;
                let x = eval_math(&fx, sw, sh, config.width, config.height);
                let y = eval_math(&fy, sw, sh, config.width, config.height);
                builder = builder.with_position(LogicalPosition::new(x, y));
            }
        }
    }

    let window = builder.build(event_loop)?;
    let window_id = window.id();
    let ipc_proxy = proxy.clone();
    
    let sys_base = sys.base_dir.clone();
    let sys_is_dev = sys.is_dev;

    let webview = WebViewBuilder::new(window)?
        .with_web_context(context)
        .with_custom_protocol("frontier".into(), move |request| {
            let uri = request.uri().to_string();
            let url_path = uri.replace("frontier://", "");
            let clean_path = url_path.split('?').next().unwrap_or(&url_path).trim_end_matches('/');
            
            let (mime, body): (String, Cow<[u8]>) = if sys_is_dev {
                let file_path = sys_base.join("app/frontend").join(clean_path);
                match fs::read(&file_path) {
                    Ok(bytes) => (get_mime_type(&file_path), Cow::Owned(bytes)),
                    Err(_) => ("text/html".into(), Cow::Owned(format!("<h1>Erro 404 (Dev)</h1><p>{:?}</p>", file_path).into_bytes()))
                }
            } else {
                let embed_path = format!("frontend/{}", clean_path);
                match Assets::get(&embed_path) {
                    Some(content) => (get_mime_from_str(clean_path), content.data),
                    None => ("text/html".into(), Cow::Owned(format!("<h1>Erro 404 (Prod)</h1><p>Asset: {}</p>", embed_path).into_bytes()))
                }
            };

            Response::builder()
                .header(header::CONTENT_TYPE, mime)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(body)
                .map_err(|_| wry::Error::InitScriptError)
        })
        .with_url(&format!("frontier://{}", file_name))?
        .with_devtools(sys.is_dev)
        .with_ipc_handler(move |window, req| {
            let wid = window.id();
            let mut parts = req.splitn(2, '|');
            let cmd = parts.next().unwrap_or("");
            let arg = parts.next().unwrap_or("").to_string();
            if cmd == "open" { let _ = ipc_proxy.send_event(FrontierEvent::OpenWindow(arg)); }
            else { let _ = ipc_proxy.send_event(FrontierEvent::RunCommand(wid, format!("{}|{}", cmd, arg))); }
        })
        .build()?;

    app_state.webviews.insert(window_id, webview);
    app_state.persistence.insert(window_id, PersistenceConfig { should_save: config.persistent, save_file: save_file_path });
    Ok(())
}

fn get_mime_type(path: &Path) -> String {
    get_mime_from_str(path.to_string_lossy().as_ref())
}

fn get_mime_from_str(path: &str) -> String {
    if path.ends_with(".html") { return "text/html".into(); }
    if path.ends_with(".js") { return "text/javascript".into(); }
    if path.ends_with(".css") { return "text/css".into(); }
    if path.ends_with(".png") { return "image/png".into(); }
    if path.ends_with(".jpg") { return "image/jpeg".into(); }
    if path.ends_with(".svg") { return "image/svg+xml".into(); }
    if path.ends_with(".json") { return "application/json".into(); }
    "application/octet-stream".into()
}

fn load_icon_from_disk(path: &Path) -> Option<Icon> {
    if let Ok(img) = image::open(path) {
        let resized = img.resize(32, 32, FilterType::Lanczos3);
        let (w, h) = resized.dimensions();
        let rgba = resized.into_rgba8().into_raw();
        Some(Icon::from_rgba(rgba, w, h).unwrap())
    } else {
        None
    }
}

fn parse_html_config(html: &str, filename: &str) -> PageConfig {
    let re_title = Regex::new(r"<title>(.*?)</title>").unwrap();
    let re_meta = Regex::new(r#"<meta\s+name=["']frontier-(.*?)["']\s+content=["'](.*?)["']\s*/?>"#).unwrap();

    let title = re_title.captures(html).map(|c| c[1].to_string()).unwrap_or("App".into());
    let mut width = 800.0;
    let mut height = 600.0;
    let mut x = None;
    let mut y = None;
    let mut resizable = true;
    let mut persistent = false;
    let mut maximized = false;
    let mut minimizable = true;
    let mut maximizable = true;
    let mut min_width = None;
    let mut min_height = None;
    let mut id = filename.replace(".", "_");
    let mut icon_path = None;

    for caps in re_meta.captures_iter(html) {
        let key = &caps[1];
        let val = &caps[2];
        match key {
            "width" => width = val.parse().unwrap_or(800.0),
            "height" => height = val.parse().unwrap_or(600.0),
            "min-width" => min_width = val.parse().ok(),
            "min-height" => min_height = val.parse().ok(),
            "x" => x = Some(val.to_string()),
            "y" => y = Some(val.to_string()),
            "resizable" => resizable = val != "false",
            "persistent" => persistent = val == "true",
            "maximized" => maximized = val == "true",
            "minimizable" => minimizable = val != "false",
            "maximizable" => maximizable = val != "false",
            "id" => id = val.to_string(),
            "icon" => icon_path = Some(val.to_string()),
            _ => {}
        }
    }
    PageConfig {
        title, width, height, x, y, resizable, maximized, persistent, id, icon_path, min_width, min_height, minimizable, maximizable
    }
}

fn eval_math(formula: &str, sw: f64, sh: f64, ww: f64, wh: f64) -> f64 {
    let mut context = HashMapContext::new();
    let _ = context.set_value("screen_w".into(), Value::Float(sw));
    let _ = context.set_value("screen_h".into(), Value::Float(sh));
    let _ = context.set_value("win_w".into(), Value::Float(ww));
    let _ = context.set_value("win_h".into(), Value::Float(wh));
    eval_number_with_context(formula, &context).unwrap_or(0.0)
}

fn recompile_and_notify(sys_arc: &Arc<Mutex<SystemState>>, file_path: &Path) -> (String, bool) {
    let mut sys = sys_arc.lock().unwrap();
    
    if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
        if let Some(module) = sys.modules_map.get(ext).cloned() {
            let stem = file_path.file_stem().unwrap().to_str().unwrap().to_string();
            
            if let Some(dev) = &module.dev {
                if dev.strategy == "build" {
                    if let Some(rule) = &module.build {
                        let exe_ext = if cfg!(windows) { "exe" } else { "" };
                        let out_name = format!("{}.{}", stem, exe_ext);
                        let out_path = sys.dev_cache.join(&out_name);
                        
                        let cmd_str = rule.command
                            .replace("%IN%", file_path.to_str().unwrap())
                            .replace("%OUT%", out_path.to_str().unwrap());
                        
                        let mut cmd = if cfg!(windows) { Command::new("cmd") } else { Command::new("sh") };
                        
                        if cfg!(windows) {
                            cmd.args(["/C", &cmd_str]);
                            cmd.creation_flags(CREATE_NO_WINDOW);
                        } else {
                            cmd.arg("-c").arg(&cmd_str);
                        }

                        // Captura saída silenciosa
                        return match cmd.output() {
                            Ok(output) => {
                                if output.status.success() {
                                    if let Some(meta) = sys.commands.get_mut(&stem) {
                                        meta.filename = out_path.to_str().unwrap().to_string();
                                    }
                                    (format!("Binário '{}' atualizado!", stem), true)
                                } else {
                                    (format!("ERRO Compilação {}:\n{}", stem, String::from_utf8_lossy(&output.stderr)), false)
                                }
                            },
                            Err(e) => (format!("Erro crítico: {}", e), false),
                        }
                    }
                }
            }
            return (format!("Script '{}' atualizado.", stem), true);
        }
    }
    ("Arquivo desconhecido alterado".into(), true)
}

fn scan_environment(base: &Path, dev_cache: &Path, is_dev: bool) -> (HashMap<String, RuntimeMeta>, HashMap<String, ModuleManifest>) {
    let mut commands = HashMap::new();
    let mut modules = HashMap::new();
    
    if is_dev {
        let modules_dir = base.join("modules");
        let backend_dir = base.join("app/backend");
        
        if modules_dir.exists() {
            for entry in WalkDir::new(&modules_dir).min_depth(1).max_depth(2) {
                let entry = entry.unwrap();
                if entry.file_name() == "manifest.toml" {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(m) = toml::from_str::<ModuleManifest>(&content) {
                            modules.insert(m.extension.clone(), m.clone());
                        }
                    }
                }
            }
        }
        if backend_dir.exists() {
            for entry in fs::read_dir(backend_dir).unwrap().flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
                    
                    if let Some(module) = modules.get(ext) {
                        let mut final_filename = format!("app/backend/{}", path.file_name().unwrap().to_str().unwrap());
                        
                        if let Some(dev) = &module.dev {
                            if dev.strategy == "build" {
                                if let Some(rule) = &module.build {
                                    let exe_ext = if cfg!(windows) { "exe" } else { "" };
                                    let out_name = format!("{}.{}", stem, exe_ext);
                                    let out_path = dev_cache.join(&out_name);
                                    
                                    if !out_path.exists() {
                                        let cmd_str = rule.command
                                            .replace("%IN%", path.to_str().unwrap())
                                            .replace("%OUT%", out_path.to_str().unwrap());
                                        let _ = if cfg!(windows) {
                                            Command::new("cmd").args(["/C", &cmd_str]).creation_flags(CREATE_NO_WINDOW).status()
                                        } else {
                                            Command::new("sh").arg("-c").arg(&cmd_str).status()
                                        };
                                    }
                                    final_filename = out_path.to_str().unwrap().to_string();
                                }
                            }
                        }
                        commands.insert(stem, RuntimeMeta {
                            trigger: "".to_string(),
                            filename: final_filename,
                            interpreter: module.interpreter.clone(),
                            suppress_window: module.suppress_window,
                        });
                    }
                }
            }
        }
    } else {
        for entry in fs::read_dir(base).unwrap().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") && path.to_string_lossy().ends_with(".meta.json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(meta) = serde_json::from_str::<RuntimeMeta>(&content) {
                        commands.insert(meta.trigger.clone(), meta);
                    }
                }
            }
        }
    }
    (commands, modules)
}

fn execute_backend(sys: &Arc<Mutex<SystemState>>, trigger: &str, args: &str) -> String {
    let (commands, base_dir) = {
        let lock = sys.lock().unwrap();
        (lock.commands.clone(), lock.base_dir.clone())
    };

    if let Some(meta) = commands.get(trigger) {
        let full_path = if Path::new(&meta.filename).is_absolute() {
            PathBuf::from(&meta.filename)
        } else {
            base_dir.join(&meta.filename)
        };

        let mut cmd = if let Some(int) = &meta.interpreter {
            let mut parts = int.split_whitespace();
            let mut c = Command::new(parts.next().unwrap());
            c.args(parts).arg(&full_path);
            c
        } else {
            Command::new(&full_path)
        };
        
        cmd.args(args.split_whitespace()).current_dir(&base_dir);

        #[cfg(target_os = "windows")]
        {
            if meta.suppress_window {
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
        }

        match cmd.output() {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(e) => format!("Erro: {}", e),
        }
    } else {
        format!("Comando não encontrado: {}", trigger)
    }
}
