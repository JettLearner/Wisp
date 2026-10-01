//! wc-common — Wallpaper Connecter 共享类型库
//!
//! 包含跨进程共享的事件定义、配置结构、统一富文本 AST 与归一化框架。
//! core 和 ui 两个进程都依赖本 crate，确保事件 schema 和配置格式一致。

pub mod config;
pub mod events;
pub mod rich_text;

pub use config::AppConfig;
pub use events::{AiSourceInfo, AiStatus, Event, EventPayload, PeerKind};
