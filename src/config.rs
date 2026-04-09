use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen: String,
    pub data_dir: PathBuf,
    pub site_title: String,
    pub site_url: String,
    pub telegram_bot_token: String,
    pub telegram_chat_id: i64,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
listen = "127.0.0.1:8123"
data_dir = "/var/lib/guestbook"
site_title = "ily.rs"
site_url = "https://ily.rs"
telegram_bot_token = "123:ABC"
telegram_chat_id = 12345
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.listen, "127.0.0.1:8123");
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/guestbook"));
        assert_eq!(config.site_title, "ily.rs");
        assert_eq!(config.site_url, "https://ily.rs");
        assert_eq!(config.telegram_chat_id, 12345);
    }
}
