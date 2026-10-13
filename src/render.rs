use crate::ascii;
use crate::config::Config;
use crate::entries::Entry;

pub const DEFAULT_TEMPLATE: &str = include_str!("../templates/default.html");
pub const DEFAULT_SIGN_TEMPLATE: &str = include_str!("../templates/sign.html");
pub const DEFAULT_SUCCESS_TEMPLATE: &str = include_str!("../templates/success.html");
pub const DEFAULT_STYLE: &str = include_str!("../templates/default.css");

fn style_tag(config: &Config) -> String {
    let css = if config.style.is_empty() {
        DEFAULT_STYLE
    } else {
        &config.style
    };
    format!("<style>\n{css}\n  </style>")
}

pub fn render_page(template: &str, config: &Config, entries: &[Entry], form_html: &str) -> String {
    let entries_html = render_entries(entries, config);
    template
        .replace("{{title}}", &config.site_title)
        .replace("{{form}}", form_html)
        .replace("{{entries}}", &entries_html)
        .replace("{{style}}", &style_tag(config))
        .replace("{{base}}", &config.base_path)
}

pub fn render_sign_page(config: &Config, error: Option<&str>) -> String {
    let template = config.sign_template.as_deref().unwrap_or(DEFAULT_SIGN_TEMPLATE);
    let form = if config.enable_submissions {
        render_form(config)
    } else {
        String::new()
    };
    let error = error
        .map(|e| format!("<p class=\"guestbook-error\">{}</p>", escape_html(e)))
        .unwrap_or_default();
    template
        .replace("{{title}}", &config.site_title)
        .replace("{{error}}", &error)
        .replace("{{form}}", &form)
        .replace("{{style}}", &style_tag(config))
        .replace("{{base}}", &config.base_path)
}

fn js_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '<' => out.push_str("\\x3c"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn render_drawing_section(config: &Config) -> String {
    let w = config.canvas_width;
    let h = config.canvas_height;
    let max_size = (w / 16).max(20);
    let size = (w / 45).max(5);
    let (art, styles, columns, rows) = if config.drawing_ascii {
        let buttons = if config.drawing_style_choice() {
            "<button type=\"button\" class=\"guestbook-tool\" data-style=\"ascii\">ASCII</button><button type=\"button\" class=\"guestbook-tool\" data-style=\"image\">Normal</button>"
        } else {
            ""
        };
        (
            "<pre class=\"guestbook-pad-art\" aria-hidden=\"true\"></pre>",
            format!(
                "{buttons}<input type=\"hidden\" name=\"drawing_style\" value=\"{}\">",
                config.default_drawing_style()
            ),
            config.drawing_ascii_columns,
            config.drawing_ascii_rows(),
        )
    } else {
        ("", String::new(), 0, 0)
    };
    format!(
        r##"<span class="guestbook-label">{label}</span>
<div class="guestbook-pad">{art}<canvas class="guestbook-canvas" width="{w}" height="{h}" aria-label="Drawing canvas"></canvas></div>
<div class="guestbook-row guestbook-drawing-tools">{styles}<input type="range" class="guestbook-size-slider" min="1" max="{max_size}" value="{size}" aria-label="Brush size"><button type="button" class="guestbook-tool" data-action="undo">Undo</button><button type="button" class="guestbook-tool" data-action="clear">Clear</button></div>
<input type="hidden" name="drawing"><script>(function(){{
  var pad=document.querySelector('.guestbook-pad'),c=pad.querySelector('canvas'),x=c.getContext('2d'),art=pad.querySelector('pre'),
      tools=document.querySelector('.guestbook-drawing-tools'),sl=tools.querySelector('.guestbook-size-slider'),hid=document.querySelector('[name=drawing]'),
      W=c.width,H=c.height,R={ramp},C={columns},N={rows},d=false,lx,ly,h=[],
      style=document.querySelector('[name=drawing_style]');
  x.lineCap='round';x.lineJoin='round';x.strokeStyle='#000';x.fillStyle='#000';x.lineWidth=+sl.value;
  function setStyle(s){{
    style.value=s;pad.classList.toggle('guestbook-pad-ascii',s==='ascii');
    tools.querySelectorAll('[data-style]').forEach(function(b){{b.setAttribute('aria-pressed',b.dataset.style===s)}});
    show();
  }}
  if(style)tools.querySelectorAll('[data-style]').forEach(function(b){{b.addEventListener('click',function(){{setStyle(b.dataset.style)}})}});
  function show(){{
    if(!art||style.value!=='ascii')return;
    var p=x.getImageData(0,0,W,H).data,s='';
    for(var r=0;r<N;r++){{
      var y0=Math.floor(r*H/N),y1=Math.min(H,Math.max(y0+1,Math.floor((r+1)*H/N)));
      for(var q=0;q<C;q++){{
        var x0=Math.floor(q*W/C),x1=Math.min(W,Math.max(x0+1,Math.floor((q+1)*W/C))),t=0;
        for(var yy=y0;yy<y1;yy++)for(var xx=x0;xx<x1;xx++)t+=p[(yy*W+xx)*4+3];
        s+=R[Math.floor(Math.floor(t/((y1-y0)*(x1-x0)))*(R.length-1)/255)];
      }}
      s+='\n';
    }}
    art.textContent=s;
  }}
  function pos(e){{var b=c.getBoundingClientRect(),t=e.touches?e.touches[0]:e;return[(t.clientX-b.left)*W/b.width,(t.clientY-b.top)*H/b.height]}}
  function save(){{if(h.length>=20)h.shift();h.push(x.getImageData(0,0,W,H))}}
  function down(e){{e.preventDefault();save();d=true;var p=pos(e);lx=p[0];ly=p[1];x.beginPath();x.arc(lx,ly,x.lineWidth/2,0,Math.PI*2);x.fill();show()}}
  function move(e){{if(!d)return;e.preventDefault();var p=pos(e);x.beginPath();x.moveTo(lx,ly);x.lineTo(p[0],p[1]);x.stroke();lx=p[0];ly=p[1];show()}}
  function up(){{d=false}}
  c.addEventListener('mousedown',down);c.addEventListener('mousemove',move);window.addEventListener('mouseup',up);
  c.addEventListener('touchstart',down);c.addEventListener('touchmove',move);c.addEventListener('touchend',up);
  sl.addEventListener('input',function(){{x.lineWidth=+sl.value}});
  tools.querySelector('[data-action=undo]').addEventListener('click',function(){{if(h.length){{x.putImageData(h.pop(),0,0);show()}}}});
  tools.querySelector('[data-action=clear]').addEventListener('click',function(){{save();x.clearRect(0,0,W,H);hid.value='';show()}});
  c.closest('form').addEventListener('submit',function(){{
    var px=x.getImageData(0,0,W,H).data,ink=false;
    for(var i=3;i<px.length;i+=4)if(px[i]){{ink=true;break}}
    if(!ink){{hid.value='';return}}
    var f=document.createElement('canvas');f.width=W;f.height=H;
    var fx=f.getContext('2d');fx.fillStyle='#fff';fx.fillRect(0,0,W,H);fx.drawImage(c,0,0);
    hid.value=f.toDataURL('image/png');
  }});
  if(style)setStyle(style.value);
}})();</script>
"##,
        label = config.label_drawing,
        ramp = js_string(ascii::RAMP),
    )
}

fn render_voice_section(config: &Config) -> String {
    format!(
        r##"<span class="guestbook-label">{label}</span>
<div class="guestbook-row guestbook-voice-controls"></div><div class="guestbook-voice-playback"></div><input type="hidden" name="voice_note"><script>(function(){{
  var maxDur={max_dur},
      ctl=document.querySelector('.guestbook-voice-controls'),
      pb=document.querySelector('.guestbook-voice-playback'),
      hid=document.querySelector('[name=voice_note]'),
      rec=null,chunks=[],iv=null,st=0;
  function fmt(s){{var m=Math.floor(s/60),sec=s%60;return m+':'+(sec<10?'0':'')+sec}}
  function button(text,fn){{var b=document.createElement('button');b.type='button';b.className='guestbook-tool';b.textContent=text;b.addEventListener('click',fn);ctl.appendChild(b);return b}}
  function stop(){{rec.stop();rec.stream.getTracks().forEach(function(t){{t.stop()}})}}
  function setInit(){{
    if(rec&&rec.state==='recording')stop();
    rec=null;chunks=[];clearInterval(iv);iv=null;pb.innerHTML='';hid.value='';ctl.innerHTML='';
    button({record},startRec);
  }}
  function setRec(){{
    ctl.innerHTML='';
    var b=button('',stop);b.classList.add('recording');b.setAttribute('aria-live','polite');
    st=Date.now();b.textContent='Stop '+fmt(0)+' / '+fmt(maxDur);
    iv=setInterval(function(){{
      var el=Math.floor((Date.now()-st)/1000);b.textContent='Stop '+fmt(el)+' / '+fmt(maxDur);
      if(el>=maxDur)stop();
    }},250);
  }}
  function setResult(){{
    clearInterval(iv);iv=null;
    var blob=new Blob(chunks,{{type:'audio/webm;codecs=opus'}});
    ctl.innerHTML='';
    button('Re-record',function(){{setInit();startRec()}});
    button('Discard',setInit);
    var au=document.createElement('audio');au.controls=true;au.preload='metadata';au.src=URL.createObjectURL(blob);
    pb.appendChild(au);
    var rd=new FileReader();rd.onload=function(){{hid.value=rd.result}};rd.readAsDataURL(blob);
  }}
  function startRec(){{
    chunks=[];hid.value='';pb.innerHTML='';
    navigator.mediaDevices.getUserMedia({{audio:true}}).then(function(stream){{
      rec=new MediaRecorder(stream,{{mimeType:'audio/webm;codecs=opus'}});
      rec.ondataavailable=function(e){{if(e.data.size>0)chunks.push(e.data)}};
      rec.onstop=setResult;
      rec.start();setRec();
    }}).catch(function(){{ctl.textContent='Microphone access denied.'}});
  }}
  setInit();
}})();</script>
"##,
        label = config.label_voice_note,
        record = js_string(&config.voice_note_record_text),
        max_dur = config.voice_note_max_duration,
    )
}

pub fn render_form(config: &Config) -> String {
    let website_section = if config.enable_website_links {
        format!(
            "<div class=\"guestbook-field\"><label class=\"guestbook-label\" for=\"website\">{label}</label><input class=\"guestbook-input\" id=\"website\" name=\"website\"></div>",
            label = config.label_website
        )
    } else {
        String::new()
    };

    let captcha_section = if config.enable_captcha {
        format!(
            "<label class=\"guestbook-label\" for=\"captcha\">{label}</label>\n<input class=\"guestbook-input\" id=\"captcha\" name=\"captcha\" required>\n",
            label = config.captcha_question
        )
    } else {
        String::new()
    };

    let drawing_section = if config.enable_drawings {
        render_drawing_section(config)
    } else {
        String::new()
    };

    let voice_note_section = if config.enable_voice_notes {
        render_voice_section(config)
    } else {
        String::new()
    };

    format!(
        r#"<form class="guestbook-form" method="post" action="{base}/submit" accept-charset="UTF-8">
<div class="guestbook-pair"><div class="guestbook-field"><label class="guestbook-label" for="name">{label_name}</label><input class="guestbook-input" id="name" name="name" required></div>{website_section}</div>
<label class="guestbook-label" for="message">{label_message}</label>
<textarea class="guestbook-textarea" id="message" name="message" rows="4"></textarea>
{drawing_section}{voice_note_section}{captcha_section}<input name="url" aria-hidden="true" style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0)" tabindex="-1" autocomplete="off"><div class="guestbook-row"><button class="guestbook-button" type="submit">{button}</button></div>
</form>"#,
        base = config.base_path,
        label_name = config.label_name,
        label_message = config.label_message,
        button = config.button_text,
    )
}

pub fn render_success_page(config: &Config) -> String {
    let template = config.success_template.as_deref().unwrap_or(DEFAULT_SUCCESS_TEMPLATE);
    template
        .replace("{{title}}", &config.site_title)
        .replace("{{style}}", &style_tag(config))
        .replace("{{base}}", &config.base_path)
}

pub fn render_error_page(config: &Config, error: &str) -> String {
    render_sign_page(config, Some(error))
}

fn website_href(website: &str) -> String {
    if website.contains("://") {
        website.to_string()
    } else {
        format!("https://{website}")
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn render_entries(entries: &[Entry], config: &Config) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let mut html = String::from("<dl class=\"entries\">");
    for entry in entries {
        html.push_str(&render_entry(entry, config));
    }
    html.push_str("</dl>");
    html
}

/// Render an ISO date (`YYYY-MM-DD`) with the given strftime format.
///
/// Falls back to the raw ISO string if the date can't be parsed or the format
/// string is invalid, so a misconfigured `BOOK_DATE_FORMAT` degrades to the
/// stored value rather than panicking.
fn format_date(iso: &str, format: &str) -> String {
    use std::fmt::Write;
    match chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
        Ok(d) => {
            let mut out = String::new();
            // write! propagates fmt errors (from bad specifiers) instead of
            // panicking, unlike DelayedFormat::to_string().
            if write!(out, "{}", d.format(format)).is_ok() {
                out
            } else {
                iso.to_string()
            }
        }
        Err(_) => iso.to_string(),
    }
}

fn render_entry(entry: &Entry, config: &Config) -> String {
    let name = if config.enable_html_injection {
        entry.meta.name.clone()
    } else {
        escape_html(&entry.meta.name)
    };
    let name_html = if config.enable_website_links && !entry.meta.website.is_empty() {
        format!(
            "<a class=\"entry-website\" href=\"{}\">{}</a>",
            escape_html(&website_href(&entry.meta.website)),
            name
        )
    } else {
        name
    };
    let body = if config.enable_html_injection {
        entry.body.clone()
    } else {
        escape_html(&entry.body)
    };
    let ascii_drawing = if !entry.meta.drawing.is_empty() && config.renders_ascii(&entry.meta.drawing_style) {
        std::fs::read(config.data_dir.join("drawings").join(&entry.meta.drawing))
            .ok()
            .and_then(|bytes| ascii::render_png(&bytes, config.drawing_ascii_columns))
    } else {
        None
    };
    let drawing_html = if let Some(art) = ascii_drawing {
        format!(
            "<dd class=\"entry-drawing-wrap\"><pre class=\"entry-drawing entry-drawing-ascii\" role=\"img\" aria-label=\"Drawing by {}\">{}</pre></dd>",
            escape_html(&entry.meta.name),
            escape_html(&art)
        )
    } else if !entry.meta.drawing.is_empty() {
        format!(
            "<dd class=\"entry-drawing-wrap\"><img class=\"entry-drawing\" src=\"{}/drawings/{}\" alt=\"Drawing by {}\"></dd>",
            config.base_path,
            escape_html(&entry.meta.drawing),
            escape_html(&entry.meta.name)
        )
    } else {
        String::new()
    };
    let voice_note_html = if !entry.meta.voice_note.is_empty() {
        format!(
            "<dd class=\"entry-voice-note-wrap\"><audio controls preload=\"metadata\" src=\"{}/voice_notes/{}\"></audio></dd>",
            config.base_path,
            escape_html(&entry.meta.voice_note)
        )
    } else {
        String::new()
    };
    let body_html = if body.is_empty() {
        String::new()
    } else {
        format!("<dd class=\"entry-body\">{body}</dd>")
    };
    // Dates are stored as ISO (YYYY-MM-DD, optionally with a time suffix). The
    // visible date is rendered with the configured strftime format; the title
    // keeps the full ISO date so an unambiguous form is available on hover.
    let iso = &entry.meta.date[..10];
    let date = format_date(iso, &config.date_format);
    format!(
        "<dt class=\"entry-header\" id=\"{id}\" title=\"{iso}\"><span class=\"entry-date\">{date}</span><span class=\"entry-name\">{name_html}</span></dt>{body_html}{drawing_html}{voice_note_html}",
        id = escape_html(&entry.id),
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
            date_format: "%Y-%m-%d".into(),

            #[cfg(feature = "telegram")]
            telegram_bot_token: None,
            #[cfg(feature = "telegram")]
            telegram_chat_id: None,
            telegram_retry_interval: 20,
            telegram_retry_limit: 3,
            telegram_reminder_interval: 86400,
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
            canvas_width: 400,
            canvas_height: 200,
            drawing_ascii: false,
            drawing_image: true,
            drawing_ascii_default: true,
            drawing_ascii_columns: 80,
            enable_voice_notes: false,
            voice_note_max_duration: 20,
            message_required: false,
            drawing_required: false,
            voice_note_required: false,
            content_required: true,
            template: None,
            success_template: None,
            sign_template: None,
            style: String::new(),
            button_text: "sign".into(),
            label_name: "name".into(),
            label_website: "website (optional)".into(),
            label_message: "message (optional)".into(),
            label_drawing: "drawing (optional)".into(),
            label_voice_note: "voice note (optional)".into(),
            voice_note_record_text: "record".into(),
            base_path: String::new(),
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
                drawing_style: String::new(),
                voice_note: String::new(),
                status: Status::Approved,
            },
            body: body.into(),
        }
    }

    #[test]
    fn test_format_date_default_is_iso() {
        assert_eq!(format_date("2026-07-03", "%Y-%m-%d"), "2026-07-03");
    }

    #[test]
    fn test_format_date_custom_format() {
        assert_eq!(format_date("2026-07-03", "%d/%m/%y"), "03/07/26");
    }

    #[test]
    fn test_format_date_unsupported_specifier_falls_back_to_iso() {
        // A time specifier can't be filled from a date-only value; it must
        // degrade to the stored value rather than panicking.
        assert_eq!(format_date("2026-07-03", "%H:%M"), "2026-07-03");
    }

    #[test]
    fn test_format_date_unparseable_falls_back() {
        assert_eq!(format_date("not-a-date", "%d/%m/%y"), "not-a-date");
    }

    #[test]
    fn test_render_entry_uses_configured_date_format() {
        let mut config = test_config();
        config.date_format = "%d/%m/%y".into();
        let entry = make_entry("alice", "2026-04-09", "Hello!");
        let html = render_entry(&entry, &config);
        // Visible date reformatted; title keeps the ISO form.
        assert!(html.contains(r#"<span class="entry-date">09/04/26</span>"#));
        assert!(html.contains(r#"title="2026-04-09""#));
    }

    #[test]
    fn test_render_default_template() {
        let config = test_config();
        let html = render_page(DEFAULT_TEMPLATE, &config, &[], "");
        assert!(html.contains("<title>test</title>"));
        assert!(html.contains("href=\"/sign\""));
    }

    #[test]
    fn test_render_default_sign_template() {
        let mut config = test_config();
        config.base_path = "/guestbook".into();
        let html = render_sign_page(&config, None);
        assert!(html.contains("<title>test</title>"));
        assert!(html.contains("guestbook-form"));
        assert!(html.contains("href=\"/guestbook/\""));
        assert!(!html.contains("<p class=\"guestbook-error\">"));
    }

    #[test]
    fn test_render_sign_page_escapes_error() {
        let config = test_config();
        let html = render_sign_page(&config, Some("<b>bad</b>"));
        assert!(html.contains("<p class=\"guestbook-error\">&lt;b&gt;bad&lt;/b&gt;</p>"));
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
        assert!(html.contains("id=\"test\""));
        assert!(html.contains("<dl class=\"entries\">"));
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
        config.button_text = "submit".into();
        config.label_name = "Name:".into();
        let form = render_form(&config);
        assert!(form.contains("submit"));
        assert!(form.contains("Name:"));
    }

    #[test]
    fn test_render_form_pairs_name_and_website() {
        let form = render_form(&test_config());
        let pair = &form[form.find("guestbook-pair").unwrap()..form.find("for=\"message\"").unwrap()];
        assert!(pair.contains("name=\"name\""));
        assert!(pair.contains("name=\"website\""));
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
    fn test_render_entry_prepends_scheme_to_bare_website() {
        let config = test_config();
        let mut entry = make_entry("bob", "2026-04-09", "Hi!");
        entry.meta.website = "bob.com".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains(r#"href="https://bob.com">"#));
    }

    #[test]
    fn test_render_entry_keeps_existing_scheme() {
        assert_eq!(website_href("http://bob.com"), "http://bob.com");
        assert_eq!(website_href("https://bob.com"), "https://bob.com");
        assert_eq!(website_href("gemini://bob.com"), "gemini://bob.com");
        assert_eq!(website_href("bob.com/page"), "https://bob.com/page");
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
        assert!(!html.contains("<script>alert("));
    }

    #[test]
    fn test_render_form_uses_base_path_for_submit() {
        let mut config = test_config();
        config.base_path = "/guestbook".into();
        let form = render_form(&config);
        assert!(form.contains("action=\"/guestbook/submit\""));
        assert!(!form.contains("action=\"/submit\""));
    }

    #[test]
    fn test_render_form_default_submit_path_unchanged() {
        let config = test_config();
        let form = render_form(&config);
        assert!(form.contains("action=\"/submit\""));
    }

    #[test]
    fn test_render_entry_drawing_uses_base_path() {
        let mut config = test_config();
        config.base_path = "/guestbook".into();
        let mut entry = make_entry("alice", "2026-04-09", "");
        entry.meta.drawing = "abc.png".into();
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], "");
        assert!(html.contains("src=\"/guestbook/drawings/abc.png\""));
        assert!(!html.contains("src=\"/drawings/abc.png\""));
    }

    #[test]
    fn test_render_entry_voice_note_uses_base_path() {
        let mut config = test_config();
        config.base_path = "/guestbook".into();
        let mut entry = make_entry("bob", "2026-04-09", "");
        entry.meta.voice_note = "1.webm".into();
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], "");
        assert!(html.contains("src=\"/guestbook/voice_notes/1.webm\""));
        assert!(!html.contains("src=\"/voice_notes/1.webm\""));
    }

    #[test]
    fn test_render_page_substitutes_base_placeholder() {
        let mut config = test_config();
        config.base_path = "/guestbook".into();
        let custom = "<html><a href=\"{{base}}/\">home</a> {{form}} {{entries}} {{style}} {{title}}</html>";
        let html = render_page(custom, &config, &[], "");
        assert!(html.contains("href=\"/guestbook/\""));
        assert!(!html.contains("{{base}}"));
    }

    #[test]
    fn test_render_success_page_substitutes_base_placeholder() {
        let mut config = test_config();
        config.base_path = "/guestbook".into();
        config.success_template = Some("<a href=\"{{base}}/\">back</a> {{title}} {{style}}".into());
        let html = render_success_page(&config);
        assert!(html.contains("href=\"/guestbook/\""));
        assert!(!html.contains("{{base}}"));
    }

    #[test]
    fn test_render_error_page_back_link_uses_base_path() {
        let mut config = test_config();
        config.base_path = "/guestbook".into();
        let html = render_error_page(&config, "oops");
        assert!(html.contains("href=\"/guestbook/\""));
        assert!(!html.contains("href=\"/\""));
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
    fn test_render_form_shows_drawing_toggle_when_enabled() {
        let mut config = test_config();
        config.enable_drawings = true;
        let form = render_form(&config);
        assert!(form.contains("guestbook-pad"));
        assert!(form.contains("guestbook-drawing-tools"));
        assert!(form.contains("guestbook-size-slider"));
        assert!(form.contains("name=\"drawing\""));
        assert!(!form.contains("guestbook-pad-art"));
        assert!(!form.contains("guestbook-swatch"));
    }

    #[test]
    fn test_render_form_ascii_pad() {
        let mut config = test_config();
        config.enable_drawings = true;
        config.drawing_ascii = true;
        config.drawing_ascii_columns = 84;
        config.canvas_width = 554;
        config.canvas_height = 253;
        let form = render_form(&config);
        assert!(form.contains("guestbook-pad-ascii"));
        assert!(form.contains("guestbook-pad-art"));
        assert!(form.contains("C=84,N=23"));
        assert!(form.contains(r#"R=" .'`^\",:;Il!i~+_-?][}{1)(|\\/tfj"#));
    }

    #[test]
    fn test_render_form_hides_drawing_when_disabled() {
        let config = test_config();
        let form = render_form(&config);
        assert!(!form.contains("guestbook-pad"));
        assert!(!form.contains("name=\"drawing\""));
    }

    #[test]
    fn test_render_entry_with_drawing() {
        let config = test_config();
        let mut entry = make_entry("alice", "2026-04-09", "Hello!");
        entry.meta.drawing = "2026-04-09-abc123.png".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains(r#"<img class="entry-drawing" src="/drawings/2026-04-09-abc123.png" alt="Drawing by alice">"#));
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
        assert!(html.contains(r#"<img class="entry-drawing" src="/drawings/2026-04-09-abc123.png" alt="Drawing by alice">"#));
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

    #[test]
    fn test_render_success_page_default() {
        let config = test_config();
        let html = render_success_page(&config);
        assert!(html.contains("<title>test</title>"));
        assert!(html.contains("pending approval"));
        assert!(html.contains("back"));
        assert!(html.contains("<style>"));
    }

    #[test]
    fn test_render_success_page_custom_template() {
        let mut config = test_config();
        config.success_template = Some("<p>{{title}} - sent!</p>".into());
        let html = render_success_page(&config);
        assert_eq!(html, "<p>test - sent!</p>");
    }

    #[test]
    fn test_render_error_page() {
        let config = test_config();
        let html = render_error_page(&config, "Name and message are required.");
        assert!(html.contains("<title>test</title>"));
        assert!(html.contains("Name and message are required."));
        assert!(html.contains("back"));
        assert!(html.contains("<style>"));
    }

    #[test]
    fn test_render_entry_with_voice_note() {
        let config = test_config();
        let mut entry = make_entry("alice", "2026-04-10", "Hello!");
        entry.meta.voice_note = "1744300800_abcd1234.webm".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains(r#"<audio controls preload="metadata" src="/voice_notes/1744300800_abcd1234.webm">"#));
    }

    #[test]
    fn test_render_entry_voice_note_works_without_html_injection() {
        let mut config = test_config();
        config.enable_html_injection = false;
        let mut entry = make_entry("alice", "2026-04-10", "<script>xss</script>");
        entry.meta.voice_note = "1744300800_abcd1234.webm".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(html.contains(r#"<audio controls preload="metadata" src="/voice_notes/1744300800_abcd1234.webm">"#));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_render_entry_without_voice_note() {
        let config = test_config();
        let entry = make_entry("alice", "2026-04-10", "Hello!");
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[entry], &form);
        assert!(!html.contains("<audio"));
    }

    #[test]
    fn test_render_form_shows_voice_note_when_enabled() {
        let mut config = test_config();
        config.enable_voice_notes = true;
        let form = render_form(&config);
        assert!(form.contains("guestbook-voice-controls"));
        assert!(form.contains("guestbook-voice-playback"));
        assert!(form.contains("name=\"voice_note\""));
    }

    #[test]
    fn test_render_form_hides_voice_note_when_disabled() {
        let config = test_config();
        let form = render_form(&config);
        assert!(!form.contains("guestbook-voice-controls"));
        assert!(!form.contains("name=\"voice_note\""));
    }

    #[test]
    fn test_render_entry_ascii_drawing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("drawings")).unwrap();
        let mut png = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut png, 2, 2);
            enc.set_color(png::ColorType::Grayscale);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&[0, 0, 0, 0]).unwrap();
        }
        std::fs::write(dir.path().join("drawings/abc.png"), &png).unwrap();
        let mut config = test_config();
        config.data_dir = dir.path().to_path_buf();
        config.drawing_ascii = true;
        config.drawing_ascii_columns = 2;
        let mut entry = make_entry("<alice>", "2026-04-09", "");
        entry.meta.drawing = "abc.png".into();
        let html = render_entry(&entry, &config);
        assert!(html.contains(r#"<pre class="entry-drawing entry-drawing-ascii" role="img" aria-label="Drawing by &lt;alice&gt;">$$</pre>"#));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn test_render_entry_ascii_falls_back_to_image_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config();
        config.data_dir = dir.path().to_path_buf();
        config.drawing_ascii = true;
        let mut entry = make_entry("alice", "2026-04-09", "");
        entry.meta.drawing = "gone.png".into();
        let html = render_entry(&entry, &config);
        assert!(html.contains(r#"<img class="entry-drawing" src="/drawings/gone.png""#));
    }

    fn ascii_entry(style: &str) -> (tempfile::TempDir, Entry) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("drawings")).unwrap();
        let mut png = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut png, 2, 2);
            enc.set_color(png::ColorType::Grayscale);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&[0, 0, 0, 0]).unwrap();
        }
        std::fs::write(dir.path().join("drawings/abc.png"), &png).unwrap();
        let mut entry = make_entry("alice", "2026-04-09", "");
        entry.meta.drawing = "abc.png".into();
        entry.meta.drawing_style = style.into();
        (dir, entry)
    }

    fn renders_ascii_entry(ascii: bool, image: bool, ascii_default: bool, style: &str) -> bool {
        let (dir, entry) = ascii_entry(style);
        let mut config = test_config();
        config.data_dir = dir.path().to_path_buf();
        config.drawing_ascii = ascii;
        config.drawing_image = image;
        config.drawing_ascii_default = ascii_default;
        config.drawing_ascii_columns = 2;
        render_entry(&entry, &config).contains("entry-drawing-ascii")
    }

    #[test]
    fn test_entry_style_honoured_when_both_allowed() {
        assert!(renders_ascii_entry(true, true, false, "ascii"));
        assert!(!renders_ascii_entry(true, true, true, "image"));
    }

    #[test]
    fn test_entry_without_style_uses_default() {
        assert!(renders_ascii_entry(true, true, true, ""));
        assert!(!renders_ascii_entry(true, true, false, ""));
    }

    #[test]
    fn test_forced_style_overrides_entry() {
        assert!(renders_ascii_entry(true, false, false, "image"));
        assert!(!renders_ascii_entry(false, true, true, "ascii"));
    }

    #[test]
    fn test_render_form_style_buttons_only_with_choice() {
        let mut config = test_config();
        config.enable_drawings = true;
        config.drawing_ascii = true;
        config.drawing_image = true;
        config.drawing_ascii_default = false;
        let form = render_form(&config);
        assert!(form.contains("data-style=\"ascii\""));
        assert!(form.contains("name=\"drawing_style\" value=\"image\""));
        config.drawing_image = false;
        let form = render_form(&config);
        assert!(!form.contains("data-style=\"ascii\""));
        assert!(form.contains("name=\"drawing_style\" value=\"ascii\""));
    }

    #[test]
    fn test_render_form_brush_reads_slider_by_class() {
        let mut config = test_config();
        config.enable_drawings = true;
        config.drawing_ascii = true;
        let form = render_form(&config);
        assert!(form.contains("sl=tools.querySelector('.guestbook-size-slider')"));
    }
}
