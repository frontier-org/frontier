# ⚡ Frontier Documentation

**Frontier** is a language-agnostic Graphical User Interface (GUI) Engine. It allows you to create native and portable Desktop applications for Windows, where the Backend can be written in any language (C, Python, Java, Go, Batch, Node) and the Frontend is built with modern Web technologies.

---

## 📂 1. Project Structure

A healthy Frontier project follows this structure:

```text
/MyProject
│
├── frontier.cmd           # CLI (Command Line Interface)
├── frontier.toml          # Executable Metadata (Version, EXE Icon)
│
├── app/
│   ├── frontend/          # HTML, CSS, JS and Window Icons
│   │   ├── index.html
│   │   └── style.css
│   └── backend/           # Your Scripts and Source Code
│       ├── calculate.c
│       ├── script.py
│       └── ComplexApp!java_gradle/  (Folder as Backend)
│
├── modules/               # Language Definitions (Compilers/Interpreters)
│   ├── mod_c/
│   └── mod_python/
│
└── .frontier/             # Engine (Rust, Cache, Build System) - Don't touch
```

---

## ⚙️ 2. Executable Configuration (`frontier.toml`)

This file controls **only** the metadata of the final `.exe` file generated on Windows. Window configurations (size, position) are now controlled by HTML.

**File:** `frontier.toml`
```toml
[app]
name = "MySuperApp"        # Final file name (e.g. MySuperApp.exe)
version = "1.0.0"          # Version (appears in File Properties)
description = "Description"  # File description
copyright = "© 2025 Corp"  # Copyright
author = "Dev Name"        # Author

[window]
# Icon that appears in Windows Explorer and Taskbar.
# MUST BE A VALID .ICO (don't rename png).
icon = "app/frontend/icon.ico" 
```

---

## 🖥️ 3. Frontend & Window Management

Frontier treats HTML as the "window configuration". You control native window behavior using **Meta Tags** in the `<head>`.

### Available Settings (Meta Tags)

| Meta Name | Example Value | Description |
| :--- | :--- | :--- |
| `frontier-title` | "My App" | Window Title (Or use `<title>` tag). |
| `frontier-width` | `800` | Initial width. |
| `frontier-height` | `600` | Initial height. |
| `frontier-min-width`| `400` | Minimum allowed width. |
| `frontier-min-height`| `300` | Minimum allowed height. |
| `frontier-x` | `(screen_w - win_w) / 2` | Horizontal Position. Accepts Math Formulas. |
| `frontier-y` | `0` | Vertical Position (0 = Top). Accepts Formulas. |
| `frontier-resizable`| `true` / `false` | Allows border resizing. |
| `frontier-maximized`| `true` / `false` | Starts maximized. |
| `frontier-minimizable`| `true` / `false` | Shows/Hides minimize button. |
| `frontier-maximizable`| `true` / `false` | Shows/Hides maximize button. |
| `frontier-icon` | `icon.png` | Title bar icon (path relative to HTML). |
| `frontier-persistent`| `true` | Save/Restore position and size on close. |
| `frontier-id` | `main_window` | Unique ID for persistence save file. |

### Math Formulas
In `x` and `y` tags, you can use variables:
*   `screen_w`: Monitor width.
*   `screen_h`: Monitor height.
*   `win_w`: Window width.
*   `win_h`: Window height.

**Complete HTML Example:**
```html
<!DOCTYPE html>
<html>
<head>
    <title>Admin Panel</title>
    <!-- Center window -->
    <meta name="frontier-x" content="(screen_w - win_w) / 2">
    <meta name="frontier-y" content="(screen_h - win_h) / 2">
    <!-- Size and Icon -->
    <meta name="frontier-width" content="1024">
    <meta name="frontier-height" content="768">
    <meta name="frontier-icon" content="assets/admin.png">
    <!-- Persistence -->
    <meta name="frontier-persistent" content="true">
    <meta name="frontier-id" content="admin_panel">
</head>
<body>
    <h1>App Running</h1>
    <button onclick="run()">Execute Backend</button>
    <script>
        // IPC API
        function run() {
            // Syntax: "backend_file|arguments"
            window.ipc.postMessage('calculate|10 20');
        }
        
        // Receive Response
        window.Frontier = {
            dispatch: (type, msg) => {
                console.log(msg); // Receives from Rust
            }
        };
    </script>
</body>
</html>
```

### Opening New Windows
You can open secondary windows (popups) via JS:
```javascript
// Opens popup.html in a new native window
window.ipc.postMessage('open|popup.html');
```

---

## 🧱 4. Backend Implementation

Place your files in `app/backend/`. Frontier detects the extension and looks up the corresponding module.

### Supported Backend Types

1.  **Single File (`script.py`, `code.c`)**
    *   Frontier uses the file name as the trigger.
    *   Ex: `app/backend/analyze.py` -> Trigger: `analyze`.

2.  **Project Folder (`Name!extension`)**
    *   Use for complex projects (Java Gradle, C Make, Node Modules).
    *   The folder must be named: `CommandName!module_extension`.
    *   Ex: Folder `app/backend/Benchmark!java`.
    *   Frontier enters the folder, runs the build defined in the `java` module and generates the executable.
    *   Trigger: `Benchmark`.

### Arguments
Everything you pass in JS (`window.ipc.postMessage('trigger|arg1 arg2')`) is forwarded to the binary/script as command-line arguments (`argv`).

---

## 📦 5. Module Creation (`modules/`)

A module teaches Frontier how to compile or run a language.
Create a folder in `modules/module_name/` and add a `manifest.toml`.

### `manifest.toml` Reference

```toml
name = "Readable Name"
version = "1.0.0"       # For update system
extension = "py"        # Extension this module controls

# (Optional) Interpreter to run the final file
# Use this for script languages (Python, JS, Bat) or Bytecode (Java)
interpreter = "python" 

# (Optional) If true, hides the black console window when running
suppress_window = true

# BUILD CONFIGURATION (Production and Dev "Build Strategy")
[build]
# Magic Variables:
# %IN%  -> Absolute path of source file (or project folder)
# %OUT% -> Absolute path where Frontier expects the final file
command = "gcc %IN% -o %OUT%"

# DEV CONFIGURATION (Hot Reload)
[dev]
# "interpreter": Does nothing on save, just runs. (Python, JS)
# "build": Runs the [build] command every time the file is saved. (C, Go, Rust)
strategy = "interpreter"
```

### Practical Examples

**Python (Script):**
```toml
extension = "py"
interpreter = "python"
suppress_window = true
[dev]
strategy = "interpreter"
```

**C (Native):**
```toml
extension = "c"
suppress_window = true
[build]
command = "gcc %IN% -o %OUT%"
[dev]
strategy = "build"
```

**Java Gradle (Folder):**
```toml
extension = "java"
interpreter = "java -jar"
[build]
# Frontier automatically sets the working directory inside the folder
command = "call gradle build -x test && copy /Y build\\libs\\app.jar %OUT%"
[dev]
strategy = "build"
```

---

## 💻 6. CLI (Command Line)

Use the `.\frontier` script at the root.

*   **`.\frontier dev`**
    *   Starts development mode.
    *   Enables **Hot Reload** (changes in Front or Back are reflected immediately).
    *   Reads files directly from the `app/` folder.
    *   Compiles binaries (C/Go) to temporary cache.
*   **`.\frontier build`**
    *   Starts production mode.
    *   Compiles all scripts and projects.
    *   Generates a single executable in `dist/`.
    *   This executable is **static** (doesn't need DLLs alongside).
*   **`.\frontier install <url>`**
    *   Downloads modules from the internet.
    *   Supports `gh:user/repo` (GitHub).
    *   Supports `https://.../file.zip`.
    *   Supports `--folder name` to download subfolders from monorepos.
*   **`.\frontier clean`**
    *   Cleans temporary folders (`target`, `assets`, `dist`). Use if something goes wrong.

---

## 🛡️ Technical Notes

1.  **Persistence:** Window data (and cookies/localstorage) are saved in `%LOCALAPPDATA%\AppName`.
