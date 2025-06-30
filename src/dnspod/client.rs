// Import constants from the sibling module.
use super::constants::*;
use crate::error::{DdnsError, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, instrument, trace, warn};

// --- API Response Models ---
#[derive(Deserialize, Debug)]
struct Status {
    code: String,
    message: String,
}

// Record.List
#[derive(Deserialize, Debug)]
struct ListResponse {
    status: Status,
    records: Option<Vec<ApiRecord>>,
}

//  Record.Create
#[derive(Deserialize, Debug)]
struct CreateResponse {
    status: Status,
    record: CreatedRecord,
}

//  Record.Modify
#[derive(Deserialize, Debug)]
struct ModifyResponse {
    status: Status,
    record: ApiRecord,
}

#[derive(Deserialize, Debug)]
pub struct CreatedRecord {
    pub id: u64,
    pub name: String,
    pub status: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApiRecord {
    pub id: u64,
    pub value: String,
    #[serde(rename = "type")]
    pub record_type: String,
}

// --- Internal State Management ---
#[derive(Default, Debug, Clone)]
struct RecordState {
    id: u64,
    ip: String,
}

#[derive(Default, Debug)]
struct DdnsState {
    a: Option<RecordState>,
    aaaa: Option<RecordState>,
}

/// An asynchronous, stateful client for the DNSPod DDNS service.
#[derive(Clone)]
pub struct DnspodClient {
    client: reqwest::Client,
    token: String,
    domain: String,
    sub_domain: String,
    state: Arc<Mutex<DdnsState>>,
}

impl DnspodClient {
    /// Initializes the client and fetches the initial state from DNSPod.
    #[instrument(skip(token))]
    pub async fn new(token: String, domain: String, sub_domain: String) -> Result<Self> {
        info!(
            "👋 Initializing DNSPod client for [{}.{}]",
            sub_domain, domain
        );
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let s = Self {
            client,
            token,
            domain,
            sub_domain,
            state: Arc::new(Mutex::new(DdnsState::default())),
        };

        // Fetch initial state to populate record IDs and IPs.
        s.refresh_state().await?;

        Ok(s)
    }

    /// The main update logic. Checks if the IP has changed and calls the appropriate API.
    #[instrument(skip(self), err, fields(ip = %current_ip))]
    pub async fn update_if_needed(&self, current_ip: &str) -> Result<()> {
        let is_ipv4 = match current_ip.parse::<IpAddr>() {
            Ok(IpAddr::V4(_)) => true,
            Ok(IpAddr::V6(_)) => false,
            Err(_) => return Err(DdnsError::InvalidIpFormat(current_ip.to_string())),
        };
        let record_type = if is_ipv4 { "A" } else { "AAAA" };

        // Lock the state for the duration of this check.
        let mut state = self.state.lock().await;
        let record_state_opt = if is_ipv4 {
            &mut state.a
        } else {
            &mut state.aaaa
        };

        match record_state_opt {
            Some(cached_state) => {
                // We have a cached state (ID and IP).
                if cached_state.ip == current_ip {
                    trace!(
                        "✅ [{}] IP has not changed from '{}'. No update needed.",
                        record_type, cached_state.ip
                    );
                    return Ok(());
                }
                info!(
                    "🔄 [{}] IP has changed from '{}' to '{}'. Updating record...",
                    record_type, cached_state.ip, current_ip
                );
                match self
                    .modify_record(record_type, cached_state.id, current_ip)
                    .await
                {
                    Ok(_) => {
                        // Update successful, update cached IP.
                        cached_state.ip = current_ip.to_string();
                    }
                    Err(DdnsError::Api { code, .. }) if code == "8" => {
                        // "Record ID Error (8)"! Our state is stale.
                        warn!("❗️Record ID is outdated. Refreshing state and retrying...");
                        // Drop the lock before calling another method on `self` to avoid deadlock.
                        drop(state);
                        self.refresh_state().await?;
                        // After refreshing, the state might be correct now.
                        // The next tick of the loop will handle the update.
                        // Or we could retry immediately here. For simplicity, we'll let the next tick handle it.
                        return Err(DdnsError::Api {
                            code: "8".to_string(),
                            message:
                                "State refreshed after Record ID error. The next cycle will retry."
                                    .to_string(),
                        });
                    }
                    Err(e) => return Err(e),
                }
            }
            None => {
                // No cached state, means no record exists. Create it.
                info!(
                    "✨ No {} No existing record found. Creating new record with IP '{}'...",
                    record_type, current_ip
                );
                drop(state); // Drop lock before async operation
                let created_record = self.create_record(record_type, current_ip).await?;
                info!(
                    "✅ Successfully created new record. ID: {}, Name: {}, Status: {}",
                    created_record.id, created_record.name, created_record.status
                );
                let mut state = self.state.lock().await; // Re-acquire lock
                let new_state = RecordState {
                    id: created_record.id,
                    ip: current_ip.to_string(),
                };
                if is_ipv4 {
                    state.a = Some(new_state);
                } else {
                    state.aaaa = Some(new_state);
                }
            }
        }
        Ok(())
    }

    /// Fetches all records and updates the internal state.
    #[instrument(skip(self))]
    async fn refresh_state(&self) -> Result<()> {
        trace!("🌐 Refreshing local record state from DNSPod...");
        let records = self.list_records().await?;
        let mut state = self.state.lock().await;

        // Reset current state
        state.a = None;
        state.aaaa = None;

        for record in records {
            let record_state = RecordState {
                id: record.id,
                ip: record.value,
            };
            if record.record_type.eq_ignore_ascii_case("A") {
                state.a = Some(record_state);
            } else if record.record_type.eq_ignore_ascii_case("AAAA") {
                state.aaaa = Some(record_state);
            }
        }
        info!(
            "💾 State refreshed: A record found ({}), AAAA record found ({})",
            state.a.is_some(),
            state.aaaa.is_some()
        );
        Ok(())
    }

    /// (Private) Calls Record.List API.
    async fn list_records(&self) -> Result<Vec<ApiRecord>> {
        let mut params: HashMap<&'static str, &str> = HashMap::new();
        params.insert("login_token", &self.token);
        params.insert("format", "json");
        params.insert("domain", &self.domain);
        params.insert("sub_domain", &self.sub_domain);

        let url = format!("{}{}", API_BASE, API_RECORD_LIST);
        let res: ListResponse = robust_post(&self.client, &url, &params).await?;

        if res.status.code != "1" {
            if res.status.code == "10" {
                return Ok(vec![]);
            }
            return Err(DdnsError::Api {
                code: res.status.code,
                message: res.status.message,
            });
        }
        Ok(res.records.unwrap_or_default())
    }

    /// (Private) Calls Record.Modify API.
    async fn modify_record(
        &self,
        record_type: &str,
        record_id: u64,
        ip: &str,
    ) -> Result<ApiRecord> {
        let mut params: HashMap<&'static str, &str> = HashMap::new();
        let record_id_str = record_id.to_string();

        params.insert("login_token", &self.token);
        params.insert("format", "json");
        params.insert("domain", &self.domain);
        params.insert("record_id", &record_id_str);
        params.insert("sub_domain", &self.sub_domain);
        params.insert("record_type", record_type);
        params.insert("record_line", "默认");
        params.insert("value", ip);

        let url = format!("{}{}", API_BASE, API_RECORD_MODIFY);
        let res: ModifyResponse = robust_post(&self.client, &url, &params).await?;

        if res.status.code != "1" {
            return Err(DdnsError::Api {
                code: res.status.code,
                message: res.status.message,
            });
        }
        Ok(res.record)
    }

    /// (Private) Calls Record.Create API.
    async fn create_record(&self, record_type: &str, ip: &str) -> Result<CreatedRecord> {
        let mut params: HashMap<&'static str, &str> = HashMap::new();
        params.insert("login_token", &self.token);
        params.insert("format", "json");
        params.insert("domain", &self.domain);
        params.insert("sub_domain", &self.sub_domain);
        params.insert("record_type", record_type);
        params.insert("record_line", "默认");
        params.insert("value", ip);

        let url = format!("{}{}", API_BASE, API_RECORD_CREATE);
        let res: CreateResponse = robust_post(&self.client, &url, &params).await?;
        if res.status.code != "1" {
            return Err(DdnsError::Api {
                code: res.status.code,
                message: res.status.message,
            });
        }

        Ok(res.record)
    }
}

/// A helper function for making robust POST requests to the DNSPod API.
/// It first gets the response as text, then tries to parse it, providing
/// a detailed error with the raw body on failure.
async fn robust_post<T: for<'de> Deserialize<'de>>(
    client: &reqwest::Client,
    url: &str,
    params: &HashMap<&'static str, &str>,
) -> Result<T> {
    let response = client.post(url).form(params).send().await?;

    let body_text = response.text().await?;

    match serde_json::from_str(&body_text) {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err(DdnsError::ApiResponseDecode {
            body: body_text,
            source: e,
        }),
    }
}
