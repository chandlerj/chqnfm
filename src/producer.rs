use std::time::Duration;
use crate::{metadata::TrackInfo, state::{AppState, CHUNK_SIZE}};
use log::{info, error};
const INTERVAL: Duration = Duration::from_millis(100);

pub async fn run(state: AppState) {
    loop {
        let path = loop {
            if let Some(p) = state.queue.lock().await.pop_front() {
                break p;
            }
            state.notify.notified().await;
        };

        let info = TrackInfo::read(&path);
        state.meta_tx.send_replace(info.ok());

        match tokio::fs::read(&path).await {
            Ok(data) => {
                for chunk in data.chunks(CHUNK_SIZE) {
                    let _ = state.tx.send(bytes::Bytes::copy_from_slice(chunk));
                    tokio::time::sleep(INTERVAL).await;
                }
            }
            Err(e) => eprintln!("Failed to read {:?}: {e}", path),
        }
    }
}
