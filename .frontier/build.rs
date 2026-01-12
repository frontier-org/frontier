// Copyright 2026 The Frontier Framework Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 1. Ensure folders for RustEmbed (PAYLOAD REMOVAL)
    let folders = ["assets", "assets/frontend"]; 
    
    for folder in folders {
        if !Path::new(folder).exists() {
            let _ = fs::create_dir_all(folder);
        }
    }
    
    // Dummy file to avoid error if assets is empty
    if !Path::new("assets/.keep").exists() { 
        let _ = fs::write("assets/.keep", ""); 
    }

    // 2. Configure Windows Resources (Icon and Metadata)
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        
        // Try to find the icon in the original App folder (Relative Path)
        let icon_original = Path::new("../app/frontend/icon.ico");
        
        // Try to use absolute path to ensure 'rc.exe' can find it
        if let Ok(abs_path) = fs::canonicalize(icon_original) {
            res.set_icon(abs_path.to_str().unwrap());
        } else if Path::new("icon.ico").exists() {
            // Fallback: Try in the root of .frontier if it exists
            res.set_icon("icon.ico");
        } else {
            println!("cargo:warning=⚠️  Icon not found at: {:?}", icon_original);
        }

        // Inject metadata via Environment Variables (from Manager)
        if let Ok(v) = env::var("FRONTIER_APP_VERSION") { res.set("FileVersion", &v); res.set("ProductVersion", &v); }
        if let Ok(v) = env::var("FRONTIER_APP_NAME") { res.set("ProductName", &v); res.set("InternalName", &v); }
        if let Ok(v) = env::var("FRONTIER_APP_DESC") { res.set("FileDescription", &v); }
        if let Ok(v) = env::var("FRONTIER_APP_COPYRIGHT") { res.set("LegalCopyright", &v); }

        if let Err(e) = res.compile() {
            println!("cargo:warning=WinRes compilation error: {}", e);
        }
    }
    
    // Monitoring
    println!("cargo:rerun-if-changed=../app/frontend/icon.ico");
    println!("cargo:rerun-if-changed=icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FRONTIER_APP_VERSION");
}