use std::path::PathBuf;

use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;

use crate::entries::{self, Entry, Status};

fn format_entry_list(entries: &[Entry], status_label: &str) -> String {
    if entries.is_empty() {
        return format!("No {status_label} entries.");
    }
    let mut lines = vec![format!("{} {}:", entries.len(), status_label)];
    for entry in entries {
        let preview: String = entry.body.chars().take(30).collect();
        let ellipsis = if entry.body.chars().count() > 30 { "..." } else { "" };
        lines.push(format!(
            "- {} ({}) \"{}{}\"\n  /view_{}",
            entry.meta.name, entry.meta.date, preview, ellipsis, entry.short_id()
        ));
    }
    lines.join("\n")
}

/// Send a notification to Telegram about a new entry.
async fn notify(bot: &Bot, chat_id: ChatId, entry: &Entry) {
    let short_id = entry.short_id();
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
pub async fn bot_task(bot: Bot, chat_id: ChatId, data_dir: PathBuf) {
    let handler = Update::filter_message().endpoint(
        |bot: Bot, msg: Message, data_dir: PathBuf, chat_id: ChatId| async move {
            let text = msg.text().unwrap_or("");
            // Only respond to the configured chat
            if msg.chat.id != chat_id {
                return respond(());
            }

            let entries_dir = data_dir.join("entries");

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
            } else if text == "/pending" {
                let list = entries::read_by_status(&entries_dir, entries::Status::Pending);
                bot.send_message(msg.chat.id, format_entry_list(&list, "pending")).await?;
            } else if text == "/approved" {
                let list = entries::read_by_status(&entries_dir, entries::Status::Approved);
                bot.send_message(msg.chat.id, format_entry_list(&list, "approved")).await?;
            } else if text == "/denied" {
                let list = entries::read_by_status(&entries_dir, entries::Status::Denied);
                bot.send_message(msg.chat.id, format_entry_list(&list, "denied")).await?;
            }

            Ok(())
        },
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![data_dir, chat_id])
        .build()
        .dispatch()
        .await;
}
