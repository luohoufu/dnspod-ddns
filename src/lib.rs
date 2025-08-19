pub mod core;
pub mod error;
pub mod notify;
pub mod probe;
pub mod utils;

// Export API
pub use core::DnspodClient;
pub use error::{DdnsError, Result};
pub use notify::HttpClient;
pub use probe::{NetworkProbe, NetworkStatus};
