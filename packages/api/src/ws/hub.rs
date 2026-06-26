use crate::models::ServerMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 64;

pub struct GameHub {
    channels: RwLock<HashMap<Uuid, broadcast::Sender<ServerMessage>>>,
}

impl GameHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            channels: RwLock::new(HashMap::new()),
        })
    }

    pub async fn subscribe(&self, game_id: Uuid) -> broadcast::Receiver<ServerMessage> {
        let mut channels = self.channels.write().await;
        let sender = channels
            .entry(game_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0);
        sender.subscribe()
    }

    pub async fn broadcast(&self, game_id: Uuid, msg: ServerMessage) {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&game_id) {
            if sender.receiver_count() > 0 {
                let _ = sender.send(msg);
            }
        }
    }

    pub async fn cleanup(&self, game_id: Uuid) {
        let mut channels = self.channels.write().await;
        if let Some(sender) = channels.get(&game_id) {
            if sender.receiver_count() == 0 {
                channels.remove(&game_id);
            }
        }
    }
}
