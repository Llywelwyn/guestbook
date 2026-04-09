use crate::entries::Entry;

pub const DEFAULT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{title}}</title>
  <style>
    body {
      max-width: 70ch;
      line-height: 1.5;
      margin: 0 auto;
      padding: 1rem;
    }
  </style>
</head>
<body>
  <h1>guestbook</h1>
  {{form}}
  {{entries}}
</body>
</html>
"#;

pub fn render_page(template: &str, title: &str, entries: &[Entry], form_html: &str) -> String {
    let entries_html = render_entries(entries);
    template
        .replace("{{title}}", title)
        .replace("{{form}}", form_html)
        .replace("{{entries}}", &entries_html)
}

fn render_entries(entries: &[Entry]) -> String {
    let mut html = String::new();
    for entry in entries {
        html.push_str(&render_entry(entry));
    }
    html
}

fn render_entry(entry: &Entry) -> String {
    let mut header = format!(
        "  <div class=\"entry\">\n    <p>{} - <b>{}</b>",
        entry.meta.date, entry.meta.name
    );
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
    fn test_render_default_template() {
        let html = render_page(DEFAULT_TEMPLATE, "ily.rs", &[], FORM_HTML);
        assert!(html.contains("<title>ily.rs</title>"));
        assert!(html.contains("action=\"/submit\""));
    }

    #[test]
    fn test_render_custom_template() {
        let custom = "<html>{{title}} {{form}} {{entries}}</html>";
        let html = render_page(custom, "my site", &[], FORM_HTML);
        assert!(html.contains("my site"));
        assert!(html.contains("action=\"/submit\""));
    }

    #[test]
    fn test_render_entry_no_website() {
        let entry = make_entry("alice", "2026-04-09", "Hello!");
        let html = render_page(DEFAULT_TEMPLATE, "test", &[entry], FORM_HTML);
        assert!(html.contains("<b>alice</b>"));
        assert!(html.contains("Hello!"));
        assert!(!html.contains("<hr"));
    }

    #[test]
    fn test_render_entry_with_website() {
        let mut entry = make_entry("bob", "2026-04-09", "Hi!");
        entry.meta.website = "https://bob.com".into();
        let html = render_page(DEFAULT_TEMPLATE, "test", &[entry], FORM_HTML);
        assert!(html.contains(r#"<a href="https://bob.com">"#));
    }

    #[test]
    fn test_render_preserves_html_in_body() {
        let entry = make_entry("carol", "2026-04-09", "<b>Bold</b> <script>alert(1)</script>");
        let html = render_page(DEFAULT_TEMPLATE, "test", &[entry], FORM_HTML);
        assert!(html.contains("<b>Bold</b>"));
        assert!(html.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn test_render_empty_form_when_closed() {
        let html = render_page(DEFAULT_TEMPLATE, "test", &[], "");
        assert!(!html.contains("action=\"/submit\""));
    }
}
