// Copyright (c) 2026 The Frontier Framework Authors
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception OR MIT

// Backend Processing Module
// 
// This module handles the detection and compilation of backend files.
// It reads module manifests and coordinates the build process.

use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::collections::HashMap;
use walkdir::WalkDir;

#[derive(Deserialize, Clone)]
pub struct ModuleManifest {
    pub extension: String,
    pub interpreter: Option<String>,
    #[serde(default = "default_suppress")]
    pub suppress_window: bool,
    pub build: Option<BuildRule>,
}

fn default_suppress() -> bool {
    true
}

#[derive(Deserialize, Clone)]
pub struct BuildRule {
    pub command: String,
}

#[derive(serde::Serialize)]
pub struct RuntimeMeta {
    pub trigger: String,
    pub filename: String,
    pub interpreter: Option<String>,
    pub suppress_window: bool,
}

// Safely splits the command into parts, respecting quotes.
fn split_shell_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in cmd.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ' ' && !in_quotes {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() { args.push(current); }
    args
}

// Load all module manifests from the modules directory
pub fn load_modules(modules_path: &Path) -> HashMap<String, ModuleManifest> {
    let mut builders = HashMap::new();

    if modules_path.exists() {
        for entry in WalkDir::new(modules_path)
            .min_depth(1)
            .max_depth(2)
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_name() == "manifest.toml" {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(manifest) = toml::from_str::<ModuleManifest>(&content) {
                        builders.insert(manifest.extension.clone(), manifest);
                    }
                }
            }
        }
    }

    builders
}

// Process backend files and generate metadata
pub fn process_backend_files(
    backend_path: &Path,
    assets_path: &Path,
    modules: &HashMap<String, ModuleManifest>,
) {
    if !backend_path.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(backend_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if let Some(module) = modules.get(ext) {
                    process_single_file(&path, assets_path, module);
                }
            }
        }
    }
}

// Process a single backend file with its module
fn process_single_file(
    file_path: &Path,
    assets_path: &Path,
    module: &ModuleManifest,
) {
    let stem = file_path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");

    if let Some(rule) = &module.build {
        // Agnostic extension logic:
        // If it's a JAR engine, use .jar. Otherwise, default to original or .exe.
        let out_filename = if let Some(interp) = &module.interpreter {
            if interp.contains("-jar") {
                format!("{}.jar", stem)
            } else {
                file_path.file_name().unwrap().to_str().unwrap().to_string()
            }
        } else {
            let exe_ext = if cfg!(windows) { "exe" } else { "" };
            if exe_ext.is_empty() { stem.to_string() } else { format!("{}.{}", stem, exe_ext) }
        };

        let out_path = assets_path.join(&out_filename);
        
        // Prepare paths for replacement
        let in_str = file_path.to_str().unwrap_or("");
        let out_str = out_path.to_str().unwrap_or("");
        
        // Split the command from manifest into parts (same as system.rs)
        let cmd_parts: Vec<String> = split_shell_args(&rule.command)
            .into_iter()
            .map(|part| {
                part.replace("%IN%", in_str).replace("%OUT%", out_str)
            })
            .collect();

        if cmd_parts.is_empty() {
            return;
        }

        println!("   > Building {}", stem);

        // Execute command as an array of arguments. 
        // Rust handles spaces and batch files automatically on Windows.
        let status = Command::new(&cmd_parts[0])
            .args(&cmd_parts[1..])
            .status()
            .unwrap_or_else(|_| std::process::ExitStatus::default());

        if !status.success() {
            panic!("Failed to build {}", stem);
        }

        // Generate metadata pointing to the correct final file (e.g. .jar)
        let meta = RuntimeMeta {
            trigger: stem.to_string(),
            filename: out_filename,
            interpreter: module.interpreter.clone(),
            suppress_window: module.suppress_window,
        };

        if let Ok(json) = serde_json::to_string(&meta) {
            let _ = fs::write(assets_path.join(format!("{}.meta.json", stem)), json);
        }
    } else {
        // No build rule: just copy the file and generate metadata
        let out_filename = file_path.file_name().unwrap().to_str().unwrap();
        let _ = fs::copy(file_path, assets_path.join(out_filename));
        
        let meta = RuntimeMeta {
            trigger: stem.to_string(),
            filename: out_filename.to_string(),
            interpreter: module.interpreter.clone(),
            suppress_window: module.suppress_window,
        };

        if let Ok(json) = serde_json::to_string(&meta) {
            let _ = fs::write(assets_path.join(format!("{}.meta.json", stem)), json);
        }
    }
}