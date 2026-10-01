//! 进程内事件总线
//!
//! 模块间不直接调用，全部通过 EventBus 发布/订阅事件解耦。
//! 发布者调用 [`EventBus::publish`]，订阅者通过 [`EventBus::subscribe`] 获取接收端。
//!
//! 设计为异步、无锁（用 parking_lot 快速锁保护订阅者列表），
//! 发布操作不会阻塞订阅者（mpsc 有界通道，满了丢弃最旧事件并告警）。

use crate::module::Module;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use wc_common::Event;

/// 每个订阅者的通道容量
const CHANNEL_CAPACITY: usize = 1024;

/// 事件总线
#[derive(Clone)]
pub struct EventBus {
    inner: Arc<Mutex<EventBusInner>>,
}

struct EventBusInner {
    subscribers: HashMap<String, mpsc::Sender<Event>>,
    next_id: u64,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(EventBusInner {
                subscribers: HashMap::new(),
                next_id: 0,
            })),
        }
    }

    /// 订阅所有事件，返回接收端和订阅 ID
    pub fn subscribe(&self) -> (String, mpsc::Receiver<Event>) {
        let mut inner = self.inner.lock();
        let id = format!("sub_{}", inner.next_id);
        inner.next_id += 1;
        let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        inner.subscribers.insert(id.clone(), tx);
        drop(inner);
        tracing::debug!("New subscriber: {}", id);
        (id, rx)
    }

    /// 取消订阅
    pub fn unsubscribe(&self, id: &str) {
        let mut inner = self.inner.lock();
        inner.subscribers.remove(id);
        drop(inner);
        tracing::debug!("Unsubscribed: {}", id);
    }

    /// 发布事件给所有订阅者
    ///
    /// 如果某个订阅者的通道满了，丢弃该事件并告警（不阻塞发布者）。
    pub async fn publish(&self, event: Event) {
        let event_type = format!("{:?}", event.payload);
        let subscribers: Vec<mpsc::Sender<Event>> = {
            let inner = self.inner.lock();
            inner.subscribers.values().cloned().collect()
        };

        for tx in subscribers {
            if let Err(_) = tx.try_send(event.clone()) {
                tracing::warn!(
                    "Subscriber channel full or closed, dropping event: {}",
                    event_type
                );
            }
        }
    }

    /// 当前订阅者数量
    pub fn subscriber_count(&self) -> usize {
        self.inner.lock().subscribers.len()
    }

    /// 注册一个模块到事件总线（模块自动订阅事件）
    ///
    /// 模块会在后台 task 中运行，接收事件并调用 `on_event`。
    pub fn register_module<M: Module + 'static>(&self, mut module: M) {
        let (id, mut rx) = self.subscribe();
        let bus = self.clone();
        tokio::spawn(async move {
            module.on_start(&bus).await;
            while let Some(event) = rx.recv().await {
                module.on_event(&event).await;
            }
            module.on_stop().await;
            tracing::debug!("Module task ended: {}", id);
        });
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc_common::{Event, EventPayload, PeerKind};

    fn make_event() -> Event {
        Event::new(PeerKind::Core, EventPayload::Ping { seq: 1 })
    }

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let (_id, mut rx) = bus.subscribe();
        bus.publish(make_event()).await;
        let received = rx.recv().await.unwrap();
        assert!(matches!(received.payload, EventPayload::Ping { seq: 1 }));
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let (_id1, mut rx1) = bus.subscribe();
        let (_id2, mut rx2) = bus.subscribe();
        bus.publish(make_event()).await;
        assert!(rx1.recv().await.is_some());
        assert!(rx2.recv().await.is_some());
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let bus = EventBus::new();
        let (id, mut rx) = bus.subscribe();
        bus.unsubscribe(&id);
        bus.publish(make_event()).await;
        // 通道已关闭，应该收到 None
        assert!(rx.recv().await.is_none());
    }
}
