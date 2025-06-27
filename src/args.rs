use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The domain name, e.g., "example.com"
    #[arg(short, long, env = "DNSPOD_DOMAIN")]
    pub domain: String,
    /// The sub-domain name, e.g., "home"
    #[arg(short, long, env = "DNSPOD_SUB_DOMAIN")]
    pub sub_domain: String,
    /// Your DNSPod API token, in "ID,Token" format
    #[arg(short, long, env = "DNSPOD_TOKEN")]
    pub token: String,
    /// Check interval in seconds. Set to 0 to run only once.
    #[arg(short, long, env = "DNSPOD_INTERVAL", default_value_t = 10)]
    pub interval: u64,
    /// Enable IPv6 (AAAA record) update.
    #[arg(long, env = "DNSPOD_IPV6_ENABLED", default_value_t = false)]
    pub ipv6: bool,
}
