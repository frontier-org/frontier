# 📊 Development Logging System

**Frontier** has a comprehensive logging system that provides detailed visibility during development mode. All logs are **dead code** in production builds and will not appear in the final executable.

## Log Categories & Icons

### 📄 Window Management
```text
📄 [WINDOW] teste.html         → Local file window opened
📦 [SPAWN] https://example.com → New spawned window with config
```

### 📦 Asset Loading
```text
📦 [ASSET] index.html (text/html)      → File successfully loaded (with MIME type)
❌ [ASSET] Not found: missing.js        → File not found error
```

### 🌐 Browser Routing
```text
🌐 [BROWSER] Opening: https://github.com              → Opening in system browser
⏱️ [BROWSER] Deduped (within 2s): https://github.com → Deduplication prevented duplicate tab
```

### 📍 URL Routing & Security
```text
📍 [ROUTING] Frontier: frontier://app/teste.html            → Internal Frontier protocol
📍 [ROUTING] Internal (whitelisted): https://kaiohsg.dev   → Allowed internal navigation
📍 [ROUTING] Browser (whitelisted): https://github.com     → External browser URL
📍 [ROUTING] Blocked: https://suspicious.com              → Security-blocked URL
```

### 🚫 Security
```text
🚫 [SECURITY] Blocked access to: https://malicious.com → Access denied by whitelist
```

### 💬 IPC Communication
```text
💬 [IPC] open: teste.html           → Window.ipc.postMessage('open|teste.html')
💬 [IPC] spawn: popup.html          → Window.ipc.postMessage('spawn|popup.html')
💬 [IPC] exec: math 10 50           → Window.ipc.postMessage('math|10 50')
💬 [IPC] exec: (no args)            → Command with no arguments
```

## Example Development Session

```text
📄 [WINDOW] index.html
📦 [ASSET] index.html (text/html)
📦 [ASSET] style.css (text/css)
📦 [ASSET] frontier-api.js (application/javascript)

💬 [IPC] exec: math 10 50
🌐 [BROWSER] Opening: https://www.google.com
⏱️ [BROWSER] Deduped (within 2s): https://www.google.com

💬 [IPC] open: teste.html
📄 [WINDOW] teste.html
📦 [ASSET] teste.html (text/html)
📦 [ASSET] style.css (text/css)

📍 [ROUTING] Frontier: frontier://app/teste.html
📍 [ROUTING] Internal (whitelisted): https://kaiohsg.dev
📍 [ROUTING] Browser (whitelisted): https://github.com
🚫 [SECURITY] Blocked access to: https://malicious.com
```

## Log Behavior

### Development Mode (`frontier dev`)
✅ All logs are printed to the terminal in real-time
✅ Uses emoji prefixes for visual categorization
✅ Includes detailed information (file types, URLs, deduplication state)
✅ Non-intrusive - doesn't interfere with application functionality

### Production Mode (`frontier build` → `.exe`)
✅ All logs are **dead code** - completely removed during compilation
✅ Zero runtime overhead
✅ Executable runs silently without terminal output
✅ Same functionality, cleaner deployment

## Why This Matters

The logging system helps debug:
- **Asset loading issues** - See which files are requested and found/missing
- **URL routing problems** - Understand how URLs are categorized and processed
- **IPC communication** - Track commands sent from JavaScript to Rust
- **Security violations** - Identify blocked URLs and understand why
- **Browser integration** - Monitor external URL opens and deduplication
- **Window spawning** - See all window creation events and configuration

## Implementation Details

All logs use conditional compilation:
```rust
if sys_is_dev { eprintln!("📦 [ASSET] {} ({})", resource, mime); }
```

This pattern ensures:
1. **Zero overhead in production** - Conditions are evaluated at compile-time
2. **Optimized binaries** - Dead code elimination removes all logging statements
3. **Development transparency** - Full visibility during development

## Usage Tips

- Pipe output to a file: `frontier dev > development.log 2>&1`
- Filter by category: `frontier dev 2>&1 | grep "\[IPC\]"`
- Monitor in real-time: Keep the terminal visible while testing
- Check security issues: Search for `[ROUTING] Blocked` to find rejected URLs

Copyright (c) 2026 The Frontier Framework Authors  
SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception OR MIT