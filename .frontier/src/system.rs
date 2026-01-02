use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeMeta {
    pub trigger: String,
    pub filename: String,
    pub interpreter: Option<String>,
    #[serde(default = "default_true")]
    pub suppress_window: bool,
}

fn default_true() -> bool { true }

#[derive(Deserialize, Clone, Debug)]
pub struct ModuleManifest {
    pub extension: String,
    pub interpreter: Option<String>,
    #[serde(default = "default_true")]
    pub suppress_window: bool,
    #[cfg(debug_assertions)]
    pub build: Option<BuildRule>,
}

#[cfg(debug_assertions)]
#[derive(Deserialize, Clone, Debug)]
pub struct BuildRule {
    pub command: String,
}

pub struct SystemState {
    pub commands: HashMap<String, RuntimeMeta>,
    #[cfg(debug_assertions)]
    pub modules_map: HashMap<String, ModuleManifest>,
    pub base_dir: PathBuf,
    pub data_dir: PathBuf,
    #[cfg(debug_assertions)]
    pub dev_cache: PathBuf,
    pub allowed_internal: Vec<String>,
    pub allowed_browser: Vec<String>,
    pub is_dev: bool,
    pub window_icon: Option<wry::application::window::Icon>,
}

/// Safely splits the command defined in manifest.toml into parts
/// Only compiled in debug mode to avoid 'dead_code' warnings in release
#[cfg(debug_assertions)]
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

pub fn execute_backend(system: &SystemState, trigger: &str, args: &str) -> String {
    #[allow(unused_mut)]
    if let Some(mut meta) = system.commands.get(trigger).cloned() {
        
        #[cfg(debug_assertions)]
        if system.is_dev {
            let file_path = if std::path::Path::new(&meta.filename).is_absolute() {
                std::path::PathBuf::from(&meta.filename)
            } else {
                system.base_dir.join(&meta.filename)
            };

            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                if let Some(module) = system.modules_map.get(ext) {
                    if let Some(build_rule) = &module.build {
                        let output_path = system.dev_cache.join(format!("{}.exe", trigger));
                        let _ = std::fs::remove_file(&output_path);

                        let in_str = file_path.to_str().unwrap_or("");
                        let out_str = output_path.to_str().unwrap_or("");

                        // Split the command template FIRST, then replace variables.
                        // This ensures that paths with spaces are kept as a single argument.
                        let cmd_parts: Vec<String> = split_shell_args(&build_rule.command)
                            .into_iter()
                            .map(|part| part.replace("%IN%", in_str).replace("%OUT%", out_str))
                            .collect();

                        if !cmd_parts.is_empty() {
                            let status = Command::new(&cmd_parts[0])
                                .args(&cmd_parts[1..])
                                .current_dir(&system.base_dir)
                                .status();

                            if let Ok(s) = status {
                                if s.success() && output_path.exists() {
                                    meta.filename = output_path.to_string_lossy().to_string();
                                } else {
                                    return format!("Build step failed for trigger '{}'. Please verify your module manifest and compiler installation.", trigger);
                                }
                            }
                        }
                    }
                }
            }
        }

        let run_path = if std::path::Path::new(&meta.filename).is_absolute() {
            std::path::PathBuf::from(&meta.filename)
        } else {
            system.base_dir.join(&meta.filename)
        };

        if !run_path.exists() { return format!("ERROR: Not found: {:?}", run_path); }

        let mut cmd = if let Some(interpreter) = &meta.interpreter {
            let mut parts = interpreter.split_whitespace();
            let mut c = Command::new(parts.next().unwrap());
            c.args(parts);
            c.arg(&run_path);
            c
        } else {
            Command::new(&run_path)
        };

        cmd.args(args.split_whitespace());
        cmd.current_dir(&system.base_dir);

        #[cfg(target_os = "windows")]
        if meta.suppress_window { cmd.creation_flags(CREATE_NO_WINDOW); }

        match cmd.output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => format!("Execution failed: {}", e),
        }
    } else {
        format!("Command '{}' not registered", trigger)
    }
}