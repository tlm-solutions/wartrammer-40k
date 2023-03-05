{ config, pkgs, lib, ... }:
let
  cfg = config.TLMS.wartrammer;
in
{
  options.TLMS.wartrammer = with lib; {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Wether to enable wartrammer-40k
      '';
    };
    port = mkOption {
      type = types.port;
      default = 7680;
      description = ''
        On which port to expose wartrammer
      '';
    };
    user = mkOption {
      type = types.str;
      default = "wartrammer";
      description = ''
        As which user wartrammer should run
      '';
    };
    group = mkOption {
      type = types.str;
      default = "wartrammer";
      description = ''
        Which group wartrammer user is in
      '';
    };
    region = mkOption {
      type = types.int;
      default = -1;
      description = ''
        Which region does wartrammer run in
      '';
    };
  };

  config = lib.mkIf cfg.enable {

    services = {
      nginx = {
        enable = true;
        recommendedProxySettings = true;
        virtualHosts = {
          "wartrammer" = {
            locations = {
              "/api/" = {
                proxyPass = "http://127.0.0.1:${toString cfg.port}";
              };
              "/" = {
                root = pkgs.wartrammer-frontend;
                index = "index.html";
              };
              "/wartrammer-40k/" = {
                root = "/var/lib/";
                extraConfig = ''
                  autoindex on;
                '';
              };
              "/formatted.csv" = {
                root = "/var/lib/data-accumulator/";
                extraConfig = ''
                  autoindex on;
                '';
              };
            };
          };
        };
      };
    };

    systemd.services."setup-wartrammer" = {
      wantedBy = [ "multi-user.target" "data-accumulator.service" ];
      script = ''
        mkdir -p /var/lib/wartrammer-40k
        chmod 755 /var/lib/wartrammer-40k
        chown ${config.systemd.services.wartrammer.serviceConfig.User} /var/lib/wartrammer-40k
      '';

      serviceConfig = {
        Type = "oneshot";
      };
    };

    systemd.services."wartrammer" = {
      enable = true;
      wantedBy = [ "multi-user.target" "setup-wartrammer.service" ];
      script = ''
        exec ${pkgs.wartrammer-backend}/bin/wartrammer-40k --port ${toString cfg.port} --region ${toString cfg.region} &
      '';

      environment = {
        "PATH_DATA" = "/var/lib/wartrammer-40k/times.json";
        "IN_DATA" = "/var/lib/wartrammer-40k/formatted.csv";
        "OUT_DATA" = "/var/lib/wartrammer-40k/out.csv";
        "CSV_FILE_R09" = "/var/lib/wartrammer-40k/formatted.csv";
        "CSV_FILE_RAW" = "/var/lib/wartrammer-40k/raw.csv";
        "RUST_LOG" = "debug";
      };

      serviceConfig = {
        Type = "forking";
        User = cfg.user;
        Restart = "on-failure";
        StartLimitBurst = "2";
        StartLimitIntervalSec = "150s";
      };
    };

    users.users."${cfg.user}" = {
      name = "${cfg.user}";
      group = "${cfg.group}";
      description = "guy that runs wartrammer-40k";
      isSystemUser = true;
    };
  };
}
