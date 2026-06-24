use std::time::Duration;
use crate::state::{AppState, CHUNK_SIZE};
const INTERVAL: Duration = Duration::from_millis(500);

use log::{info, error};

pub async fn run(state: AppState) {
    loop {
        let path = loop {
            if let Some(p) = state.queue.lock().await.pop_front() {
                break p;
            }
            state.notify.notified().await;
        };

        info!("now playing: {} - {}: {}", 
            path.title,
            path.artist,
            path.album,
        );
        state.meta_tx.send_replace(Some(path.clone()));
        let track_bitrate = match &path.bitrate {
            Ok(d) => *d as u32,
            Err(e) => {
                error!("An error occurred trying to read the bitrate of the current song using default chunk size of {}: {}", CHUNK_SIZE, e);
                CHUNK_SIZE as u32
            }
        };
        let chunk_size = ((track_bitrate * 1000 / 8) * INTERVAL.as_millis() as u32 / 1000) as usize;
        match tokio::fs::read(&path.path).await {
            Ok(data) => {
                let start = tokio::time::Instant::now();
                for (i, chunk) in data.chunks(chunk_size).enumerate() {
                    let _ = state.tx.send(bytes::Bytes::copy_from_slice(chunk));
                    let next_tick = start + INTERVAL * (i as u32 + 1);
                    tokio::time::sleep_until(next_tick).await;
                }
            }
            Err(e) => eprintln!("Failed to read {:?}: {e}", path),
        }
    }
}

