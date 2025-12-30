# 🔭 Frontier Documentation

**Frontier** is a language-agnostic Graphical User Interface (GUI) Engine. It allows you to create native and portable Desktop applications for Windows, where the Backend can be written in any language (C, Python, Java, Go, Batch, Node) and the Frontend is built with modern Web technologies.

> **📊 Development Logging:** See [LOGS.md](LOGS.md) for detailed information about development-mode logging, debugging output, and monitoring your application during development.

> **💡 Exemple** Check out this [exemple](https://github.com/frontier-org/exemple) repository to see how to use Frontier across several languages and all HTML functions.

---

## 📂 1. Project Structure

A healthy Frontier project follows this structure:

```text
MyProject/
│
├── .frontier/             # Engine (Rust, Cache, Build System)
│
├── app/
|   |
│   ├── backend/           # Your Scripts and Source Code
│   │   ├── code.c
│   │   └── script.py
│   │
│   └── frontend/          # HTML, CSS, JS and Images
│       ├── icon.ico
│       ├── index.html
│       └── style.css
│
├── modules/               # Language Definitions (Compilers/Interpreters)
│   ├── mod_gcc/
│   └── mod_python/
│
├── back.bat               # Direct Backend CLI
├── front.bat              # Direct Frontend CLI
├── frontier.bat           # CLI principal
└── frontier.toml          # Project Metadata
```

---

## ⚙️ 2. Executable Configuration (`frontier.toml`)

This file controls frontier settings, and the metadata of the final `.exe` file generated on Windows.

**File:** `frontier.toml`
```toml
[app]
name = "MySuperApp"         # Final file name (e.g. MySuperApp.exe)
version = "1.0.0"           # Version (appears in File Properties)
description = "Description" # File description
copyright = "© 2025 Corp"   # Copyright
author = "Dev Name"         # Author

[window]
# Icon that appears in Windows Explorer and Taskbar.
# MUST BE A VALID .ICO (don't rename png).
icon = "app/frontend/icon.ico" 

[security]
# Enable opening in a Frontier app window for all pages.
allowed_internal = [
    "https://github.com/frontier-org/frontier/*"
]
# Enable opening in a Frontier app window for all pages.
allowed_browser = [
    "https://github.com/*"
]
```

---

## 🖥️ 3. Frontend & Window Management

Frontier treats HTML as the "window configuration". You control native window behavior using **Meta Tags** in the `<head>`.

### Available Settings (Meta Tags)

| Meta Name | Example Value | Description |
| :--- | :--- | :--- |
| `frontier-title` | `My App` | Window Title (Or use `<title>` tag). |
| `frontier-width` | `800` | Initial width. |
| `frontier-height` | `600` | Initial height. |
| `frontier-min-width`| `400` | Minimum allowed width. |
| `frontier-min-height`| `300` | Minimum allowed height. |
| `frontier-max-width`| `1920` | Maximum allowed width. |
| `frontier-max-height`| `1080` | Maximum allowed height. |
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

Frontier supports opening multiple windows. You can open secondary windows (popups) and configure them independently.

#### Method 1: Simple Window Open
```javascript
// Opens popup.html in a new native window with default configuration
window.ipc.postMessage('open|popup.html');
```

#### Method 2: Full Control with Frontier.spawn()
```javascript
Frontier.spawn('popup.html', {
    title: 'Settings Window',
    id: 'settings_window',
    width: 500,
    height: 400,
    min_width: 300,
    min_height: 300,
    max_width: 800,
    max_height: 600,
    
    // Security: Each window can have its own whitelist
    ignore_global_security: false,
    allowed_internal: ['https://api.example.com/*'],
    
    resizable: true,
    maximizable: true,
    minimizable: true,
    maximized: false,
    persistent: false,
    
    x: '(screen_w - win_w) / 2',
    y: '(screen_h - win_h) / 2'
});
```

#### Method 3: HTML Link with target="_blank"
```html
<!-- Opens new window with default config -->
<a href="frontier://app/popup.html" target="_blank">Open Settings</a>
```

#### Rules for Multiple Windows
- Each window has its own **DOM**, **CSS**, and **JavaScript context**
- Each window can have **independent security whitelists** via `allowed_internal` and `allowed_browser`
- CSS from parent window does **NOT** cascade to child windows
- Each window needs its own `<style>` or `<link rel="stylesheet">` if styling is required
- Window state is persisted independently based on `frontier-id` meta tag

#### Creating a Separate Window File
Create `app/frontend/popup.html`:
```html
<!DOCTYPE html>
<html>
<head>
    <title>Settings</title>
    <meta name="frontier-width" content="500">
    <meta name="frontier-height" content="400">
    <meta name="frontier-id" content="settings_window">
    <style>
        * { margin: 0; padding: 0; }
        body { font-family: Arial; background: #f0f0f0; padding: 20px; }
        h1 { color: #333; }
    </style>
</head>
<body>
    <h1>Settings</h1>
    <p>Independent window content</p>
    <script src="frontier-api.js"></script>
</body>
</html>
```

---

## 🧱 4. Backend Implementation

Place your files in `app/backend/`. Frontier detects the extension and looks up the corresponding module.

### Supported Backend Types

1.  **Single File (`script.py`, `code.c`)**
    *   Frontier uses the file name as the trigger.
    *   Ex: `app/backend/analyze.py` -> Trigger: `analyze`.

### Arguments
Everything you pass in JS (`window.ipc.postMessage('trigger|arg1 arg2')`) is forwarded to the binary/script as command-line arguments (`argv`).

---

## 📦 5. Module Creation (`modules/`)

A module teaches Frontier how to compile or run a language.
Create a folder in `modules/module_name/` and add a `manifest.toml`.

### `manifest.toml` Reference

```toml
name = "Readable Name"  # Module name
version = "1.0.0"       # Version
extension = "py"        # Extension this module controls

# (Optional) Interpreter to run the final file
# Use this for script languages (Python, JS, Bat) or Bytecode (Java)
interpreter = "python" 

# (Optional) If true, hides the black console window when running
suppress_window = true

# DEV CONFIGURATION (Hot Reload)
[dev]
# "interpreter": Does nothing on save, just runs. (Python, JS)
# "build": Runs the [build] command every time the file is saved. (C, Go, Rust)
strategy = "interpreter"

# BUILD CONFIGURATION (Production and Dev "Build Strategy")
[build]
# Variables:
# %IN%  -> Path of source file (or project folder)
# %OUT% -> Path where Frontier expects the final file
command = "gcc %IN% -o %OUT%"
```

### Practical Examples

**Python (Script):**
```toml
name = "Mod Python"
version = "1.0.0"
extension = "py"
interpreter = "python"

[dev]
strategy = "interpreter"
```

**C (Native):**
```toml
name = "Mod GCC"
version = "1.0.0"
extension = "c"
suppress_window = true

[dev]
strategy = "build"

[build]
command = "gcc %IN% -o %OUT%"
```

**Batch (Script):**
```toml
name = "Mod Batch"
version = "1.0.0"
extension = "bat"
interpreter = "cmd /c"

[dev]
strategy = "interpreter"
```

---

## 🌐 6. URL Routing & Security

Frontier provides multiple mechanisms to control how URLs are handled, preventing unwanted opens and ensuring security.

### URL Categories

Frontier automatically categorizes all URLs into four types:

| Category | Behavior | Examples |
| :--- | :--- | :--- |
| **Frontier** | Opens new Frontier windows | `frontier://popup.html`, `https://frontier.*` |
| **Internal** | Loads inside current window | URLs in `allowed_internal` whitelist |
| **Browser** | Opens in system browser | URLs in `allowed_browser` whitelist |
| **Blocked** | Rejected | All other URLs (security default) |

### Security Configuration (HTML Meta Tags)

Whitelist patterns work with two modes:

**Exact URL Match** (no wildcard - restricts to exact path):
```html
<!-- Allows ONLY https://mysite.com and https://mysite.com/ -->
<meta name="frontier-allowed-internal" content="https://mysite.com">
```

**Wildcard Match** (with `/*` - allows subpaths):
```html
<!-- Allows https://api.example.com/users, /posts, /auth/login, etc. -->
<meta name="frontier-allowed-internal" content="https://api.example.com/*">
```

Complete example with multiple patterns:
```html
<!-- Multiple patterns separated by commas -->
<meta name="frontier-allowed-internal" content="https://mysite.com,https://api.example.com/*,https://docs.example.com">

<!-- Whitelist for browser opens -->
<meta name="frontier-allowed-browser" content="https://github.com,https://docs.example.com/*">

<!-- Per-window override (ignores global security if true) -->
<meta name="frontier-ignore-global-security" content="false">
```

**Pattern Matching Rules:**
| Pattern | Matches | Blocks |
| :--- | :--- | :--- |
| `https://mysite.com` | `https://mysite.com`, `https://mysite.com/` | `https://mysite.com/page`, `https://mysite.com/api` |
| `https://mysite.com/*` | `https://mysite.com/page`, `https://mysite.com/api/users` | `https://othersite.com/`, `https://mysite.co` |
| `https://api.*/users` | `https://api.example.com/users`, `https://api.test.com/users` | `https://api.example.com/users/123` |

### URL Deduplication

When users click links that open external URLs (via `target="_blank"` or JavaScript), Frontier prevents duplicate tabs by:

1. **Normalizing URLs**: Removes query parameters (`?locale=en-US`) and fragments (`#section`)
2. **Atomic Locks**: Ensures only one thread can open a URL at a time
3. **Temporal Caching**: Ignores repeated opens of the same base URL within 2 seconds

This prevents issues like:
- Redirect chains opening multiple tabs (e.g., GitHub's automatic locale redirect)
- Concurrent handlers firing simultaneously
- Accidental double-clicks creating duplicate windows

**Example Scenario:**
```
User clicks: https://github.com/
GitHub redirects to: https://github.com/?locale=en-US
Result: One tab opens (not two) because both URLs normalize to the same base
```

### Programmatic Window Opening (JavaScript)

Use `Frontier.spawn()` to open new Frontier windows with full control:

```javascript
Frontier.spawn('https://github.com/frontier-org', {
    title: 'Frontier Org',
    id: 'github_window',
    icon: 'favicon.ico',
    
    // Security
    ignore_global_security: false, 
    allowed_internal: ['https://github.com/frontier-org/*'], 
    allowed_browser: ['https://github.com'], 
    
    // Size constraints
    width: 1200,
    height: 600,
    min_width: 800,
    min_height: 600,
    max_width: 1400,
    max_height: 800,
    
    // Behavior
    resizable: true,
    maximizable: false,
    minimizable: false,
    maximized: false,
    persistent: false,
    
    // Position
    x: '(screen_w - win_w) / 2',
    y: '(screen_h - win_h) / 2'
});
```

---

## 💻 7. CLI (Command Line)

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
*   **`.\back [command]`**
    *   Executes the specified command within the app/backend directory.
    *   Used for managing server-side logic, database migrations, or API configurations.
    *   Examples: .\back install, .\back migrate, or .\back test.
    *   Streamlines workflow by eliminating the need to manually cd into the backend folder.
*   **`.\front [command]`**
    *   Executes the specified command within the app/frontend directory.
    *   Used for managing the UI, installing frontend dependencies, or running linters.
    *   Examples: .\front add [package], .\front lint, or .\front tailwind.
    *   Ensures isolation between the client-side environment and the rest of the stack.

---

## 🛡️ 8. Technical Notes

1.  **Persistence:** Window data (and cookies/localstorage) are saved in `%LOCALAPPDATA%\AppName`.

## 🚧 Known Boundaries

* Static builds currently only support Windows.
* Hot reload latency for heavy backend modules.