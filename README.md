# ddns-my-public-ip

This short program sends a TSIG-signed dynamic zone update (RFC2136) to a DNS
server, which updates certain records with your public IP.

To determine the public IP, the API from <https://www.ipify.org/> is being
used, thanks!

## Config

Use env variables:

- `DNS_SERVER`: DNS server to update
- `DNS_ZONE`: DNS zone to update (no trailing dot)
- `DOMAINS`: Comma-separated list of domains to update (no trailing dot)
- `TTL`: TTL of the created A record (optional, default 60)
- `TSIG_HMAC`: TSIG HMAC algorithm, e.g. `hmac-sha256`
- `TSIG_KEY`: TSIG key name
- `TSIG_SECRET`: TSIG key secret (base64)
- `NSUPDATE`: The `nsupdate` binary to use (default `nsupdate`)


## Use with NixOS (Flakes)

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    ddns-my-public-ip.url = "github:dbrgn/ddns-my-public-ip";
  };
  outputs = {nixpkgs, ddns-my-public-ip, ...}: {
    nixosConfigurations.my-hostname = nixpkgs.lib.nixosSystem {
      modules = [
        ddns-my-public-ip.nixosModules.default
        ({config, ...}: {
          services.ddns-my-public-ip = {
            enable = true;
            domains = "a.example.com,b.example.com";
            # ... TODO: Add config options sample from the module.
          };
        })
      ];
    };
  };
}
```
