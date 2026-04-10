use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub site_title: String,

    #[cfg(feature = "telegram")]
    pub telegram_bot_token: Option<String>,
    #[cfg(feature = "telegram")]
    pub telegram_chat_id: Option<i64>,
    #[cfg(feature = "telegram")]
    pub telegram_retry_interval: u64,
    #[cfg(feature = "telegram")]
    pub telegram_retry_limit: u32,
    #[cfg(feature = "telegram")]
    pub telegram_reminder_interval: u64,
    pub enable_honeypot: bool,
    pub max_name_length: usize,
    pub max_message_length: usize,
    pub max_website_length: usize,
    pub enable_submissions: bool,
    pub enable_website_links: bool,
    pub enable_html_injection: bool,
    pub enable_captcha: bool,
    pub captcha_question: String,
    pub captcha_answer: String,
    pub captcha_exact: bool,
    pub captcha_casesensitive: bool,
    pub enable_drawings: bool,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub enable_voice_notes: bool,
    pub voice_note_max_duration: u32,
    pub template: Option<String>,
    pub success_template: Option<String>,
    pub style: String,
    pub form_prompt: String,
    pub button_text: String,
    pub label_name: String,
    pub label_website: String,
    pub label_message: String,
    pub textarea_width: u32,
    pub textarea_height: u32,
}

impl Config {
    pub fn listen_addr(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// Maximum drawing file size: width * height * 4 (raw RGBA).
    /// Any valid PNG from the configured canvas will be smaller than this.
    pub fn max_drawing_bytes(&self) -> usize {
        self.canvas_width as usize * self.canvas_height as usize * 4
    }

    /// Maximum voice note file size: duration * 10KB.
    /// Generous cap — real WebM/Opus clips are much smaller.
    pub fn max_voice_note_bytes(&self) -> usize {
        self.voice_note_max_duration as usize * 10 * 1024
    }

    pub fn from_env() -> Result<Self, String> {
        Ok(Config {
            port: env::var("BOOK_PORT")
                .unwrap_or_else(|_| "8123".into())
                .parse()
                .map_err(|_| "BOOK_PORT must be a number")?,
            data_dir: env::var("BOOK_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./data")),
            site_title: env::var("BOOK_SITE_TITLE").unwrap_or_else(|_| "guestbook".into()),

            #[cfg(feature = "telegram")]
            telegram_bot_token: env::var("BOOK_TELEGRAM_BOT_TOKEN").ok(),
            #[cfg(feature = "telegram")]
            telegram_chat_id: env::var("BOOK_TELEGRAM_CHAT_ID")
                .ok()
                .map(|v| v.parse().map_err(|_| "BOOK_TELEGRAM_CHAT_ID must be an integer"))
                .transpose()?,
            #[cfg(feature = "telegram")]
            telegram_retry_interval: env::var("BOOK_TELEGRAM_RETRY_INTERVAL")
                .unwrap_or_else(|_| "20".into())
                .parse()
                .map_err(|_| "BOOK_TELEGRAM_RETRY_INTERVAL must be a number")?,
            #[cfg(feature = "telegram")]
            telegram_retry_limit: env::var("BOOK_TELEGRAM_RETRY_LIMIT")
                .unwrap_or_else(|_| "3".into())
                .parse()
                .map_err(|_| "BOOK_TELEGRAM_RETRY_LIMIT must be a number")?,
            #[cfg(feature = "telegram")]
            telegram_reminder_interval: env::var("BOOK_TELEGRAM_REMINDER_INTERVAL")
                .unwrap_or_else(|_| "86400".into())
                .parse()
                .map_err(|_| "BOOK_TELEGRAM_REMINDER_INTERVAL must be a number")?,
            enable_honeypot: env::var("BOOK_ENABLE_HONEYPOT")
                .map(|v| v != "false")
                .unwrap_or(true),
            max_name_length: env::var("BOOK_MAX_NAME_LENGTH")
                .unwrap_or_else(|_| "0".into())
                .parse()
                .map_err(|_| "BOOK_MAX_NAME_LENGTH must be a number")?,
            max_message_length: env::var("BOOK_MAX_MESSAGE_LENGTH")
                .unwrap_or_else(|_| "0".into())
                .parse()
                .map_err(|_| "BOOK_MAX_MESSAGE_LENGTH must be a number")?,
            max_website_length: env::var("BOOK_MAX_WEBSITE_LENGTH")
                .unwrap_or_else(|_| "0".into())
                .parse()
                .map_err(|_| "BOOK_MAX_WEBSITE_LENGTH must be a number")?,
            enable_submissions: env::var("BOOK_ENABLE_SUBMISSIONS")
                .map(|v| v != "false")
                .unwrap_or(true),
            enable_website_links: env::var("BOOK_ENABLE_WEBSITE_LINKS")
                .map(|v| v != "false")
                .unwrap_or(true),
            enable_html_injection: env::var("BOOK_ENABLE_HTML_INJECTION")
                .map(|v| v != "false")
                .unwrap_or(false),
            enable_captcha: env::var("BOOK_ENABLE_CAPTCHA")
                .map(|v| v != "false")
                .unwrap_or(false),
            captcha_question: env::var("BOOK_CAPTCHA_QUESTION")
                .unwrap_or_default(),
            captcha_answer: env::var("BOOK_CAPTCHA_ANSWER")
                .unwrap_or_default(),
            captcha_exact: env::var("BOOK_CAPTCHA_EXACT")
                .map(|v| v != "false")
                .unwrap_or(false),
            captcha_casesensitive: env::var("BOOK_CAPTCHA_CASESENSITIVE")
                .map(|v| v != "false")
                .unwrap_or(false),
            enable_drawings: env::var("BOOK_ENABLE_DRAWINGS")
                .map(|v| v != "false")
                .unwrap_or(false),
            canvas_width: env::var("BOOK_CANVAS_WIDTH")
                .unwrap_or_else(|_| "400".into())
                .parse()
                .map_err(|_| "BOOK_CANVAS_WIDTH must be a number")?,
            canvas_height: env::var("BOOK_CANVAS_HEIGHT")
                .unwrap_or_else(|_| "200".into())
                .parse()
                .map_err(|_| "BOOK_CANVAS_HEIGHT must be a number")?,
            enable_voice_notes: env::var("BOOK_ENABLE_VOICE_NOTES")
                .map(|v| v != "false")
                .unwrap_or(false),
            voice_note_max_duration: env::var("BOOK_VOICE_NOTE_MAX_DURATION")
                .unwrap_or_else(|_| "20".into())
                .parse()
                .map_err(|_| "BOOK_VOICE_NOTE_MAX_DURATION must be a number")?,
            template: env::var("BOOK_TEMPLATE").ok().map(|path| {
                std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("failed to read template {path}: {e}"))
            }),
            success_template: env::var("BOOK_SUCCESS_TEMPLATE").ok().map(|path| {
                std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("failed to read success template {path}: {e}"))
            }),
            style: env::var("BOOK_STYLE_FILE")
                .ok()
                .map(|path| {
                    std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("failed to read style file {path}: {e}"))
                })
                .or_else(|| env::var("BOOK_STYLE").ok())
                .unwrap_or_default(),
            form_prompt: env::var("BOOK_FORM_PROMPT")
                .unwrap_or_else(|_| "Thanks for visiting. Sign the guestbook!".into()),
            button_text: env::var("BOOK_BUTTON_TEXT")
                .unwrap_or_else(|_| "sign".into()),
            label_name: env::var("BOOK_LABEL_NAME")
                .unwrap_or_else(|_| "Your name:".into()),
            label_website: env::var("BOOK_LABEL_WEBSITE")
                .unwrap_or_else(|_| "Your website (optional):".into()),
            label_message: env::var("BOOK_LABEL_MESSAGE")
                .unwrap_or_else(|_| "Your message:".into()),
            textarea_width: env::var("BOOK_TEXTAREA_WIDTH")
                .unwrap_or_else(|_| "400".into())
                .parse()
                .map_err(|_| "BOOK_TEXTAREA_WIDTH must be a number")?,
            textarea_height: env::var("BOOK_TEXTAREA_HEIGHT")
                .unwrap_or_else(|_| "150".into())
                .parse()
                .map_err(|_| "BOOK_TEXTAREA_HEIGHT must be a number")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_from_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("BOOK_PORT", "9999");
        env::set_var("BOOK_DATA_DIR", "/tmp/gb");
        env::set_var("BOOK_SITE_TITLE", "test.rs");
        env::set_var("BOOK_TELEGRAM_BOT_TOKEN", "123:ABC");
        env::set_var("BOOK_TELEGRAM_CHAT_ID", "12345");

        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 9999);
        assert_eq!(config.listen_addr(), "127.0.0.1:9999");
        assert_eq!(config.data_dir, PathBuf::from("/tmp/gb"));
        assert_eq!(config.site_title, "test.rs");
        #[cfg(feature = "telegram")]
        assert_eq!(config.telegram_bot_token.as_deref(), Some("123:ABC"));
        #[cfg(feature = "telegram")]
        assert_eq!(config.telegram_chat_id, Some(12345));

        // Clean up
        env::remove_var("BOOK_PORT");
        env::remove_var("BOOK_DATA_DIR");
        env::remove_var("BOOK_SITE_TITLE");
        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");
    }

    #[test]
    fn test_defaults() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("BOOK_TELEGRAM_BOT_TOKEN", "123:ABC");
        env::set_var("BOOK_TELEGRAM_CHAT_ID", "12345");

        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 8123);
        assert_eq!(config.data_dir, PathBuf::from("./data"));
        assert_eq!(config.site_title, "guestbook");

        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");
    }

    #[test]
    fn test_telegram_optional() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");

        let config = Config::from_env().unwrap();
        #[cfg(feature = "telegram")]
        assert!(config.telegram_bot_token.is_none());
        #[cfg(feature = "telegram")]
        assert!(config.telegram_chat_id.is_none());
    }

    #[test]
    fn test_enable_website_links_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("BOOK_TELEGRAM_BOT_TOKEN", "123:ABC");
        env::set_var("BOOK_TELEGRAM_CHAT_ID", "12345");
        env::remove_var("BOOK_ENABLE_WEBSITE_LINKS");

        let config = Config::from_env().unwrap();
        assert!(config.enable_website_links);

        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");
    }

    #[test]
    fn test_enable_website_links_false() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("BOOK_TELEGRAM_BOT_TOKEN", "123:ABC");
        env::set_var("BOOK_TELEGRAM_CHAT_ID", "12345");
        env::set_var("BOOK_ENABLE_WEBSITE_LINKS", "false");

        let config = Config::from_env().unwrap();
        assert!(!config.enable_website_links);

        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");
        env::remove_var("BOOK_ENABLE_WEBSITE_LINKS");
    }

    #[test]
    fn test_enable_html_injection_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("BOOK_TELEGRAM_BOT_TOKEN", "123:ABC");
        env::set_var("BOOK_TELEGRAM_CHAT_ID", "12345");
        env::remove_var("BOOK_ENABLE_HTML_INJECTION");

        let config = Config::from_env().unwrap();
        assert!(!config.enable_html_injection);

        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");
    }

    #[test]
    fn test_enable_html_injection_true() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("BOOK_TELEGRAM_BOT_TOKEN", "123:ABC");
        env::set_var("BOOK_TELEGRAM_CHAT_ID", "12345");
        env::set_var("BOOK_ENABLE_HTML_INJECTION", "true");

        let config = Config::from_env().unwrap();
        assert!(config.enable_html_injection);

        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");
        env::remove_var("BOOK_ENABLE_HTML_INJECTION");
    }

    #[test]
    fn test_enable_drawings_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::remove_var("BOOK_ENABLE_DRAWINGS");
        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");

        let config = Config::from_env().unwrap();
        assert!(!config.enable_drawings);
        assert_eq!(config.canvas_width, 400);
        assert_eq!(config.canvas_height, 200);
        assert_eq!(config.max_drawing_bytes(), 400 * 200 * 4);
    }

    #[test]
    fn test_enable_voice_notes_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::remove_var("BOOK_ENABLE_VOICE_NOTES");
        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");

        let config = Config::from_env().unwrap();
        assert!(!config.enable_voice_notes);
        assert_eq!(config.voice_note_max_duration, 20);
        assert_eq!(config.max_voice_note_bytes(), 20 * 10 * 1024);
    }

    #[test]
    fn test_success_template_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::remove_var("BOOK_SUCCESS_TEMPLATE");
        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");

        let config = Config::from_env().unwrap();
        assert!(config.success_template.is_none());
    }
}
