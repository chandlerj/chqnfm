use std::time::Duration;
use crate::{metadata::TrackInfo, state::{AppState, CHUNK_SIZE}};
const INTERVAL: Duration = Duration::from_millis(100);

use log::{info, error};

pub async fn run(state: AppState) {
    loop {
        let path = loop {
            if let Some(p) = state.queue.lock().await.pop_front() {
                break p;
            }
            state.notify.notified().await;
        };

        let info = TrackInfo::read(&path).ok();
        info!("now playing: {} - {}: {}", 
            info.as_ref().unwrap().title,
            info.as_ref().unwrap().artist,
            info.as_ref().unwrap().album,
        );
        state.meta_tx.send_replace(info.clone());
        let track_bitrate = match &info.as_ref().unwrap().bitrate {
            Ok(d) => *d as u32,
            Err(e) => {
                error!("An error occurred trying to read the bitrate of the current song using default chunk size of {}: {}", CHUNK_SIZE, e);
                CHUNK_SIZE as u32
            }
        };
        let chunk_size = ((track_bitrate * 1000 / 8) * INTERVAL.as_millis() as u32 / 1000) as usize;
        match tokio::fs::read(&path).await {
            Ok(data) => {
                for chunk in data.chunks(chunk_size) {
                    let _ = state.tx.send(bytes::Bytes::copy_from_slice(chunk));
                    tokio::time::sleep(INTERVAL).await;
                }
            }
            Err(e) => eprintln!("Failed to read {:?}: {e}", path),
        }
    }
}

