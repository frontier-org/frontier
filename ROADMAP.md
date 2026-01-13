# 🗺️ Frontier Roadmap

A strategic overview of the development phases for **Frontier**, from initial setup to final static binary distribution.

## 🏆 Completed

* **CLI Route Mapping:** Finalize logic for `.\frontier`, `.\back`, and `.\front` to ensure correct directory scoping.
* **Project Scaffolding:** Standardize the `app/frontend/` and `app/backend/` hierarchy to ensure the Engine finds `index.html` and source files.
* **Metadata Implementation:** Configure `frontier.toml` to handle `[app]` metadata (Name, Version, Copyright) for Windows file properties.
* **Window Meta-Tag Parser:** Implement the engine's ability to read `frontier-width`, `frontier-height`, and `frontier-title` from the HTML `<head>`.
* **Window Size Constraints:** Add support for `frontier-min-width/height` and `frontier-max-width/height` meta tags to control resizing limits.
* **Math Formula Resolver:** Build the logic to calculate `screen_w`, `win_w`, and center the window via `frontier-x/y` formulas.
* **Module Manifest System:** Create the initial `manifest.toml` structure for `modules/` to define how `.c`, `.py`, and other extensions are handled.
* **Persistence Layer (ID/Save):** Setup the `%LOCALAPPDATA%` storage logic based on the `frontier-id` meta tag.
* **IPC Bridge Foundation:** Establish the `window.ipc.postMessage` communication link between the Webview and the Backend triggers.
* **Icon Handling:** Integrate the logic to pull the `.ico` from the path defined in `frontier.toml` for the final executable.
* **URL Routing & Security:** Implement comprehensive URL classification (Frontier/Internal/Browser/Blocked) with whitelist support via HTML meta tags.
* **URL Deduplication System:** Develop atomic locks and query-parameter normalization to prevent duplicate browser opens when redirect chains occur.
* **Dependency Audit:** Verify that the `.frontier/` folder (Rust/Build System) is correctly isolated from the user source code.

## 🏗️ Upcoming Development

* **Cross-Platform Linux Support:** Implement a Linux-compatible build pipeline, ensuring core engine functionality and Webview bindings across distributions using a unified source code.
* **Module Distribution System:** Develop a system for remote module acquisition via Git or Zip, enabling distribution of assets beyond `manifest.toml` with integrated versioning and update checks. All module specifications, including versions and origin links, will be recorded and managed directly within the `frontier.toml` file.
* **Multi-Extension & Directory-Based Modules:** 
    * **Unified Manifests:** Update `manifest.toml` to support multiple extensions (e.g., `extension = ["c", "cpp"]`) within a single file.
    * **Directory Scoping:** Implement `dir_extension` for folder-based modules, allowing for specialized lifecycle hooks like `[dir_dev]` and `[dir_build]` for complex compilation or asset processing.
* **Standalone Engine Distribution:** Transition Frontier into a single pre-compiled binary. This removes the need for the `.frontier/` source folder and Rust/C compilers, allowing users to build apps by simply providing the `frontier.toml`, `app/` and `modules/` folder.
* **Universal Cross-Platform Module Logic:** Enable unified `.fs` scripts to execute seamlessly across Windows and Linux by providing native OS-detection triggers, allowing a single module to handle platform-specific toolchains and commands while maintaining a consistent codebase for environment orchestration. 
* **Self-Contained Mobile Packaging:** Expand the engine to generate native mobile containers (APK/IPA) where the module system acts as a cross-compilation layer, embedding the language runtimes and pre-compiled backend binaries directly into the application's internal assets for standalone execution.

## ⚠️ Maintenance & Stability

* **Path to .ico:** I forgot to add the function to the actual path of the `.ico` file.
* **Folders are not embedded:** Folders within `app/frontend/` are not being embedded.

SPDX-License-Identifier: Apache-2.0  
Copyright (c) 2026 The Frontier Framework Authors