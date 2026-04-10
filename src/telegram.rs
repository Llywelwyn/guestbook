use std::path::PathBuf;

use teloxide::prelude::*;
use teloxide::types::ParseMode;
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
            entry.meta.name, entry.meta.date, preview, ellipsis, entry.id
        ));
    }
    lines.join("\n")
}

/// Escape special characters for Telegram MarkdownV2.
fn escape_md(s: &str) -> String {
    let special = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if special.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Format a bot command, escaping underscores for MarkdownV2.
fn cmd(name: &str, id: &str) -> String {
    format!("/{}\\_{}", name, id)
}

/// Format an entry as a Telegram message with bold headers and contextual commands.
fn format_entry_message(entry: &Entry) -> String {
    let mut parts = Vec::new();

    parts.push(format!("*Name*\n{}", escape_md(&entry.meta.name)));

    if !entry.meta.website.is_empty() {
        parts.push(format!("*Website*\n{}", escape_md(&entry.meta.website)));
    }

    parts.push(format!("*Message*\n{}", escape_md(&entry.body)));

    // Attached media commands
    let has_drawing = !entry.meta.drawing.is_empty();
    let has_voice = !entry.meta.voice_note.is_empty();
    if has_drawing || has_voice {
        let mut attached = vec!["*Attached*".to_string()];
        if has_drawing {
            attached.push(cmd("drawing", &entry.id));
        }
        if has_voice {
            attached.push(cmd("voice\\_note", &entry.id));
        }
        parts.push(attached.join("\n"));
    }

    // Moderation section with status and contextual commands
    let status_text = match entry.meta.status {
        Status::Pending => "Currently pending\\.",
        Status::Approved => "Currently approved\\.",
        Status::Denied => "Currently denied\\.",
    };
    let commands = match entry.meta.status {
        Status::Pending => format!("{}\n{}", cmd("allow", &entry.id), cmd("deny", &entry.id)),
        Status::Approved => format!("{}\n{}", cmd("deny", &entry.id), cmd("reply", &entry.id)),
        Status::Denied => format!("{}\n{}", cmd("allow", &entry.id), cmd("delete", &entry.id)),
    };
    parts.push(format!("*Moderation*\n{status_text}\n\n{commands}"));

    parts.join("\n\n")
}

/// Send a formatted message with Markdown parsing.
async fn send_md(bot: &Bot, chat_id: ChatId, text: &str) -> Result<Message, teloxide::RequestError> {
    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::MarkdownV2)
        .await
}

/// Send a notification about a new entry, retrying on failure.
async fn notify(bot: &Bot, chat_id: ChatId, entry: &Entry, retry_interval: u64, retry_limit: u32) {
    let text = format_entry_message(entry);
    if send_md(bot, chat_id, &text).await.is_ok() {
        return;
    }
    tracing::warn!("failed to send notification for entry {}, spawning retry task", entry.id);
    let bot = bot.clone();
    let id = entry.id.clone();
    let text = text.clone();
    tokio::spawn(async move {
        for attempt in 1..=retry_limit {
            tokio::time::sleep(std::time::Duration::from_secs(retry_interval)).await;
            tracing::info!("retry {attempt}/{retry_limit} for entry {id}");
            match send_md(&bot, chat_id, &text).await {
                Ok(_) => {
                    tracing::info!("retry succeeded for entry {id}");
                    return;
                }
                Err(e) => {
                    tracing::warn!("retry {attempt}/{retry_limit} failed for entry {id}: {e}");
                }
            }
        }
        tracing::error!("all {retry_limit} retries exhausted for entry {id}");
    });
}

/// Listen for new entries on the channel and send Telegram notifications.
pub async fn notification_task(
    bot: Bot,
    chat_id: ChatId,
    mut rx: Receiver<(Entry, Option<Vec<u8>>, Option<Vec<u8>>)>,
    retry_interval: u64,
    retry_limit: u32,
) {
    while let Some((entry, _drawing_bytes, _voice_bytes)) = rx.recv().await {
        notify(&bot, chat_id, &entry, retry_interval, retry_limit).await;
    }
}

/// Periodically check for pending entries and send a reminder.
pub async fn reminder_task(bot: Bot, chat_id: ChatId, data_dir: PathBuf, interval_secs: u64) {
    let entries_dir = data_dir.join("entries");
    loop {
        let pending = entries::read_by_status(&entries_dir, Status::Pending);
        if !pending.is_empty() {
            let text = format!("📬 *Pending reminder*\n\n{}", escape_md(&format_entry_list(&pending, "pending")));
            if let Err(e) = send_md(&bot, chat_id, &text).await {
                tracing::error!("failed to send pending reminder: {e}");
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
    }
}

/// Run the Telegram bot that listens for commands.
pub async fn bot_task(bot: Bot, chat_id: ChatId, data_dir: PathBuf) {
    let commands = vec![
        teloxide::types::BotCommand::new("pending", "List pending entries"),
        teloxide::types::BotCommand::new("approved", "List approved entries"),
        teloxide::types::BotCommand::new("denied", "List denied entries"),
    ];
    if let Err(e) = bot.set_my_commands(commands).await {
        tracing::error!("failed to set bot commands: {e}");
    }

    let handler = Update::filter_message().endpoint(
        |bot: Bot, msg: Message, data_dir: PathBuf, chat_id: ChatId| async move {
            let text = msg.text().unwrap_or("");
            if msg.chat.id != chat_id {
                return respond(());
            }

            let entries_dir = data_dir.join("entries");

            if let Some(id) = text.strip_prefix("/allow_") {
                match entries::set_status(&entries_dir, id, Status::Approved) {
                    Ok(name) => {
                        send_md(&bot, msg.chat.id, &format!("Approved \\({}\\)\\.\n{}", escape_md(&name), cmd("reply", id))).await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, e).await?;
                    }
                }
            } else if let Some(id) = text.strip_prefix("/deny_") {
                match entries::set_status(&entries_dir, id, Status::Denied) {
                    Ok(name) => {
                        send_md(&bot, msg.chat.id, &format!("Denied \\({}\\)\\.\n{}", escape_md(&name), cmd("delete", id))).await?;
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
            } else if let Some(id) = text.strip_prefix("/view_") {
                match entries::find_entry(&entries_dir, id) {
                    Ok(entry) => {
                        let text = format_entry_message(&entry);
                        send_md(&bot, msg.chat.id, &text).await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, e).await?;
                    }
                }
            } else if let Some(id) = text.strip_prefix("/drawing_") {
                match entries::find_entry(&entries_dir, id) {
                    Ok(entry) if !entry.meta.drawing.is_empty() => {
                        let drawing_path = data_dir.join("drawings").join(&entry.meta.drawing);
                        if let Ok(bytes) = std::fs::read(&drawing_path) {
                            bot.send_photo(
                                msg.chat.id,
                                teloxide::types::InputFile::memory(bytes).file_name("drawing.png"),
                            ).await?;
                        } else {
                            bot.send_message(msg.chat.id, "Drawing file not found.").await?;
                        }
                    }
                    Ok(_) => {
                        bot.send_message(msg.chat.id, "No drawing attached.").await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, e).await?;
                    }
                }
            } else if let Some(id) = text.strip_prefix("/voice_note_") {
                match entries::find_entry(&entries_dir, id) {
                    Ok(entry) if !entry.meta.voice_note.is_empty() => {
                        let vn_path = data_dir.join("voice_notes").join(&entry.meta.voice_note);
                        if let Ok(bytes) = std::fs::read(&vn_path) {
                            bot.send_voice(
                                msg.chat.id,
                                teloxide::types::InputFile::memory(bytes).file_name("voice_note.webm"),
                            ).await?;
                        } else {
                            bot.send_message(msg.chat.id, "Voice note file not found.").await?;
                        }
                    }
                    Ok(_) => {
                        bot.send_message(msg.chat.id, "No voice note attached.").await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, e).await?;
                    }
                }
            } else if let Some(rest) = text.strip_prefix("/reply_") {
                let (id, reply) = match rest.split_once('\n') {
                    Some((id, reply)) => (id.trim(), reply),
                    None => {
                        bot.send_message(msg.chat.id, "Usage: /reply_ID\nYour reply text").await?;
                        return respond(());
                    }
                };
                if reply.trim().is_empty() {
                    bot.send_message(msg.chat.id, "Reply text is empty.").await?;
                    return respond(());
                }
                match entries::append_reply(&entries_dir, id, reply) {
                    Ok(name) => {
                        bot.send_message(msg.chat.id, format!("Reply added to {name}'s entry."))
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, e).await?;
                    }
                }
            } else if let Some(id) = text.strip_prefix("/confirm_delete_") {
                match entries::delete_entry(&data_dir, id) {
                    Ok(name) => {
                        bot.send_message(msg.chat.id, format!("Deleted ({name}).")).await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, e).await?;
                    }
                }
            } else if let Some(id) = text.strip_prefix("/delete_") {
                match entries::find_entry(&entries_dir, id) {
                    Ok(entry) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("Delete {}'s entry? This cannot be undone.\n\n/confirm_delete_{}", entry.meta.name, entry.id),
                        ).await?;
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
        .dependencies(dptree::deps![data_dir, chat_id])
        .build()
        .dispatch()
        .await;
}
