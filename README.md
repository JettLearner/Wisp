# Wisp

Windows desktop floating tool — floating ball, music widget, transparent AI app windows.

> Author: **周璟琦-Jett** | License: **GPL-3.0** | Version: **v1.0.0**

---

## Features

### Music Widget
- Integrates Windows SMTC (System Media Transport Controls) for real-time playback info
- Shows: album art, track title, artist, lyrics, progress bar
- Controls: play/pause, next, previous
- Lyrics fetched from NetEase Cloud Music public API, parsed from LRC, synced to playback position
- Fully transparent background; text color auto-switches between black/white based on background brightness

### Floating Ball
- Docks to screen top; slides out after 1.5s hover, retracts after 2s away
- Click = open music widget
- Right-click menu: toggle transparency / restore transparency / music / settings / exit
- Visible on all Windows virtual desktops

### Transparent AI App Windows
- Direct Windows API (`SetWindowLong` + `SetLayeredWindowAttributes`) — no DLL injection, no debug mode, no restart needed
- Supported apps: Doubao, VSCode, Gemini, Qwen, CodeBuddy, Ollama, DeepSeek, Kimi, Tencent Yuanbao, and more
- One-click toggle from right-click menu

### Settings
- Clear log cache
- About: version, author, license

---

## Requirements

- **Windows 10 2004+** (requires `WS_EX_LAYERED` extended style)
- **.NET 8 Desktop Runtime** (required for framework-dependent build; self-contained build bundles it)
- Network connection (for lyric search)

---

## Installation

### Option 1: Download installer (recommended)

Go to [Releases](https://github.com/JettLearner/Wisp/releases):

| File | Size | Description |
|------|------|-------------|
| `Wisp-Setup.exe` | ~66 MB | Self-contained, bundles .NET runtime |
| `Wisp-Setup-LooseFiles.exe` | ~7 MB | Framework-dependent, requires .NET 8 |

Install wizard includes: GPL-3.0 license confirmation, custom install path, shortcuts, auto-close running instance.

### Option 2: Build from source

#### Prerequisites
- Rust 1.80+ (MSVC toolchain)
- .NET 8 SDK
- Visual Studio Build Tools (C++ desktop workload)

#### Build

```bash
# 1. Build Rust core
cargo build --release -p wc-core

# 2. Build WPF UI (self-contained single file)
cd wc-ui-wpf
dotnet publish -c Release -r win-x64 --self-contained true /p:PublishSingleFile=true /p:EnableCompressionInSingleFile=true

# 3. Build installer
cd ../installer/WcInstaller
dotnet publish -c Release
```

---

## Project Structure

```
Wisp/
├── crates/
│   ├── wc-common/          # Shared types: events, config, rich-text AST, app rules
│   └── wc-core/            # Rust daemon: event bus, WebSocket, heartbeat, wallpaper sampling
├── wc-ui-wpf/             # C# WPF UI
│   ├── Helpers/           # Acrylic, config, i18n, transparency, WebSocket client
│   └── Windows/          # Floating ball, music widget, settings
└── installer/
    └── WcInstaller/       # WPF install wizard
        └── Pages/         # 9 wizard pages
```

### Dual-process architecture
- **wc-core.exe** (Rust): daemon, starts WebSocket server, spawns UI process
- **WcUiWpf.exe** (C# WPF): UI process, connects via WebSocket, renders all windows
- Bidirectional heartbeat; either process saves state and exits gracefully if the other dies

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Daemon | Rust + tokio + tungstenite (WebSocket) |
| UI | C# WPF + .NET 8 |
| Window effects | Windows API (SetWindowCompositionAttribute / SetLayeredWindowAttributes) |
| Media | Windows SMTC |
| Lyrics | NetEase Cloud Music public API + LRC parser |
| Installer | Custom WPF wizard |

---

## Development

```bash
# Rust check
cargo check --workspace

# Run core (dev mode)
cargo run -p wc-core

# WPF dev
cd wc-ui-wpf
dotnet build
```

---

## License

[GPL-3.0-or-later](LICENSE)

---

<p align="center">
  Made with ❤️ by 周璟琦-Jett
</p>
