//! 配置管理器 — 加载、保存、热更新 config.json
//!
//! 配置文件路径：`%APPDATA%\wallpaper-connecter\config.json`
//! 首次启动时如果文件不存在，使用默认配置并写入。

use crate::event_bus::EventBus;
use crate::module::Module;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use wc_common::{AppConfig, Event, EventPayload, PeerKind};

/// 配置管理器
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// 创建配置管理器，从文件加载（不存在则用默认）
    pub fn new() -> Self {
        let config_dir = Self::config_dir();
        let config_path = config_dir.join("config.json");

        let config = if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                    Ok(cfg) => {
                        tracing::info!("Config loaded from {}", config_path.display());
                        cfg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse config, using default: {}", e);
                        AppConfig::default()
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to read config, using default: {}", e);
                    AppConfig::default()
                }
            }
        } else {
            tracing::info!("Config file not found, using default");
            let cfg = AppConfig::default();
            if let Err(e) = Self::save_config(&config_path, &cfg) {
                tracing::error!("Failed to write default config: {}", e);
            }
            cfg
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        }
    }

    /// 获取配置目录（不存在则创建）
    fn config_dir() -> PathBuf {
        let app_data = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wallpaper-connecter");
        if !app_data.exists() {
            if let Err(e) = std::fs::create_dir_all(&app_data) {
                tracing::error!("Failed to create config dir: {}", e);
            }
        }
        app_data
    }

    /// 保存配置到文件
    fn save_config(path: &PathBuf, config: &AppConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Serialize config failed: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("Write config failed: {}", e))?;
        Ok(())
    }

    /// 获取当前配置的克隆
    pub fn get(&self) -> AppConfig {
        self.config.read().clone()
    }

    /// 获取配置的 Arc 引用（只读）
    pub fn config_arc(&self) -> Arc<RwLock<AppConfig>> {
        self.config.clone()
    }

    /// 更新配置并持久化，返回变更的字段名列表
    pub fn update(&self, new_config: AppConfig) -> Vec<String> {
        let old = self.config.read().clone();
        let changed_fields = Self::diff_configs(&old, &new_config);

        if !changed_fields.is_empty() {
            *self.config.write() = new_config.clone();
            if let Err(e) = Self::save_config(&self.config_path, &new_config) {
                tracing::error!("Failed to persist config: {}", e);
            }
            tracing::info!("Config updated, changed fields: {:?}", changed_fields);
        }

        changed_fields
    }

    /// 简单对比两个配置，返回变更的顶层字段名
    fn diff_configs(old: &AppConfig, new: &AppConfig) -> Vec<String> {
        let mut changed = Vec::new();
        if old.general != new.general {
            changed.push("general".to_string());
        }
        if old.floating_ball != new.floating_ball {
            changed.push("floating_ball".to_string());
        }
        if old.chat_window != new.chat_window {
            changed.push("chat_window".to_string());
        }
        if old.wallpaper_sampler != new.wallpaper_sampler {
            changed.push("wallpaper_sampler".to_string());
        }
        if old.desktop_control != new.desktop_control {
            changed.push("desktop_control".to_string());
        }
        if old.adaptation_layer != new.adaptation_layer {
            changed.push("adaptation_layer".to_string());
        }
        if old.music_dock != new.music_dock {
            changed.push("music_dock".to_string());
        }
        if old.process != new.process {
            changed.push("process".to_string());
        }
        changed
    }

    /// 配置文件路径
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

// 配置管理器也实现 Module trait，可以注册到 EventBus
// （第一版只做基础加载保存，事件处理后续扩展）
#[async_trait]
impl Module for ConfigManager {
    fn name(&self) -> &str {
        "ConfigManager"
    }

    async fn on_event(&mut self, event: &Event) {
        if let EventPayload::ConfigChanged { .. } = &event.payload {
            tracing::debug!("ConfigChanged event received");
        }
    }
}
