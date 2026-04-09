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

      forwardAuth = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "URL for forward_auth (e.g. localhost:9090). When set, all requests are authenticated via forward_auth before proxying.";
      };
    };

    security = {
      enableSubmissions = mkOption {
        type = types.bool;
        default = true;
        description = "Allow new guestbook submissions. When false, the form is hidden and submissions are rejected.";
      };

      enableHtmlInjection = mkOption {
        type = types.bool;
        default = false;
        description = "Allow raw HTML/JS in entry names and message bodies. When false, HTML is escaped. Website URLs are always escaped.";
      };

      enableWebsiteLinks = mkOption {
        type = types.bool;
        default = true;
        description = "Show website field in form and render website links in entries. When false, the input is hidden, submitted values are ignored, and existing links are not displayed.";
      };

      enableHoneypot = mkOption {
        type = types.bool;
        default = true;
        description = "Enable honeypot field for spam prevention.";
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
        description = "Custom CSS injected into a style tag. Use class names: .guestbook-form, .guestbook-prompt, .guestbook-label, .guestbook-input, .guestbook-textarea, .guestbook-button, .entry-header, .entry-name, .entry-website, .entry-body, .entry-separator";
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

      separator = mkOption {
        type = types.str;
        default = "------------------------------------------------------------";
        description = "Separator between guestbook entries.";
      };

      greeting = mkOption {
        type = types.str;
        default = "Thanks for visiting. Sign the guestbook!";
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
          default = "Your name:";
          description = "Label for the name field.";
        };

        website = mkOption {
          type = types.str;
          default = "Your website (optional):";
          description = "Label for the website field.";
        };

        message = mkOption {
          type = types.str;
          default = "Your message:";
          description = "Label for the message field.";
        };
      };

      message = {
        rows = mkOption {
          type = types.int;
          default = 8;
          description = "Number of rows for the message textarea.";
        };

        cols = mkOption {
          type = types.int;
          default = 60;
          description = "Number of columns for the message textarea.";
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

          BOOK_ENABLE_HONEYPOT = if cfg.security.enableHoneypot then "true" else "false";
          BOOK_ENABLE_SUBMISSIONS = if cfg.security.enableSubmissions then "true" else "false";
          BOOK_ENABLE_HTML_INJECTION = if cfg.security.enableHtmlInjection then "true" else "false";
          BOOK_ENABLE_WEBSITE_LINKS = if cfg.security.enableWebsiteLinks then "true" else "false";
          BOOK_ENABLE_CAPTCHA = if cfg.security.captcha.enable then "true" else "false";
          BOOK_CAPTCHA_QUESTION = cfg.security.captcha.question;
          BOOK_CAPTCHA_ANSWER = cfg.security.captcha.answer;
          BOOK_CAPTCHA_EXACT = if cfg.security.captcha.exact then "true" else "false";
          BOOK_CAPTCHA_CASESENSITIVE = if cfg.security.captcha.caseSensitive then "true" else "false";
          BOOK_MAX_NAME_LENGTH = toString cfg.limits.name;
          BOOK_MAX_MESSAGE_LENGTH = toString cfg.limits.message;
          BOOK_MAX_WEBSITE_LENGTH = toString cfg.limits.website;
          BOOK_SEPARATOR = cfg.styles.separator;
          BOOK_STYLE = cfg.styles.css;
          BOOK_FORM_PROMPT = cfg.styles.greeting;
          BOOK_BUTTON_TEXT = cfg.styles.labels.submit;
          BOOK_LABEL_NAME = cfg.styles.labels.name;
          BOOK_LABEL_WEBSITE = cfg.styles.labels.website;
          BOOK_LABEL_MESSAGE = cfg.styles.labels.message;
          BOOK_TEXTAREA_ROWS = toString cfg.styles.message.rows;
          BOOK_TEXTAREA_COLS = toString cfg.styles.message.cols;
        } // lib.optionalAttrs (cfg.styles.cssFile != null) {
          BOOK_STYLE_FILE = cfg.styles.cssFile;
        } // lib.optionalAttrs (cfg.styles.templateFile != null) {
          BOOK_TEMPLATE = cfg.styles.templateFile;
        } // lib.optionalAttrs cfg.telegram.enable {
          BOOK_TELEGRAM_CHAT_ID = toString cfg.telegram.chatId;
        };
        serviceConfig = {
          Type = "simple";
          ExecStartPre = "+${pkgs.writeShellScript "guestbook-prepare" ''
            mkdir -p ${cfg.dataDir}/entries
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
        ${lib.optionalString (cfg.caddy.forwardAuth != null) ''
        forward_auth ${cfg.caddy.forwardAuth} {
            uri /api/auth
        }
        ''}
        reverse_proxy localhost:${toString cfg.port}
        encode zstd gzip
      '';
    })
  ]);
}
