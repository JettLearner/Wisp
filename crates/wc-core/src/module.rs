//! 模块 trait — 所有功能模块的统一接口
//!
//! 模块实现 [`Module`] 后，通过 [`EventBus::register_module`] 注册，
//! 自动订阅事件并在后台 task 中运行。
//!
//! 生命周期：`on_start` → 循环接收事件 `on_event` → `on_stop`

use async_trait::async_trait;
use wc_common::Event;

use crate::event_bus::EventBus;

/// 模块 trait
#[async_trait]
pub trait Module: Send + Sync {
    /// 模块名称（用于日志和调试）
    fn name(&self) -> &str;

    /// 模块启动时调用（一次性初始化）
    async fn on_start(&mut self, _bus: &EventBus) {}

    /// 收到事件时调用
    async fn on_event(&mut self, _event: &Event) {}

    /// 模块停止时调用（清理资源）
    async fn on_stop(&mut self) {}
}
