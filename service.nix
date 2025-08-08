{ self }:
{
  lib,
  config,
  pkgs,
  ...
}:
let
  botBin = lib.getExe self.packages.${pkgs.system}.default;
in
with lib;
{
  options.services.claude-discord-bot = {
    enable = mkEnableOption "Claude Discord Bot";

    discordTokenFile = mkOption {
      type = types.str;
      description = "Path containing the Discord bot token";
    };

    databasePath = mkOption {
      type = types.path;
      default = "/var/lib/claude-discord-bot/bot.redb";
      description = "Path to the redb database file";
    };

    logLevel = mkOption {
      type = types.str;
      default = "INFO";
      description = "Log level, one of (INFO, WARN, ERROR, DEBUG, TRACE)";
    };
  };

  config = mkIf config.services.claude-discord-bot.enable {
    systemd.services.claude-discord-bot = {
      description = "Claude Discord Bot";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        ExecStart = lib.concatStringsSep " " [
          "${botBin}"
          "--discord-token-file ${toString config.services.claude-discord-bot.discordTokenFile}"
          "--database-path ${config.services.claude-discord-bot.databasePath}"
          "--log-level ${config.services.claude-discord-bot.logLevel}"
        ];
        StateDirectory = "claude-discord-bot";
        StateDirectoryMode = "0700";
        Restart = "always";
        RestartSec = "5min";
        StartLimitBurst = 1;
        User = "claude-discord-bot";
        Group = "claude-discord-bot";
      };
    };

    users.users.claude-discord-bot = {
      isSystemUser = true;
      group = "claude-discord-bot";
    };

    users.groups.claude-discord-bot = { };
  };
}
