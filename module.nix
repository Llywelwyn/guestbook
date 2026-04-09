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
