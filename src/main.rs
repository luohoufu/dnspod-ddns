mod args;

use anyhow::Result;
use args::Args;
use clap::Parser;
use dnspod::DnspodClient;
use reqwest::Client;
use std::env;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tokio::time::{self, Duration};
use tracing::{error, info, trace, warn};

// The single, dual-stack-aware IP detection service.
const IP_SERVICE_URL: &str = "https://test.ipw.cn";

/// Asynchronously gets the public IP address using a pre-configured client.
async fn get_public_ip(client: &Client, url: &str) -> Result<String> {
    let response = client.get(url).send().await?.error_for_status()?;

    Ok(response.text().await?.trim().to_string())
}

/// The main application entry point.
#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "info,ddns=info,dnspod=info");
        }
    }
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    // Initialize the stateful DnspodClient.
    let dnspod_client = DnspodClient::new(args.token, args.domain, args.sub_domain).await?;

    // --- 关键修正：为 IPv4 和 IPv6 创建独立的 reqwest 客户端 ---

    // Client that is forced to use IPv4
    let http_client_v4 = Client::builder()
        .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED)) // Bind to 0.0.0.0
        .timeout(Duration::from_secs(10))
        .build()?;

    // Client that is forced to use IPv6 (only build if needed)
    let http_client_v6 = if args.ipv6 {
        Some(
            Client::builder()
                .local_address(IpAddr::V6(Ipv6Addr::UNSPECIFIED)) // Bind to ::
                .timeout(Duration::from_secs(10))
                .build()?,
        )
    } else {
        None
    };

    // --- Execution Flow ---
    if args.interval == 0 {
        info!("Running in single-shot mode...");
        run_ddns_checks(&dnspod_client, &http_client_v4, http_client_v6.as_ref()).await;
        info!("DDNS check finished.");
    } else {
        info!("Starting DDNS check loop every {} seconds.", args.interval);
        let mut interval = time::interval(Duration::from_secs(args.interval));
        loop {
            interval.tick().await;
            run_ddns_checks(&dnspod_client, &http_client_v4, http_client_v6.as_ref()).await;
        }
    }

    Ok(())
}

/// Runs the DDNS checks for IPv4 and, if enabled, IPv6 concurrently.
async fn run_ddns_checks(
    dnspod_client: &DnspodClient,
    http_client_v4: &Client,
    http_client_v6: Option<&Client>,
) {
    info!("Running scheduled DDNS checks...");

    let dnspod_client_v4 = dnspod_client.clone();
    let http_client_v4 = http_client_v4.clone();

    let v4_handle = tokio::spawn(async move {
        trace!("[IPv4] Starting check...");
        if let Ok(ip) = get_public_ip(&http_client_v4, IP_SERVICE_URL).await {
            if let Err(e) = dnspod_client_v4.update_if_needed(&ip).await {
                warn!("[IPv4] Update process failed: {}", e);
            }
        } else {
            trace!("[IPv4] Could not get public IPv4.");
        }
    });

    if let Some(client_v6) = http_client_v6 {
        let dnspod_client_v6 = dnspod_client.clone();
        let http_client_v6 = client_v6.clone();
        tokio::spawn(async move {
            trace!("[IPv6] Starting check...");
            if let Ok(ip) = get_public_ip(&http_client_v6, IP_SERVICE_URL).await {
                if let Err(e) = dnspod_client_v6.update_if_needed(&ip).await {
                    warn!("[IPv6] Update process failed: {}", e);
                }
            } else {
                trace!("[IPv6] Could not get public IPv6 (network may not support it).");
            }
        });
    }

    if let Err(e) = v4_handle.await {
        error!("[IPv4] DDNS task panicked: {}", e);
    }
}
