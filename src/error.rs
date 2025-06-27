use thiserror::Error;

/// Custom error types for the dnspod library.
#[derive(Error, Debug)]
pub enum DdnsError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Failed to parse JSON response: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("DNSPod API returned an error: {message} (Code: {code})")]
    Api { code: String, message: String },

    #[error("The provided IP address is not a valid IPv4 address: {0}")]
    InvalidIpFormat(String),
}

/// A convenience type alias for `Result` with our custom error type.
pub type Result<T> = std::result::Result<T, DdnsError>;
