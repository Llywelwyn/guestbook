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

    siteUrl = mkOption {
      type = types.str;
      description = "Base URL of the main site (for absolute nav links).";
    };

    telegramChatId = mkOption {
      type = types.int;
      description = "Telegram chat ID for moderation messages.";
    };

    telegramBotTokenFile = mkOption {
      type = types.path;
      description = "Path to a file containing the Telegram bot token.";
    };

    honeypot = mkOption {
      type = types.bool;
      default = true;
      description = "Enable honeypot field for spam prevention.";
    };

    maxNameLength = mkOption {
      type = types.int;
      default = 50;
      description = "Maximum length for names. 0 for unlimited.";
    };

    maxMessageLength = mkOption {
      type = types.int;
      default = 1000;
      description = "Maximum length for messages. 0 for unlimited.";
    };

    maxWebsiteLength = mkOption {
      type = types.int;
      default = 100;
      description = "Maximum length for website URLs. 0 for unlimited.";
    };

    openRegistration = mkOption {
      type = types.bool;
      default = true;
      description = "Allow new guestbook submissions. When false, the form is hidden and submissions are rejected.";
    };

    separator = mkOption {
      type = types.str;
      default = "------------------------------------------------------------";
      description = "Separator between guestbook entries.";
    };

    style = mkOption {
      type = types.str;
      default = "";
      description = "Custom CSS injected into a style tag. Use class names: .guestbook-form, .guestbook-prompt, .guestbook-label, .guestbook-input, .guestbook-textarea, .guestbook-button, .entry-header, .entry-name, .entry-website, .entry-body, .entry-separator";
    };

    styleFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Path to a CSS file. Takes precedence over style.";
    };

    formPrompt = mkOption {
      type = types.str;
      default = "If you visited my site, please sign my guestbook!";
      description = "Text shown above the form.";
    };

    buttonText = mkOption {
      type = types.str;
      default = "sign";
      description = "Submit button text.";
    };

    labelName = mkOption {
      type = types.str;
      default = "Your name:";
      description = "Label for the name field.";
    };

    labelWebsite = mkOption {
      type = types.str;
      default = "Your website (optional):";
      description = "Label for the website field.";
    };

    labelMessage = mkOption {
      type = types.str;
      default = "Your message:";
      description = "Label for the message field.";
    };

    textareaRows = mkOption {
      type = types.int;
      default = 8;
      description = "Number of rows for the message textarea.";
    };

    textareaCols = mkOption {
      type = types.int;
      default = 60;
      description = "Number of columns for the message textarea.";
    };

    templateFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Custom HTML template file with {{title}}, {{form}}, and {{entries}} placeholders. Uses built-in default if null.";
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
          BOOK_SITE_URL = cfg.siteUrl;
          BOOK_TELEGRAM_CHAT_ID = toString cfg.telegramChatId;
          BOOK_HONEYPOT = if cfg.honeypot then "true" else "false";
          BOOK_MAX_NAME_LENGTH = toString cfg.maxNameLength;
          BOOK_MAX_MESSAGE_LENGTH = toString cfg.maxMessageLength;
          BOOK_MAX_WEBSITE_LENGTH = toString cfg.maxWebsiteLength;
          BOOK_OPEN_REGISTRATION = if cfg.openRegistration then "true" else "false";
          BOOK_SEPARATOR = cfg.separator;
          BOOK_STYLE = cfg.style;
        } // lib.optionalAttrs (cfg.styleFile != null) {
          BOOK_STYLE_FILE = cfg.styleFile;
          BOOK_FORM_PROMPT = cfg.formPrompt;
          BOOK_BUTTON_TEXT = cfg.buttonText;
          BOOK_LABEL_NAME = cfg.labelName;
          BOOK_LABEL_WEBSITE = cfg.labelWebsite;
          BOOK_LABEL_MESSAGE = cfg.labelMessage;
          BOOK_TEXTAREA_ROWS = toString cfg.textareaRows;
          BOOK_TEXTAREA_COLS = toString cfg.textareaCols;
        } // lib.optionalAttrs (cfg.templateFile != null) {
          BOOK_TEMPLATE = cfg.templateFile;
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
          export BOOK_TELEGRAM_BOT_TOKEN="$(< "${cfg.telegramBotTokenFile}")"
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
        reverse_proxy localhost:${toString cfg.port}
        encode zstd gzip
      '';
    })
  ]);
}
