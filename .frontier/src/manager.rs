use std::fs;
use std::path::Path;
use std::process::Command;
use serde::Deserialize;
use walkdir::WalkDir;

// --- CAMINHOS ---
const APP_DIR: &str = "app";
const MODULES_DIR: &str = "modules";
const ASSETS_DIR: &str = ".frontier/assets";
const DIST_DIR: &str = "dist";
const BASE_DIR: &str = ".frontier";
// const PAYLOAD_DIR REMOVIDO

// --- CONFIG ---
#[derive(Deserialize)]
struct ConfigTOML { 
    app: Option<AppInfo>,
    window: Option<WindowInfo>
}

#[derive(Deserialize, Clone)]
struct AppInfo { 
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    copyright: Option<String>,
}

#[derive(Deserialize)]
struct WindowInfo { 
    icon: Option<String>
}

#[derive(Deserialize)] 
struct ModuleManifest { extension: String, interpreter: Option<String>, #[serde(default = "default_suppress")] suppress_window: bool, build: Option<BuildRule> }
fn default_suppress() -> bool { true }
#[derive(Deserialize)] struct BuildRule { command: String }
#[derive(serde::Serialize)] struct RuntimeMeta { trigger: String, filename: String, interpreter: Option<String>, suppress_window: bool }

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "install" { return; }

    println!("🚀 FRONTIER BUILDER (Cleanest)");
    
    // 1. Limpeza
    if Path::new(DIST_DIR).exists() { let _ = fs::remove_dir_all(DIST_DIR); }
    let _ = fs::remove_dir_all(ASSETS_DIR);
    // Limpa payload velho se existir
    let _ = fs::remove_dir_all(".frontier/payload");
    
    fs::create_dir_all(ASSETS_DIR).unwrap();
    fs::create_dir_all(format!("{}/frontend", ASSETS_DIR)).unwrap();
    fs::create_dir_all(DIST_DIR).unwrap();

    println!("⚙️  Configuração...");
    let app_info = process_config();

    println!("📦 Processando scripts...");
    process_user_backend();
    copy_frontend();

    println!("⚙️  Compilando Core...");
    run_cargo_build("core", &app_info);

    let final_name = app_info.name.clone().unwrap_or("MeuApp".into());
    finalize_dist_folder(&final_name);
}

fn process_config() -> AppInfo {
    let src = Path::new("frontier.toml");
    let dst = Path::new(ASSETS_DIR).join("frontier.toml");
    
    let mut info = AppInfo { name: Some("App".into()), version: None, description: None, copyright: None };

    if src.exists() {
        fs::copy(src, &dst).expect("Erro copiar TOML");
        let content = fs::read_to_string(src).unwrap();
        if let Ok(cfg) = toml::from_str::<ConfigTOML>(&content) {
            if let Some(app) = cfg.app { info = app; }
            if let Some(win) = &cfg.window {
                // Ícone da Janela
                if let Some(icon_path) = &win.icon {
                    let isrc = Path::new(icon_path);
                    if isrc.exists() {
                        let ext = isrc.extension().unwrap_or_default();
                        let idst = Path::new(ASSETS_DIR).join("app_icon").with_extension(ext);
                        let _ = fs::copy(isrc, idst);
                    }
                }
                // Ícone do EXE: Não precisamos mais copiar para .frontier se o build.rs
                // estiver configurado para ler o caminho original (../app/frontend/icon.ico).
                // Mas se quiser garantir, pode manter a cópia aqui.
                // Como atualizamos o build.rs para olhar "para fora", podemos remover a cópia.
            }
        }
    }
    info
}

fn run_cargo_build(bin_name: &str, info: &AppInfo) {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--manifest-path", ".frontier/Cargo.toml", "--release", "--bin", bin_name]);
    
    if let Some(v) = &info.name { cmd.env("FRONTIER_APP_NAME", v); }
    if let Some(v) = &info.version { cmd.env("FRONTIER_APP_VERSION", v); }
    if let Some(v) = &info.description { cmd.env("FRONTIER_APP_DESC", v); }
    if let Some(v) = &info.copyright { cmd.env("FRONTIER_APP_COPYRIGHT", v); }

    let status = cmd.status().expect("Falha ao rodar cargo");
    if !status.success() { panic!("Erro build {}", bin_name); }
}

fn finalize_dist_folder(app_name: &str) {
    let target_dir = Path::new(BASE_DIR).join("target/release");
    let dist_dir = Path::new(DIST_DIR);
    let core_name = "core.exe";
    let final_exe_name = format!("{}.exe", app_name);
    
    let src_exe = target_dir.join(core_name);
    let dst_exe = dist_dir.join(&final_exe_name);

    if !src_exe.exists() { panic!("ERRO CRÍTICO: Executável não gerado."); }

    fs::copy(&src_exe, &dst_exe).expect("Falha ao copiar executável");
    
    println!("\n✅ SUCESSO!");
    println!("📁 App Nativo: {}/{}", DIST_DIR, final_exe_name);
}

fn process_user_backend() {
    let modules_path = Path::new(MODULES_DIR);
    let backend_path = Path::new(APP_DIR).join("backend");
    let assets_out = Path::new(ASSETS_DIR);
    let mut builders = std::collections::HashMap::new();

    if modules_path.exists() {
        for entry in WalkDir::new(modules_path).min_depth(1).max_depth(2) {
            let entry = entry.unwrap();
            if entry.file_name() == "manifest.toml" {
                let content = fs::read_to_string(entry.path()).unwrap();
                if let Ok(m) = toml::from_str::<ModuleManifest>(&content) {
                    builders.insert(m.extension.clone(), m);
                }
            }
        }
    }

    if backend_path.exists() {
        for entry in fs::read_dir(backend_path).expect("App backend nao achado").flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let stem = path.file_stem().unwrap().to_str().unwrap();
                
                if let Some(module) = builders.get(ext) {
                    let out_filename;
                    if let Some(rule) = &module.build {
                        if module.interpreter.is_some() {
                            out_filename = path.file_name().unwrap().to_str().unwrap().to_string();
                        } else {
                            let exe_ext = if cfg!(windows) { "exe" } else { "" };
                            out_filename = format!("{}.{}", stem, exe_ext);
                        }
                        
                        let out_path = assets_out.join(&out_filename);
                        let cmd_str = rule.command.replace("%IN%", path.to_str().unwrap()).replace("%OUT%", out_path.to_str().unwrap());
                        println!("   > Build {}", stem);
                        
                        let status = if cfg!(windows) {
                            Command::new("cmd").args(["/C", &cmd_str]).status().unwrap()
                        } else {
                            Command::new("sh").arg("-c").arg(&cmd_str).status().unwrap()
                        };
                        if !status.success() { panic!("Falha build {}", stem); }
                    } else {
                        out_filename = path.file_name().unwrap().to_str().unwrap().to_string();
                        fs::copy(&path, assets_out.join(&out_filename)).unwrap();
                    }
                    
                    let meta = RuntimeMeta { 
                        trigger: stem.to_string(), 
                        filename: out_filename, 
                        interpreter: module.interpreter.clone(), 
                        suppress_window: module.suppress_window 
                    };
                    let json = serde_json::to_string(&meta).unwrap();
                    fs::write(assets_out.join(format!("{}.meta.json", stem)), json).unwrap();
                }
            }
        }
    }
}

fn copy_frontend() {
    let src = Path::new(APP_DIR).join("frontend");
    let dst = Path::new(ASSETS_DIR).join("frontend");
    if src.exists() {
        for entry in WalkDir::new(src).min_depth(1) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let filename = entry.file_name();
                fs::copy(entry.path(), dst.join(filename)).unwrap();
            }
        }
    }
}
