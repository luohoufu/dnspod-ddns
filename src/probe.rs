use reqwest::Client;
use std::time::Duration;
use tokio::time;
use tracing::{info, trace, warn};

/// A simple, lightweight URL to check for general internet connectivity.
/// Using a high-availability service is recommended.
const PROBE_URL: &str = "https://baidu.com";

/// Represents the network status after a probe check.
/// The `PartialEq` and `Eq` traits are derived to allow for direct comparison `==`.
#[derive(Debug, PartialEq, Eq)]
pub enum NetworkStatus {
    /// The network was already online during the last check.
    AlreadyOnline,
    /// The network has just recovered from a down state.
    JustRecovered,
}

/// A stateful network connectivity probe that implements exponential backoff.
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
                .timeout(Duration::from_secs(5)) // Use a short timeout for probes
                .build()
                .expect("Failed to build the network probe's HTTP client"),
            consecutive_failures: 0,
            base_backoff_secs: 5, // Start with a 5-second backoff after the first failure
            max_backoff_secs: 300, // Max backoff of 5 minutes to avoid excessive waiting
        }
    }

    /// Waits until network connectivity is established and returns the status.
    ///
    /// This function will block asynchronously until it can successfully make a
    /// HEAD request to the probe URL. It implements exponential backoff on failure
    /// and informs the caller if the network has just recovered.
    pub async fn wait_for_network(&mut self) -> NetworkStatus {
        // Remember if the network was already in a failed state when we started.
        let was_down = self.consecutive_failures > 0;

        loop {
            trace!("Probing network connectivity at '{}'...", PROBE_URL);
            match self.client.head(PROBE_URL).send().await {
                Ok(response) if response.status().is_success() => {
                    // Success! Now, determine the status to return.
                    if was_down {
                        // If it was down before, it has just recovered.
                        info!("✅ Network connectivity has been restored.");
                        self.consecutive_failures = 0; // Reset failure counter
                        return NetworkStatus::JustRecovered;
                    } else {
                        // If it was not down, it's just normally online.
                        self.consecutive_failures = 0; // Ensure counter is zero
                        return NetworkStatus::AlreadyOnline;
                    }
                }
                Ok(response) => {
                    // Got a response, but it's not a success code (e.g., 404, 500).
                    // This still indicates a problem.
                    self.handle_failure(format!(
                        "Probe failed with HTTP status: {}",
                        response.status()
                    ))
                    .await;
                }
                Err(e) => {
                    // A lower-level network error occurred (e.g., DNS resolution failed, timeout).
                    self.handle_failure(format!("Probe encountered a network error: {}", e))
                        .await;
                }
            }
        }
    }

    /// (Private) Handles a failed probe attempt by logging and sleeping with backoff.
    async fn handle_failure(&mut self, reason: String) {
        self.consecutive_failures += 1;

        if self.consecutive_failures == 1 {
            // Log the very first failure in a series with a higher severity.
            warn!(
                "🚨 Network connectivity lost. Will retry with exponential backoff. Reason: {}",
                reason
            );
        } else {
            // Subsequent failures are less verbose to avoid log spam.
            trace!(
                "Network still down (failure #{}), reason: {}",
                self.consecutive_failures, reason
            );
        }

        // Calculate and apply the exponential backoff delay.
        let backoff_secs = (self.base_backoff_secs * 2u64.pow(self.consecutive_failures - 1))
            .min(self.max_backoff_secs);
        let backoff_duration = Duration::from_secs(backoff_secs);

        trace!(
            "Waiting for {:?} before the next network probe.",
            backoff_duration
        );
        time::sleep(backoff_duration).await;
    }
}
