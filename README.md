[![Crates.io Version](https://img.shields.io/crates/v/guestbook)](https://crates.io/crates/guestbook)
[![Crates.io License](https://img.shields.io/crates/l/guestbook)](./LICENSE)

`guestbook` is a self-hosted guestbook web service with:
- entries stored in plaintext,
- a [drawing canvas](#drawing) for visitors to sketch alongside their message,
- [voice notes](#voice-notes) for visitors to record a short audio clip,
- notifications and moderation via [Telegram](#telegram),
- spam prevention via honeypot and/or [captcha](#captcha),
- fairly customisable [styling](#customisation),

and more, written in Rust, and inspired by [t0.vc/g](https://t0.vc/g).

`guestbook` is a single-page guestbook designed for personal sites. There's a form for visitors to submit a name, and optionally a message, a link to their own site, a drawing, or a voice note. Entries are written to plain text files with TOML frontmatter, and are initially marked as pending. The frontmatter can be manually edited to mark entries as approved or denied, or a Telegram bot can be hooked up for notifications and moderation (drawings and voice notes are fetched on demand via `/drawing_<id>` and `/voice_note_<id>` so the chat doesn't fill up with attachments). Running the Telegram bot just requires handing over a bot token.

Everything is configured through environment variables (see [`.env.example`](#default-config) for the defaults). If you're hosting with Nix, there's a flake that can set up the `guestbook` service end-to-end, running on a systemd service with a Caddy reverse proxy. Optionally, just ignore the flake and set up all the extra stuff yourself.

Aesthetically, essentially all of the HTML and CSS can be configured. There's a default template included for both, but you can take them and change both to your liking. Just point the template and/or style variables at your replacements.

---

### Installation

<p>
  <sup>
    <a href="#build">Build</a>&nbsp;·
    <a href="#nixos">NixOS</a>
  </sup>
</p>

#### Build

`guestbook` is written in [Rust](https://www.rust-lang.org). Clone the repo and build with `cargo`.

```bash
git clone https://git.ily.rs/lew/guestbook
cd guestbook
cp .env.example .env  # edit with your values
cargo run --release
```

Alternatively, install directly from [crates.io](https://crates.io/crates/guestbook) with `cargo install guestbook`. The binary uses the current working directory for its `.env` and data, so run it from whichever directory you want it to operate out of.

This will run the site on localhost on the port you've configured, or `8123` by default. I'll leave exposing it to the web to you, but personally I run [my guestbook](https://ily.rs/g) through a reverse proxy with [Caddy](https://caddyserver.com).

#### NixOS

[NixOS](https://nixos.org) users can use the included flake, which builds the binary via [crane](https://github.com/ipetkov/crane) and exports a module that sets up the systemd service, user, and optionally a [Caddy](https://caddyserver.com) reverse proxy.

```nix
# flake.nix
{
  inputs.guestbook.url = "git+https://git.ily.rs/lew/guestbook";

  outputs = { self, nixpkgs, guestbook, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [
        guestbook.nixosModules.default
        {
          services.guestbook = {
            enable = true;
            package = guestbook.packages.x86_64-linux.default;
            siteTitle = "my guestbook";
            telegram = {
              enable = true;
              botTokenFile = "/run/secrets/guestbook-bot-token";
              chatId = 12345;
            };
            caddy = {
              enable = true;
              domain = "guestbook.example.com";
            };
          };
        }
      ];
    };
  };
}
```

---

### Configuration

`guestbook` is configured entirely through environment variables. For local development, copy `.env.example` to `.env`. For NixOS, the [module](#nixos-module) maps all options to environment variables for you.

Running `guestbook` with no env vars will give you a working guestbook on `localhost:8123` with the default config below. Notably, no Telegram moderation. That requires a bot token, and is probably the most important thing to set up.

#### Default Config

```bash
# Port to listen on (binds to 127.0.0.1).
# BOOK_PORT=8123

# Directory for guestbook entry files.
# BOOK_DATA_DIR=./data

# Site title shown in nav and page title.
# BOOK_SITE_TITLE=guestbook

# strftime format for entry dates (e.g. %d/%m/%y). The full ISO date is kept
# in each entry's title attribute. An invalid format falls back to the ISO date.
# BOOK_DATE_FORMAT=%Y-%m-%d

# Telegram bot token. Optional — if unset, telegram moderation is disabled.
# BOOK_TELEGRAM_BOT_TOKEN=your-bot-token-here

# Telegram chat ID for moderation messages. Required if bot token is set.
# BOOK_TELEGRAM_CHAT_ID=0

# Seconds between retry attempts for failed Telegram notifications.
# BOOK_TELEGRAM_RETRY_INTERVAL=20

# Maximum number of retry attempts for failed Telegram notifications.
# BOOK_TELEGRAM_RETRY_LIMIT=3

# Seconds between pending entry reminders. Set to 0 to disable.
# BOOK_TELEGRAM_REMINDER_INTERVAL=86400

# Enable honeypot field for spam prevention.
# BOOK_ENABLE_HONEYPOT=true

# Allow new guestbook submissions. When false, the form is hidden and submissions are rejected.
# BOOK_ENABLE_SUBMISSIONS=true

# Show website field in form and render website links in entries.
# When false, the input is hidden, submitted values are ignored, and existing links are not displayed.
# BOOK_ENABLE_WEBSITE_LINKS=true

# Allow raw HTML/JS in entry names and message bodies. When false, HTML is escaped.
# Website URLs are always escaped regardless of this setting.
# BOOK_ENABLE_HTML_INJECTION=false

# Enable captcha on submission form.
# BOOK_ENABLE_CAPTCHA=false

# Captcha question displayed as a label.
# BOOK_CAPTCHA_QUESTION=What is my name?

# Captcha answer to validate against.
# BOOK_CAPTCHA_ANSWER=lew

# Require exact match (true) or just "contains" (false).
# BOOK_CAPTCHA_EXACT=false

# Require case-sensitive match.
# BOOK_CAPTCHA_CASESENSITIVE=false

# Maximum length for names. 0 for unlimited.
# BOOK_MAX_NAME_LENGTH=0

# Maximum length for messages. 0 for unlimited.
# BOOK_MAX_MESSAGE_LENGTH=0

# Maximum length for website URLs. 0 for unlimited.
# BOOK_MAX_WEBSITE_LENGTH=0

# Path to a CSS file. Takes precedence over BOOK_STYLE. Uses built-in default if unset.
# BOOK_STYLE_FILE=./templates/default.css

# Custom CSS injected into a style tag.
# Classes: .guestbook-error, .guestbook-form, .guestbook-pair, .guestbook-field,
#          .guestbook-label, .guestbook-input, .guestbook-textarea, .guestbook-row,
#          .guestbook-button, .guestbook-tool, .guestbook-pad, .guestbook-pad-ascii,
#          .guestbook-pad-art, .guestbook-canvas, .guestbook-drawing-tools,
#          .guestbook-size-slider, .guestbook-voice-controls, .guestbook-voice-playback,
#          .entries, .entry-header, .entry-date, .entry-name, .entry-website,
#          .entry-body, .entry-drawing-wrap, .entry-drawing, .entry-drawing-ascii,
#          .entry-voice-note-wrap
# BOOK_STYLE=

# Submit button text.
# BOOK_BUTTON_TEXT=Submit Entry

# Label for the name field.
# BOOK_LABEL_NAME=Your name

# Label for the website field.
# BOOK_LABEL_WEBSITE=Link a website (optional)

# Label for the message field.
# BOOK_LABEL_MESSAGE=Leave a message (optional)

# Label for the drawing field (when BOOK_ENABLE_DRAWINGS=true).
# BOOK_LABEL_DRAWING=Leave a drawing (optional)

# Label for the voice note field (when BOOK_ENABLE_VOICE_NOTES=true).
# BOOK_LABEL_VOICE_NOTE=Leave a voice note (optional)

# Initial text on the voice note record button.
# BOOK_VOICE_NOTE_RECORD_TEXT=Start recording

# Custom HTML template file with {{title}}, {{form}}, {{entries}}, {{style}}, and {{base}} placeholders.
# Uses built-in default if unset.
# BOOK_TEMPLATE=./templates/default.html

# Custom sign page template, served at /sign. Supports {{title}}, {{error}}, {{form}},
# {{style}}, and {{base}} placeholders. Rejected submissions re-render this page with {{error}} filled.
# Uses built-in templates/sign.html if unset.
# BOOK_SIGN_TEMPLATE=./templates/sign.html

# Custom success page template shown after a successful submission.
# Supports {{title}}, {{style}}, and {{base}} placeholders. Use <script> for dynamic behavior.
# Uses built-in templates/success.html if unset.
# BOOK_SUCCESS_TEMPLATE=./templates/success.html

# URL prefix the guestbook is mounted at. Empty serves at the domain root.
# When set (e.g. /guestbook), all routes (/, /sign, /submit, /drawings/*, /voice_notes/*)
# are mounted under the prefix, and form actions and asset URLs include it.
# Templates can interpolate the prefix with the {{base}} placeholder.
# BOOK_BASE_PATH=

# Enable drawing canvas in submission form. Drawings are stored as PNG files in DATA_DIR/drawings/.
# BOOK_ENABLE_DRAWINGS=false

# Drawing canvas width in pixels.
# BOOK_CANVAS_WIDTH=320

# Drawing canvas height in pixels.
# BOOK_CANVAS_HEIGHT=200

# Allow drawings shown as normal images.
# BOOK_DRAWING_IMAGE=true

# Allow drawings shown as ASCII text, on the canvas while drawing and in entries.
# With both styles allowed, visitors choose per drawing; with one, it's forced.
# PNGs are always stored, so these can be changed freely.
# BOOK_DRAWING_ASCII=false

# Start the canvas in ASCII mode when visitors can choose. Also used for drawings
# with no recorded choice.
# BOOK_DRAWING_ASCII_DEFAULT=true

# Width of an ASCII drawing in characters. Rows follow from the drawing's aspect ratio.
# BOOK_DRAWING_ASCII_COLUMNS=80

# Enable voice note recording in submission form. Voice notes are stored as WebM files in DATA_DIR/voice_notes/.
# BOOK_ENABLE_VOICE_NOTES=false

# Maximum voice note duration in seconds. Max file size is derived as duration * 10KB.
# BOOK_VOICE_NOTE_MAX_DURATION=20

# Require a non-empty message field. Individual checks take priority over BOOK_CONTENT_REQUIRED.
# BOOK_MESSAGE_REQUIRED=false

# Require a drawing. No-op when BOOK_ENABLE_DRAWINGS=false.
# BOOK_DRAWING_REQUIRED=false

# Require a voice note. No-op when BOOK_ENABLE_VOICE_NOTES=false.
# BOOK_VOICE_NOTE_REQUIRED=false

# Require at least one of message, drawing, or voice note. Set to false to allow name-only submissions.
# BOOK_CONTENT_REQUIRED=true
```

#### NixOS Module

```nix
services.guestbook = {
  enable = false;
  # package = <package>;  -- required when enabled
  port = 8123;
  dataDir = "/srv/guestbook/data";
  siteTitle = "guestbook";
  dateFormat = "%Y-%m-%d"; # strftime; e.g. "%d/%m/%y"
  user = "guestbook";
  group = "guestbook";

  caddy = {
    enable = false;
    # domain = <str>;  -- required when enabled
    forwardAuth = {
      enable = false;
      # address = <str>;  -- required when enabled, e.g. "localhost:9090"
      uri = "/api/auth";
      copyHeaders = []; # e.g. [ "Remote-User" "Remote-Email" ]
    };
  };

  submissions.enable = true;
  websites.enable = true;

  drawing = {
    enable = false;
    required = false;
    image.enable = true;
    ascii = {
      enable = false;
      default = true;
      columns = 80;
    };
  };

  voice = {
    enable = false;
    required = false;
  };

  message.required = false;
  content.required = true;

  telegram = {
    enable = false;
    # botTokenFile = <path>;  -- required when enabled
    # chatId = <int>;         -- required when enabled
    retry = {
      interval = 20;
      limit = 3;
    };
    reminderInterval = 86400;
  };

  security = {
    htmlInjection.enable = false;
    honeypot.enable = true;
    captcha = {
      enable = false;
      question = "";
      answer = "";
      exact = false;
      caseSensitive = false;
    };
  };

  limits = {
    name.length = 0;
    message.length = 0;
    website.length = 0;
    drawing = {
      width = 320;
      height = 200;
    };
    voice.duration = 20;
  };

  styles = {
    css = "";
    cssFile = null;
    templateFile = null;
    signTemplateFile = null;
    successTemplateFile = null;
    labels = {
      submit = "Submit Entry";
      name = "Your name";
      website = "Link a website (optional)";
      message = "Leave a message (optional)";
      drawing = "Leave a drawing (optional)";
      voice = "Leave a voice note (optional)";
      voiceRecord = "Start recording";
    };
  };
};
```

---

### Drawing

Set `BOOK_ENABLE_DRAWINGS=true` to add a drawing canvas to the form. Visitors draw in black with mouse or touch, with a brush size slider and undo/clear buttons; on submit, the canvas is converted to a base64 PNG data URL in a hidden field. Drawings are stored as PNGs in `{data_dir}/drawings/` and rendered above the message body, independent of the HTML injection setting.

Server-side validation checks the PNG magic bytes (`\x89PNG\r\n\x1a\n`), then reads width/height from the IHDR chunk and rejects anything that doesn't match `BOOK_CANVAS_WIDTH` x `BOOK_CANVAS_HEIGHT`. Max file size is derived from canvas dimensions (`w * h * 4`, the raw RGBA ceiling). A 2MB request body limit is enforced on all form submissions.

Set `BOOK_DRAWING_ASCII=true` to allow drawings as ASCII text. With `BOOK_DRAWING_IMAGE` left on, the canvas gets ASCII and Normal buttons and each visitor picks; the choice is saved as `drawing_style` in the entry, and entries without one use `BOOK_DRAWING_ASCII_DEFAULT`. Set `BOOK_DRAWING_IMAGE=false` to force ASCII for every drawing, old ones included. In ASCII mode the canvas becomes an invisible layer over a grid of glyphs that redraws as the visitor draws, and entries show each stored PNG converted to the same glyphs server-side, `BOOK_DRAWING_ASCII_COLUMNS` characters wide. Darker cells get denser glyphs. The PNGs are always what's stored, so changing these settings loses nothing.

When Telegram moderation is enabled, the notification includes a `/drawing_<id>` command to view the drawing on demand.

---

### Voice Notes

Set `BOOK_ENABLE_VOICE_NOTES=true` to let visitors record a short audio clip alongside their message. Recording uses the browser's MediaRecorder API (WebM/Opus format). The form shows a record button that starts recording on click, with a timer counting up to the configured max duration (`BOOK_VOICE_NOTE_MAX_DURATION`, default 20 seconds). After recording, visitors can listen back, re-record, or discard.

Server-side validation checks the WebM magic bytes (`\x1a\x45\xdf\xa3`) and enforces a file size cap derived from the max duration (`duration * 10KB`). Voice notes are stored as WebM files in `{data_dir}/voice_notes/` and rendered as native `<audio>` elements below the entry header, independent of the HTML injection setting.

When Telegram moderation is enabled, the notification includes a `/voice_note_<id>` command to listen on demand.

---

### Telegram

To enable Telegram moderation, create a bot via [@BotFather](https://t.me/BotFather) and set `BOOK_TELEGRAM_BOT_TOKEN` to the token it gives you. Set `BOOK_TELEGRAM_CHAT_ID` to the chat ID where you want notifications sent: the easiest way to find this is to message the bot and check the [getUpdates](https://api.telegram.org/bot<token>/getUpdates) endpoint.

When a visitor submits an entry, the bot sends a notification with the entry details and moderation commands. If the send fails, it retries in the background (`BOOK_TELEGRAM_RETRY_INTERVAL`, `BOOK_TELEGRAM_RETRY_LIMIT`). A periodic reminder will remind you about any pending entries once a day by default (`BOOK_TELEGRAM_REMINDER_INTERVAL` seconds, 0 to disable).

#### Commands

```bash
# List pending, approved, or denied entries.
/pending
/approved
/denied

# View the full details of an entry.
/view_<id>

# View entry attachments, if they exist.
/drawing_<id>
/voice_note_<id>

# Approve and deny entries.
/allow_<id>
/deny_<id>

# Append a reply to an entry.
# Reply is a multi-line command. Your reply will be appended
# to the guestbook entry, prefixed by `>>`.
/reply_<id>
[response]

# Delete an entry.
/delete_<id>
/confirm_delete_<id>
```

---

### Entry Format

Each entry is a plain text file in `{data_dir}/entries/`. The filename is a 4-character base36 ID (e.g., `ab3c.txt`). Drawings and voice notes share the same ID (`ab3c.png`, `ab3c.webm`) in their respective directories. Entries can be anchor linked via `#id`.

```
+++
name = "someone"
date = "2026-04-09T12:00:00"
website = "https://example.com"
drawing = "ab3c.png"
drawing_style = "ascii"
voice_note = "ab3c.webm"
status = "approved"
+++
Message body here. This is what someone
entered into the 'message' field.

>>  This is a reply. You can append
>>  to a message manually, and format
>>  yourself, or /reply_<id> to the bot.
```

`status` is either `pending`, `approved`, or `denied`. Only approved entries are displayed. `drawing` and `voice_note` fields link to their respective attachments, or nothing if they're empty. `drawing_style` is `ascii` or `image`, recording how the visitor chose to show their drawing; it's omitted when ASCII drawings are off. State is all stored in these files so you can moderate however you like, either via the built-in bot or just by manually editing the `status` field yourself.

---

### Customisation

#### Default Template

```html
<!--
  Default guestbook template.
  Copy this file and point BOOK_TEMPLATE at your copy to customize.

  Placeholders are inserted with double curly braces, e.g. curly-title-curly.

  Available placeholders:

    title   - Site title (BOOK_SITE_TITLE). Useful in <title> and headings.
    form    - The submission form, for templates that want it inline rather
              than on the sign page (see sign.html). Empty when
              BOOK_ENABLE_SUBMISSIONS=false.
    entries - Approved guestbook entries, newest first.
    style   - Custom CSS from BOOK_STYLE or BOOK_STYLE_FILE, wrapped in
              a <style> tag. Uses built-in default.css when neither is set.
    base    - URL prefix the guestbook is mounted at (BOOK_BASE_PATH).
              Empty when serving at the domain root.

  See default.css for available CSS classes on rendered elements.
-->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{title}}</title>
  {{style}}
</head>
<body>
<div class="page-container">
<h1>{{title}}</h1>

<p><a href="{{base}}/sign">Sign the guestbook</a></p>

<h1>entries</h1>
{{entries}}
</div>
</body>
</html>
```

#### Sign Page

```html
<!--
  Default sign page, served at /sign.
  Copy this file and point BOOK_SIGN_TEMPLATE at your copy to customize.

  Available placeholders:

    title - Site title (BOOK_SITE_TITLE).
    error - A paragraph (class guestbook-error) explaining why the last submission
            was rejected. Empty on a fresh visit.
    form  - The submission form (labels, inputs, buttons). Controlled by
            BOOK_LABEL_NAME, BOOK_LABEL_WEBSITE, BOOK_LABEL_MESSAGE,
            BOOK_LABEL_DRAWING, BOOK_LABEL_VOICE_NOTE, BOOK_BUTTON_TEXT.
            Empty when BOOK_ENABLE_SUBMISSIONS=false.
    style - Custom CSS (same as the main template).
    base  - URL prefix the guestbook is mounted at (BOOK_BASE_PATH).
-->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{title}}</title>
  {{style}}
</head>
<body>
<div class="page-container">
<p><a href="{{base}}/">&#8592; back</a></p>
{{error}}
{{form}}
</div>
</body>
</html>
```

#### Success Page

```html
<!--
  Default success page shown after a guestbook submission.
  Copy this file and point BOOK_SUCCESS_TEMPLATE at your copy to customize.

  Available placeholders:

    title - Site title (BOOK_SITE_TITLE).
    style - Custom CSS (same as the main template).
    base  - URL prefix the guestbook is mounted at (BOOK_BASE_PATH).

  Everything else is static — write whatever you want. Use <script> for
  dynamic behavior like showing the current time.
-->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{title}}</title>
  {{style}}
</head>
<body>
<div class="page-container">
<p>Thanks! Your message is pending approval.</p>
<p><a href="{{base}}/">&#8592; back</a></p>
</div>
</body>
</html>
```

#### Default CSS

```css
/* Page container */
body {
  margin: 0;
  line-height: 1.5;
}
.page-container {
  max-width: 70ch;
  padding: 1rem;
  word-wrap: break-word;
}

/* Form */
.guestbook-error {
  font-weight: bold;
}
.guestbook-label {
  display: block;
  font-style: oblique;
}
.guestbook-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1em;
}
.guestbook-input,
.guestbook-textarea {
  display: block;
  box-sizing: border-box;
  width: 100%;
  margin-bottom: 0.6em;
  font: inherit;
}
.guestbook-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5em;
  margin-bottom: 0.6em;
}

/* Drawings */
.guestbook-pad {
  position: relative;
  width: fit-content;
  max-width: 100%;
  margin-bottom: 0.4em;
}
.guestbook-canvas {
  display: block;
  max-width: 100%;
  height: auto;
  border: 1px solid #000;
  background: #fff;
  cursor: crosshair;
  touch-action: none;
}
.guestbook-pad-ascii {
  border: 1px solid #000;
}
.guestbook-pad-art {
  margin: 0;
  line-height: 1;
}
.guestbook-pad-ascii .guestbook-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  border: 0;
  opacity: 0;
}
.guestbook-size-slider {
  width: 6em;
}
.entry-drawing {
  max-width: 100%;
  background: #fff;
}
.entry-drawing-ascii {
  margin: 0;
  line-height: 1;
  overflow: hidden;
}

/* Voice notes */
.guestbook-voice-playback:empty {
  display: none;
}
audio {
  display: block;
  height: 2em;
}

/* Entries */
.entries {
  margin: 0;
  line-height: 1;
}
.entries dt:not(:first-child) {
  margin-top: 0.5rem;
}
.entry-date {
  margin-right: 1em;
}
.entry-name {
  font-weight: bold;
}
.entry-website {}
.entry-body {
  white-space: pre-wrap;
}
```

---

### License

```
MIT License

Copyright (c) 2026 Lewis Wynne

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
