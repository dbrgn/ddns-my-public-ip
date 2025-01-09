use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};

static NSUPDATE: Option<&str> = option_env!("NSUPDATE"); // The `nsupdate` binary to use (default `nsupdate`)

const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_RESET: &str = "\x1b[0m";

const GET_PUBLIC_IPV4_ENDPOINT: &str = "https://api.ipify.org";
const GET_PUBLIC_IPV6_ENDPOINT: &str = "https://api6.ipify.org";

struct PublicIps {
    v4: String,
    v6: Option<String>,
}

fn fetch_public_ip() -> Result<PublicIps> {
    let ipv4: String = reqwest::blocking::get(GET_PUBLIC_IPV4_ENDPOINT)
        .context("Failed to fetch public IPv4")?
        .text()
        .context("Failed to read public IPv4 response body")?;
    let ipv6: Option<String> = reqwest::blocking::get(GET_PUBLIC_IPV6_ENDPOINT)
        .map_err(|e| eprintln!("{ANSI_YELLOW}Note: Failed to fetch public IPv6: {e}{ANSI_RESET}"))
        .and_then(|response| {
            response
                .text()
                .map_err(|e| eprintln!("Failed to read public IPv6 response body: {e}"))
        })
        .ok();
    Ok(PublicIps { v4: ipv4, v6: ipv6 })
}

/// Return env variable [`name`]. If it is not defined, fall back to
/// [`default`]. If no default is passed in, an error will be printed and the
/// program will abort.
fn get_var(name: &str, default: Option<&str>) -> String {
    match (std::env::var(name), default) {
        (Ok(val), _) => val,
        (Err(_), Some(val)) => val.to_string(),
        (Err(_), None) => {
            eprintln!("{ANSI_RED}Missing environment variable: {name}{ANSI_RESET}");
            std::process::exit(1);
        }
    }
}

struct Config {
    /// The `nsupdate` compatible binary to use
    nsupdate: &'static str,
    /// DNS server to update
    server: String,
    /// DNS zone to update (no trailing dot)
    zone: String,
    /// Comma-separated list of domains to update
    domains: Vec<String>,
    /// TTL of the created A record (default 60)
    ttl: usize,
    /// TSIG HMAC algorithm, e.g. `hmac-sha256`
    tsig_hmac: String,
    /// TSIG key name
    tsig_key: String,
    /// TSIG key secret (base64)
    tsig_secret: String,
}

fn get_config() -> Config {
    // Get runtime variables
    let server = get_var("DNS_SERVER", None);
    let zone = get_var("DNS_ZONE", None);
    let domains = get_var("DOMAINS", None);
    let ttl = get_var("TTL", Some("60"));
    let tsig_hmac = get_var("TSIG_HMAC", None);
    let tsig_key = get_var("TSIG_KEY", None);
    let tsig_secret = get_var("TSIG_SECRET", None);

    // Get compile time variables
    let nsupdate = NSUPDATE.unwrap_or("nsupdate");

    // Parse string values
    let parsed_domains: Vec<String> = domains.split(',').map(String::from).collect();
    let parsed_ttl: usize = ttl.parse().unwrap_or_else(|_| {
        eprintln!(
            "{ANSI_YELLOW}Note: Failed to parse TTL {:?} as number, falling back to default value{ANSI_RESET}",
            ttl
        );
        60
    });

    Config {
        nsupdate,
        server,
        zone,
        domains: parsed_domains,
        ttl: parsed_ttl,
        tsig_hmac,
        tsig_key,
        tsig_secret,
    }
}

fn main() -> Result<()> {
    // Get config from env
    let config = get_config();

    // Fetch public IPs
    let ips = fetch_public_ip()?;
    println!("Fetched own public IP:");
    println!("  IPv4: {}", ips.v4);
    if let Some(ipv6) = ips.v6.as_ref() {
        println!("  IPv6: {ipv6}");
    } else {
        println!("  IPv6: n/a");
    }
    println!();

    println!("Running DNS zone update with '{}':", config.nsupdate);
    println!("  Server: {}", config.server);
    println!("  Zone: {}", config.zone);
    println!("  TSIG Key: {}", config.tsig_key);
    println!("  Domains: {:?}", config.domains);
    println!("  TTL: {}", config.ttl);
    println!();

    let mut child = Command::new(config.nsupdate)
        .arg("-v")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()
        .context(format!(
            "Failed to spawn '{}' child process",
            config.nsupdate
        ))?;
    {
        let mut stdin = child.stdin.take().expect("Failed to get stdin of child");

        // Destructure config
        let Config {
            server,
            zone,
            domains,
            ttl,
            tsig_hmac,
            tsig_key,
            tsig_secret,
            ..
        } = config;

        // Authenticate
        stdin.write_all(format!("key {tsig_hmac}:{tsig_key} {tsig_secret}\n").as_bytes())?;

        // Connect to server
        stdin.write_all(format!("server {server}\n").as_bytes())?;

        // Select zone
        stdin.write_all(format!("zone {zone}.\n").as_bytes())?;

        // Update IPv4
        for domain in &domains {
            stdin.write_all(format!("update delete {domain}. in A\n").as_bytes())?;
            stdin.write_all(format!("update add {domain}. {ttl} in A {}\n", ips.v4).as_bytes())?;
        }

        // Update IPv6
        for domain in &domains {
            stdin.write_all(format!("update delete {domain}. in AAAA\n").as_bytes())?;
            if let Some(ref ipv6) = ips.v6 {
                stdin.write_all(
                    format!("update add {domain}. {ttl} in AAAA {}\n", ipv6).as_bytes(),
                )?;
            }
        }

        // Submit
        stdin.write_all(b"send\n")?;
        stdin.write_all(b"quit\n")?;
    }

    // Wait for the command to finish
    let status = child.wait()?;
    if status.success() {
        println!("{ANSI_GREEN}✅ Zone update executed successfully!{ANSI_RESET}");
    } else {
        eprintln!("{ANSI_RED}Failed to send zone update: Exit status {status}{ANSI_RESET}");
        std::process::exit(1);
    }

    Ok(())
}
