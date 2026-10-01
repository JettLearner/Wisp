//! 事件定义 — 进程内 EventBus 和跨进程 WebSocket 共用同一套 schema。
//!
//! 所有事件通过 [`Event`] 包装，载荷在 [`EventPayload`] 中。
//! 模块间不直接调用，全部通过事件解耦。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 进程身份
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerKind {
    /// 核心守护进程
    Core,
    /// 界面进程
    Ui,
}

impl PeerKind {
    pub fn opposite(&self) -> Self {
        match self {
            PeerKind::Core => PeerKind::Ui,
            PeerKind::Ui => PeerKind::Core,
        }
    }
}

/// 统一事件包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub source: PeerKind,
    pub payload: EventPayload,
}

impl Event {
    pub fn new(source: PeerKind, payload: EventPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            source,
            payload,
        }
    }
}

/// 事件载荷 — 所有模块间通信的事件在此枚举。
///
/// 设计原则：新增事件只加变体，不改已有变体的字段，保证向后兼容。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum EventPayload {
    // ── 悬浮球 ──────────────────────────────────────
    /// 悬浮球从隐藏态滑出
    BallRevealed { x: f64, y: f64 },
    /// 悬浮球被点击（触发 Win+D + 藏图标 + 展开聊天窗）
    BallClicked { x: f64, y: f64 },
    /// 悬浮球收回隐藏
    BallHidden,

    // ── 桌面控制 ────────────────────────────────────
    /// 已执行 ToggleDesktop（等同 Win+D，最小化所有窗口）
    DesktopShown,
    /// 桌面图标已隐藏
    IconsHidden,
    /// 桌面图标已恢复
    IconsRestored,

    // ── 聊天窗 ──────────────────────────────────────
    /// 聊天窗已打开
    ChatWindowOpened {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    /// 聊天窗位置/尺寸变化（拖动节流后发出）
    ChatWindowMoved {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    /// 聊天窗已关闭
    ChatWindowClosed,

    // ── 壁纸取色 / 主题 ─────────────────────────────
    /// 壁纸采样完成，产出渐变主题
    WallpaperSampled {
        primary_color: String,
        secondary_color: String,
        gradient_css: String,
        text_color: String,
    },
    /// 主题已变更（UI 应用新 CSS 变量后发出）
    ThemeChanged,

    // ── AI 适配层 ───────────────────────────────────
    /// 可用 AI 源列表变化（新启动/退出）
    AiListChanged { apps: Vec<AiSourceInfo> },
    /// 用户选中某个 AI
    AiSelected { app_id: String },
    /// 用户选中某个会话
    SessionSelected {
        app_id: String,
        session_id: String,
    },
    /// 对话状态更新（完整快照）
    ConversationStateUpdated {
        app_id: String,
        state: ConversationState,
    },
    /// 流式输出增量
    StreamingToken {
        app_id: String,
        session_id: String,
        delta: String,
    },
    /// 用户发送消息
    UserMessageSent {
        app_id: String,
        session_id: String,
        text: String,
    },
    /// 停止生成
    GenerationStopped { app_id: String },

    // ── 音乐窗 ──────────────────────────────────────
    /// 媒体播放状态变化
    MusicStateChanged {
        title: String,
        artist: String,
        album_art: Option<String>,
        is_playing: bool,
    },

    // ── 设置面板 ────────────────────────────────────
    /// 设置面板已打开
    SettingsOpened,
    /// 设置面板已关闭
    SettingsClosed,
    /// 设置面板脏状态变化（有未保存更改）
    SettingsDirtyChanged { dirty: bool },
    /// 配置已变更并持久化
    ConfigChanged { fields: Vec<String> },

    // ── 进程管理 ────────────────────────────────────
    /// 心跳 ping
    Ping { seq: u64 },
    /// 心跳 pong
    Pong { seq: u64 },
    /// 对端进程死亡（连续未收到心跳）
    PeerDied { peer: PeerKind },
    /// 发起有序关闭
    ShutdownInitiated { reason: String },
    /// 窗口状态已保存（崩溃恢复用）
    WindowStateSaved,
}

/// AI 源信息（分栏显示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSourceInfo {
    pub app_id: String,
    pub app_name: String,
    pub status: AiStatus,
    pub model: Option<String>,
}

/// AI 运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiStatus {
    /// 空闲
    Idle,
    /// 思考中
    Thinking,
    /// 流式输出中
    Streaming,
    /// 错误
    Error,
    /// 离线/未运行
    Offline,
}

/// 统一对话状态 schema — 所有 AI 适配脚本必须输出此格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub app_id: String,
    pub app_name: String,
    pub status: AiStatus,
    pub model: Option<String>,
    pub session_id: String,
    pub session_title: String,
    pub messages: Vec<ChatMessage>,
    pub tools_called: Vec<String>,
    pub error: Option<String>,
}

/// 单条聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: i64,
    pub status: MessageStatus,
    /// 流式增量（仅 status=streaming 时有值）
    #[serde(default)]
    pub streaming_delta: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Complete,
    Streaming,
    Error,
}
