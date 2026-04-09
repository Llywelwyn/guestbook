use crate::config::Config;
use crate::entries::Entry;

pub const DEFAULT_TEMPLATE: &str = include_str!("../templates/default.html");
pub const DEFAULT_STYLE: &str = include_str!("../templates/default.css");

pub fn render_page(template: &str, config: &Config, entries: &[Entry], form_html: &str) -> String {
    let entries_html = render_entries(entries, config);
    let css = if config.style.is_empty() {
        DEFAULT_STYLE
    } else {
        &config.style
    };
    let style = format!("<style>\n{css}\n  </style>");
    template
        .replace("{{title}}", &config.site_title)
        .replace("{{form}}", form_html)
        .replace("{{entries}}", &entries_html)
        .replace("{{style}}", &style)
}

pub fn render_form(config: &Config) -> String {
    let website_section = if config.enable_website_links {
        format!(
            "\n<label class=\"guestbook-label\">{}</label>\n<input class=\"guestbook-input\" name=\"website\">\n",
            config.label_website
        )
    } else {
        String::new()
    };

    let captcha_section = if config.enable_captcha {
        format!(
            "\n<label class=\"guestbook-label\">{}</label>\n<input class=\"guestbook-input\" name=\"captcha\" required>\n",
            config.captcha_question
        )
    } else {
        String::new()
    };

    let drawing_section = if config.enable_drawings {
        format!(
            r##"<label class="guestbook-label">{label}</label>
<canvas class="guestbook-canvas" width="{w}" height="{h}"></canvas>
<a href="#" class="guestbook-canvas-reset">Reset</a>
<input type="hidden" name="drawing">
<script>
(function(){{
  var c=document.querySelector('.guestbook-canvas'),
      x=c.getContext('2d'),
      d=false,lx,ly;
  function pos(e){{var r=c.getBoundingClientRect();
    return[e.clientX-r.left,e.clientY-r.top]}}
  function tpos(e){{var r=c.getBoundingClientRect(),t=e.touches[0];
    return[t.clientX-r.left,t.clientY-r.top]}}
  c.addEventListener('mousedown',function(e){{d=true;var p=pos(e);lx=p[0];ly=p[1]}});
  c.addEventListener('mousemove',function(e){{if(!d)return;var p=pos(e);
    x.beginPath();x.moveTo(lx,ly);x.lineTo(p[0],p[1]);x.stroke();lx=p[0];ly=p[1]}});
  c.addEventListener('mouseup',function(){{d=false}});
  c.addEventListener('mouseleave',function(){{d=false}});
  c.addEventListener('touchstart',function(e){{e.preventDefault();var p=tpos(e);lx=p[0];ly=p[1]}});
  c.addEventListener('touchmove',function(e){{e.preventDefault();var p=tpos(e);
    x.beginPath();x.moveTo(lx,ly);x.lineTo(p[0],p[1]);x.stroke();lx=p[0];ly=p[1]}});
  document.querySelector('.guestbook-canvas-reset').addEventListener('click',function(e){{
    e.preventDefault();x.clearRect(0,0,c.width,c.height)}});
  c.closest('form').addEventListener('submit',function(){{
    var px=new Uint32Array(x.getImageData(0,0,c.width,c.height).data.buffer);
    if(px.some(function(v){{return v!==0}})){{
      c.closest('form').querySelector('[name=drawing]').value=c.toDataURL('image/png');
    }}
  }});
}})();
</script>"##,
            label = config.label_drawing,
            w = config.canvas_width,
            h = config.canvas_height,
        )
    } else {
        String::new()
    };

    format!(
        r#"<span class="guestbook-prompt">{prompt}</span>
<form class="guestbook-form" method="post" action="/submit" accept-charset="UTF-8">
<label class="guestbook-label">{label_name}</label>
<input class="guestbook-input" name="name" required>
{website_section}
<label class="guestbook-label">{label_message}</label>
<textarea class="guestbook-textarea" name="message" style="width:{tw}px;height:{th}px" required></textarea>
{captcha_section}
{drawing_section}
<input name="url" style="display:none" tabindex="-1" autocomplete="off">
<button class="guestbook-button" type="submit">{button}</button>
</form>"#,
        prompt = config.form_prompt,
        label_name = config.label_name,
        website_section = website_section,
        label_message = config.label_message,
        tw = config.textarea_width,
        th = config.textarea_height,
        captcha_section = captcha_section,
        drawing_section = drawing_section,
        button = config.button_text,
    )
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn render_entries(entries: &[Entry], config: &Config) -> String {
    let mut html = String::new();
    for entry in entries {
        html.push_str(&render_entry(entry, config));
    }
    html
}

fn render_entry(entry: &Entry, config: &Config) -> String {
    let name = if config.enable_html_injection {
        entry.meta.name.clone()
    } else {
        escape_html(&entry.meta.name)
    };
    let mut header = format!(
        "<span class=\"entry-header\"><span class=\"entry-date\">{}</span> - <span class=\"entry-name\">{}</span>",
        &entry.meta.date[..10], name
    );
    if config.enable_website_links && !entry.meta.website.is_empty() {
        let website = escape_html(&entry.meta.website);
        header.push_str(&format!(
            " (<a class=\"entry-website\" href=\"{}\">{}</a>)",
            website, website
        ));
    }
    header.push_str("</span>");
    let body = if config.enable_html_injection {
        entry.body.clone()
    } else {
        escape_html(&entry.body)
    };
    let drawing_html = if !entry.meta.drawing.is_empty() {
        format!(
            "\n<img class=\"entry-drawing\" src=\"/drawings/{}\">",
            escape_html(&entry.meta.drawing)
        )
    } else {
        String::new()
    };
    format!(
        "\n{header}\n{drawing_html}\n<span class=\"entry-body\">{body}</span>\n\n<span class=\"entry-separator\">{}</span>\n",
        config.separator
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entries::{Entry, EntryMeta, Status};
    use std::path::PathBuf;

    fn test_config() -> Config {
        Config {
            port: 0,
            data_dir: PathBuf::from("./data"),
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
            template: None,
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

    fn make_entry(name: &str, date: &str, body: &str) -> Entry {
        Entry {
            id: "test".into(),
            meta: EntryMeta {
                name: name.into(),
                date: date.into(),
                website: String::new(),
                drawing: String::new(),
                status: Status::Approved,
            },
            body: body.into(),
        }
    }

    #[test]
    fn test_render_default_template() {
        let config = test_config();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[], &form);
        assert!(html.contains("<title>test</title>"));
        assert!(html.contains("guestbook-form"));
    }

    #[test]
    fn test_render_custom_template() {
        let config = test_config();
        let custom = "<html>{{title}} {{form}} {{entries}} {{style}}</html>";
        let form = render_form(&config);
        let html = render_page(custom, &config, &[], &form);
        assert!(html.contains("test"));
        assert!(html.contains("guestbook-form"));
    }

    #[test]
    fn test_render_entry_classes() {
        let config = test_config();
        let entry = make_entry("alice", "2026-04-09", "Hello!");
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains("entry-header"));
        assert!(html.contains("entry-name"));
        assert!(html.contains("entry-body"));
        assert!(html.contains("entry-separator"));
    }

    #[test]
    fn test_render_entry_with_website() {
        let config = test_config();
        let mut entry = make_entry("bob", "2026-04-09", "Hi!");
        entry.meta.website = "https://bob.com".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains("entry-website"));
        assert!(html.contains(r#"href="https://bob.com">"#));
    }

    #[test]
    fn test_render_preserves_html_in_body() {
        let mut config = test_config();
        config.enable_html_injection = true;
        let entry = make_entry("carol", "2026-04-09", "<b>Bold</b>");
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains("<b>Bold</b>"));
    }

    #[test]
    fn test_render_empty_form_when_closed() {
        let config = test_config();
        let html = render_page(DEFAULT_TEMPLATE, &config, &[], "");
        assert!(!html.contains("action=\"/submit\""));
    }

    #[test]
    fn test_render_custom_style() {
        let mut config = test_config();
        config.style = ".entry-name { color: red; }".into();
        let html = render_page(DEFAULT_TEMPLATE, &config, &[], "");
        assert!(html.contains(".entry-name { color: red; }"));
        assert!(html.contains("<style>"));
    }

    #[test]
    fn test_render_form_custom_labels() {
        let mut config = test_config();
        config.form_prompt = "Leave a note!".into();
        config.button_text = "submit".into();
        config.label_name = "Name:".into();
        let form = render_form(&config);
        assert!(form.contains("Leave a note!"));
        assert!(form.contains("submit"));
        assert!(form.contains("Name:"));
    }

    #[test]
    fn test_render_form_custom_textarea() {
        let mut config = test_config();
        config.textarea_width = 500;
        config.textarea_height = 200;
        let form = render_form(&config);
        assert!(form.contains("width:500px"));
        assert!(form.contains("height:200px"));
    }

    #[test]
    fn test_render_form_hides_website_when_disabled() {
        let mut config = test_config();
        config.enable_website_links = false;
        let form = render_form(&config);
        assert!(!form.contains("name=\"website\""));
        assert!(!form.contains(&config.label_website));
    }

    #[test]
    fn test_render_form_shows_website_when_enabled() {
        let config = test_config();
        let form = render_form(&config);
        assert!(form.contains("name=\"website\""));
        assert!(form.contains(&config.label_website));
    }

    #[test]
    fn test_render_entry_always_escapes_website() {
        let config = test_config();
        let mut entry = make_entry("bob", "2026-04-09", "Hi!");
        entry.meta.website = "https://example.com?a=1&b=2".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains("href=\"https://example.com?a=1&amp;b=2\""));
        assert!(!html.contains("href=\"https://example.com?a=1&b=2\""));
    }

    #[test]
    fn test_render_entry_hides_website_when_disabled() {
        let mut config = test_config();
        config.enable_website_links = false;
        let mut entry = make_entry("bob", "2026-04-09", "Hi!");
        entry.meta.website = "https://bob.com".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(!html.contains("href=\"https://bob.com\""));
        assert!(!html.contains("class=\"entry-website\""));
    }

    #[test]
    fn test_render_entry_escapes_html_when_injection_disabled() {
        let mut config = test_config();
        config.enable_html_injection = false;
        let entry = make_entry("<b>hacker</b>", "2026-04-09", "<script>alert('xss')</script>");
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains("&lt;b&gt;hacker&lt;/b&gt;"));
        assert!(html.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_render_entry_preserves_html_when_injection_enabled() {
        let mut config = test_config();
        config.enable_html_injection = true;
        let entry = make_entry("carol", "2026-04-09", "<b>Bold</b>");
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains("<b>Bold</b>"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<b>test</b> & \"quotes\" 'apos'"),
            "&lt;b&gt;test&lt;/b&gt; &amp; &quot;quotes&quot; &#x27;apos&#x27;"
        );
    }

    #[test]
    fn test_render_form_shows_canvas_when_drawings_enabled() {
        let mut config = test_config();
        config.enable_drawings = true;
        let form = render_form(&config);
        assert!(form.contains("<canvas"));
        assert!(form.contains("class=\"guestbook-canvas\""));
        assert!(form.contains("name=\"drawing\""));
        assert!(form.contains("Reset"));
    }

    #[test]
    fn test_render_form_hides_canvas_when_drawings_disabled() {
        let config = test_config();
        let form = render_form(&config);
        assert!(!form.contains("<canvas"));
        assert!(!form.contains("name=\"drawing\""));
    }

    #[test]
    fn test_render_entry_with_drawing() {
        let config = test_config();
        let mut entry = make_entry("alice", "2026-04-09", "Hello!");
        entry.meta.drawing = "2026-04-09-abc123.png".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains(r#"<img class="entry-drawing" src="/drawings/2026-04-09-abc123.png">"#));
    }

    #[test]
    fn test_render_entry_drawing_works_without_html_injection() {
        let mut config = test_config();
        config.enable_html_injection = false;
        let mut entry = make_entry("alice", "2026-04-09", "<script>xss</script>");
        entry.meta.drawing = "2026-04-09-abc123.png".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        // Drawing renders regardless
        assert!(html.contains(r#"<img class="entry-drawing" src="/drawings/2026-04-09-abc123.png">"#));
        // But body HTML is escaped
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_render_entry_without_drawing() {
        let config = test_config();
        let entry = make_entry("alice", "2026-04-09", "Hello!");
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(!html.contains("<img class=\"entry-drawing\""));
    }
}
