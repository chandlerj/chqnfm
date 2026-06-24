use std::{collections::VecDeque, path::PathBuf, sync::Arc};
use bytes::Bytes;
use tokio::sync::{broadcast, watch, Mutex, Notify};
use crate::metadata::TrackInfo;

pub const CHUNK_SIZE: u32 = 3000;
pub const CHANNEL_CAPACITY: usize = 128;

#[derive(Clone, Debug)]
pub struct AppState {
    pub tx:         Arc<broadcast::Sender<Bytes>>,
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
}
