/// Frontier Manager - Build System and Package Orchestrator
/// 
/// The Manager is responsible for:
/// - Reading and validating project configuration
/// - Processing backend source files
/// - Managing module dependencies
/// - Coordinating the build process
/// - Packaging the final executable

mod config;
mod backend;
mod assets;
mod build;

use std::fs;
use std::path::Path;

// --- CONSTANTS ---
const APP_DIR: &str = "app";
const MODULES_DIR: &str = "modules";
const ASSETS_DIR: &str = ".frontier/assets";
const DIST_DIR: &str = "dist";
const BASE_DIR: &str = ".frontier";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "install" {
        return;
    }

    println!("🚀 FRONTIER BUILDER (Cleanest)");

    // 1. Cleanup
    if Path::new(DIST_DIR).exists() {
        let _ = fs::remove_dir_all(DIST_DIR);
    }
    let _ = fs::remove_dir_all(ASSETS_DIR);
    let _ = fs::remove_dir_all(".frontier/payload");

    fs::create_dir_all(ASSETS_DIR).expect("Failed to create assets directory");
    fs::create_dir_all(format!("{}/frontend", ASSETS_DIR)).expect("Failed to create frontend directory");
    fs::create_dir_all(DIST_DIR).expect("Failed to create dist directory");

    println!("⚙️  Loading configuration...");
    let app_config = config::load_config(Path::new("frontier.toml"));

    println!("📦 Processing backend files...");
    process_backend();
    copy_frontend_assets();

    println!("⚙️  Compiling Core...");
    compile_core(&app_config);

    let final_name = app_config.name.clone().unwrap_or_else(|| "MyApp".into());
    finalize_distribution(&final_name);
}

/// Load backend modules and process files
fn process_backend() {
    let modules_path = Path::new(MODULES_DIR);
    let backend_path = Path::new(APP_DIR).join("backend");
    let assets_path = Path::new(ASSETS_DIR);

    let modules = backend::load_modules(modules_path);
    backend::process_backend_files(&backend_path, assets_path, &modules);
}

/// Copy frontend assets to build directory
fn copy_frontend_assets() {
    let src = Path::new(APP_DIR).join("frontend");
    let dst = Path::new(ASSETS_DIR).join("frontend");

    fs::create_dir_all(&dst).ok();
    assets::copy_frontend_files(&src, &dst);

    // Copy icon if present
    if let Some(window_cfg) = config::load_window_config(Path::new("frontier.toml")) {
        if let Some(icon_path) = window_cfg.icon {
            let icon_src = Path::new(&icon_path);
            let _ = assets::copy_icon(icon_src, Path::new(ASSETS_DIR));
        }
    }
}

/// Compile the core binary using cargo
fn compile_core(app_config: &config::AppConfig) {
    let build_config = build::BuildConfig {
        app_name: app_config.name.clone(),
        version: app_config.version.clone(),
        description: app_config.description.clone(),
        copyright: app_config.copyright.clone(),
    };

    match build::run_cargo_build(
        Path::new(".frontier/Cargo.toml"),
        "core",
        &build_config,
    ) {
        Ok(_) => println!("✅ Core compiled successfully"),
        Err(e) => panic!("{}", e),
    }
}

/// Move the compiled executable to dist/ and rename it
fn finalize_distribution(app_name: &str) {
    let target_dir = Path::new(BASE_DIR).join("target/release");
    let dist_dir = Path::new(DIST_DIR);
    let core_name = "core.exe";
    let final_exe_name = format!("{}.exe", app_name);

    let src_exe = target_dir.join(core_name);
    let dst_exe = dist_dir.join(&final_exe_name);

    match build::finalize_executable(&src_exe, &dst_exe) {
        Ok(_) => {
            println!("\n✅ SUCCESS!");
            println!("📁 Native App: {}/{}", DIST_DIR, final_exe_name);
        }
        Err(e) => panic!("{}", e),
    }
}
