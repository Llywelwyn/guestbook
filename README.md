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

`guestbook` is a single binary that serves a single-page guestbook aimed at personal sites. There's a form for visitors to submit a name, message, and optionally a link to their own site. Visitors can also draw a picture or leave a voice note if those features are enabled. Entries are written to plain text files with TOML frontmatter, and are initially marked as pending. The frontmatter can be manually edited to mark entries as approved or denied, or a Telegram bot can be hooked up for notifications and moderation (drawings are sent as photos and voice notes as voice messages so you can review them before approving). Running the Telegram bot just requires handing over a bot token, and it'll run off the same binary.

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
            features.telegram = {
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

# Telegram bot token. Optional — if unset, telegram moderation is disabled.
# BOOK_TELEGRAM_BOT_TOKEN=your-bot-token-here

# Telegram chat ID for moderation messages. Required if bot token is set.
# BOOK_TELEGRAM_CHAT_ID=0

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

# Separator between guestbook entries.
# BOOK_SEPARATOR=------------------------------------------------------------

# Path to a CSS file. Takes precedence over BOOK_STYLE. Uses built-in default if unset.
# BOOK_STYLE_FILE=./templates/default.css

# Custom CSS injected into a style tag.
# Classes: .guestbook-form, .guestbook-prompt, .guestbook-label, .guestbook-input,
#          .guestbook-textarea, .guestbook-button, .entry-header, .entry-date, .entry-name,
#          .entry-website, .entry-body, .entry-separator
# BOOK_STYLE=

# Text shown above the form.
# BOOK_FORM_PROMPT=Thanks for visiting. Sign the guestbook!

# Submit button text.
# BOOK_BUTTON_TEXT=sign

# Label for the name field.
# BOOK_LABEL_NAME=Your name:

# Label for the website field.
# BOOK_LABEL_WEBSITE=Your website (optional):

# Label for the message field.
# BOOK_LABEL_MESSAGE=Your message:

# Message textarea width in pixels.
# BOOK_TEXTAREA_WIDTH=400

# Message textarea height in pixels.
# BOOK_TEXTAREA_HEIGHT=150

# Custom HTML template file with {{title}}, {{prompt}}, {{form}}, {{entries}}, and {{style}} placeholders.
# Uses built-in default if unset.
# BOOK_TEMPLATE=./templates/default.html

# Custom success page template shown after a successful submission.
# Supports {{title}} and {{style}} placeholders. Use <script> for dynamic behavior.
# Uses built-in templates/success.html if unset.
# BOOK_SUCCESS_TEMPLATE=./templates/success.html

# Enable drawing canvas in submission form. Drawings are stored as PNG files in DATA_DIR/drawings/.
# BOOK_ENABLE_DRAWINGS=false

# Drawing canvas width in pixels.
# BOOK_CANVAS_WIDTH=400

# Drawing canvas height in pixels.
# BOOK_CANVAS_HEIGHT=200

# Enable voice note recording in submission form. Voice notes are stored as WebM files in DATA_DIR/voice_notes/.
# BOOK_ENABLE_VOICE_NOTES=false

# Maximum voice note duration in seconds. Max file size is derived as duration * 10KB.
# BOOK_VOICE_NOTE_MAX_DURATION=20
```

#### NixOS Module

```nix
services.guestbook = {
  enable = false;
  # package = <package>;  -- required when enabled
  port = 8123;
  dataDir = "/srv/guestbook/data";
  siteTitle = "guestbook";
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

  features = {
    submissions.enable = true;
    websites.enable = true;
    drawing = {
      enable = false;
      canvasWidth = 400;
      canvasHeight = 200;
    };
    voiceNote = {
      enable = false;
      maxDuration = 20;
    };
    telegram = {
      enable = false;
      # botTokenFile = <path>;  -- required when enabled
      # chatId = <int>;         -- required when enabled
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
  };

  limits = {
    name = 0;
    message = 0;
    website = 0;
  };

  styles = {
    css = "";
    cssFile = null;
    templateFile = null;
    successTemplateFile = null;
    separator = "------------------------------------------------------------";
    greeting = "Thanks for visiting. Sign the guestbook!";
    labels = {
      submit = "sign";
      name = "Your name:";
      website = "Your website (optional):";
      message = "Your message:";
    };
    message = {
      width = 400;
      height = 150;
    };
  };
};
```

---

### Drawing

Set `BOOK_ENABLE_DRAWINGS=true` to add a drawing canvas to the form. Visitors draw with mouse or touch; on submit, the canvas is converted to a base64 PNG data URL in a hidden field. Drawings are stored as PNGs in `{data_dir}/drawings/` and rendered above the message body, independent of the HTML injection setting.

Server-side validation checks the PNG magic bytes (`\x89PNG\r\n\x1a\n`), then reads width/height from the IHDR chunk and rejects anything that doesn't match `BOOK_CANVAS_WIDTH` x `BOOK_CANVAS_HEIGHT`. Max file size is derived from canvas dimensions (`w * h * 4`, the raw RGBA ceiling). A 2MB request body limit is enforced on all form submissions.

When Telegram moderation is enabled, drawings are sent as photos in the notification so you can see them before approving.

---

### Voice Notes

Set `BOOK_ENABLE_VOICE_NOTES=true` to let visitors record a short audio clip alongside their message. Recording uses the browser's MediaRecorder API (WebM/Opus format). The form shows an "add a voice note" link that starts recording on click, with a timer counting up to the configured max duration (`BOOK_VOICE_NOTE_MAX_DURATION`, default 20 seconds). After recording, visitors can listen back, re-record, or discard.

Server-side validation checks the WebM magic bytes (`\x1a\x45\xdf\xa3`) and enforces a file size cap derived from the max duration (`duration * 10KB`). Voice notes are stored as WebM files in `{data_dir}/voice_notes/` and rendered as native `<audio>` elements below the entry header, independent of the HTML injection setting.

When Telegram moderation is enabled, voice notes are sent as voice messages in the notification so you can hear them before approving.

---

### Telegram

To enable Telegram moderation, create a bot via [@BotFather](https://t.me/BotFather) and set `BOOK_TELEGRAM_BOT_TOKEN` to the token it gives you. Set `BOOK_TELEGRAM_CHAT_ID` to the chat ID where you want notifications sent: the easiest way to find this is to message the bot and check the [getUpdates](https://api.telegram.org/bot<token>/getUpdates) endpoint.

When a visitor submits an entry, the bot sends a message with the entry details and `/allow_<id>` and `/deny_<id>` commands, as well as any drawing or voice note attached. Tap either command to approve or deny. If you approve something and later want to deny it, or vice versa, just hit the opposite option and it'll work as expected.

---

### Entry Format

Each entry is a plain text file in `{data_dir}/entries/`. The filename is `{epoch}_{uuid}.txt`. If the entry has a drawing, the drawing is stored as `{epoch}_{uuid}.png` in `{data_dir}/drawings/` with the same prefix. Voice notes work the same way, stored as `{epoch}_{uuid}.webm` in `{data_dir}/voice_notes/`.

```
+++
name = "someone"
date = "2026-04-09T12:00:00"
website = "https://example.com"
drawing = "1744185600_abcd1234.png"
voice_note = "1744300800_abcd1234.webm"
status = "pending"
+++
Message body here.
```

The `status` field can be `pending`, `approved`, or `denied`. Only approved entries are displayed. The `drawing` and `voice_note` fields are empty when there's no drawing or voice note. To moderate without Telegram, just edit the file and change `status` to `approved` or `denied`.

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
    prompt  - The form prompt text (BOOK_FORM_PROMPT), wrapped in a
              <span class="guestbook-prompt">. Empty when submissions
              are disabled. Place anywhere relative to the form.
    form    - The submission form (labels, inputs, button). Controlled by
              BOOK_LABEL_NAME, BOOK_LABEL_WEBSITE, BOOK_LABEL_MESSAGE,
              BOOK_BUTTON_TEXT, BOOK_TEXTAREA_WIDTH, BOOK_TEXTAREA_HEIGHT.
              Empty when BOOK_ENABLE_SUBMISSIONS=false.
    entries - Approved guestbook entries, newest first. Entry separator
              controlled by BOOK_SEPARATOR.
    style   - Custom CSS from BOOK_STYLE or BOOK_STYLE_FILE, wrapped in
              a <style> tag. Uses built-in default.css when neither is set.

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
{{title}}

guestbook
=========

{{prompt}}
{{form}}

entries
=======
{{entries}}
</div>
</body>
</html>
```

#### Success Page

After a successful submission, visitors see a success page. The default is built into the binary from `templates/success.html`. To customise it, copy the file and point `BOOK_SUCCESS_TEMPLATE` at your copy. The `{{title}}` and `{{style}}` placeholders work the same as in the main template. Use `<script>` for dynamic behavior like showing the current time.

Validation errors (empty fields, wrong captcha, etc.) show a simple error page with the error message and a back link. This page is not customisable.

#### Default CSS

```css
/* Page container */
.page-container {
  max-width: 70ch;
  margin: 0 auto;
  padding: 1rem;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/* Form */
.guestbook-prompt {}
.guestbook-form {}
.guestbook-label {}
.guestbook-input {}
.guestbook-textarea {
  box-sizing: border-box;
}
.guestbook-button {
  display: block;
  margin-top: 1em;
}

/* Drawings */
.guestbook-canvas {
  border: 1px solid #000;
  cursor: crosshair;
  display: block;
}
.guestbook-drawing-content {
  display: block;
  margin-bottom: 1em;
}
.entry-drawing {
  max-width: 100%;
}

/* Voice notes */
.guestbook-voice-record.recording {
  color: red;
}
.guestbook-voice-timer {
  font-variant-numeric: tabular-nums;
}
audio {
  display: block;
  margin-top: 0.6em;
  height: 2em;
}

/* Entries */
.entry-header {}
.entry-date {}
.entry-name {}
.entry-website {}
.entry-body {}
.entry-separator {}
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
