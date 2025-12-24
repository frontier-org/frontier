/// Build Management Module
/// 
/// This module handles the compilation process using Cargo.
/// It coordinates the building of the core binary.

use std::process::Command;
use std::path::Path;

/// Cargo build configuration
pub struct BuildConfig {
    pub app_name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
}

/// Run cargo build with the specified configuration
pub fn run_cargo_build(manifest_path: &Path, bin_name: &str, config: &BuildConfig) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--manifest-path", 
              manifest_path.to_str().unwrap(), 
              "--release", 
              "--bin", 
              bin_name]);

    // Pass metadata as environment variables
    if let Some(name) = &config.app_name {
        cmd.env("FRONTIER_APP_NAME", name);
    }
    if let Some(version) = &config.version {
        cmd.env("FRONTIER_APP_VERSION", version);
    }
    if let Some(desc) = &config.description {
        cmd.env("FRONTIER_APP_DESC", desc);
    }
    if let Some(copyright) = &config.copyright {
        cmd.env("FRONTIER_APP_COPYRIGHT", copyright);
    }

    let status = cmd.status()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;

    if !status.success() {
        return Err(format!("Cargo build failed for binary: {}", bin_name));
    }

    Ok(())
}

/// Copy the final executable to the distribution directory
pub fn finalize_executable(
    source_exe: &Path,
    dest_exe: &Path,
) -> Result<(), String> {
    if !source_exe.exists() {
        return Err("CRITICAL ERROR: Executable not generated.".to_string());
    }

    std::fs::copy(source_exe, dest_exe)
        .map_err(|e| format!("Failed to copy executable: {}", e))?;

    Ok(())
}
