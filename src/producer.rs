use std::time::Duration;
use crate::{
    state::{AppState, CHUNK_SIZE},
    metadata::TrackInfo
};
use tokio::sync::oneshot;
use log::{info, error};

const INTERVAL: Duration = Duration::from_millis(500);

pub async fn run(state: AppState) {
    let mut next_song: Option<(TrackInfo, oneshot::Receiver<Vec<u8>>)> = None;

    let mut tick = tokio::time::Instant::now();
    loop {

        let track = loop {
            if let Some(p) = state.queue.lock().await.pop_front() {
                break p;
            }
            else {
                info!("the queue is empty")
            }
            state.notify.notified().await;
        };

        let data = match next_song.take() {
            Some((pt, rx)) if pt.path == track.path => {
                info!("prefetch hit for {:?}", track.path);
                match rx.await {
                    Ok(d) => d,
                    Err(e) => {
                        error!("Failed to read from prefetch: {e}");
                        match tokio::fs::read(&track.path).await {
                            Ok(t) => t,
                            Err(e) => {
                                error!("Failed to read {:?}: {e}", track);
                                break;
                            }
                        }
                    }
                }
            }
            _ => {
                info!("prefetch miss, reading fresh");
                match tokio::fs::read(&track.path).await{
                Ok(t) => t,
                Err(e) => { 
                    error!("Failed to read {:?}: {e}", track);
                    break;
                },
            }}
        };

        info!("now playing: {} - {}: {}", 
            track.title,
            track.artist,
            track.album,
        );

        state.meta_tx.send_replace(Some(track.clone()));

        // prefetch next song
        if let Some(next) = state.queue.lock().await.front().cloned() {
            let (tx, rx) = oneshot::channel();
            let path = next.path.clone();
            info!("caching the next song in queue: {} - {}: {}", next.title, next.artist, next.album);
            tokio::spawn(async move {
                if let Ok(data) = tokio::fs::read(&path).await {
                    let _ = tx.send(data);
                }
            });
            next_song = Some((next, rx));
        }

        let track_bitrate = match &track.bitrate {
            Ok(d) => *d as u32,
            Err(e) => {
                error!("An error occurred trying to read the bitrate of the current song using default chunk size of {}: {}", CHUNK_SIZE, e);
                CHUNK_SIZE as u32
            }
        };
        let chunk_size = ((track_bitrate * 1000 / 8) * INTERVAL.as_millis() as u32 / 1000) as usize;
        let stripped = strip_id3(&data);
        for chunk in stripped.chunks(chunk_size) {
            let _ = state.tx.send(bytes::Bytes::copy_from_slice(chunk));
            tick += INTERVAL;
            tokio::time::sleep_until(tick).await;
        }
    }
}

// disclosure: this was written by AI (Claude 4.6 Sonnet: Medium [thinking]) to fix gaps
// between tracks caused by reading the id3 tags at the start and end of songs. the bytestream
// would be interrupted by the metadata embedded in the file... so we pull it out
fn strip_id3(data: &[u8]) -> &[u8] {
    let mut start = 0;
    let end = data.len();

    if data.len() > 10 && &data[0..3] == b"ID3" {
        let size = ((data[6] as usize & 0x7F) << 21)
            | ((data[7] as usize & 0x7F) << 14)
            | ((data[8] as usize & 0x7F) << 7)
            | (data[9] as usize & 0x7F);
        start = 10 + size;
    }

    let end = if end >= 128 && &data[end - 128..end - 125] == b"TAG" {
        end - 128
    } else {
        end
    };

    &data[start..end]
}
