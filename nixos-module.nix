self:
{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.ddns-my-public-ip;
in
with lib;
{
  options.services.ddns-my-public-ip = {
    enable = mkEnableOption "DDNS My Public IP";
    package = mkPackageOption pkgs "ddns-my-public-ip" { };

    dnsServer = mkOption {
      type = types.str;
      description = "DNS server to update";
      example = "example.com";
    };
    dnsZone = mkOption {
      type = types.str;
      description = "DNS zone to update (no trailing dot)";
      example = "site1.example.com";
    };
    domains = mkOption {
      type = types.str;
      description = "Comma-separated list of domains to update (no trailing dot)";
      example = "ip.site1.example.com,webcam.site1.example.com";
    };
    environmentFile = mkOption {
      description = ''
        Path to an EnvironmentFile containing required environment
        variables as per documentation.
      '';
      type = types.path;
    };
  };

  config = mkIf cfg.enable {
    nixpkgs.overlays = [ self.overlays.default ];

    systemd.services.ddns-my-public-ip = {
      # TODO: Full service configuration (run after network-online, etc etc).
      #       See https://search.nixos.org/options?channel=24.11&from=0&size=100&sort=relevance&type=packages&query=systemd.services.%3Cname%3E
      #       Example here (not oneShot, but otherwise nice with temp. user): https://github.com/zhaofengli/attic/blob/main/nixos/atticd.nix#L200
      description = "";
      wants = [ "network-online.target" ];

      environment = {
        DNS_SERVER = cfg.dnsServer;
        DNS_ZONE = cfg.dnsZone;
        DOMAINS = cfg.domains;
      };

      serviceConfig = {
        ExecStart = "${getExe cfg.package}";
        EnvironmentFile = cfg.environmentFile;
      };
    };

    systemd.timers.ddns-my-public-ip = {
      # TODO: See https://search.nixos.org/options?channel=24.11&from=0&size=100&sort=relevance&type=packages&query=systemd.timer
      description = "";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = cfg.startAt; # TODO: Config option?
      };

      # Wait for network before running
      after = "network-online.target";
      wants = "network-online.target";
    };
  };
}
