// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 The Frontier Framework Authors

use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct WindowConfig {
    pub icon: Option<String>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct SecurityConfig {
    #[serde(default)]
    pub allowed_internal: Vec<String>, // Open inside the App
    #[serde(default)]
    pub allowed_browser: Vec<String>,  // Open in system browser (Chrome/Edge)
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FrontierToml {
    pub app: Option<AppConfig>,
    pub window: Option<WindowConfig>,
    pub security: Option<SecurityConfig>,
}

#[allow(dead_code)]
pub fn load_config(config_path: &Path) -> AppConfig {
    let mut config = AppConfig { name: Some("App".into()), version: None, description: None, copyright: None };
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(parsed) = toml::from_str::<FrontierToml>(&content) {
            if let Some(app) = parsed.app { config = app; }
        }
    }
    config
}

#[allow(dead_code)]
pub fn load_security_config(config_path: &Path) -> SecurityConfig {
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(parsed) = toml::from_str::<FrontierToml>(&content) {
            if let Some(sec) = parsed.security { return sec; }
        }
    }
    SecurityConfig { allowed_internal: vec![], allowed_browser: vec![] }
}

#[allow(dead_code)]
pub fn load_window_config(config_path: &Path) -> Option<WindowConfig> {
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(parsed) = toml::from_str::<FrontierToml>(&content) { return parsed.window; }
    }
    None
}