//! 进程监控器 — WebSocket 服务端 + 双向心跳 + 死亡检测 + 有序退出
//!
//! 职责：
//! 1. 启动 WebSocket 服务端，监听 127.0.0.1（仅本地回环，安全）
//! 2. 生成认证 token，ui 进程连接时必须携带
//! 3. spawn ui 进程（wc-ui.exe），传入端口和 token
//! 4. 双向心跳：双方互发 Ping/Pong
//! 5. 连续 N 次未收到心跳 → 判定对端死亡 → 保存状态 → 有序退出
//! 6. 收到 ShutdownInitiated → 通知 ui → 等待超时 → 强制退出

use crate::config_manager::ConfigManager;
use crate::event_bus::EventBus;
use crate::module::Module;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use wc_common::{Event, EventPayload, PeerKind};

/// 进程监控器
pub struct ProcessSupervisor {
    config: Arc<parking_lot::RwLock<wc_common::AppConfig>>,
    ws_port: Mutex<Option<u16>>,
    ws_token: Mutex<String>,
    ui_process: Mutex<Option<std::process::Child>>,
}

impl ProcessSupervisor {
    pub fn new(config_manager: &ConfigManager) -> Self {
        Self {
            config: config_manager.config_arc(),
            ws_port: Mutex::new(None),
            ws_token: Mutex::new(String::new()),
            ui_process: Mutex::new(None),
        }
    }

    /// 生成随机认证 token
    fn generate_token() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 获取当前 WebSocket 端口
    pub fn ws_port(&self) -> Option<u16> {
        *self.ws_port.lock()
    }

    /// 获取当前认证 token
    pub fn ws_token(&self) -> String {
        self.ws_token.lock().clone()
    }

    /// 启动 WebSocket 服务端并 spawn UI 进程
    pub async fn start(&self, bus: &EventBus) -> Result<(), String> {
        let cfg = self.config.read().clone();
        let requested_port = cfg.adaptation_layer.ws_port;

        let addr: SocketAddr = format!("127.0.0.1:{}", requested_port).parse().unwrap();
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind WebSocket port: {}", e))?;
        let actual_port = listener.local_addr().unwrap().port();

        let token = if cfg.adaptation_layer.ws_token.is_empty() {
            Self::generate_token()
        } else {
            cfg.adaptation_layer.ws_token.clone()
        };

        *self.ws_port.lock() = Some(actual_port);
        *self.ws_token.lock() = token.clone();

        tracing::info!("WebSocket server listening on 127.0.0.1:{}", actual_port);

        self.spawn_ui_process(actual_port, &token);

        let bus_clone = bus.clone();
        let token_clone = token.clone();
        let config_clone = self.config.clone();

        tokio::spawn(async move {
            if let Ok((stream, peer)) = listener.accept().await {
                tracing::info!("UI connected from {}", peer);

                let ws_stream = tokio_tungstenite::accept_async(stream)
                    .await
                    .expect("WebSocket handshake failed");

                let (mut ws_sink, mut ws_stream) = ws_stream.split();

                // 认证：第一条消息必须是 token
                let auth_ok = if let Some(Ok(msg)) = ws_stream.next().await {
                    if let Message::Text(text) = msg {
                        text.trim() == token_clone
                    } else {
                        false
                    }
                } else {
                    false
                };

                if !auth_ok {
                    tracing::error!("UI authentication failed");
                    let _ = ws_sink.send(Message::Close(None)).await;
                    return;
                }
                tracing::info!("UI authenticated");

                // 通道：bus -> ws_sink（发送事件给 UI）
                let (bus_to_ws_tx, mut bus_to_ws_rx) = mpsc::channel::<Event>(256);

                // 订阅 bus 事件，转发给 WebSocket
                let bus_for_forward = bus_clone.clone();
                let bus_to_ws_tx_clone = bus_to_ws_tx.clone();
                tokio::spawn(async move {
                    let (_id, mut rx) = bus_for_forward.subscribe();
                    while let Some(event) = rx.recv().await {
                        // 只转发 source=Core 的事件给 UI（避免回环）
                        if event.source == PeerKind::Core {
                            if bus_to_ws_tx_clone.send(event).await.is_err() {
                                break;
                            }
                        }
                    }
                });

                let heartbeat_interval = config_clone.read().process.heartbeat_interval_ms;
                let death_threshold = config_clone.read().process.death_threshold_misses;
                let mut missed_pongs: u32 = 0;
                let mut ping_seq: u64 = 0;

                let mut heartbeat_tick =
                    tokio::time::interval(Duration::from_millis(heartbeat_interval));

                loop {
                    tokio::select! {
                        _ = heartbeat_tick.tick() => {
                            ping_seq += 1;
                            let ping_event = Event::new(
                                PeerKind::Core,
                                EventPayload::Ping { seq: ping_seq },
                            );
                            if let Ok(json) = serde_json::to_string(&ping_event) {
                                if ws_sink.send(Message::Text(json.into())).await.is_err() {
                                    tracing::error!("Failed to send Ping, UI disconnected");
                                    break;
                                }
                            }
                            missed_pongs += 1;
                            tracing::trace!("Sent Ping seq={}, missed_pongs={}", ping_seq, missed_pongs);

                            // 检查死亡
                            if missed_pongs >= death_threshold {
                                tracing::error!(
                                    "UI heartbeat missed {} times, declaring dead",
                                    missed_pongs
                                );
                                Self::handle_peer_death(&bus_clone, PeerKind::Ui).await;
                                break;
                            }
                        }

                        // 从 bus 接收事件，转发给 WebSocket
                        Some(event) = bus_to_ws_rx.recv() => {
                            if let Ok(json) = serde_json::to_string(&event) {
                                if ws_sink.send(Message::Text(json.into())).await.is_err() {
                                    tracing::error!("WebSocket send failed");
                                    break;
                                }
                            }
                        }

                        // 从 WebSocket 接收 UI 的事件
                        msg = ws_stream.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    if let Ok(event) = serde_json::from_str::<Event>(&text) {
                                        match &event.payload {
                                            EventPayload::Pong { seq } => {
                                                missed_pongs = 0;
                                                tracing::trace!("Received Pong seq={}", seq);
                                            }
                                            EventPayload::Ping { seq } => {
                                                // UI 发的 Ping，回 Pong
                                                let pong = Event::new(
                                                    PeerKind::Core,
                                                    EventPayload::Pong { seq: *seq },
                                                );
                                                if let Ok(json) = serde_json::to_string(&pong) {
                                                    let _ = ws_sink.send(Message::Text(json.into())).await;
                                                }
                                            }
                                            _ => {
                                                // UI 发来的其他事件，发布到 bus
                                                tracing::debug!(
                                                    "Received event from UI: {:?}",
                                                    event.payload
                                                );
                                                bus_clone.publish(event).await;
                                            }
                                        }
                                    }
                                }
                                Some(Ok(Message::Close(_))) => {
                                    tracing::info!("UI closed WebSocket connection");
                                    break;
                                }
                                Some(Err(e)) => {
                                    tracing::error!("WebSocket error: {}", e);
                                    break;
                                }
                                None => {
                                    tracing::info!("UI WebSocket stream ended");
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                tracing::info!("UI connection handler ended, initiating shutdown");
                // UI 连接断开（崩溃或正常关闭）→ 按设计 core 也有序退出
                Self::handle_peer_death(&bus_clone, PeerKind::Ui).await;
            } else {
                tracing::error!("Failed to accept UI connection");
            }
        });

        Ok(())
    }

    /// spawn UI 进程
    fn spawn_ui_process(&self, port: u16, token: &str) {
        // 候选路径：1) 同目录  2) 新TFM输出  3) 旧TFM输出
        let candidates: Vec<std::path::PathBuf> = std::env::current_exe().ok().map(|exe| {
            let dir = exe.parent().unwrap().to_path_buf();
            let wpf_bin = dir.join("..").join("..").join("wc-ui-wpf").join("bin").join("Debug");
            vec![
                dir.join("WcUiWpf.exe"),
                wpf_bin.join("net8.0-windows10.0.19041.0").join("WcUiWpf.exe"),
                wpf_bin.join("net8.0-windows").join("WcUiWpf.exe"),
            ]
        }).unwrap_or_default();

        let ui_exe = candidates.iter().find(|p| p.exists()).cloned();

        if let Some(ui_exe) = ui_exe {
            match Command::new(&ui_exe)
                .arg("--port")
                .arg(port.to_string())
                .arg("--token")
                .arg(token)
                .spawn()
            {
                Ok(child) => {
                    tracing::info!("Spawned UI process: {}", ui_exe.display());
                    *self.ui_process.lock() = Some(child);
                }
                Err(e) => {
                    tracing::error!("Failed to spawn UI process: {}", e);
                }
            }
        } else {
            tracing::warn!("UI executable not found in any candidate path");
        }
    }

    /// 处理对端死亡：保存状态 → 发事件 → 有序退出
    async fn handle_peer_death(bus: &EventBus, peer: PeerKind) {
        tracing::error!("Peer {:?} died, initiating orderly shutdown", peer);

        // 发布 PeerDied 事件
        bus.publish(Event::new(
            PeerKind::Core,
            EventPayload::PeerDied { peer },
        ))
        .await;

        // 保存窗口状态（第一版：写一个占位文件，后续模块会填充真实数据）
        Self::save_window_state();

        // 发布 ShutdownInitiated
        bus.publish(Event::new(
            PeerKind::Core,
            EventPayload::ShutdownInitiated {
                reason: format!("Peer {:?} died", peer),
            },
        ))
        .await;

        // 等待各模块清理，然后退出进程
        tokio::spawn(async {
            let shutdown_timeout = 2000; // 从配置读，第一版写死
            tokio::time::sleep(Duration::from_millis(shutdown_timeout)).await;
            tracing::info!("Shutdown timeout reached, exiting");
            std::process::exit(0);
        });
    }

    /// 保存窗口状态到文件（崩溃恢复用）
    fn save_window_state() {
        let state_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("wallpaper-connecter");
        if !state_dir.exists() {
            let _ = std::fs::create_dir_all(&state_dir);
        }
        let state_path = state_dir.join("window-state.json");

        // 第一版：写占位状态，后续 WindowManager 会填充真实数据
        let placeholder = serde_json::json!({
            "saved_at": chrono::Utc::now().to_rfc3339(),
            "reason": "peer_died",
            "windows": []
        });

        match serde_json::to_string_pretty(&placeholder) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&state_path, content) {
                    tracing::error!("Failed to save window state: {}", e);
                } else {
                    tracing::info!("Window state saved to {}", state_path.display());
                }
            }
            Err(e) => {
                tracing::error!("Failed to serialize window state: {}", e);
            }
        }
    }

    /// 发起有序关闭（用户主动退出时调用）
    pub async fn initiate_shutdown(&self, bus: &EventBus, reason: &str) {
        bus.publish(Event::new(
            PeerKind::Core,
            EventPayload::ShutdownInitiated {
                reason: reason.to_string(),
            },
        ))
        .await;

        // 等待 UI 进程退出
        if let Some(mut child) = self.ui_process.lock().take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
        std::process::exit(0);
    }
}

#[async_trait]
impl Module for ProcessSupervisor {
    fn name(&self) -> &str {
        "ProcessSupervisor"
    }

    async fn on_start(&mut self, bus: &EventBus) {
        if let Err(e) = self.start(bus).await {
            tracing::error!("Failed to start ProcessSupervisor: {}", e);
        }
    }

    async fn on_event(&mut self, event: &Event) {
        match &event.payload {
            EventPayload::ShutdownInitiated { reason } => {
                tracing::info!("Shutdown initiated: {}", reason);
            }
            _ => {}
        }
    }
}
