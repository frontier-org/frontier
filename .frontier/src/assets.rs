// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 The Frontier Framework Authors

// Assets Management Module
// 
// This module handles copying and organizing frontend assets (HTML, CSS, JS, images)
// from the app/frontend directory to the build assets directory.

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Copy frontend files to the assets directory
pub fn copy_frontend_files(src: &Path, dst: &Path) {
    if !src.exists() {
        return;
    }

    for entry in WalkDir::new(src).min_depth(1) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_type().is_file() {
            let filename = entry.file_name();
            let dest_path = dst.join(filename);

            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let _ = fs::copy(entry.path(), &dest_path);
        }
    }
}

// Copy and organize icon file
pub fn copy_icon(icon_path: &Path, assets_path: &Path) -> Option<()> {
    if !icon_path.exists() {
        return None;
    }

    let ext = icon_path.extension()?;
    let dest = assets_path.join("app_icon").with_extension(ext);

    fs::copy(icon_path, dest).ok()?;
    Some(())
}
