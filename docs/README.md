# 🌐 Frontier

***The world opens up. Cross the frontier.***

[![Release](https://img.shields.io/github/v/release/frontier-org/frontier?include_prereleases)](https://github.com/frontier-org/frontier/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/frontier-org/frontier/total?include_prereleases)](https://github.com/frontier-org/frontier/releases)
[![VirusTotal](https://img.shields.io/badge/virustotal-status-navy)](https://www.virustotal.com/gui/search/https%253A%252F%252Fgithub%252Ecom%252Ffrontier-org%252Ffrontier%252Farchive%252Frefs%252Fheads%252Fmain%252Ezip)

**Frontier** is a Agnostic and Native Graphical User Interface (GUI) Framework. It allows developers to create interfaces using Web technologies (**HTML5, CSS3, JavaScript**) to control backends written in any language (**C, Python, Rust, Go, Node.js**), consolidating the result into a **Single Static Executable** for Windows.

## 📖 Documentation

* 🔭 **[MANUAL.md](./MANUAL.md):** Detailed technical documentation on how to build apps, configure modules, and use the IPC bridge.
* 🗺️ **[ROADMAP.md](./ROADMAP.md):** Strategic overview of development phases, completed features, and the transition to a standalone framework.
* 📊 **[LOGS.md](./LOGS.md):** Overview of the development logging system, event categorization (IPC, Routing, Assets), and zero-overhead production behavior.

## How to Get Frontier?

### Using Command

1. Use the **command below** in the working directory.

```powershell
powershell -nop "iex(irm https://frontier-fw.dev/get.ps1)"
```

2. Give the project folder a name, or leave it blank for the current folder.

### Using Frontier.zip

1. Download [**Frontier.zip**](https://github.com/frontier-org/frontier/releases).
2. Extract the files to the project folder.

### Requirements

You will need to have the [Rust](https://rust-lang.org/tools/install/) and MSVC from [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/) installed (note that rustup can install MSVC for you).

The project is now ready to use. See [MANUAL.md](./MANUAL.md) for more information.

## System Architecture

The architecture is based on two distinct Rust binaries that operate in different lifecycles:

### 🛠️ The Manager (`manager.rs`) - "The Builder"
Acts as CLI, build system and package orchestrator.

* **Configuration:** Reads the `frontier.toml` file to define metadata (version, copyright) and visual resources (icons).
* **Module Management:** Identifies languages in the `app/backend` folder and executes necessary pre-compilation based on each module's rules.
* **Packaging:** Groups assets (HTML, CSS, JS) and compiled binaries.
* **Build Pipeline:** Invokes the Rust compiler (`Cargo`) to generate the Core and organizes delivery to the `dist/` folder.

### 🧠 The Core (`core.rs`) - "The Runtime"
Is the engine of the final executable (e.g. `MyApp.exe`).

* **Native WebView:** Renders the interface through the operating system's engine (Edge WebView2 on Windows), statically linked to eliminate external DLL dependencies.
* **`frontier://` Protocol:** Virtual filesystem that serves content directly from memory (Production) or disk (Dev), mitigating CORS errors.
* **IPC (Inter-Process Communication):** Communication bridge that receives commands from JavaScript (`window.ipc.postMessage`) and dispatches execution to the backend binary or script in the background.
* **Window Orchestration:** Defines window properties (dimensions, icon, resizing, min/max constraints) dynamically via `<meta>` tags in HTML.
* **URL Routing & Security:** Implements multi-category URL handling (Frontier/Internal/Browser/Blocked) with atomic deduplication to prevent duplicate opens.
* **State Persistence:** Automatically stores window coordinates and state in `%LOCALAPPDATA%`, restoring user experience on restart.

## Lifecycle and Data Flow

### Development Mode (`.\frontier dev`)
1.  Sets the `FRONTIER_DEV` environment flag.
2.  The **Core** scans `app/backend` for sources (e.g. `.c`, `.go`).
3.  **On-the-fly Compilation:** If detected, invokes the local compiler (e.g. GCC) to generate binaries in a temporary cache (`.frontier/target/dev_cache`).
4.  **Hot Reload:** A *watcher* monitors changes. Changes in Front trigger a `reload`; changes in Back trigger a silent recompilation.

### Production Mode (`.\frontier build`)
1.  The **Manager** cleans and prepares the assets directory.
2.  Backend scripts are compiled and moved into the internal bundle.
3.  **Resource Injection:** Generates a dynamic `build.rs` to embed the `.ico` icon and metadata directly into the Windows executable manifest.
4.  **Static Compilation:** Core is compiled in `Release` mode (Static MSVC).
5.  **Bundling:** Uses the `rust-embed` macro to "consume" all assets, resulting in a single independent binary.

## Features Matrix

| Feature | Status | Technical Description |
| :--- | :---: | :--- |
| **Single Executable** | ✅ | A simple, small binary encompasses your entire application. |
| **HTML Configuration** | ✅ | Layout and behavior defined by `<meta>` tags (width, height, min/max constraints). |
| **Hot Reload** | ✅ | Real-time update for Front and Backend. |
| **Polyglot Support** | ✅ | Modular architecture that accepts any binary via `manifest.toml`. |
| **URL Routing & Security** | ✅ | Multi-category URL handling with whitelist support and atomic deduplication. |
| **Window Size Constraints** | ✅ | Configurable min/max width and height to control resizing behavior. |
| **Multiple Windows** | ✅ | Open independent windows with separate DOM, styles, and security policies. |
| **Development Logs** | ✅ | Detailed logging in dev mode without polluting production builds. |

## License

*Licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [The MIT License](https://opensource.org/license/mit). See [LICENSE-APACHE](https://github.com/frontier-org/frontier?tab=Apache-2.0-1-ov-file#readme), [LICENSE-MIT](https://github.com/frontier-org/frontier?tab=MIT-2-ov-file#readme), [NOTICE](https://github.com/frontier-org/frontier/blob/main/NOTICE) and [LICENSE folder](https://github.com/frontier-org/frontier/tree/main/LICENSE) for details.*

Copyright (c) 2026 The Frontier Framework Authors  
SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception OR MIT