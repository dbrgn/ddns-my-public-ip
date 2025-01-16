# ddns-my-public-ip

This short program sends a TSIG-signed dynamic zone update (RFC2136) to a DNS
server, which updates certain records with your public IP.

To determine the public IP, the API from <https://www.ipify.org/> is being
used, thanks!

## Config

Buildtime env variables:

- `NSUPDATE`: The `nsupdate` binary to use (default `nsupdate`)

Runtime env variables:

- `DNS_SERVER`: DNS server to update
- `DNS_ZONE`: DNS zone to update (no trailing dot)
- `DOMAINS`: Comma-separated list of domains to update (no trailing dot)
- `TTL`: TTL of the created A record (optional, default 60)
- `TSIG_HMAC`: TSIG HMAC algorithm, e.g. `hmac-sha256`
- `TSIG_KEY`: TSIG key name
- `TSIG_SECRET`: TSIG key secret (base64)


## Use with NixOS (Flakes)

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    ddns-my-public-ip = {
      url = "github:dbrgn/ddns-my-public-ip";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {nixpkgs, ddns-my-public-ip, ...}: {
    nixosConfigurations.my-hostname = nixpkgs.lib.nixosSystem {
      modules = [
        ddns-my-public-ip.nixosModules.default
        ({config, ...}: {
          services.ddns-my-public-ip = {
            enable = true;

            dnsServer = "example.com";
            dnsZone = "site1.example.com";
            domains = ["ip.site1.example.com" "webcam.site1.example.com"];
            ttl = 60;
            tsigHmac = "hmac-sha256";
            tsigKey = "mykey";
            tsigSecretFile = "${pkgs.writeText "ddns-tsig-secret" ''
              TSIG_SECRET=dGVzdC10c2lnLXNlY3JldAo=
            ''}";
            timerInterval = "1min";
          };
        })
      ];
    };
  };
}
```
