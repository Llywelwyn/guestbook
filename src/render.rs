use crate::config::Config;
use crate::entries::Entry;

pub const DEFAULT_TEMPLATE: &str = include_str!("../templates/default.html");
pub const DEFAULT_SUCCESS_TEMPLATE: &str = include_str!("../templates/success.html");
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
            "<label class=\"guestbook-label\" for=\"website\">{label}</label>\n<input class=\"guestbook-input\" id=\"website\" name=\"website\">\n",
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
        format!(
            r##"<span class="guestbook-label">{label}</span>
<span class="guestbook-drawing-wrap"><span class="guestbook-drawing-tools"></span><span class="guestbook-drawing-content"></span></span><input type="hidden" name="drawing"><script>(function(){{
  var inl=document.querySelector('.guestbook-drawing-tools'),
      cnt=document.querySelector('.guestbook-drawing-content'),
      hid=document.querySelector('[name=drawing]'),
      c,x,d=false,lx,ly,h=[],col='#000',sz=5;
  function pos(e){{var r=c.getBoundingClientRect(),sx=c.width/r.width,sy=c.height/r.height;return[(e.clientX-r.left)*sx,(e.clientY-r.top)*sy]}}
  function tpos(e){{var r=c.getBoundingClientRect(),t=e.touches[0],sx=c.width/r.width,sy=c.height/r.height;return[(t.clientX-r.left)*sx,(t.clientY-r.top)*sy]}}
  function save(){{if(h.length>=20)h.shift();h.push(x.getImageData(0,0,c.width,c.height))}}
  function dot(px,py){{x.beginPath();x.arc(px,py,sz/2,0,Math.PI*2);x.fillStyle=col;x.fill()}}
  function bindCanvas(){{
    x=c.getContext('2d');x.strokeStyle=col;x.lineWidth=sz;x.lineCap='round';x.lineJoin='round';
    c.addEventListener('mousedown',function(e){{save();d=true;var p=pos(e);lx=p[0];ly=p[1];dot(lx,ly)}});
    c.addEventListener('mousemove',function(e){{if(!d)return;var p=pos(e);x.beginPath();x.moveTo(lx,ly);x.lineTo(p[0],p[1]);x.stroke();lx=p[0];ly=p[1]}});
    c.addEventListener('mouseup',function(){{d=false}});
    c.addEventListener('mouseleave',function(){{d=false}});
    c.addEventListener('touchstart',function(e){{e.preventDefault();save();var p=tpos(e);lx=p[0];ly=p[1];dot(lx,ly)}});
    c.addEventListener('touchmove',function(e){{e.preventDefault();var p=tpos(e);x.beginPath();x.moveTo(lx,ly);x.lineTo(p[0],p[1]);x.stroke();lx=p[0];ly=p[1]}});
  }}
  var sw=[{{c:'#000',n:'black'}},{{c:'#e03131',n:'red'}},{{c:'#2f9e44',n:'green'}},{{c:'#1971c2',n:'blue'}}];
  sw.forEach(function(s,i){{
    var sp=document.createElement('span');
    sp.className='guestbook-swatch'+(i===0?' active':'');
    sp.style.background=s.c;
    sp.setAttribute('role','button');sp.setAttribute('aria-label',s.n);
    sp.addEventListener('click',function(){{
      inl.querySelectorAll('.guestbook-swatch').forEach(function(el){{el.classList.remove('active')}});
      sp.classList.add('active');col=s.c;x.strokeStyle=col;
    }});
    inl.appendChild(sp);
  }});
  var sl=document.createElement('input');
  sl.type='range';sl.className='guestbook-size-slider';sl.min='1';sl.max='20';sl.value='5';sl.setAttribute('aria-label','Brush size');
  sl.addEventListener('input',function(){{sz=parseInt(sl.value);x.lineWidth=sz}});
  inl.appendChild(document.createTextNode(' '));inl.appendChild(sl);
  inl.appendChild(document.createTextNode(' | '));
  var undo=document.createElement('a');undo.href='#';undo.textContent='undo';
  undo.addEventListener('click',function(e){{e.preventDefault();if(h.length)x.putImageData(h.pop(),0,0)}});
  inl.appendChild(undo);
  inl.appendChild(document.createTextNode(' | '));
  var clr=document.createElement('a');clr.href='#';clr.textContent='clear';
  clr.addEventListener('click',function(e){{
    e.preventDefault();h=[];x.clearRect(0,0,c.width,c.height);hid.value='';
  }});
  inl.appendChild(clr);
  c=document.createElement('canvas');c.className='guestbook-canvas';c.width={w};c.height={h};c.setAttribute('aria-label','Drawing canvas');
  cnt.appendChild(c);bindCanvas();
  c.closest('form').addEventListener('submit',function(){{
    var px=new Uint32Array(x.getImageData(0,0,c.width,c.height).data.buffer);
    if(px.some(function(v){{return v!==0}})){{hid.value=c.toDataURL('image/png')}}
  }});
}})();</script>"##,
            label = config.label_drawing,
            w = config.canvas_width,
            h = config.canvas_height,
        )
    } else {
        String::new()
    };

    let voice_note_section = if config.enable_voice_notes {
        format!(
            r##"<span class="guestbook-label">{label}</span>
<span class="guestbook-voice-wrap"><span class="guestbook-voice-controls"></span><span class="guestbook-voice-playback"></span></span><input type="hidden" name="voice_note"><script>(function(){{
  var maxDur={max_dur};
  var inl=document.querySelector('.guestbook-voice-controls'),
      pb=document.querySelector('.guestbook-voice-playback'),
      hid=document.querySelector('[name=voice_note]'),
      rec=null,chunks=[],iv=null,st=0;
  function fmt(s){{var m=Math.floor(s/60),sec=s%60;return m+':'+(sec<10?'0':'')+sec}}
  function setInit(){{
    if(rec&&rec.state==='recording'){{rec.stop();rec.stream.getTracks().forEach(function(t){{t.stop()}})}}
    rec=null;chunks=[];clearInterval(iv);iv=null;pb.innerHTML='';hid.value='';
    inl.innerHTML='';
    var a=document.createElement('a');a.href='#';a.className='guestbook-voice-record';
    a.textContent='{record}';
    a.addEventListener('click',function(e){{e.preventDefault();startRec()}});
    inl.appendChild(a);
  }}
  function setRec(){{
    inl.innerHTML='';
    var a=document.createElement('a');a.href='#';a.className='guestbook-voice-record recording';
    a.textContent='stop recording';
    a.addEventListener('click',function(e){{e.preventDefault();rec.stop();rec.stream.getTracks().forEach(function(t){{t.stop()}})}});
    inl.appendChild(a);inl.appendChild(document.createTextNode(' '));
    var t=document.createElement('span');t.className='guestbook-voice-timer';t.setAttribute('aria-live','polite');t.setAttribute('aria-label','Recording timer');inl.appendChild(t);
    st=Date.now();t.textContent=fmt(0)+' / '+fmt(maxDur);
    iv=setInterval(function(){{
      var el=Math.floor((Date.now()-st)/1000);t.textContent=fmt(el)+' / '+fmt(maxDur);
      if(el>=maxDur){{rec.stop();rec.stream.getTracks().forEach(function(t){{t.stop()}})}}
    }},250);
  }}
  function setResult(){{
    clearInterval(iv);iv=null;
    var blob=new Blob(chunks,{{type:'audio/webm;codecs=opus'}});
    inl.innerHTML='';
    var re=document.createElement('a');re.href='#';re.textContent='re-record';
    re.addEventListener('click',function(e){{e.preventDefault();setInit();startRec()}});
    var disc=document.createElement('a');disc.href='#';disc.textContent='discard';
    disc.addEventListener('click',function(e){{e.preventDefault();setInit()}});
    inl.appendChild(re);inl.appendChild(document.createTextNode(' | '));inl.appendChild(disc);
    var url=URL.createObjectURL(blob);
    var au=document.createElement('audio');au.controls=true;au.preload='metadata';au.src=url;
    pb.appendChild(au);
    var rd=new FileReader();rd.onload=function(){{hid.value=rd.result}};rd.readAsDataURL(blob);
  }}
  function startRec(){{
    chunks=[];hid.value='';pb.innerHTML='';
    navigator.mediaDevices.getUserMedia({{audio:true}}).then(function(stream){{
      rec=new MediaRecorder(stream,{{mimeType:'audio/webm;codecs=opus'}});
      rec.ondataavailable=function(e){{if(e.data.size>0)chunks.push(e.data)}};
      rec.onstop=function(){{setResult()}};
      rec.start();setRec();
    }}).catch(function(){{
      inl.innerHTML='';
      inl.appendChild(document.createTextNode('mic access denied'));
    }});
  }}
  setInit();
}})();</script>"##,
            label = config.label_voice_note,
            record = config.voice_note_record_text,
            max_dur = config.voice_note_max_duration,
        )
    } else {
        String::new()
    };

    format!(
        r#"<form class="guestbook-form" method="post" action="/submit" accept-charset="UTF-8">
<label class="guestbook-label" for="name">{label_name}</label>
<input class="guestbook-input" id="name" name="name" required>
{website_section}<label class="guestbook-label" for="message">{label_message}</label>
<textarea class="guestbook-textarea" id="message" name="message" style="width:{tw}px;height:{th}px"></textarea>
{drawing_section}{voice_note_section}{captcha_section}<input name="url" aria-hidden="true" style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0)" tabindex="-1" autocomplete="off"><button class="guestbook-button" type="submit">{button}</button>
</form>"#,
        label_name = config.label_name,
        website_section = website_section,
        label_message = config.label_message,
        tw = config.textarea_width,
        th = config.textarea_height,
        drawing_section = drawing_section,
        voice_note_section = voice_note_section,
        captcha_section = captcha_section,
        button = config.button_text,
    )
}

pub fn render_success_page(config: &Config) -> String {
    let template = config.success_template.as_deref().unwrap_or(DEFAULT_SUCCESS_TEMPLATE);
    let css = if config.style.is_empty() {
        DEFAULT_STYLE
    } else {
        &config.style
    };
    let style = format!("<style>\n{css}\n  </style>");
    template
        .replace("{{title}}", &config.site_title)
        .replace("{{style}}", &style)
}

pub fn render_error_page(config: &Config, error: &str) -> String {
    let css = if config.style.is_empty() {
        DEFAULT_STYLE
    } else {
        &config.style
    };
    let error = escape_html(error);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>
{css}
  </style>
</head>
<body>
<div class="page-container">
<p>{error}</p>
<p><a href="/">&#8592; back</a></p>
</div>
</body>
</html>"#,
        title = config.site_title,
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

fn render_entry(entry: &Entry, config: &Config) -> String {
    let name = if config.enable_html_injection {
        entry.meta.name.clone()
    } else {
        escape_html(&entry.meta.name)
    };
    let name_html = if config.enable_website_links && !entry.meta.website.is_empty() {
        format!(
            "<a class=\"entry-website\" href=\"{}\">{}</a>",
            escape_html(&entry.meta.website),
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
    let drawing_html = if !entry.meta.drawing.is_empty() {
        format!(
            "<dd class=\"entry-drawing-wrap\"><img class=\"entry-drawing\" src=\"/drawings/{}\" alt=\"Drawing by {}\"></dd>",
            escape_html(&entry.meta.drawing),
            escape_html(&entry.meta.name)
        )
    } else {
        String::new()
    };
    let voice_note_html = if !entry.meta.voice_note.is_empty() {
        format!(
            "<dd class=\"entry-voice-note-wrap\"><audio controls preload=\"metadata\" src=\"/voice_notes/{}\"></audio></dd>",
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
    let date = &entry.meta.date[..10];
    format!(
        "<dt class=\"entry-header\" id=\"{id}\" title=\"{date}\"><span class=\"entry-date\">{date}&emsp;</span><span class=\"entry-name\">{name_html}</span></dt>{body_html}{drawing_html}{voice_note_html}",
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
            enable_voice_notes: false,
            voice_note_max_duration: 20,
            template: None,
            success_template: None,
            style: String::new(),
            button_text: "sign".into(),
            label_name: "name".into(),
            label_website: "website (optional)".into(),
            label_message: "message (optional)".into(),
            label_drawing: "drawing (optional)".into(),
            label_voice_note: "voice note (optional)".into(),
            voice_note_record_text: "record".into(),
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
                voice_note: String::new(),
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
        config.form_prompt = "Leave a note!".into();
        config.button_text = "submit".into();
        config.label_name = "Name:".into();
        let form = render_form(&config);
        let html = render_page(DEFAULT_TEMPLATE, &config, &[], &form);
        assert!(html.contains("Leave a note!"));
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
        assert!(!html.contains("<script>alert("));
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
        assert!(form.contains("guestbook-drawing-wrap"));
        assert!(form.contains("guestbook-drawing-content"));
        assert!(form.contains("name=\"drawing\""));
    }

    #[test]
    fn test_render_form_hides_drawing_when_disabled() {
        let config = test_config();
        let form = render_form(&config);
        assert!(!form.contains("guestbook-drawing-wrap"));
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
        assert!(form.contains("guestbook-voice-wrap"));
        assert!(form.contains("guestbook-voice-controls"));
        assert!(form.contains("name=\"voice_note\""));
    }

    #[test]
    fn test_render_form_hides_voice_note_when_disabled() {
        let config = test_config();
        let form = render_form(&config);
        assert!(!form.contains("guestbook-voice-wrap"));
        assert!(!form.contains("name=\"voice_note\""));
    }
}
