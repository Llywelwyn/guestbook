[![Crates.io Version](https://img.shields.io/crates/v/guestbook)](https://crates.io/crates/guestbook)
[![Crates.io License](https://img.shields.io/crates/l/guestbook)](./LICENSE)

`guestbook` is a self-hosted guestbook web service with:
- entries stored in plaintext,
- notifications and moderation via [Telegram](#telegram),
- spam prevention via honeypot and/or [captcha](#captcha),
- completely customisable [styling](#customisation),

and more, written in Rust, and inspired by [t0.vc/g](https://t0.vc/g).

`guestbook` is a single binary that serves a single-page guestbook aimed at personal sites. There's a form for visitors to submit a name, message, and optionally a link to their own site. Entries are written to plain text files with TOML frontmatter, and are initially marked as pending. The frontmatter can be manually edited to mark entries as approved or denied, or a Telegram bot can be hooked up for notifications and moderation. Running the Telegram bot just requires handing over a bot token, and it'll run off the same binary.

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
#          .guestbook-textarea, .guestbook-button, .entry-header, .entry-name,
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

# Number of rows for the message textarea.
# BOOK_TEXTAREA_ROWS=8

# Number of columns for the message textarea.
# BOOK_TEXTAREA_COLS=60

# Custom HTML template file with {{title}}, {{form}}, {{entries}}, and {{style}} placeholders.
# Uses built-in default if unset.
# BOOK_TEMPLATE=./templates/default.html
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
    forwardAuth = null; # e.g. "localhost:9090"
  };

  security = {
    enableSubmissions = true;
    enableHtmlInjection = false;
    enableWebsiteLinks = true;
    enableHoneypot = true;
    captcha = {
      enable = false;
      question = "";
      answer = "";
      exact = false;
      caseSensitive = false;
    };
  };

  telegram = {
    enable = false;
    # botTokenFile = <path>;  -- required when enabled
    # chatId = <int>;         -- required when enabled
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
    separator = "------------------------------------------------------------";
    greeting = "Thanks for visiting. Sign the guestbook!";
    labels = {
      submit = "sign";
      name = "Your name:";
      website = "Your website (optional):";
      message = "Your message:";
    };
    message = {
      rows = 8;
      cols = 60;
    };
  };
};
```

---

### Telegram

To enable Telegram moderation, create a bot via [@BotFather](https://t.me/BotFather) and set `BOOK_TELEGRAM_BOT_TOKEN` to the token it gives you. Set `BOOK_TELEGRAM_CHAT_ID` to the chat ID where you want notifications sent: the easiest way to find this is to message the bot and check the [getUpdates](https://api.telegram.org/bot<token>/getUpdates) endpoint.

When a visitor submits an entry, the bot sends a message with the entry details and `/allow_<id>` and `/deny_<id>` commands. Tap either to approve or deny. If you approve something and later want to deny it, or vice versa, just hit the opposite option and it'll work as expected.

---

### Entry Format

Each entry is a plain text file in `{data_dir}/entries/`. The filename is `{date}-{short_id}.txt`.

```
+++
name = "someone"
date = "2026-04-09"
website = "https://example.com"
status = "pending"
+++
Message body here.
```

The `status` field can be `pending`, `approved`, or `denied`. Only approved entries are displayed. To moderate without Telegram, just edit the file and change `status` to `approved` or `denied`.

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
    form    - The submission form (labels, inputs, button). Controlled by
              BOOK_FORM_PROMPT, BOOK_LABEL_NAME, BOOK_LABEL_WEBSITE,
              BOOK_LABEL_MESSAGE, BOOK_BUTTON_TEXT, BOOK_TEXTAREA_ROWS,
              BOOK_TEXTAREA_COLS. Empty when BOOK_ENABLE_SUBMISSIONS=false.
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

{{form}}

entries
=======
{{entries}}
</div>
</body>
</html>
```

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
.guestbook-textarea {}
.guestbook-button {}

/* Entries */
.entry-header {}
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
