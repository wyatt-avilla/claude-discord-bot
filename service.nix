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

    model = mkOption {
      type = types.str;
      default = "sonnet-4";
      description = "Claude model to use for the bot, one of (opus-4, sonnet-4, sonnet-3.7, sonnet-3.5, haiku-3.5)";
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
          "--model ${config.services.claude-discord-bot.model}"
          "--database-path ${config.services.claude-discord-bot.databasePath}"
          "--log-level ${config.services.claude-discord-bot.logLevel}"
        ];
        StateDirectory = "claude-discord-bot";
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
