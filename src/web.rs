use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::entries::{self, Entry, EntryMeta, Status};
use crate::render::{self, DEFAULT_TEMPLATE};

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
    #[serde(default)]
    captcha: String,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/submit", post(submit))
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

async fn submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SubmitForm>,
) -> Html<String> {
    if !state.config.enable_submissions {
        return Html("Submissions are closed.".to_string());
    }

    // Honeypot check — silently discard
    if state.config.enable_honeypot && !form.url.is_empty() {
        return Html("Thanks! Your message is pending approval.".to_string());
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
            return Html("Wrong answer.".to_string());
        }
    }

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
    let date = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let date_short = &date[..10];
    let filename = format!("{date_short}-{short_id}.txt");

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
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
            template: None,
            separator: "---".into(),
            style: String::new(),
            form_prompt: "Thanks for visiting. Sign the guestbook!".into(),
            button_text: "sign".into(),
            label_name: "Your name:".into(),
            label_website: "Your website (optional):".into(),
            label_message: "Your message:".into(),
            textarea_rows: 8,
            textarea_cols: 60,
        }
    }

    fn test_app(config: Config) -> (Router, tokio::sync::mpsc::Receiver<Entry>) {
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
}
