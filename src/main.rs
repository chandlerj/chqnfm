mod handlers;
mod metadata;
mod playlist;
mod producer;
mod state;

use std::path::PathBuf;
use axum::{Router, routing::{delete, get}};
use state::AppState;
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    const BROADCAST_IP: &str = "0.0.0.0:3000";
    let state = AppState::new();

    // CLI testing: seed the queue with a path (audio file or .m3u playlist) if provided.
    if let Some(path) = std::env::args().nth(1) {
        let tracks = playlist::expand(PathBuf::from(path)).await;
        state.queue.lock().await.extend(tracks);
        state.notify.notify_one();
    }

    tokio::spawn(producer::run(state.clone()));

    info!("the current queue is: {:#?}", state.get_queue().await);

    let app = Router::new()
        .route("/",               get(handlers::index))
        .route("/stream",         get(handlers::stream))
        .route("/now-playing",    get(handlers::now_playing))
        .route("/metadata/stream",get(handlers::metadata_stream))
        .route("/queue",          get(handlers::get_queue).post(handlers::add_track))
        .route("/queue/front",    delete(handlers::skip_track))
        .with_state(state);

    info!("server is listening on {}", BROADCAST_IP);
    let listener = match tokio::net::TcpListener::bind(BROADCAST_IP).await {
        Ok(x) => x,
        Err(e) => { panic!("Could not bind to server to address {}:\n {}", BROADCAST_IP,e) }
    };
    axum::serve(listener, app).await.unwrap();
}
