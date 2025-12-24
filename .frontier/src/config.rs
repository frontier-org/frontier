/// Configuration Management Module
/// 
/// This module handles reading and parsing the frontier.toml configuration file.
/// It separates configuration concerns from the main manager logic.

use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
}

#[derive(Deserialize)]
pub struct WindowConfig {
    pub icon: Option<String>,
}

#[derive(Deserialize)]
pub struct FrontierToml {
    pub app: Option<AppConfig>,
    pub window: Option<WindowConfig>,
}

/// Load the frontier.toml configuration file
pub fn load_config(config_path: &Path) -> AppConfig {
    let mut config = AppConfig {
        name: Some("App".into()),
        version: None,
        description: None,
        copyright: None,
    };

    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(parsed) = toml::from_str::<FrontierToml>(&content) {
                if let Some(app) = parsed.app {
                    config = app;
                }
            }
        }
    }

    config
}

/// Load window configuration from frontier.toml
pub fn load_window_config(config_path: &Path) -> Option<WindowConfig> {
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(parsed) = toml::from_str::<FrontierToml>(&content) {
                return parsed.window;
            }
        }
    }
    None
}
