self: {
  lib,
  config,
  pkgs,
  ...
}: let
  settingsFormat = pkgs.formats.toml {};
in {
  options.services.aocbot = {
    enable = lib.mkEnableOption "aocbot";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.system}.default;
    };

    logLevel = lib.mkOption {
      type = lib.types.str;
      default = "info,matrix_sdk=warn";
    };

    settings = lib.mkOption {
      inherit (settingsFormat) type;
      default = {};
    };
  };

  config = let
    cfg = config.services.aocbot;

    settings = settingsFormat.generate "config.toml" cfg.settings;

    aocbot-setup = pkgs.writeShellScriptBin "aocbot-setup" ''
      export CONFIG_PATH=${settings}
      export RUST_LOG=${lib.escapeShellArg cfg.logLevel}
      ${lib.getExe pkgs.sudo} --preserve-env=CONFIG_PATH,RUST_LOG -u aocbot ${lib.getExe cfg.package} setup
    '';
  in
    lib.mkIf cfg.enable {
      systemd.services.aocbot = {
        wantedBy = ["multi-user.target"];
        wants = ["network-online.target"];
        after = ["network-online.target"];

        serviceConfig = {
          User = "aocbot";
          Group = "aocbot";
          StateDirectory = "aocbot";
        };

        environment = {
          CONFIG_PATH = "${settings}";
          RUST_LOG = cfg.logLevel;
        };

        script = ''
          ${lib.getExe cfg.package} run
        '';
      };

      users.users.aocbot = {
        isSystemUser = true;
        group = "aocbot";
      };
      users.groups.aocbot = {};

      services.aocbot.settings = {
        users = lib.mkDefault self.users;
        matrix.store_path = lib.mkDefault "/var/lib/aocbot/store";
      };

      environment.systemPackages = [aocbot-setup];
    };
}
