use crate::error::Result;
use tracing::{info, instrument, warn};

#[derive(Clone)]
pub struct HttpClient {
    http_client: reqwest::Client,
    url_template: String,
    auth_token: Option<String>,
}

impl HttpClient {
    pub fn new(url_template: String, auth_token: Option<String>) -> Result<Self> {
        Ok(Self {
            http_client: reqwest::Client::new(),
            url_template,
            auth_token,
        })
    }

    /// Sends a GET request with the new IP address in the URL.
    #[instrument(skip(self), name = "http_get_notify", fields(new_ip))]
    pub async fn notify(&self, new_ip: &str) -> Result<()> {
        tracing::Span::current().record("new_ip", &new_ip);
        info!("🚀 Sending notification to HTTP Server...");

        // The URL template should contain the placeholder for the IP address.
        // e.g., "https://.../update?ip={IP_ADDRESS}"
        let final_url = self.url_template.replace("{IP_ADDRESS}", new_ip);

        let mut request_builder = self.http_client.get(&final_url);

        // The token is correctly placed in the header, not the URL.
        if let Some(token) = &self.auth_token {
            request_builder = request_builder.bearer_auth(token);
        }

        match request_builder.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Notification sent successfully.");
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_else(|_| "N/A".to_string());
                    warn!("❗️ Notification failed. Status: {}, Body: {}", status, body);
                }
            }
            Err(e) => {
                warn!("❗️ Error sending notification request: {}", e);
            }
        }
        // As a notifier, we log errors but don't fail the main process.
        Ok(())
    }
}
