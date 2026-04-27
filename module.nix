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

    features = {
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

        canvasWidth = mkOption {
          type = types.int;
          default = 320;
          description = "Drawing canvas width in pixels.";
        };

        canvasHeight = mkOption {
          type = types.int;
          default = 200;
          description = "Drawing canvas height in pixels.";
        };
      };

      voiceNote = {
        enable = mkOption {
          type = types.bool;
          default = false;
          description = "Enable voice note recording in the submission form. Stores WebM files in dataDir/voice_notes/.";
        };

        maxDuration = mkOption {
          type = types.int;
          default = 20;
          description = "Maximum voice note duration in seconds. Max file size is derived as duration * 10KB.";
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
    };

    limits = {
      name = mkOption {
        type = types.int;
        default = 0;
        description = "Maximum length for names. 0 for unlimited.";
      };

      message = mkOption {
        type = types.int;
        default = 0;
        description = "Maximum length for messages. 0 for unlimited.";
      };

      website = mkOption {
        type = types.int;
        default = 0;
        description = "Maximum length for website URLs. 0 for unlimited.";
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
        description = "Custom HTML template file with {{title}}, {{form}}, {{entries}}, and {{style}} placeholders. Uses built-in default if null.";
      };

      successTemplateFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Custom success page template with {{title}} and {{style}} placeholders. Uses built-in default if null.";
      };

      greeting = mkOption {
        type = types.str;
        default = "";
        description = "Text shown above the form.";
      };

      labels = {
        submit = mkOption {
          type = types.str;
          default = "sign";
          description = "Submit button text.";
        };

        name = mkOption {
          type = types.str;
          default = "name";
          description = "Label for the name field (used as both screen-reader label and placeholder).";
        };

        website = mkOption {
          type = types.str;
          default = "website (optional)";
          description = "Label for the website field (used as both screen-reader label and placeholder).";
        };

        message = mkOption {
          type = types.str;
          default = "message";
          description = "Label for the message field (used as both screen-reader label and placeholder).";
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

          BOOK_ENABLE_SUBMISSIONS = if cfg.features.submissions.enable then "true" else "false";
          BOOK_ENABLE_WEBSITE_LINKS = if cfg.features.websites.enable then "true" else "false";
          BOOK_ENABLE_DRAWINGS = if cfg.features.drawing.enable then "true" else "false";
          BOOK_ENABLE_HTML_INJECTION = if cfg.features.security.htmlInjection.enable then "true" else "false";
          BOOK_ENABLE_HONEYPOT = if cfg.features.security.honeypot.enable then "true" else "false";
          BOOK_ENABLE_CAPTCHA = if cfg.features.security.captcha.enable then "true" else "false";
          BOOK_CAPTCHA_QUESTION = cfg.features.security.captcha.question;
          BOOK_CAPTCHA_ANSWER = cfg.features.security.captcha.answer;
          BOOK_CAPTCHA_EXACT = if cfg.features.security.captcha.exact then "true" else "false";
          BOOK_CAPTCHA_CASESENSITIVE = if cfg.features.security.captcha.caseSensitive then "true" else "false";
          BOOK_MAX_NAME_LENGTH = toString cfg.limits.name;
          BOOK_MAX_MESSAGE_LENGTH = toString cfg.limits.message;
          BOOK_MAX_WEBSITE_LENGTH = toString cfg.limits.website;
          BOOK_STYLE = cfg.styles.css;
          BOOK_FORM_PROMPT = cfg.styles.greeting;
          BOOK_BUTTON_TEXT = cfg.styles.labels.submit;
          BOOK_LABEL_NAME = cfg.styles.labels.name;
          BOOK_LABEL_WEBSITE = cfg.styles.labels.website;
          BOOK_LABEL_MESSAGE = cfg.styles.labels.message;
          BOOK_CANVAS_WIDTH = toString cfg.features.drawing.canvasWidth;
          BOOK_CANVAS_HEIGHT = toString cfg.features.drawing.canvasHeight;
          BOOK_ENABLE_VOICE_NOTES = if cfg.features.voiceNote.enable then "true" else "false";
          BOOK_VOICE_NOTE_MAX_DURATION = toString cfg.features.voiceNote.maxDuration;
          BOOK_TEXTAREA_WIDTH = toString cfg.styles.message.width;
          BOOK_TEXTAREA_HEIGHT = toString cfg.styles.message.height;
        } // lib.optionalAttrs (cfg.styles.cssFile != null) {
          BOOK_STYLE_FILE = cfg.styles.cssFile;
        } // lib.optionalAttrs (cfg.styles.templateFile != null) {
          BOOK_TEMPLATE = cfg.styles.templateFile;
        } // lib.optionalAttrs (cfg.styles.successTemplateFile != null) {
          BOOK_SUCCESS_TEMPLATE = cfg.styles.successTemplateFile;
        } // lib.optionalAttrs cfg.features.telegram.enable {
          BOOK_TELEGRAM_CHAT_ID = toString cfg.features.telegram.chatId;
          BOOK_TELEGRAM_RETRY_INTERVAL = toString cfg.features.telegram.retry.interval;
          BOOK_TELEGRAM_RETRY_LIMIT = toString cfg.features.telegram.retry.limit;
          BOOK_TELEGRAM_REMINDER_INTERVAL = toString cfg.features.telegram.reminderInterval;
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
          ${lib.optionalString cfg.features.telegram.enable ''
            export BOOK_TELEGRAM_BOT_TOKEN="$(< "${cfg.features.telegram.botTokenFile}")"
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
