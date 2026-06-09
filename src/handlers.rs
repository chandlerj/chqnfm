use std::{convert::Infallible, path::PathBuf};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::{Event, KeepAlive, Sse}, IntoResponse, Response},
    Json,
};
use axum::body::Body;
use bytes::Bytes;
use serde::Deserialize;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use crate::{metadata::TrackInfo, state::AppState};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title:  String,
    artist: String,
    album:  String,
}

pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let info = state.meta_rx.borrow().clone().unwrap();
    let title = info.title.clone().unwrap_or("unknown title".into());
    let artist = info.artist.clone().unwrap_or("unknown artist".into());
    let album = info.album.clone().unwrap_or("unknown album".into());

    IndexTemplate {
        title,
        artist,
        album
    }
}

pub async fn stream(State(state): State<AppState>) -> Response {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|r| r.ok().map(Ok::<Bytes, Infallible>));

    Response::builder()
        .header("Content-Type", "audio/mpeg")
        .header("Cache-Control", "no-cache")
        .header("Transfer-Encoding", "chunked")
        .header("Access-Control-Allow-Origin", "*")
        .header("X-Content-Type-Options", "nosniff")
        .body(Body::from_stream(stream))
        .unwrap()
}

pub async fn now_playing(State(state): State<AppState>) -> Json<Option<TrackInfo>> {
    Json(state.meta_rx.borrow().clone())
}

pub async fn metadata_stream(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();

    tokio::spawn(async move {
        let mut watch_rx = state.meta_tx.subscribe();

        // Send whatever is currently playing before waiting for changes.
        if let Some(info) = watch_rx.borrow_and_update().clone() {
            let data = serde_json::to_string(&info).unwrap_or_default();
            if tx.send(Ok(Event::default().event("track").data(data))).is_err() {
                return;
            }
        }

        while watch_rx.changed().await.is_ok() {
            if let Some(info) = watch_rx.borrow_and_update().clone() {
                let data = serde_json::to_string(&info).unwrap_or_default();
                if tx.send(Ok(Event::default().event("track").data(data))).is_err() {
                    return; // Client disconnected.
                }
            }
        }
    });

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn get_queue(State(state): State<AppState>) -> Json<Vec<String>> {
    let queue = state.queue.lock().await;
    let paths = queue.iter()
        .filter_map(|p| p.to_str())
        .map(String::from)
        .collect();
    Json(paths)
}

#[derive(Deserialize)]
pub struct AddTrack {
    pub path: String,
}

pub async fn add_track(
    State(state): State<AppState>,
    Json(body): Json<AddTrack>,
) -> StatusCode {
    let tracks = crate::playlist::expand(PathBuf::from(body.path)).await;
    if tracks.is_empty() {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }
    state.queue.lock().await.extend(tracks);
    state.notify.notify_one();
    StatusCode::OK
}

pub async fn skip_track(State(state): State<AppState>) -> StatusCode {
    state.queue.lock().await.pop_front();
    StatusCode::OK
}
