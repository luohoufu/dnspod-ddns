mod args;

use anyhow::Result;
use args::Args;
use clap::Parser;
use dnspod::DnspodClient;
use std::env;
use tokio::time::{self, Duration};
use tracing::{info, trace, warn};

// The service for fetching our public IPv4 address.
const IPV4_URL: &str = "http://ns1.dnspod.net:6666";

/// Fetches the public IPv4 address from a given URL.
async fn get_public_ip(url: &str) -> Result<String> {
    Ok(reqwest::get(url).await?.text().await?.trim().to_owned())
}

/// Performs a single, complete DDNS check and update cycle.
async fn run_ddns_check(client: &DnspodClient) {
    trace!("Running DDNS check...");
    match get_public_ip(IPV4_URL).await {
        Ok(ip) => {
            if let Err(e) = client.update_record(&ip).await {
                warn!("Failed to update record: {}", e);
            }
        }
        Err(e) => trace!("Could not get public IP: {}", e),
    }
}

/// The main application entry point.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging, handling the unsafe `set_var` call.
    if env::var("RUST_LOG").is_err() {
        // This is safe at program startup before other threads are spawned.
        unsafe {
            env::set_var("RUST_LOG", "info,ddns=info,dnspod=info");
        }
    }
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let client = DnspodClient::new(args.token, args.domain, args.sub_domain)?;

    // --- Execution Flow ---
    if args.interval == 0 {
        info!("Running in single-shot mode...");
        run_ddns_check(&client).await;
        info!("DDNS check finished.");
    } else {
        info!("Starting DDNS check loop every {} seconds.", args.interval);
        let mut interval = time::interval(Duration::from_secs(args.interval));
        loop {
            // The first tick happens immediately, subsequent ticks wait for the interval.
            interval.tick().await;
            run_ddns_check(&client).await;
        }
    }

    Ok(())
}
