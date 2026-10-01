//! 配置结构 — 对应 `%APPDATA%\wallpaper-connecter\config.json`
//!
//! 所有字段都有默认值，配置文件缺失时使用默认。
//! 设置面板修改后通过 ConfigManager 持久化。

use serde::{Deserialize, Serialize};

/// 顶层配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub floating_ball: FloatingBallConfig,
    pub chat_window: ChatWindowConfig,
    pub wallpaper_sampler: WallpaperSamplerConfig,
    pub desktop_control: DesktopControlConfig,
    pub adaptation_layer: AdaptationLayerConfig,
    pub music_dock: MusicDockConfig,
    pub process: ProcessConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            floating_ball: FloatingBallConfig::default(),
            chat_window: ChatWindowConfig::default(),
            wallpaper_sampler: WallpaperSamplerConfig::default(),
            desktop_control: DesktopControlConfig::default(),
            adaptation_layer: AdaptationLayerConfig::default(),
            music_dock: MusicDockConfig::default(),
            process: ProcessConfig::default(),
        }
    }
}

// ── 通用 ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub auto_start: bool,
    pub language: String,
    pub theme_mode: String,
    pub log_level: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            language: "zh-CN".into(),
            theme_mode: "wallpaper-adaptive".into(),
            log_level: "info".into(),
        }
    }
}

// ── 悬浮球 ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloatingBallConfig {
    pub enabled: bool,
    pub diameter: u32,
    pub position_x: String,
    pub position_y: i32,
    pub trigger_height: u32,
    pub reveal_delay_ms: u64,
    pub hide_delay_ms: u64,
    pub opacity_normal: f32,
    pub opacity_hover: f32,
    pub custom_icon: Option<String>,
}

impl Default for FloatingBallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            diameter: 56,
            position_x: "center".into(),
            position_y: 0,
            trigger_height: 8,
            reveal_delay_ms: 1500,
            hide_delay_ms: 2000,
            opacity_normal: 0.85,
            opacity_hover: 1.0,
            custom_icon: None,
        }
    }
}

// ── 聊天窗 ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatWindowConfig {
    pub default_width: u32,
    pub default_height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub corner_radius: u32,
    pub backdrop_blur: u32,
    pub window_opacity: f32,
    pub gradient_opacity: f32,
    pub gradient_angle: u32,
    pub session_bar_width: u32,
    pub session_bar_collapsed: bool,
    pub ai_tab_height: u32,
    pub virtual_scroll_threshold: usize,
    pub restore_windows_on_close: bool,
}

impl Default for ChatWindowConfig {
    fn default() -> Self {
        Self {
            default_width: 900,
            default_height: 600,
            min_width: 600,
            min_height: 400,
            corner_radius: 24,
            backdrop_blur: 20,
            window_opacity: 0.85,
            gradient_opacity: 0.15,
            gradient_angle: 135,
            session_bar_width: 200,
            session_bar_collapsed: false,
            ai_tab_height: 28,
            virtual_scroll_threshold: 50,
            restore_windows_on_close: true,
        }
    }
}

// ── 壁纸采样 ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallpaperSamplerConfig {
    pub enabled: bool,
    pub method: String,
    pub dynamic_detect_interval_ms: u64,
    pub dynamic_detect_threshold_percent: f32,
    pub dynamic_sample_interval_ms: u64,
    pub color_diff_threshold_delta_e: f32,
    pub sample_delay_after_drag_ms: u64,
    pub cache_size: usize,
    pub edge_expand_percent: u32,
    pub min_brightness: f32,
    pub max_brightness: f32,
    pub fallback_to_system_accent: bool,
}

impl Default for WallpaperSamplerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: "screenshot_exclude_capture".into(),
            dynamic_detect_interval_ms: 1000,
            dynamic_detect_threshold_percent: 5.0,
            dynamic_sample_interval_ms: 3000,
            color_diff_threshold_delta_e: 8.0,
            sample_delay_after_drag_ms: 200,
            cache_size: 10,
            edge_expand_percent: 10,
            min_brightness: 0.15,
            max_brightness: 0.90,
            fallback_to_system_accent: true,
        }
    }
}

// ── 桌面控制 ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesktopControlConfig {
    pub toggle_desktop_on_ball_click: bool,
    pub hide_icons_on_ball_click: bool,
    pub restore_icons_on_chat_close: bool,
}

impl Default for DesktopControlConfig {
    fn default() -> Self {
        Self {
            toggle_desktop_on_ball_click: true,
            hide_icons_on_ball_click: true,
            restore_icons_on_chat_close: true,
        }
    }
}

// ── AI 适配层 ───────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptationLayerConfig {
    pub adapters_dir: String,
    pub poll_interval_ms: u64,
    pub session_refresh_interval_ms: u64,
    pub allow_shell_exec: bool,
    pub http_allow_remote: bool,
    pub ws_port: u16,
    pub ws_token: String,
}

impl Default for AdaptationLayerConfig {
    fn default() -> Self {
        Self {
            adapters_dir: "%APPDATA%/wallpaper-connecter/adapters".into(),
            poll_interval_ms: 2000,
            session_refresh_interval_ms: 5000,
            allow_shell_exec: false,
            http_allow_remote: false,
            ws_port: 0,
            ws_token: String::new(),
        }
    }
}

// ── 音乐窗 ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicDockConfig {
    pub enabled: bool,
    pub width: u32,
    pub height: u32,
    pub corner_radius: u32,
    pub window_opacity: f32,
    pub edge_hide: bool,
    pub edge_hide_delay_ms: u64,
}

impl Default for MusicDockConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            width: 280,
            height: 72,
            corner_radius: 18,
            window_opacity: 0.80,
            edge_hide: false,
            edge_hide_delay_ms: 3000,
        }
    }
}

// ── 进程管理 ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub heartbeat_interval_ms: u64,
    pub death_threshold_misses: u32,
    pub shutdown_timeout_ms: u64,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_ms: 500,
            death_threshold_misses: 3,
            shutdown_timeout_ms: 2000,
        }
    }
}
