use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};

static SERVER: &str = env!("DNS_SERVER"); // DNS server to update
static ZONE: &str = env!("DNS_ZONE"); // DNS zone to update (no trailing dot)
static DOMAINS: &str = env!("DOMAINS"); // Comma-separated list of domains to update
static TTL: Option<&str> = option_env!("TTL"); // TTL of the created A record (default 60)
static TSIG_HMAC: &str = env!("TSIG_HMAC"); // TSIG HMAC algorithm, e.g. "hmac-sha256"
static TSIG_KEY: &str = env!("TSIG_KEY"); // TSIG key name
static TSIG_SECRET: &str = env!("TSIG_SECRET"); // TSIG key secret (base64)

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

fn main() -> Result<()> {
    let ips = fetch_public_ip()?;
    println!("Fetched own public IP:");
    println!("  IPv4: {}", ips.v4);
    if let Some(ipv6) = ips.v6.as_ref() {
        println!("  IPv6: {ipv6}");
    } else {
        println!("  IPv6: n/a");
    }

    // Parse env variables
    let domains: Vec<&str> = DOMAINS.split(',').collect();
    let ttl: usize = TTL.unwrap_or("60").parse().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to parse TTL: {:?}", e);
        60
    });

    println!("Running DNS zone update:");
    println!("  Server: {SERVER}");
    println!("  Zone: {ZONE}");
    println!("  TSIG Key: {TSIG_KEY}");
    println!("  Domains: {domains:?}");
    println!("  TTL: {ttl}");

    let mut child = Command::new("nsupdate")
        .arg("-v")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()?;
    {
        let mut stdin = child.stdin.take().expect("Failed to get stdin of child");

        // Authenticate
        stdin.write_all(format!("key {TSIG_HMAC}:{TSIG_KEY} {TSIG_SECRET}\n").as_bytes())?;

        // Connect to server
        stdin.write_all(format!("server {SERVER}\n").as_bytes())?;

        // Select zone
        stdin.write_all(format!("zone {ZONE}.\n").as_bytes())?;

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
