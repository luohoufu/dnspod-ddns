mod args;
mod dnspod;
mod error;
mod probe;

use anyhow::Result;
use args::Args;
use clap::Parser;
use dnspod::{API_BASE, DnspodClient};
use probe::{NetworkProbe, NetworkStatus};
use reqwest::Client;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::EnvFilter;

// The single, dual-stack-aware IP detection service.
const IP_SERVICE_URL: &str = "https://test.ipw.cn";

/// Asynchronously gets the public IP address using a pre-configured client.
async fn get_public_ip(client: &Client) -> Result<String> {
    let response = client
        .get(IP_SERVICE_URL)
        .send()
        .await?
        .error_for_status()?;

    Ok(response.text().await?.trim().to_string())
}

/// The main application entry point.
#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,ddns=info,dnspod=info"));

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .without_time()
        .with_env_filter(filter)
        .init();

    let args = Args::parse();
    let dnspod_client = DnspodClient::new(args.token, args.domain, args.sub_domain).await?;

    let http_client_v4 = Client::builder()
        .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        .timeout(Duration::from_secs(10))
        .build()?;

    let http_client_v6 = if args.ipv6 {
        Some(
            Client::builder()
                .local_address(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
                .timeout(Duration::from_secs(10))
                .build()?,
        )
    } else {
        None
    };

    if args.interval == 0 {
        info!("🚀 Running in single-shot mode...");
        run_ddns_checks(&dnspod_client, &http_client_v4, http_client_v6.as_ref()).await;
        info!("✅ DDNS check finished.");
    } else {
        info!(
            "🔄 Starting DDNS check loop, running every {} seconds.",
            args.interval
        );
        let mut interval = time::interval(Duration::from_secs(args.interval));
        let mut probe = NetworkProbe::new();
        loop {
            interval.tick().await;

            // 1. Wait for the core service to be healthy.
            //    This will block with exponential backoff until the service is reachable.
            let status = probe.wait_for_service(API_BASE).await;

            // 2. Log a message about the service status.
            if status == NetworkStatus::JustRecovered {
                info!(
                    "🌐 DNSPod service at '{}' has recovered. Proceeding with checks.",
                    API_BASE
                );
            } else {
                trace!("DNSPod service is online. Proceeding with checks.");
            }

            // 3. Run the DDNS checks for IPv4/IPv6 now that we know the service is up.
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
    debug!("🔎 Starting scheduled DDNS check cycle...");
    let mut tasks: Vec<JoinHandle<()>> = Vec::new();

    // --- IPv4 Task ---
    let dnspod_client_v4 = dnspod_client.clone();
    let http_client_v4 = http_client_v4.clone();
    let v4_task = tokio::spawn(async move {
        trace!("[IPv4] 🕵️ Starting check...");
        match get_public_ip(&http_client_v4).await {
            Ok(ip) => {
                if let Err(e) = dnspod_client_v4.update_if_needed(&ip).await {
                    warn!("🚨 [IPv4] Update process failed: {}", e);
                }
            }
            Err(e) => {
                trace!("[IPv4] 💨 Could not get public IPv4: {}", e);
            }
        }
    });
    tasks.push(v4_task);

    // --- IPv6 Task ---
    if let Some(client_v6) = http_client_v6 {
        let dnspod_client_v6 = dnspod_client.clone();
        let http_client_v6 = client_v6.clone();
        let v6_task = tokio::spawn(async move {
            trace!("[IPv6] 🕵️ Starting check...");
            match get_public_ip(&http_client_v6).await {
                Ok(ip) => {
                    if let Err(e) = dnspod_client_v6.update_if_needed(&ip).await {
                        warn!("🚨 [IPv6] Update process failed: {}", e);
                    }
                }
                Err(e) => {
                    trace!("[IPv6] 💨 Could not get public IPv6: {}", e);
                }
            }
        });
        tasks.push(v6_task);
    }

    for handle in tasks {
        if let Err(e) = handle.await {
            error!("💥 A DDNS task panicked: {}", e);
        }
    }

    debug!("🏁 DDNS check cycle finished.");
}
