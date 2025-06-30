use reqwest::Client;
use std::time::Duration;
use tokio::time;
use tracing::{info, trace, warn};

/// Represents the network status after a probe check.
#[derive(Debug, PartialEq, Eq)]
pub enum NetworkStatus {
    /// The service was already online during the last check.
    AlreadyOnline,
    /// The service has just recovered from an unreachable state.
    JustRecovered,
}

/// A stateful service connectivity probe that implements exponential backoff.
pub struct NetworkProbe {
    client: Client,
    consecutive_failures: u32,
    base_backoff_secs: u64,
    max_backoff_secs: u64,
}

impl NetworkProbe {
    /// Creates a new NetworkProbe with default settings.
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5)) // Short timeout for probes
                .build()
                .expect("Failed to build the network probe's HTTP client"),
            consecutive_failures: 0,
            base_backoff_secs: 5,  // Start with a 5-second backoff
            max_backoff_secs: 300, // Max backoff of 5 minutes
        }
    }

    /// Waits until the specified service is reachable and returns the status.
    ///
    /// This function will block asynchronously until it can successfully make a
    /// HEAD request to the `service_url`. It implements exponential backoff on failure.
    pub async fn wait_for_service(&mut self, service_url: &str) -> NetworkStatus {
        let was_down = self.consecutive_failures > 0;

        loop {
            trace!("Probing service connectivity at '{}'...", service_url);
            match self.client.head(service_url).send().await {
                Ok(response) if response.status().is_success() => {
                    if was_down {
                        info!("✅ Service at '{}' has become reachable.", service_url);
                        self.consecutive_failures = 0;
                        return NetworkStatus::JustRecovered;
                    } else {
                        self.consecutive_failures = 0;
                        return NetworkStatus::AlreadyOnline;
                    }
                }
                Ok(response) => {
                    self.handle_failure(
                        service_url,
                        format!("Probe failed with HTTP status: {}", response.status()),
                    )
                    .await;
                }
                Err(e) => {
                    self.handle_failure(
                        service_url,
                        format!("Probe encountered a network error: {}", e),
                    )
                    .await;
                }
            }
        }
    }

    /// (Private) Handles a failed probe by logging and sleeping with backoff.
    async fn handle_failure(&mut self, service_url: &str, reason: String) {
        self.consecutive_failures += 1;

        if self.consecutive_failures == 1 {
            warn!(
                "🚨 Service at '{}' is unreachable. Will retry with backoff. Reason: {}",
                service_url, reason
            );
        } else {
            trace!(
                "Service '{}' still down (failure #{}), reason: {}",
                service_url, self.consecutive_failures, reason
            );
        }

        let backoff_secs = (self.base_backoff_secs * 2u64.pow(self.consecutive_failures - 1))
            .min(self.max_backoff_secs);
        let backoff_duration = Duration::from_secs(backoff_secs);

        trace!("Waiting for {:?} before the next probe.", backoff_duration);
        time::sleep(backoff_duration).await;
    }
}
