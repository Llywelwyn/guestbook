mod config;
mod entries;
mod render;
mod web;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::load("config.toml").expect("failed to load config.toml");
    let listen = config.listen.clone();

    let (tx, _rx) = tokio::sync::mpsc::channel(32);

    let state = Arc::new(web::AppState { config, tx });
    let app = web::router(state);

    tracing::info!("listening on {listen}");
    let listener = tokio::net::TcpListener::bind(&listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
