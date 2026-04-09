use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub site_title: String,

    pub telegram_bot_token: String,
    pub telegram_chat_id: i64,
    pub enable_honeypot: bool,
    pub max_name_length: usize,
    pub max_message_length: usize,
    pub max_website_length: usize,
    pub enable_submissions: bool,
    pub enable_website_links: bool,
    pub enable_html_injection: bool,
    pub template: Option<String>,
    pub separator: String,
    pub style: String,
    pub form_prompt: String,
    pub button_text: String,
    pub label_name: String,
    pub label_website: String,
    pub label_message: String,
    pub textarea_rows: u32,
    pub textarea_cols: u32,
}

impl Config {
    pub fn listen_addr(&self) -> String {
        format!("127.0.0.1:{}", self.port)
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

            telegram_bot_token: env::var("BOOK_TELEGRAM_BOT_TOKEN")
                .map_err(|_| "BOOK_TELEGRAM_BOT_TOKEN is required")?,
            telegram_chat_id: env::var("BOOK_TELEGRAM_CHAT_ID")
                .map_err(|_| "BOOK_TELEGRAM_CHAT_ID is required")?
                .parse()
                .map_err(|_| "BOOK_TELEGRAM_CHAT_ID must be an integer")?,
            enable_honeypot: env::var("BOOK_ENABLE_HONEYPOT")
                .map(|v| v != "false")
                .unwrap_or(true),
            max_name_length: env::var("BOOK_MAX_NAME_LENGTH")
                .unwrap_or_else(|_| "50".into())
                .parse()
                .map_err(|_| "BOOK_MAX_NAME_LENGTH must be a number")?,
            max_message_length: env::var("BOOK_MAX_MESSAGE_LENGTH")
                .unwrap_or_else(|_| "1000".into())
                .parse()
                .map_err(|_| "BOOK_MAX_MESSAGE_LENGTH must be a number")?,
            max_website_length: env::var("BOOK_MAX_WEBSITE_LENGTH")
                .unwrap_or_else(|_| "100".into())
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
                .unwrap_or(true),
            separator: env::var("BOOK_SEPARATOR")
                .unwrap_or_else(|_| "------------------------------------------------------------".into()),
            template: env::var("BOOK_TEMPLATE").ok().map(|path| {
                std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("failed to read template {path}: {e}"))
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
                .unwrap_or_else(|_| "If you visited my site, please sign my guestbook!".into()),
            button_text: env::var("BOOK_BUTTON_TEXT")
                .unwrap_or_else(|_| "sign".into()),
            label_name: env::var("BOOK_LABEL_NAME")
                .unwrap_or_else(|_| "Your name:".into()),
            label_website: env::var("BOOK_LABEL_WEBSITE")
                .unwrap_or_else(|_| "Your website (optional):".into()),
            label_message: env::var("BOOK_LABEL_MESSAGE")
                .unwrap_or_else(|_| "Your message:".into()),
            textarea_rows: env::var("BOOK_TEXTAREA_ROWS")
                .unwrap_or_else(|_| "8".into())
                .parse()
                .map_err(|_| "BOOK_TEXTAREA_ROWS must be a number")?,
            textarea_cols: env::var("BOOK_TEXTAREA_COLS")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .map_err(|_| "BOOK_TEXTAREA_COLS must be a number")?,
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
        assert_eq!(config.telegram_chat_id, 12345);

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
    fn test_missing_required() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");

        let result = Config::from_env();
        assert!(result.is_err());
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
        assert!(config.enable_html_injection);

        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");
    }

    #[test]
    fn test_enable_html_injection_false() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("BOOK_TELEGRAM_BOT_TOKEN", "123:ABC");
        env::set_var("BOOK_TELEGRAM_CHAT_ID", "12345");
        env::set_var("BOOK_ENABLE_HTML_INJECTION", "false");

        let config = Config::from_env().unwrap();
        assert!(!config.enable_html_injection);

        env::remove_var("BOOK_TELEGRAM_BOT_TOKEN");
        env::remove_var("BOOK_TELEGRAM_CHAT_ID");
        env::remove_var("BOOK_ENABLE_HTML_INJECTION");
    }
}
