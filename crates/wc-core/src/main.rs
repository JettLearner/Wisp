#![windows_subsystem = "windows"]
//! wc-core — Wallpaper Connecter 核心守护进程
//!
//! 职责：
//! - 加载配置
//! - 启动事件总线
//! - 启动 WebSocket 服务端（供 UI 进程连接）
//! - spawn UI 进程
//! - 双向心跳监控，对端死亡时有序退出
//! - 后续模块（壁纸采样、桌面控制、AI 适配层）在此注册

mod config_manager;
mod event_bus;
mod module;
mod process_supervisor;
mod wallpaper_sampler;

use config_manager::ConfigManager;
use event_bus::EventBus;
use process_supervisor::ProcessSupervisor;
use std::sync::Arc;
use wallpaper_sampler::WallpaperSampler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wc_core=info,wc_common=info".into()),
        )
        .with_target(false)
        .init();

    tracing::info!("========================================");
    tracing::info!("  Wallpaper Connecter Core v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("========================================");

    let config_manager = Arc::new(ConfigManager::new());
    tracing::info!("Config loaded from {}", config_manager.config_path().display());

    let bus = EventBus::new();

    let supervisor = ProcessSupervisor::new(&config_manager);
    bus.register_module(supervisor);



    let wallpaper_sampler = WallpaperSampler::new();
    bus.register_module(wallpaper_sampler);

    tracing::info!("Core running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl_c");

    tracing::info!("Ctrl+C received, shutting down...");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    tracing::info!("Core exited.");
}
