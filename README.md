# Wisp

Windows 桌面悬浮工具 — 悬浮球入口、音乐悬浮窗、AI 软件窗口透明效果。

> 创作者：**周璟琦-Jett** | 协议：**GPL-3.0** | 版本：**v1.0.0**

---

## 功能

### 🎵 音乐悬浮窗
- 接入 Windows SMTC（系统媒体控制），自动获取正在播放的歌曲
- 显示：专辑封面、歌名、艺术家、歌词、进度条
- 控制：播放/暂停、上一首、下一首
- 歌词通过网易云音乐公开 API 搜索 LRC 文件，按播放进度同步高亮
- 背景全透明，文字颜色根据背景亮度自动切换黑白

### 🎱 悬浮球
- 屏幕顶部吸附，鼠标悬停 1.5 秒滑出，离开 2 秒收回
- 点击 = 打开音乐悬浮窗
- 右键菜单：开启透明效果 / 恢复透明度 / 音乐 / 设置 / 退出
- 所有 Windows 虚拟桌面均显示

### 🪟 AI 软件窗口透明效果
- 通过 Windows API（`SetWindowLong` + `SetLayeredWindowAttributes`）直接调透明度
- 无需 DLL 注入、无需调试模式、无需重启 AI 软件
- 支持：豆包、VSCode、Gemini、Qwen、CodeBuddy、Ollama、DeepSeek、Kimi、腾讯元宝 等
- 右键菜单一键开启/关闭

### ⚙️ 设置面板
- 清除日志缓存
- 关于：版本、创作者、开源协议

---

## 系统要求

- **Windows 10 2004+**（需要 `WS_EX_LAYERED` 扩展样式）
- **.NET 8 Desktop Runtime**（框架依赖版需要；单文件版已内嵌）
- 网络连接（歌词搜索用）

---

## 安装

### 方式一：下载安装包（推荐）

前往 [Releases](https://github.com/JettLearner/Wisp/releases) 下载：

| 文件 | 大小 | 说明 |
|------|------|------|
| `WallpaperConnecter-Setup.exe` | ~66 MB | 单文件版，内嵌 .NET 运行时，开箱即用 |
| `WallpaperConnecter-Setup-LooseFiles.exe` | ~7 MB | 松散文件版，需自行安装 .NET 8 |

安装向导包含：
1. GPL-3.0 许可协议确认
2. 自定义安装路径
3. 桌面/开始菜单快捷方式
4. 自动关闭运行中的旧版本

### 方式二：从源码编译

#### 前置要求
- Rust 1.80+（MSVC 工具链）
- .NET 8 SDK
- Visual Studio Build Tools（C++ 桌面开发）

#### 编译步骤

```bash
# 1. 编译 Rust 核心
cargo build --release -p wc-core

# 2. 编译 WPF UI（自包含单文件）
cd wc-ui-wpf
dotnet publish -c Release -r win-x64 --self-contained true /p:PublishSingleFile=true /p:EnableCompressionInSingleFile=true

# 3. 编译安装向导
cd ../installer/WcInstaller
dotnet publish -c Release
```

---

## 项目结构

```
Wisp/
├── crates/
│   ├── wc-common/          # 共享类型：事件、配置、富文本 AST、APP 规则
│   └── wc-core/            # Rust 守护进程：事件总线、WebSocket、心跳、壁纸取色
├── wc-ui-wpf/              # C# WPF 界面
│   ├── Helpers/            # Acrylic、配置、I18n、透明度、WebSocket 客户端
│   └── Windows/            # 悬浮球、音乐窗、设置窗
└── installer/
    └── WcInstaller/       # 安装向导（WPF）
        └── Pages/          # 9 个向导页面
```

### 双进程架构
- **wc-core.exe**（Rust）：守护进程，启动 WebSocket 服务，spawn UI 进程
- **WcUiWpf.exe**（C# WPF）：界面进程，连接 WebSocket，渲染所有窗口
- 双向心跳监控，任一进程死亡时另一方保存状态后有序退出

---

## 技术栈

| 层 | 技术 |
|----|------|
| 守护进程 | Rust + tokio + tungstenite（WebSocket） |
| 界面 | C# WPF + .NET 8 |
| 窗口效果 | Windows API（SetWindowCompositionAttribute / SetLayeredWindowAttributes） |
| 媒体集成 | Windows SMTC（System Media Transport Controls） |
| 歌词 | 网易云音乐公开 API + LRC 解析 |
| 安装 | 自研 WPF 安装向导 |

---

## 开发

```bash
# Rust 检查
cargo check --workspace

# 运行 core（开发模式）
cargo run -p wc-core

# WPF 开发
cd wc-ui-wpf
dotnet build
```

---

## License

[GPL-3.0-or-later](LICENSE) — 详见 [LICENSE](LICENSE) 文件。

---

<p align="center">
  用 ❤️ 创作 by 周璟琦-Jett
</p>