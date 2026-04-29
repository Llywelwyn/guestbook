{ config, lib, pkgs, ... }:
let
  inherit (lib) mkOption mkEnableOption types mkIf mkMerge;
  cfg = config.services.guestbook;
in
{
  options.services.guestbook = {
    enable = mkEnableOption "guestbook service";

    package = mkOption {
      type = types.package;
      description = "The guestbook package to use.";
    };

    port = mkOption {
      type = types.port;
      default = 8123;
      description = "Port to listen on (binds to 127.0.0.1).";
    };

    dataDir = mkOption {
      type = types.str;
      default = "/srv/guestbook/data";
      description = "Directory for guestbook entry files.";
    };

    siteTitle = mkOption {
      type = types.str;
      default = "guestbook";
      description = "Site title shown in nav and page title.";
    };

    basePath = mkOption {
      type = types.str;
      default = "";
      example = "/guestbook";
      description = ''
        URL prefix the guestbook is mounted at. Empty serves at the domain root.
        When set, all routes (/, /submit, /drawings/*, /voice_notes/*) are
        mounted under the prefix, and form actions and asset URLs include it.
        Templates can interpolate the prefix with the {{base}} placeholder.
      '';
    };

    user = mkOption {
      type = types.str;
      default = "guestbook";
      description = "User to run the service as.";
    };

    group = mkOption {
      type = types.str;
      default = "guestbook";
      description = "Group to run the service as.";
    };

    caddy = {
      enable = mkEnableOption "Caddy reverse proxy for guestbook";

      domain = mkOption {
        type = types.str;
        description = "Domain for the Caddy virtual host.";
      };

      forwardAuth = {
        enable = mkEnableOption "forward_auth for Caddy";

        address = mkOption {
          type = types.str;
          description = "Address of the auth service (e.g. localhost:9090).";
        };

        uri = mkOption {
          type = types.str;
          default = "/api/auth";
          description = "URI to send auth subrequests to.";
        };

        copyHeaders = mkOption {
          type = types.listOf types.str;
          default = [];
          description = "Headers to copy from the auth response to the proxied request.";
        };
      };
    };

    submissions = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Allow new guestbook submissions. When false, the form is hidden and submissions are rejected.";
      };
    };

    websites = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Show website field in form and render website links in entries. When false, the input is hidden, submitted values are ignored, and existing links are not displayed.";
      };
    };

    drawing = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable the drawing canvas in the submission form. Stores PNG files in dataDir/drawings/.";
      };

      required = mkOption {
        type = types.bool;
        default = false;
        description = "Require a drawing on every submission. No-op when drawing.enable=false.";
      };
    };

    voice = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable voice note recording in the submission form. Stores WebM files in dataDir/voice_notes/.";
      };

      required = mkOption {
        type = types.bool;
        default = false;
        description = "Require a voice note on every submission. No-op when voice.enable=false.";
      };
    };

    message = {
      required = mkOption {
        type = types.bool;
        default = false;
        description = "Require a non-empty message on every submission. Individual checks take priority over content.required.";
      };
    };

    content = {
      required = mkOption {
        type = types.bool;
        default = true;
        description = "Require at least one of message, drawing, or voice note. Set to false to allow name-only submissions.";
      };
    };

    telegram = {
      enable = mkEnableOption "Telegram moderation notifications";

      botTokenFile = mkOption {
        type = types.path;
        description = "Path to a file containing the Telegram bot token.";
      };

      chatId = mkOption {
        type = types.int;
        description = "Telegram chat ID for moderation messages.";
      };

      retry = {
        interval = mkOption {
          type = types.int;
          default = 20;
          description = "Seconds between retry attempts for failed Telegram notifications.";
        };

        limit = mkOption {
          type = types.int;
          default = 3;
          description = "Maximum number of retry attempts for failed Telegram notifications.";
        };
      };

      reminderInterval = mkOption {
        type = types.int;
        default = 86400;
        description = "Seconds between pending entry reminders. Set to 0 to disable.";
      };
    };

    security = {
      htmlInjection = {
        enable = mkOption {
          type = types.bool;
          default = false;
          description = "Allow raw HTML/JS in entry names and message bodies. When false, HTML is escaped. Website URLs are always escaped.";
        };
      };

      honeypot = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Enable honeypot field for spam prevention.";
        };
      };

      captcha = {
        enable = mkEnableOption "captcha on submission form";

        question = mkOption {
          type = types.str;
          default = "";
          description = "Captcha question displayed as a label.";
        };

        answer = mkOption {
          type = types.str;
          default = "";
          description = "Captcha answer to validate against.";
        };

        exact = mkOption {
          type = types.bool;
          default = false;
          description = "Require exact match. When false, the answer just needs to be contained in the response.";
        };

        caseSensitive = mkOption {
          type = types.bool;
          default = false;
          description = "Require case-sensitive match.";
        };
      };
    };

    limits = {
      name.length = mkOption {
        type = types.int;
        default = 0;
        description = "Maximum length for names. 0 for unlimited.";
      };

      message.length = mkOption {
        type = types.int;
        default = 0;
        description = "Maximum length for messages. 0 for unlimited.";
      };

      website.length = mkOption {
        type = types.int;
        default = 0;
        description = "Maximum length for website URLs. 0 for unlimited.";
      };

      drawing = {
        width = mkOption {
          type = types.int;
          default = 320;
          description = "Drawing canvas width in pixels.";
        };

        height = mkOption {
          type = types.int;
          default = 200;
          description = "Drawing canvas height in pixels.";
        };
      };

      voice.duration = mkOption {
        type = types.int;
        default = 20;
        description = "Maximum voice note duration in seconds. Max file size is derived as duration * 10KB.";
      };
    };

    styles = {
      css = mkOption {
        type = types.str;
        default = "";
        description = "Custom CSS injected into a style tag. Use class names: .guestbook-form, .guestbook-prompt, .guestbook-label, .guestbook-input, .guestbook-textarea, .guestbook-button, .guestbook-canvas, .entry, .entry-header, .entry-date, .entry-name, .entry-website, .entry-body, .entry-drawing";
      };

      cssFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to a CSS file. Takes precedence over css.";
      };

      templateFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Custom HTML template file with {{title}}, {{form}}, {{entries}}, {{style}}, and {{base}} placeholders. Uses built-in default if null.";
      };

      successTemplateFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Custom success page template with {{title}}, {{style}}, and {{base}} placeholders. Uses built-in default if null.";
      };

      labels = {
        submit = mkOption {
          type = types.str;
          default = "Submit Entry";
          description = "Submit button text.";
        };

        name = mkOption {
          type = types.str;
          default = "Your name";
          description = "Label for the name field.";
        };

        website = mkOption {
          type = types.str;
          default = "Link a website (optional)";
          description = "Label for the website field.";
        };

        message = mkOption {
          type = types.str;
          default = "Leave a message (optional)";
          description = "Label for the message field.";
        };

        drawing = mkOption {
          type = types.str;
          default = "Leave a drawing (optional)";
          description = "Label for the drawing field (when drawing.enable=true).";
        };

        voice = mkOption {
          type = types.str;
          default = "Leave a voice note (optional)";
          description = "Label for the voice note field (when voice.enable=true).";
        };

        voiceRecord = mkOption {
          type = types.str;
          default = "Start recording";
          description = "Initial text on the voice note record button.";
        };
      };

      message = {
        width = mkOption {
          type = types.int;
          default = 320;
          description = "Message textarea width in pixels.";
        };

        height = mkOption {
          type = types.int;
          default = 150;
          description = "Message textarea height in pixels.";
        };
      };
    };
  };

  config = mkIf cfg.enable (mkMerge [
    {
      systemd.services.guestbook = {
        description = "Guestbook for ${cfg.siteTitle}";
        after = [ "network-online.target" ];
        wants = [ "network-online.target" ];
        wantedBy = [ "multi-user.target" ];
        environment = {
          BOOK_PORT = toString cfg.port;
          BOOK_DATA_DIR = cfg.dataDir;
          BOOK_SITE_TITLE = cfg.siteTitle;

          BOOK_ENABLE_SUBMISSIONS = if cfg.submissions.enable then "true" else "false";
          BOOK_ENABLE_WEBSITE_LINKS = if cfg.websites.enable then "true" else "false";
          BOOK_ENABLE_DRAWINGS = if cfg.drawing.enable then "true" else "false";
          BOOK_ENABLE_HTML_INJECTION = if cfg.security.htmlInjection.enable then "true" else "false";
          BOOK_ENABLE_HONEYPOT = if cfg.security.honeypot.enable then "true" else "false";
          BOOK_ENABLE_CAPTCHA = if cfg.security.captcha.enable then "true" else "false";
          BOOK_CAPTCHA_QUESTION = cfg.security.captcha.question;
          BOOK_CAPTCHA_ANSWER = cfg.security.captcha.answer;
          BOOK_CAPTCHA_EXACT = if cfg.security.captcha.exact then "true" else "false";
          BOOK_CAPTCHA_CASESENSITIVE = if cfg.security.captcha.caseSensitive then "true" else "false";
          BOOK_MAX_NAME_LENGTH = toString cfg.limits.name.length;
          BOOK_MAX_MESSAGE_LENGTH = toString cfg.limits.message.length;
          BOOK_MAX_WEBSITE_LENGTH = toString cfg.limits.website.length;
          BOOK_STYLE = cfg.styles.css;
          BOOK_BUTTON_TEXT = cfg.styles.labels.submit;
          BOOK_LABEL_NAME = cfg.styles.labels.name;
          BOOK_LABEL_WEBSITE = cfg.styles.labels.website;
          BOOK_LABEL_MESSAGE = cfg.styles.labels.message;
          BOOK_LABEL_DRAWING = cfg.styles.labels.drawing;
          BOOK_LABEL_VOICE_NOTE = cfg.styles.labels.voice;
          BOOK_VOICE_NOTE_RECORD_TEXT = cfg.styles.labels.voiceRecord;
          BOOK_CANVAS_WIDTH = toString cfg.limits.drawing.width;
          BOOK_CANVAS_HEIGHT = toString cfg.limits.drawing.height;
          BOOK_ENABLE_VOICE_NOTES = if cfg.voice.enable then "true" else "false";
          BOOK_VOICE_NOTE_MAX_DURATION = toString cfg.limits.voice.duration;
          BOOK_MESSAGE_REQUIRED = if cfg.message.required then "true" else "false";
          BOOK_DRAWING_REQUIRED = if cfg.drawing.required then "true" else "false";
          BOOK_VOICE_NOTE_REQUIRED = if cfg.voice.required then "true" else "false";
          BOOK_CONTENT_REQUIRED = if cfg.content.required then "true" else "false";
          BOOK_TEXTAREA_WIDTH = toString cfg.styles.message.width;
          BOOK_TEXTAREA_HEIGHT = toString cfg.styles.message.height;
          BOOK_BASE_PATH = cfg.basePath;
        } // lib.optionalAttrs (cfg.styles.cssFile != null) {
          BOOK_STYLE_FILE = cfg.styles.cssFile;
        } // lib.optionalAttrs (cfg.styles.templateFile != null) {
          BOOK_TEMPLATE = cfg.styles.templateFile;
        } // lib.optionalAttrs (cfg.styles.successTemplateFile != null) {
          BOOK_SUCCESS_TEMPLATE = cfg.styles.successTemplateFile;
        } // lib.optionalAttrs cfg.telegram.enable {
          BOOK_TELEGRAM_CHAT_ID = toString cfg.telegram.chatId;
          BOOK_TELEGRAM_RETRY_INTERVAL = toString cfg.telegram.retry.interval;
          BOOK_TELEGRAM_RETRY_LIMIT = toString cfg.telegram.retry.limit;
          BOOK_TELEGRAM_REMINDER_INTERVAL = toString cfg.telegram.reminderInterval;
        };
        serviceConfig = {
          Type = "simple";
          ExecStartPre = "+${pkgs.writeShellScript "guestbook-prepare" ''
            mkdir -p ${cfg.dataDir}/entries ${cfg.dataDir}/drawings ${cfg.dataDir}/voice_notes
            chown -R ${cfg.user}:${cfg.group} ${cfg.dataDir}
          ''}";
          Restart = "on-failure";
          User = cfg.user;
          Group = cfg.group;
          ReadWritePaths = [ cfg.dataDir ];
        };
        script = ''
          ${lib.optionalString cfg.telegram.enable ''
            export BOOK_TELEGRAM_BOT_TOKEN="$(< "${cfg.telegram.botTokenFile}")"
          ''}
          exec ${cfg.package}/bin/guestbook
        '';
      };

      users.users.${cfg.user} = {
        isSystemUser = true;
        group = cfg.group;
        home = cfg.dataDir;
      };

      users.groups.${cfg.group} = {};
    }

    (mkIf cfg.caddy.enable {
      services.caddy.virtualHosts.${cfg.caddy.domain}.extraConfig = ''
        ${lib.optionalString cfg.caddy.forwardAuth.enable ''
        forward_auth ${cfg.caddy.forwardAuth.address} {
            uri ${cfg.caddy.forwardAuth.uri}
            ${lib.optionalString (cfg.caddy.forwardAuth.copyHeaders != [])
              "copy_headers ${lib.concatStringsSep " " cfg.caddy.forwardAuth.copyHeaders}"}
        }
        ''}
        reverse_proxy localhost:${toString cfg.port}
        encode zstd gzip
      '';
    })
  ]);
}
