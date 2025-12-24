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
    pub minimizable: bool,
    pub maximizable: bool,
}

/// Parse HTML to extract window configuration from meta tags
pub fn parse_html_config(html: &str, filename: &str) -> PageConfig {
    let re_title = Regex::new(r"<title>(.*?)</title>").unwrap();
    let re_meta = Regex::new(r#"<meta\s+name=["']frontier-(.*?)["']\s+content=["'](.*?)["']\s*/?>"#).unwrap();

    let mut config = PageConfig {
        title: re_title
            .captures(html)
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| "App".into()),
        width: 800.0,
        height: 600.0,
        x: None,
        y: None,
        resizable: true,
        maximized: false,
        persistent: false,
        id: filename.replace('.', "_"),
        icon_path: None,
        min_width: None,
        min_height: None,
        minimizable: true,
        maximizable: true,
    };

    for caps in re_meta.captures_iter(html) {
        let key = &caps[1];
        let val = &caps[2];

        match key {
            "width" => config.width = val.parse().unwrap_or(800.0),
            "height" => config.height = val.parse().unwrap_or(600.0),
            "min-width" => config.min_width = val.parse().ok(),
            "min-height" => config.min_height = val.parse().ok(),
            "persistent" => config.persistent = val == "true",
            "maximized" => config.maximized = val == "true",
            "minimizable" => config.minimizable = val != "false",
            "maximizable" => config.maximizable = val != "false",
            "icon" => config.icon_path = Some(val.into()),
            "id" => config.id = val.into(),
            "x" => config.x = Some(val.into()),
            "y" => config.y = Some(val.into()),
            _ => {}
        }
    }

    config
}

/// Evaluate mathematical expressions in position formulas
/// 
/// Supports variables: screen_w, screen_h, win_w, win_h
pub fn evaluate_math_expression(formula: &str, screen_width: f64, screen_height: f64, window_width: f64, window_height: f64) -> f64 {
    let mut context = HashMapContext::new();
    let _ = context.set_value("screen_w".into(), Value::Float(screen_width));
    let _ = context.set_value("screen_h".into(), Value::Float(screen_height));
    let _ = context.set_value("win_w".into(), Value::Float(window_width));
    let _ = context.set_value("win_h".into(), Value::Float(window_height));

    eval_number_with_context(formula, &context).unwrap_or(0.0)
}
