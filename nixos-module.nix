self:
{
  config,
  pkgs,
  lib,
  ...
}:
with lib;
let
  cfg = config.services.ddns-my-public-ip;
in
{
  # Define the options that can be set for this module
  options.services.ddns-my-public-ip = {
    enable = mkEnableOption "ddns-my-public-ip";
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
    domains = mkOption { # TODO make this a list
      type = types.str;
      description = "Comma-separated list of domains to update (no trailing dot)";
      example = "ip.site1.example.com,webcam.site1.example.com";
    };
    ttl = mkOption {
      type = types.int;
      description = "TTL of the created A record (optional, default 60)";
      default = 60;
      example = 180;
    };
    tsigHmac = mkOption {
      type = types.str;
      description = "HMAC algorithm for TSIG authentication";
      example = "hmac-sha256";
    };
    tsigKey = mkOption {
      type = types.str;
      description = "Key name for TSIG authentication";
      example = "my-tsig-key";
    };
    tsigSecret = mkOption {
      type = types.str;
      description = "TSIG secret (base64-encoded)";
      example = "c2VjcmV0c2VjcmV0OTM4NzQ5ODI3MzQ5ODcyMwo=";
    };
    timerInterval = mkOption {
      type = types.str;
      description = "How often to run the systemd timer?";
      default = "5min";
      example = "5min";
    };
  };

  # Define what other settings and services should be active if a user enabled
  # this module
  config = mkIf cfg.enable {
    nixpkgs.overlays = [ self.overlays.default ];

    systemd.services.ddns-my-public-ip = {
      description = "A service that updates one or more domains on a DNS server with your public IP";
      wants = [ "network-online.target" ];
      after = [ "network-online.target" ];

      environment = {
        DNS_SERVER = cfg.dnsServer;
        DNS_ZONE = cfg.dnsZone;
        DOMAINS = cfg.domains;
        TTL = toString cfg.ttl;
        TSIG_HMAC = cfg.tsigHmac;
        TSIG_KEY = cfg.tsigKey;
        TSIG_SECRET = cfg.tsigSecret;
      };

      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${cfg.package}/bin/ddns-my-public-ip";
      };
    };

    systemd.timers.ddns-my-public-ip = {
      description = "Periodically run ddns-my-public-ip";
      wantedBy = [ "timers.target" ];

      timerConfig = {
        # Run 10s after boot
        OnBootSec = "10s";
        # Run periodically according to config
        OnUnitActiveSec = cfg.timerInterval;
      };

      # Wait for network before running
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
    };
  };
}
