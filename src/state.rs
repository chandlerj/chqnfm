use std::{collections::VecDeque, sync::Arc};
use bytes::Bytes;
use tokio::sync::{broadcast, watch, Mutex, Notify};
use crate::{
    metadata::{TrackInfo},
    songbank::Songbank
};

pub const CHUNK_SIZE: u32 = 3000;
pub const CHANNEL_CAPACITY: usize = 128;

#[derive(Clone, Debug)]
pub struct AppState {
    pub tx:         Arc<broadcast::Sender<Bytes>>,
    pub bank:       Arc<Mutex<Songbank>>,
    pub queue:      Arc<Mutex<VecDeque<TrackInfo>>>,
    pub notify:     Arc<Notify>,
    pub meta_tx:    Arc<watch::Sender<Option<TrackInfo>>>,
    pub meta_rx:    watch::Receiver<Option<TrackInfo>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (meta_tx, meta_rx) = watch::channel(None);
        Self {
            tx:      Arc::new(tx),
            bank:    Arc::new(Mutex::new(Songbank::new())),
            queue:   Arc::new(Mutex::new(VecDeque::new())),
            notify:  Arc::new(Notify::new()),
            meta_tx: Arc::new(meta_tx),
            meta_rx,
        }
    }
    
    pub async fn get_queue_str(&self) -> Vec<String> {
        self.queue
            .lock()
            .await
            .iter()
            .map(|p| format!("{} - {}: {}", p.title, p.artist, p.album))
            .collect()
    }

    pub async fn build_songbank(&mut self, path: String) {
        self.bank.lock().await.build_songbank(path).await
    }
}
