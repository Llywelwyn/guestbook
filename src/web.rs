use axum::{
    extract::DefaultBodyLimit,
    extract::Path as AxumPath,
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use base64::Engine;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::entries::{self, Entry, EntryMeta, Status};
use crate::render::{self, DEFAULT_TEMPLATE, render_error_page, render_success_page};

pub struct AppState {
    pub config: Config,
    pub tx: tokio::sync::mpsc::Sender<(Entry, Option<Vec<u8>>)>,
}

#[derive(Deserialize)]
pub struct SubmitForm {
    name: String,
    #[serde(default)]
    website: String,
    message: String,
    #[serde(default)]
    url: String, // honeypot
    #[serde(default)]
    captcha: String,
    #[serde(default)]
    drawing: String,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/submit", post(submit))
        .route("/drawings/{filename}", get(serve_drawing))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .with_state(state)
}

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let entries_dir = state.config.data_dir.join("entries");
    let entries = entries::read_approved(&entries_dir);
    let form = if state.config.enable_submissions {
        render::render_form(&state.config)
    } else {
        String::new()
    };
    let template = state.config.template.as_deref().unwrap_or(DEFAULT_TEMPLATE);
    let html = render::render_page(
        template,
        &state.config,
        &entries,
        &form,
    );
    Html(html)
}

async fn serve_drawing(
    State(state): State<Arc<AppState>>,
    AxumPath(filename): AxumPath<String>,
) -> Response {
    // Validate filename: only safe chars + .png
    if !filename.ends_with(".png")
        || !filename[..filename.len() - 4]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = state.config.data_dir.join("drawings").join(&filename);
    match std::fs::read(&path) {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SubmitForm>,
) -> Html<String> {
    if !state.config.enable_submissions {
        return Html(render_error_page(&state.config, "Submissions are closed."));
    }

    // Honeypot check — silently discard
    if state.config.enable_honeypot && !form.url.is_empty() {
        return Html(render_success_page(&state.config));
    }

    // Validation
    let name = form.name.trim().to_string();
    let message = form.message.trim().to_string();
    let website = if state.config.enable_website_links {
        form.website.trim().to_string()
    } else {
        String::new()
    };

    // Captcha check
    if state.config.enable_captcha {
        let input = form.captcha.trim();
        let answer = &state.config.captcha_answer;
        let ok = if state.config.captcha_casesensitive {
            if state.config.captcha_exact {
                input == answer
            } else {
                input.contains(answer.as_str())
            }
        } else {
            let input_lower = input.to_lowercase();
            let answer_lower = answer.to_lowercase();
            if state.config.captcha_exact {
                input_lower == answer_lower
            } else {
                input_lower.contains(&answer_lower)
            }
        };
        if !ok {
            return Html(render_error_page(&state.config, "Wrong answer."));
        }
    }

    if name.is_empty() || message.is_empty() {
        return Html(render_error_page(&state.config, "Name and message are required."));
    }
    let max_name = state.config.max_name_length;
    if max_name > 0 && name.len() > max_name {
        return Html(render_error_page(&state.config, &format!("Name is too long (max {max_name} chars).")));
    }
    let max_web = state.config.max_website_length;
    if max_web > 0 && website.len() > max_web {
        return Html(render_error_page(&state.config, &format!("Website is too long (max {max_web} chars).")));
    }
    let max_msg = state.config.max_message_length;
    if max_msg > 0 && message.len() > max_msg {
        return Html(render_error_page(&state.config, &format!("Message is too long (max {max_msg} chars).")));
    }

    // Process drawing if enabled and provided
    let drawing_bytes: Option<Vec<u8>> = if state.config.enable_drawings && !form.drawing.is_empty() {
        let b64 = form.drawing
            .strip_prefix("data:image/png;base64,")
            .unwrap_or("");
        if b64.is_empty() {
            None
        } else {
            let bytes = match base64::engine::general_purpose::STANDARD.decode(b64) {
                Ok(b) => b,
                Err(_) => return Html(render_error_page(&state.config, "Invalid drawing data.")),
            };
            let max = state.config.max_drawing_bytes();
            if max > 0 && bytes.len() > max {
                return Html(render_error_page(&state.config, &format!("Drawing is too large (max {} bytes).", max)));
            }

            // Validate PNG: magic bytes + IHDR dimensions match configured canvas
            if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
                return Html(render_error_page(&state.config, "Invalid drawing format."));
            }
            let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
            let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
            if width != state.config.canvas_width || height != state.config.canvas_height {
                return Html(render_error_page(&state.config, "Invalid drawing dimensions."));
            }

            Some(bytes)
        }
    } else {
        None
    };

    let now = chrono::Utc::now();
    let epoch = now.timestamp();
    let short_id = &Uuid::new_v4().to_string()[..8];
    let prefix = format!("{epoch}_{short_id}");
    let date = now.format("%Y-%m-%dT%H:%M:%S").to_string();
    let filename = format!("{prefix}.txt");

    // Save drawing with the same prefix as the entry
    let drawing_filename = if let Some(ref bytes) = drawing_bytes {
        let drawing_name = format!("{prefix}.png");
        let drawings_dir = state.config.data_dir.join("drawings");
        std::fs::create_dir_all(&drawings_dir).ok();
        if let Err(e) = std::fs::write(drawings_dir.join(&drawing_name), bytes) {
            tracing::error!("failed to write drawing: {e}");
            return Html(render_error_page(&state.config, "Something went wrong. Please try again."));
        }
        drawing_name
    } else {
        String::new()
    };
    let entry = Entry {
        id: filename.trim_end_matches(".txt").to_string(),
        meta: EntryMeta {
            name,
            date,
            website,
            drawing: drawing_filename,
            voice_note: String::new(),
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
        return Html(render_error_page(&state.config, "Something went wrong. Please try again."));
    }

    // Notify telegram task
    let _ = state.tx.send((entry, drawing_bytes)).await;

    Html(render_success_page(&state.config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use base64::Engine;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_config(dir: &std::path::Path) -> Config {
        Config {
            port: 0,
            data_dir: dir.to_path_buf(),
            site_title: "test".into(),

            telegram_bot_token: None,
            telegram_chat_id: None,
            enable_honeypot: true,
            max_name_length: 0,
            max_message_length: 0,
            max_website_length: 0,
            enable_submissions: true,
            enable_website_links: true,
            enable_html_injection: false,
            enable_captcha: false,
            captcha_question: String::new(),
            captcha_answer: String::new(),
            captcha_exact: false,
            captcha_casesensitive: false,
            enable_drawings: false,
            label_drawing: "Draw (optional):".into(),
            canvas_width: 400,
            canvas_height: 200,
            enable_voice_notes: false,
            label_voice_note: "Voice note (optional):".into(),
            voice_note_max_duration: 20,
            template: None,
            success_template: None,
            separator: "---".into(),
            style: String::new(),
            form_prompt: "Thanks for visiting. Sign the guestbook!".into(),
            button_text: "sign".into(),
            label_name: "Your name:".into(),
            label_website: "Your website (optional):".into(),
            label_message: "Your message:".into(),
            textarea_width: 400,
            textarea_height: 150,
        }
    }

    fn test_app(config: Config) -> (Router, tokio::sync::mpsc::Receiver<(Entry, Option<Vec<u8>>)>) {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let state = Arc::new(AppState { config, tx });
        (router(state), rx)
    }

    async fn post_form(app: &Router, body: &str) -> (StatusCode, String) {
        let req = Request::builder()
            .method("POST")
            .uri("/submit")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(body.to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    async fn get_index(app: &Router) -> String {
        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn test_enable_submissions_shows_form() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);
        let html = get_index(&app).await;
        assert!(html.contains("action=\"/submit\""));
    }

    #[tokio::test]
    async fn test_closed_registration_hides_form() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_submissions = false;
        let (app, _rx) = test_app(config);
        let html = get_index(&app).await;
        assert!(!html.contains("action=\"/submit\""));
    }

    #[tokio::test]
    async fn test_closed_registration_rejects_submit() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_submissions = false;
        let (app, _rx) = test_app(config);
        let (status, body) = post_form(&app, "name=test&message=hello").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Submissions are closed"));
    }

    #[tokio::test]
    async fn test_honeypot_discards() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=bot&message=spam&url=http://spam.com").await;
        assert!(body.contains("Thanks!"));
        // No entry file should exist
        let entries: Vec<_> = std::fs::read_dir(dir.path().join("entries"))
            .into_iter()
            .flatten()
            .collect();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_honeypot_disabled_allows_url_field() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_honeypot = false;
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=user&message=hello&url=http://mysite.com").await;
        assert!(body.contains("pending approval"));
        let count = std::fs::read_dir(dir.path().join("entries"))
            .unwrap()
            .count();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_max_name_length() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.max_name_length = 5;
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=toolong&message=hi").await;
        assert!(body.contains("too long"));
    }

    #[tokio::test]
    async fn test_max_name_length_zero_unlimited() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.max_name_length = 0;
        let (app, _rx) = test_app(config);
        let long_name = "a".repeat(200);
        let (_, body) = post_form(&app, &format!("name={long_name}&message=hi")).await;
        assert!(body.contains("pending approval"));
    }

    #[tokio::test]
    async fn test_max_message_length() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.max_message_length = 10;
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=test&message=this+message+is+way+too+long").await;
        assert!(body.contains("too long"));
    }

    #[tokio::test]
    async fn test_max_website_length() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.max_website_length = 5;
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=test&message=hi&website=http://toolong.com").await;
        assert!(body.contains("too long"));
    }

    #[tokio::test]
    async fn test_custom_template() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.template = Some("<html><nav>custom nav</nav>{{form}}{{entries}}</html>".into());
        let (app, _rx) = test_app(config);
        let html = get_index(&app).await;
        assert!(html.contains("custom nav"));
        assert!(html.contains("action=\"/submit\""));
    }

    #[tokio::test]
    async fn test_valid_submission_creates_entry() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello").await;
        assert!(body.contains("pending approval"));
        let count = std::fs::read_dir(dir.path().join("entries"))
            .unwrap()
            .count();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_website_field_disabled_ignores_website() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_website_links = false;
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello&website=http://evil.com").await;
        assert!(body.contains("pending approval"));
        let entries_dir = dir.path().join("entries");
        let files: Vec<_> = std::fs::read_dir(&entries_dir).unwrap().collect();
        assert_eq!(files.len(), 1);
        let content = std::fs::read_to_string(files[0].as_ref().unwrap().path()).unwrap();
        assert!(content.contains("website = \"\""));
    }

    #[tokio::test]
    async fn test_website_field_disabled_hides_form_field() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_website_links = false;
        let (app, _rx) = test_app(config);
        let html = get_index(&app).await;
        assert!(!html.contains("name=\"website\""));
    }

    #[tokio::test]
    async fn test_captcha_rejects_wrong_answer() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_captcha = true;
        config.captcha_question = "What is my name?".into();
        config.captcha_answer = "lew".into();
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello&captcha=wrong").await;
        assert!(body.contains("Wrong answer"));
    }

    #[tokio::test]
    async fn test_captcha_accepts_correct_answer() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_captcha = true;
        config.captcha_question = "What is my name?".into();
        config.captcha_answer = "lew".into();
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello&captcha=lew").await;
        assert!(body.contains("pending approval"));
    }

    #[tokio::test]
    async fn test_captcha_inexact_contains() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_captcha = true;
        config.captcha_exact = false;
        config.captcha_question = "What is my name?".into();
        config.captcha_answer = "lew".into();
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello&captcha=lewis").await;
        assert!(body.contains("pending approval"));
    }

    #[tokio::test]
    async fn test_captcha_inexact_rejects_no_match() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_captcha = true;
        config.captcha_exact = false;
        config.captcha_question = "What is my name?".into();
        config.captcha_answer = "lew".into();
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello&captcha=bob").await;
        assert!(body.contains("Wrong answer"));
    }

    #[tokio::test]
    async fn test_captcha_casesensitive() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_captcha = true;
        config.captcha_question = "What is my name?".into();
        config.captcha_answer = "lew".into();
        config.captcha_casesensitive = true;
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello&captcha=Lew").await;
        assert!(body.contains("Wrong answer"));
    }

    #[tokio::test]
    async fn test_captcha_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_captcha = true;
        config.captcha_question = "What is my name?".into();
        config.captcha_answer = "lew".into();
        config.captcha_casesensitive = false;
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello&captcha=LEW").await;
        assert!(body.contains("pending approval"));
    }

    #[tokio::test]
    async fn test_captcha_disabled_skips_check() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello").await;
        assert!(body.contains("pending approval"));
    }

    #[tokio::test]
    async fn test_captcha_shows_in_form() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_captcha = true;
        config.captcha_question = "What is 2+2?".into();
        config.captcha_answer = "4".into();
        let (app, _rx) = test_app(config);
        let html = get_index(&app).await;
        assert!(html.contains("What is 2+2?"));
        assert!(html.contains("name=\"captcha\""));
    }

    async fn get_path(app: &Router, path: &str) -> (StatusCode, Vec<u8>) {
        let req = Request::builder()
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes().to_vec();
        (status, bytes)
    }

    #[tokio::test]
    async fn test_serve_drawing() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);

        let drawings_dir = dir.path().join("drawings");
        std::fs::create_dir_all(&drawings_dir).unwrap();
        let png_bytes = b"\x89PNG\r\n\x1a\nfake";
        std::fs::write(drawings_dir.join("test123.png"), png_bytes).unwrap();

        let (status, body) = get_path(&app, "/drawings/test123.png").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, png_bytes);
    }

    #[tokio::test]
    async fn test_serve_drawing_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);

        let (status, _) = get_path(&app, "/drawings/nonexistent.png").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_serve_drawing_rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);

        let (status, _) = get_path(&app, "/drawings/../entries/secret.txt").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    /// Build a fake but valid PNG with the given dimensions.
    fn fake_png(width: u32, height: u32) -> Vec<u8> {
        let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&width.to_be_bytes());
        png.extend_from_slice(&height.to_be_bytes());
        png.extend_from_slice(&[8, 6, 0, 0, 0]);
        png.extend_from_slice(&[0, 0, 0, 0]);
        png
    }

    #[tokio::test]
    async fn test_submit_with_drawing() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_drawings = true;
        config.canvas_width = 400;
        config.canvas_height = 200;
        let (app, _rx) = test_app(config);

        let png = fake_png(400, 200);
        let drawing_data = base64::engine::general_purpose::STANDARD.encode(&png);
        let data_url = format!("data:image/png;base64,{drawing_data}");
        let body = format!(
            "name=alice&message=hello&drawing={}",
            urlencoding::encode(&data_url)
        );
        let (_, resp) = post_form(&app, &body).await;
        assert!(resp.contains("pending approval"));

        let entries: Vec<_> = std::fs::read_dir(dir.path().join("entries"))
            .unwrap()
            .collect();
        assert_eq!(entries.len(), 1);
        let content = std::fs::read_to_string(entries[0].as_ref().unwrap().path()).unwrap();
        assert!(content.contains("drawing = "));

        let drawings: Vec<_> = std::fs::read_dir(dir.path().join("drawings"))
            .unwrap()
            .collect();
        assert_eq!(drawings.len(), 1);
    }

    #[tokio::test]
    async fn test_submit_without_drawing() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_drawings = true;
        let (app, _rx) = test_app(config);
        let (_, resp) = post_form(&app, "name=alice&message=hello").await;
        assert!(resp.contains("pending approval"));

        let drawings_dir = dir.path().join("drawings");
        if drawings_dir.exists() {
            let count = std::fs::read_dir(&drawings_dir).unwrap().count();
            assert_eq!(count, 0);
        }
    }

    #[tokio::test]
    async fn test_submit_drawing_too_large() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_drawings = true;
        config.canvas_width = 1;
        config.canvas_height = 1;
        let (app, _rx) = test_app(config);

        // PNG with dimensions 1x1 — max_drawing_bytes() is 4, but the fake_png itself is 33 bytes
        let png = fake_png(1, 1);
        let drawing_data = base64::engine::general_purpose::STANDARD.encode(&png);
        let data_url = format!("data:image/png;base64,{drawing_data}");
        let body = format!(
            "name=alice&message=hello&drawing={}",
            urlencoding::encode(&data_url)
        );
        let (_, resp) = post_form(&app, &body).await;
        assert!(resp.contains("too large"));
    }

    #[tokio::test]
    async fn test_submit_drawing_rejects_non_png() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_drawings = true;
        let (app, _rx) = test_app(config);

        let drawing_data = base64::engine::general_purpose::STANDARD.encode(b"not a png file at all");
        let data_url = format!("data:image/png;base64,{drawing_data}");
        let body = format!(
            "name=alice&message=hello&drawing={}",
            urlencoding::encode(&data_url)
        );
        let (_, resp) = post_form(&app, &body).await;
        assert!(resp.contains("Invalid drawing"));
    }

    #[tokio::test]
    async fn test_submit_drawing_rejects_wrong_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_drawings = true;
        config.canvas_width = 400;
        config.canvas_height = 200;
        let (app, _rx) = test_app(config);

        let png = fake_png(1920, 1080);
        let drawing_data = base64::engine::general_purpose::STANDARD.encode(&png);
        let data_url = format!("data:image/png;base64,{drawing_data}");
        let body = format!(
            "name=alice&message=hello&drawing={}",
            urlencoding::encode(&data_url)
        );
        let (_, resp) = post_form(&app, &body).await;
        assert!(resp.contains("Invalid drawing dimensions"));
    }

    #[tokio::test]
    async fn test_submit_drawing_ignored_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_drawings = false;
        let (app, _rx) = test_app(config);

        let png = fake_png(400, 200);
        let drawing_data = base64::engine::general_purpose::STANDARD.encode(&png);
        let data_url = format!("data:image/png;base64,{drawing_data}");
        let body = format!(
            "name=alice&message=hello&drawing={}",
            urlencoding::encode(&data_url)
        );
        let (_, resp) = post_form(&app, &body).await;
        assert!(resp.contains("pending approval"));

        let entries: Vec<_> = std::fs::read_dir(dir.path().join("entries"))
            .unwrap()
            .collect();
        let content = std::fs::read_to_string(entries[0].as_ref().unwrap().path()).unwrap();
        assert!(content.contains("drawing = \"\""));
    }

    #[tokio::test]
    async fn test_drawing_full_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.enable_drawings = true;
        config.canvas_width = 400;
        config.canvas_height = 200;
        let (app, _rx) = test_app(config);

        // Submit with a drawing
        let png = fake_png(400, 200);
        let drawing_data = base64::engine::general_purpose::STANDARD.encode(&png);
        let data_url = format!("data:image/png;base64,{drawing_data}");
        let body = format!(
            "name=alice&message=hello&drawing={}",
            urlencoding::encode(&data_url)
        );
        post_form(&app, &body).await;

        // Approve the entry
        let entries_dir = dir.path().join("entries");
        let entry_file = std::fs::read_dir(&entries_dir).unwrap().next().unwrap().unwrap();
        let content = std::fs::read_to_string(entry_file.path()).unwrap();
        let id = entry_file.path().file_stem().unwrap().to_str().unwrap().to_string();
        let mut entry = entries::Entry::parse(&id, &content).unwrap();
        entry.meta.status = entries::Status::Approved;
        std::fs::write(entry_file.path(), entry.to_file_contents()).unwrap();

        let drawing_filename = entry.meta.drawing.clone();
        assert!(!drawing_filename.is_empty(), "entry should have a drawing filename");

        // Verify index shows the drawing
        let html = get_index(&app).await;
        assert!(html.contains("entry-drawing"));
        assert!(html.contains(&format!("/drawings/{drawing_filename}")));

        // Verify the drawing file is served
        let (status, bytes) = get_path(&app, &format!("/drawings/{drawing_filename}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(bytes, png);
    }

    #[tokio::test]
    async fn test_submit_success_is_full_page() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello").await;
        assert!(body.contains("<!DOCTYPE html>"));
        assert!(body.contains("<title>test</title>"));
        assert!(body.contains("pending approval"));
        assert!(body.contains("back"));
    }

    #[tokio::test]
    async fn test_submit_custom_success_template() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path());
        config.success_template = Some("<p>{{title}} — sent!</p>".into());
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=alice&message=hello").await;
        assert_eq!(body, "<p>test — sent!</p>");
    }

    #[tokio::test]
    async fn test_submit_error_is_full_page() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config(dir.path());
        let (app, _rx) = test_app(config);
        let (_, body) = post_form(&app, "name=&message=").await;
        assert!(body.contains("<!DOCTYPE html>"));
        assert!(body.contains("Name and message are required"));
        assert!(body.contains("back"));
    }
}
