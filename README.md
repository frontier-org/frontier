# 🔭 Frontier: Technical Project Summary

**Frontier** is a Polyglot and Native Graphical User Interface (GUI) Engine. It allows developers to create interfaces using Web technologies (**HTML5, CSS3, JavaScript**) to control backends written in any language (**C, Python, Rust, Go, Node.js**), consolidating the result into a **Single Static Executable** for Windows.

---

## 1. System Architecture

The architecture is based on two distinct Rust binaries that operate in different lifecycles:

### 🛠️ A. The Manager (`manager.rs`) - "The Builder"
Acts as CLI, build system and package orchestrator.

* **Configuration:** Reads the `frontier.toml` file to define metadata (version, copyright) and visual resources (icons).
* **Module Management:** Identifies languages in the `app/backend` folder and executes necessary pre-compilation based on each module's rules.
* **Packaging:** Groups assets (HTML, CSS, JS) and compiled binaries.
* **Build Pipeline:** Invokes the Rust compiler (`Cargo`) to generate the Core and organizes delivery to the `dist/` folder.

### 🧠 B. The Core (`core.rs`) - "The Runtime"
Is the engine of the final executable (e.g. `MyApp.exe`).

* **Native WebView:** Renders the interface through the operating system's engine (Edge WebView2 on Windows), statically linked to eliminate external DLL dependencies.
* **`frontier://` Protocol:** Virtual filesystem that serves content directly from memory (Production) or disk (Dev), mitigating CORS errors.
* **IPC (Inter-Process Communication):** Communication bridge that receives commands from JavaScript (`window.ipc.postMessage`) and dispatches execution to the backend binary or script in the background.
* **Window Orchestration:** Defines window properties (dimensions, icon, resizing) dynamically via `<meta>` tags in HTML.
* **State Persistence:** Automatically stores window coordinates and state in `%LOCALAPPDATA%`, restoring user experience on restart.



---

## 2. Lifecycle and Data Flow

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

---

## 3. Features Matrix

| Feature | Status | Technical Description |
| :--- | :---: | :--- |
| **Single Executable** | ✅ | A simple, small binary encompasses your entire application. |
| **HTML Configuration** | ✅ | Layout and behavior defined by `<meta>` tags. |
| **Hot Reload** | ✅ | Real-time update for Front and Backend. |
| **Polyglot Support** | ✅ | Modular architecture that accepts any binary via `manifest.toml`. |
