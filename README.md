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
