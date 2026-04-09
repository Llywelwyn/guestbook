use crate::entries::Entry;

pub fn render_page(site_title: &str, site_url: &str, entries: &[Entry], form_html: &str) -> String {
    let nav_url = site_url.trim_end_matches('/');
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>guestbook - {site_title}</title>
  <link rel="stylesheet" href="/style.css">
</head>
<body>
  <nav>
    <a href="{nav_url}/">{site_title}</a> |
    <a href="{nav_url}/links/">links</a> |
    <a href="{nav_url}/now/">now</a> |
    sign the <a href="/">guestbook</a>
  </nav>
  <h1>guestbook</h1>
  <p>If you visited my site, please sign my guestbook!</p>
{form_html}
"#
    );

    for entry in entries {
        html.push_str(&render_entry(entry));
    }

    html.push_str("</body>\n</html>\n");
    html
}

fn render_entry(entry: &Entry) -> String {
    let mut header = format!("  <div class=\"entry\">\n    <p>{} - <b>{}</b>", entry.meta.date, entry.meta.name);
    if !entry.meta.website.is_empty() {
        header.push_str(&format!(
            " (<a href=\"{}\">{}</a>)",
            entry.meta.website, entry.meta.website
        ));
    }
    header.push_str("</p>\n");
    format!("{header}    {}\n  </div>\n", entry.body)
}

pub const FORM_HTML: &str = r#"  <form method="post" action="/submit">
    <input name="name" placeholder="name" required>
    <input name="website" placeholder="website (optional)">
    <textarea name="message" placeholder="message" required></textarea>
    <input name="url" style="display:none" tabindex="-1" autocomplete="off">
    <button type="submit">sign</button>
  </form>"#;

pub const STYLE_CSS: &str = "body {
  max-width: 70ch;
  line-height: 1.5;
  margin: 0 auto;
  padding: 1rem;
}
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entries::{Entry, EntryMeta, Status};

    fn make_entry(name: &str, date: &str, body: &str) -> Entry {
        Entry {
            id: "test".into(),
            meta: EntryMeta {
                name: name.into(),
                date: date.into(),
                website: String::new(),
                status: Status::Approved,
            },
            body: body.into(),
        }
    }

    #[test]
    fn test_render_page_contains_nav() {
        let html = render_page("ily.rs", "https://ily.rs", &[], FORM_HTML);
        assert!(html.contains(r#"<a href="https://ily.rs/">ily.rs</a>"#));
        assert!(html.contains(r#"<a href="https://ily.rs/links/">links</a>"#));
    }

    #[test]
    fn test_render_page_contains_form() {
        let html = render_page("ily.rs", "https://ily.rs", &[], FORM_HTML);
        assert!(html.contains(r#"action="/submit""#));
        assert!(html.contains(r#"style="display:none""#)); // honeypot
    }

    #[test]
    fn test_render_entry_no_website() {
        let entry = make_entry("alice", "2026-04-09", "Hello!");
        let html = render_page("ily.rs", "https://ily.rs", &[entry], FORM_HTML);
        assert!(html.contains("<b>alice</b>"));
        assert!(html.contains("Hello!"));
        assert!(!html.contains("<hr"));
    }

    #[test]
    fn test_render_entry_with_website() {
        let mut entry = make_entry("bob", "2026-04-09", "Hi!");
        entry.meta.website = "https://bob.com".into();
        let html = render_page("ily.rs", "https://ily.rs", &[entry], FORM_HTML);
        assert!(html.contains(r#"<a href="https://bob.com">"#));
    }

    #[test]
    fn test_render_preserves_html_in_body() {
        let entry = make_entry("carol", "2026-04-09", "<b>Bold</b> <script>alert(1)</script>");
        let html = render_page("ily.rs", "https://ily.rs", &[entry], FORM_HTML);
        assert!(html.contains("<b>Bold</b>"));
        assert!(html.contains("<script>alert(1)</script>"));
    }
}
