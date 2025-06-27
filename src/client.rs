use crate::error::{DdnsError, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use tracing::{info, instrument, trace};

// --- API Response Models ---
#[derive(Deserialize, Debug)]
struct DnspodResponse {
    status: Status,
    record: Option<Record>,
}

#[derive(Deserialize, Debug)]
struct Status {
    code: String,
    message: String,
}

#[derive(Deserialize, Debug)]
pub struct Record {
    pub value: String,
}

/// An asynchronous, efficient client for the DNSPod DDNS service.
#[derive(Clone)]
pub struct DnspodClient {
    client: reqwest::Client,
    token: String,
    domain: String,
    sub_domain: String,
}

impl DnspodClient {
    pub fn new(token: String, domain: String, sub_domain: String) -> Result<Self> {
        info!("Initializing DNSPod client for [{}.{}]", sub_domain, domain);
        Ok(Self {
            client: reqwest::Client::new(),
            token,
            domain,
            sub_domain,
        })
    }

    /// Checks and updates a DNS record using the efficient `Record.Ddns` endpoint.
    #[instrument(skip(self), err, fields(ip = %ip, record_type))]
    pub async fn update_record(&self, ip: &str) -> Result<()> {
        let record_type = match ip.parse::<IpAddr>() {
            Ok(IpAddr::V4(_)) => "A",
            Ok(IpAddr::V6(_)) => "AAAA",
            Err(_) => return Err(DdnsError::InvalidIpFormat(ip.to_string())),
        };
        tracing::Span::current().record("record_type", record_type);
        trace!("Attempting to update record...");

        let mut params: HashMap<&'static str, &str> = HashMap::new();
        params.insert("login_token", &self.token);
        params.insert("format", "json");
        params.insert("domain", &self.domain);
        params.insert("sub_domain", &self.sub_domain);
        params.insert("record_type", record_type);
        params.insert("record_line", "默认");
        params.insert("value", ip);

        let response = self
            .client
            .post("https://dnsapi.cn/Record.Ddns")
            .form(&params)
            .send()
            .await?;

        let result: DnspodResponse = response.json().await?;

        if result.status.code != "1" {
            return Err(DdnsError::Api {
                code: result.status.code,
                message: result.status.message,
            });
        }

        let updated_ip = result.record.map_or("N/A".to_string(), |r| r.value);
        info!(
            "Successfully processed {} record. Current value is {}",
            record_type, updated_ip
        );
        Ok(())
    }
}
