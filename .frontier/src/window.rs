/// Window Configuration and Management Module
/// 
/// Handles parsing HTML meta tags and managing window state.

use regex::Regex;
use evalexpr::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub maximized: bool,
}

pub struct PageConfig {
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub x: Option<String>,
    pub y: Option<String>,
    pub resizable: bool,
    pub maximized: bool,
    pub persistent: bool,
    pub id: String,
    pub icon_path: Option<String>,
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
    /// Maximum window width in pixels (prevents resizing beyond this size)
    pub max_width: Option<f64>,
    /// Maximum window height in pixels (prevents resizing beyond this size)
    pub max_height: Option<f64>,
    pub minimizable: bool,
    pub maximizable: bool,
    pub allowed_internal: Vec<String>,
    pub ignore_global_security: bool,
}

pub fn parse_html_config(html: &str, filename: &str) -> PageConfig {
    let re_title = Regex::new(r"<title>(.*?)</title>").unwrap();
    // Regex melhorada para aceitar aspas simples ou duplas e espaços
    let re_meta = Regex::new(r#"<meta\s+name=["']frontier-(.*?)["']\s+content=["'](.*?)["']\s*/?>"#).unwrap();

    let mut config = PageConfig {
        title: re_title.captures(html).map(|c| c[1].to_string()).unwrap_or_else(|| "App".into()),
        width: 800.0,
        height: 600.0,
        x: None, y: None,
        resizable: true,
        maximized: false,
        persistent: false,
        id: filename.replace('.', "_"),
        icon_path: None,
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        minimizable: true,
        maximizable: true,
        allowed_internal: Vec::new(),
        ignore_global_security: false,
    };

    for caps in re_meta.captures_iter(html) {
        let key = &caps[1];
        let val = &caps[2];
        match key {
            "title" => config.title = val.to_string(),
            "width" => config.width = val.parse().unwrap_or(800.0),
            "height" => config.height = val.parse().unwrap_or(600.0),
            "min-width" => config.min_width = val.parse().ok(),
            "min-height" => config.min_height = val.parse().ok(),
            "max-width" => config.max_width = val.parse().ok(),
            "max-height" => config.max_height = val.parse().ok(),
            "resizable" => config.resizable = val == "true",
            "maximized" => config.maximized = val == "true",
            "persistent" => config.persistent = val == "true",
            "minimizable" => config.minimizable = val != "false",
            "maximizable" => config.maximizable = val != "false",
            "icon" => config.icon_path = Some(val.into()),
            "id" => config.id = val.into(),
            "x" => config.x = Some(val.into()),
            "y" => config.y = Some(val.into()),
            "allowed-internal" => {
                config.allowed_internal = val.split(',').map(|s| s.trim().to_string()).collect();
            }
            "ignore-global-security" => config.ignore_global_security = val == "true",
            _ => {}
        }
    }
    config
}

pub fn create_manual_config(url: &str, config_str: &str) -> PageConfig {
    let mut config = PageConfig {
        title: "Frontier Window".into(),
        width: 800.0, height: 600.0,
        x: None, y: None,
        resizable: true, maximized: false, persistent: false,
        id: url.replace(|c: char| !c.is_alphanumeric(), "_"),
        icon_path: None, min_width: None, min_height: None, max_width: None, max_height: None,
        minimizable: true, maximizable: true,
        allowed_internal: Vec::new(),
        ignore_global_security: false, // Default
    };

    for part in config_str.split(',') {
        let mut pair = part.splitn(2, '=');
        if let (Some(k), Some(v)) = (pair.next(), pair.next()) {
            let key = k.trim();
            let val = v.trim();
            match key {
                "title" => config.title = val.into(),
                "width" => config.width = val.parse().unwrap_or(800.0),
                "height" => config.height = val.parse().unwrap_or(600.0),
                "min_width" => config.min_width = val.parse().ok(),
                "min_height" => config.min_height = val.parse().ok(),
                "max_width" => config.max_width = val.parse().ok(),
                "max_height" => config.max_height = val.parse().ok(),
                "x" => config.x = Some(val.into()),
                "y" => config.y = Some(val.into()),
                "resizable" => config.resizable = val == "true",
                "maximized" => config.maximized = val == "true",
                "persistent" => config.persistent = val == "true",
                "minimizable" => config.minimizable = val != "false",
                "maximizable" => config.maximizable = val != "false",
                "ignore_global_security" => {
                    config.ignore_global_security = val == "true";
                },
                "icon" => config.icon_path = Some(val.into()),
                "id" => config.id = val.into(),
                "allowed_internal" => {
                    // Importante: no JS, use | para separar URLs na lista
                    config.allowed_internal = val.split('|').map(|s| s.trim().to_string()).collect();
                },
                _ => {}
            }
        }
    }
    config
}

pub fn evaluate_math_expression(formula: &str, screen_width: f64, screen_height: f64, window_width: f64, window_height: f64) -> f64 {
    let mut context = HashMapContext::new();
    let _ = context.set_value("screen_w".into(), Value::Float(screen_width));
    let _ = context.set_value("screen_h".into(), Value::Float(screen_height));
    let _ = context.set_value("win_w".into(), Value::Float(window_width));
    let _ = context.set_value("win_h".into(), Value::Float(window_height));
    eval_number_with_context(formula, &context).unwrap_or(0.0)
}
