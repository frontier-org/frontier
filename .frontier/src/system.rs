/// System State and Backend Command Management Module
/// 
/// Manages runtime metadata, module information, and backend command execution.

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

fn default_true() -> bool {
    true
}

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
    pub is_dev: bool,
    pub window_icon: Option<wry::application::window::Icon>,
}

/// Execute a backend command with the given trigger and arguments
pub fn execute_backend(system: &SystemState, trigger: &str, args: &str) -> String {
    if let Some(mut meta) = system.commands.get(trigger).cloned() {
        // In dev mode, if the file is source code (C/C++/etc) with no interpreter,
        // try to compile it first
        #[cfg(debug_assertions)]
        if system.is_dev && meta.interpreter.is_none() {
            if meta.filename.ends_with(".c") || meta.filename.ends_with(".cpp") {
                // Try to compile the source file
                let source_path = if std::path::Path::new(&meta.filename).is_absolute() {
                    std::path::PathBuf::from(&meta.filename)
                } else {
                    system.base_dir.join(&meta.filename)
                };

                if source_path.exists() {
                    if let Some(module) = system.modules_map.get("c") {
                        if let Some(build_rule) = &module.build {
                            let output_path = system.dev_cache.join(format!("{}.exe", trigger));
                            let _ = std::fs::remove_file(&output_path);

                            let cmd_str = build_rule.command
                                .replace("%IN%", source_path.to_str().unwrap_or(""))
                                .replace("%OUT%", output_path.to_str().unwrap_or(""));

                            let status = if cfg!(windows) {
                                Command::new("cmd")
                                    .args(["/C", &cmd_str])
                                    .current_dir(&system.base_dir)
                                    .status()
                            } else {
                                Command::new("sh")
                                    .arg("-c")
                                    .arg(&cmd_str)
                                    .current_dir(&system.base_dir)
                                    .status()
                            };

                            if let Ok(s) = status {
                                if s.success() && output_path.exists() {
                                    // Update metadata to point to compiled binary
                                    meta.filename = output_path.to_string_lossy().to_string();
                                } else {
                                    return format!("Compilation failed for '{}'", trigger);
                                }
                            } else {
                                return format!("Failed to execute compiler for '{}'", trigger);
                            }
                        }
                    }
                }
            }
        }

        // Check if binary exists after potential compilation
        if meta.interpreter.is_none() && (meta.filename.ends_with(".c") || meta.filename.ends_with(".cpp")) {
            return format!("ERROR: Binary for '{}' does not exist.", trigger);
        }

        let file_path = if std::path::Path::new(&meta.filename).is_absolute() {
            std::path::PathBuf::from(&meta.filename)
        } else {
            system.base_dir.join(&meta.filename)
        };

        if !file_path.exists() {
            return format!("ERROR: Not found: {:?}", file_path);
        }

        // Build the command
        let mut cmd = if let Some(interpreter) = &meta.interpreter {
            let mut parts = interpreter.split_whitespace();
            let mut c = Command::new(parts.next().unwrap());
            c.args(parts);
            c.arg(&file_path);
            c
        } else {
            Command::new(&file_path)
        };

        cmd.args(args.split_whitespace());
        cmd.current_dir(&system.base_dir);

        #[cfg(target_os = "windows")]
        if meta.suppress_window {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        // Execute and capture output
        match cmd.output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => format!("Execution failed: {}", e),
        }
    } else {
        format!("Command '{}' not registered", trigger)
    }
}
