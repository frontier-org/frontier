use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 1. Garante pastas do RustEmbed (PAYLOAD REMOVIDO)
    let folders = ["assets", "assets/frontend"]; 
    
    for folder in folders {
        if !Path::new(folder).exists() {
            let _ = fs::create_dir_all(folder);
        }
    }
    
    // Arquivo dummy para evitar erro se assets estiver vazio
    if !Path::new("assets/.keep").exists() { 
        let _ = fs::write("assets/.keep", ""); 
    }

    // 2. Configura Recursos do Windows (Ícone e Metadados)
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        
        // Tenta achar o ícone na pasta original do App (Caminho Relativo)
        let icon_original = Path::new("../app/frontend/icon.ico");
        
        // Tenta usar caminho absoluto para garantir que o 'rc.exe' encontre
        if let Ok(abs_path) = fs::canonicalize(icon_original) {
            res.set_icon(abs_path.to_str().unwrap());
        } else if Path::new("icon.ico").exists() {
            // Fallback: Tenta na raiz do .frontier se existir
            res.set_icon("icon.ico");
        }

        // Injeta Metadados via Variáveis de Ambiente (Vindas do Manager)
        if let Ok(v) = env::var("FRONTIER_APP_VERSION") { res.set("FileVersion", &v); res.set("ProductVersion", &v); }
        if let Ok(v) = env::var("FRONTIER_APP_NAME") { res.set("ProductName", &v); res.set("InternalName", &v); }
        if let Ok(v) = env::var("FRONTIER_APP_DESC") { res.set("FileDescription", &v); }
        if let Ok(v) = env::var("FRONTIER_APP_COPYRIGHT") { res.set("LegalCopyright", &v); }

        if let Err(e) = res.compile() {
            println!("cargo:warning=Erro WinRes: {}", e);
        }
    }
    
    // Monitoramento
    println!("cargo:rerun-if-changed=../app/frontend/icon.ico");
    println!("cargo:rerun-if-changed=icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FRONTIER_APP_VERSION");
}
