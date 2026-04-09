use axum::{
    extract::State,
    http::header,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::entries::{self, Entry, EntryMeta, Status};
use crate::render::{self, FORM_HTML, STYLE_CSS};

pub struct AppState {
    pub config: Config,
    pub tx: tokio::sync::mpsc::Sender<Entry>,
}

#[derive(Deserialize)]
pub struct SubmitForm {
    name: String,
    #[serde(default)]
    website: String,
    message: String,
    #[serde(default)]
    url: String, // honeypot
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/submit", post(submit))
        .route("/style.css", get(style))
        .with_state(state)
}

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let entries_dir = state.config.data_dir.join("entries");
    let entries = entries::read_approved(&entries_dir);
    let form = if state.config.open_registration { FORM_HTML } else { "" };
    let html = render::render_page(
        &state.config.site_title,
        &state.config.site_url,
        &entries,
        form,
    );
    Html(html)
}

async fn submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SubmitForm>,
) -> Html<String> {
    if !state.config.open_registration {
        return Html("Submissions are closed.".to_string());
    }

    // Honeypot check — silently discard
    if state.config.honeypot && !form.url.is_empty() {
        return Html("Thanks! Your message is pending approval.".to_string());
    }

    // Validation
    let name = form.name.trim().to_string();
    let message = form.message.trim().to_string();
    let website = form.website.trim().to_string();

    if name.is_empty() || message.is_empty() {
        return Html("Name and message are required.".to_string());
    }
    let max_name = state.config.max_name_length;
    if max_name > 0 && name.len() > max_name {
        return Html(format!("Name is too long (max {max_name} chars)."));
    }
    let max_web = state.config.max_website_length;
    if max_web > 0 && website.len() > max_web {
        return Html(format!("Website is too long (max {max_web} chars)."));
    }
    let max_msg = state.config.max_message_length;
    if max_msg > 0 && message.len() > max_msg {
        return Html(format!("Message is too long (max {max_msg} chars)."));
    }

    let short_id = &Uuid::new_v4().to_string()[..8];
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let filename = format!("{date}-{short_id}.txt");

    let entry = Entry {
        id: filename.trim_end_matches(".txt").to_string(),
        meta: EntryMeta {
            name,
            date,
            website,
            status: Status::Pending,
        },
        body: message,
    };

    // Write to disk
    let entries_dir = state.config.data_dir.join("entries");
    std::fs::create_dir_all(&entries_dir).ok();
    let path = entries_dir.join(&filename);
    if let Err(e) = std::fs::write(&path, entry.to_file_contents()) {
        tracing::error!("failed to write entry: {e}");
        return Html("Something went wrong. Please try again.".to_string());
    }

    // Notify telegram task
    let _ = state.tx.send(entry).await;

    Html("Thanks! Your message is pending approval.".to_string())
}

async fn style() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], STYLE_CSS)
}
