pub mod dnspod;
pub mod error;
pub mod probe;
pub mod utils;

// Export API
pub use dnspod::DnspodClient;
pub use error::{DdnsError, Result};
pub use probe::{NetworkProbe, NetworkStatus};
