use std::path::PathBuf;

use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;

use crate::entries::{self, Entry, Status};

/// Send a notification to Telegram about a new entry.
async fn notify(bot: &Bot, chat_id: ChatId, entry: &Entry) {
    let short_id = entry.id.split('_').last().unwrap_or(&entry.id);
    let text = format!(
        "New guestbook entry:\n\nName: {}\nWebsite: {}\n\n{}\n\n/allow_{}\n/deny_{}",
        entry.meta.name, entry.meta.website, entry.body, short_id, short_id
    );
    if let Err(e) = bot.send_message(chat_id, &text).await {
        tracing::error!("failed to send telegram message: {e}");
    }
}

/// Listen for new entries on the channel and send Telegram notifications.
pub async fn notification_task(bot: Bot, chat_id: ChatId, mut rx: Receiver<(Entry, Option<Vec<u8>>, Option<Vec<u8>>)>) {
    while let Some((entry, drawing_bytes, voice_bytes)) = rx.recv().await {
        notify(&bot, chat_id, &entry).await;
        if let Some(bytes) = drawing_bytes {
            if let Err(e) = bot.send_photo(
                chat_id,
                teloxide::types::InputFile::memory(bytes).file_name("drawing.png"),
            ).await {
                tracing::error!("failed to send drawing photo: {e}");
            }
        }
        if let Some(bytes) = voice_bytes {
            if let Err(e) = bot.send_voice(
                chat_id,
                teloxide::types::InputFile::memory(bytes).file_name("voice_note.webm"),
            ).await {
                tracing::error!("failed to send voice note: {e}");
            }
        }
    }
}

/// Run the Telegram bot that listens for /allow_ and /deny_ commands.
pub async fn bot_task(bot: Bot, chat_id: ChatId, entries_dir: PathBuf) {
    let handler = Update::filter_message().endpoint(
        |bot: Bot, msg: Message, entries_dir: PathBuf, chat_id: ChatId| async move {
            let text = msg.text().unwrap_or("");
            // Only respond to the configured chat
            if msg.chat.id != chat_id {
                return respond(());
            }

            if let Some(id) = text.strip_prefix("/allow_") {
                match entries::set_status(&entries_dir, id, Status::Approved) {
                    Ok(name) => {
                        bot.send_message(msg.chat.id, format!("Approved ({name})."))
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, e).await?;
                    }
                }
            } else if let Some(id) = text.strip_prefix("/deny_") {
                match entries::set_status(&entries_dir, id, Status::Denied) {
                    Ok(name) => {
                        bot.send_message(msg.chat.id, format!("Denied ({name})."))
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, e).await?;
                    }
                }
            }

            Ok(())
        },
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![entries_dir, chat_id])
        .build()
        .dispatch()
        .await;
}
