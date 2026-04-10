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

    std::fs::create_dir_all(&entries_dir).ok();

    let (tx, rx) = tokio::sync::mpsc::channel::<(entries::Entry, Option<Vec<u8>>, Option<Vec<u8>>)>(32);

    // Spawn telegram tasks if configured
    match (&config.telegram_bot_token, config.telegram_chat_id) {
        (Some(token), Some(chat_id)) => {
            let chat_id = ChatId(chat_id);
            let bot = Bot::new(token);

            let notify_bot = bot.clone();
            tokio::spawn(telegram::notification_task(notify_bot, chat_id, rx));

            let cmd_entries_dir = entries_dir.clone();
            tokio::spawn(telegram::bot_task(bot, chat_id, cmd_entries_dir));
        }
        _ => {
            tracing::info!("telegram not configured, moderation notifications disabled");
        }
    }

    let state = Arc::new(web::AppState { config, tx });
    let app = web::router(state);

    // Run web server
    tracing::info!("listening on {listen}");
    let listener = tokio::net::TcpListener::bind(&listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
