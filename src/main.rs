mod config;
mod entries;
mod render;
#[cfg(feature = "telegram")]
mod telegram;
mod web;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let config = config::Config::from_env().expect("failed to load config");
    let listen = config.listen_addr();
    let entries_dir = config.data_dir.join("entries");

    std::fs::create_dir_all(&entries_dir).expect("failed to create entries directory");

    let (tx, _rx) = tokio::sync::mpsc::channel::<(entries::Entry, Option<Vec<u8>>, Option<Vec<u8>>)>(32);

    #[cfg(feature = "telegram")]
    {
        use teloxide::prelude::*;
        match (&config.telegram_bot_token, config.telegram_chat_id) {
            (Some(token), Some(chat_id)) => {
                let chat_id = ChatId(chat_id);
                let bot = Bot::new(token);

                let notify_bot = bot.clone();
                let retry_interval = config.telegram_retry_interval;
                let retry_limit = config.telegram_retry_limit;
                tokio::spawn(telegram::notification_task(notify_bot, chat_id, _rx, retry_interval, retry_limit));

                let reminder_interval = config.telegram_reminder_interval;
                if reminder_interval > 0 {
                    let reminder_bot = bot.clone();
                    let reminder_data_dir = config.data_dir.clone();
                    tokio::spawn(telegram::reminder_task(reminder_bot, chat_id, reminder_data_dir, reminder_interval));
                }

                let cmd_data_dir = config.data_dir.clone();
                tokio::spawn(telegram::bot_task(bot, chat_id, cmd_data_dir));
            }
            _ => {
                tracing::info!("telegram not configured, moderation notifications disabled");
            }
        }
    }

    #[cfg(not(feature = "telegram"))]
    tracing::info!("compiled without telegram support");

    let state = Arc::new(web::AppState { config, tx });
    let app = web::router(state);

    // Run web server
    tracing::info!("listening on {listen}");
    let listener = tokio::net::TcpListener::bind(&listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
