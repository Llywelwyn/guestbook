mod config;
mod entries;
mod render;
mod telegram;
mod web;

use std::sync::Arc;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let config = config::Config::from_env().expect("failed to load config");
    let listen = config.listen_addr();
    let entries_dir = config.data_dir.join("entries");
    let chat_id = ChatId(config.telegram_chat_id);

    std::fs::create_dir_all(&entries_dir).ok();

    let bot = Bot::new(&config.telegram_bot_token);

    let (tx, rx) = tokio::sync::mpsc::channel(32);

    let state = Arc::new(web::AppState { config, tx });
    let app = web::router(state);

    // Spawn telegram notification sender
    let notify_bot = bot.clone();
    tokio::spawn(telegram::notification_task(notify_bot, chat_id, rx));

    // Spawn telegram command listener
    let cmd_bot = bot.clone();
    let cmd_entries_dir = entries_dir.clone();
    tokio::spawn(telegram::bot_task(cmd_bot, chat_id, cmd_entries_dir));

    // Run web server
    tracing::info!("listening on {listen}");
    let listener = tokio::net::TcpListener::bind(&listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
