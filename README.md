English | [简体中文](./README_ZH.md)

# 🚀 DNSPod DDNS

A modern, fast, and reliable DDNS (Dynamic DNS) client for [DNSPod](https://www.dnspod.cn/), written in Rust. 🦀

[![CI](https://github.com/luohoufu/dnspod-ddns/actions/workflows/ci.yml/badge.svg)](https://github.com/luohoufu/dnspod-ddns/actions/workflows/ci.yml)
[![Release](https://github.com/luohoufu/dnspod-ddns/actions/workflows/release.yml/badge.svg)](https://github.com/luohoufu/dnspod-ddns/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`dnspod-ddns` automatically checks your public IP address and updates your DNSPod DNS records. It ensures your domain always points to your home network, server, or any device with a dynamic IP.

Additionally, it can send an **instant HTTP notification** to a service like a reverse proxy (e.g., OpenResty/Nginx), allowing it to immediately update its backend IP without waiting for DNS propagation.

---

## ✨ Features

- **Blazing Fast**: Built with Rust for minimal resource usage and high performance.
- **Intelligent & Robust**: Quietly waits for network recovery during outages using exponential backoff, preventing log spam and unnecessary API requests.
- **IPv4 & IPv6 Ready**: Simultaneously updates both `A` (IPv4) and `AAAA` (IPv6) records.
- **Instant HTTP Notifier (Optional)**: Can immediately notify a configurable HTTP endpoint (like a reverse proxy) after a successful IP update, bypassing DNS TTL delays.
- **Cross-Platform**: Pre-compiled binaries are available for Linux, macOS, and Windows.
- **Easy to Configure**: Configure via command-line arguments or environment variables.
- **Run Anywhere**: Works perfectly in a Docker container, on a Raspberry Pi, your NAS, or any server.

---

## 📦 Installation

You have multiple ways to get `dnspod-ddns`.

### 1. From GitHub Releases (Recommended)

The easiest way is to download a pre-compiled binary for your system from the [**GitHub Releases**](https://github.com/luohoufu/dnspod-ddns/releases) page.

1. Go to the [Releases page](https://github.com/luohoufu/dnspod-ddns/releases).
2. Download the archive for your operating system and architecture (e.g., `ddns-x86_64-unknown-linux-musl.tar.gz`).
3. Extract the archive, and you're ready to go!

```bash
# Example for Linux
wget https://github.com/luohoufu/dnspod-ddns/releases/latest/download/ddns-x86_64-unknown-linux-musl.tar.gz
tar -xzvf ddns-x86_64-unknown-linux-musl.tar.gz
sudo mv ddns /usr/local/bin/
```

### 2. From Source

```bash
git clone https://github.com/luohoufu/dnspod-ddns.git
cd dnspod-ddns
cargo build --release
# The binary will be in ./target/release/ddns
```

---

## 🚀 Usage

The tool is configured via command-line arguments or environment variables. All DNSPod-related arguments are mandatory.

### Command-Line Arguments

Run `ddns --help` to see all available options:

```text
A modern, async DDNS tool for DNSPod with IPv4/IPv6 support and an optional HTTP notifier.

Usage: ddns [OPTIONS] --token <TOKEN> --domain <DOMAIN> --sub-domain <SUB_DOMAIN>

Options:
--token <TOKEN>
Your DNSPod API token, in "ID,Token" format
[env: DNSPOD_TOKEN=]

--domain <DOMAIN>
The domain name for the DNSPod target, e.g., "example.com"
[env: DNSPOD_DOMAIN=]

--sub-domain <SUB_DOMAIN>
The sub-domain name for the DNSPod target, e.g., "home"
[env: DNSPOD_SUB_DOMAIN=]

--http-url <HTTP_URL>
(Optional) The URL for the HTTP GET notifier. Use ?ip={IP_ADDRESS} as a placeholder
[env: HTTP_URL=]

--http-token <HTTP_TOKEN>
(Optional) A Bearer token for authenticating with the HTTP API
[env: HTTP_TOKEN=]

-i, --interval <INTERVAL>
Check interval in seconds. Set to 0 to run only once
[env: UPDATE_INTERVAL_SECS=, default: 300]

--ipv6
Enable IPv6 (AAAA record) update
[env: ENABLE_IPV6=]

-h, --help
Print help information

-V, --version
Print version information
```

### Quick Start Example

#### Example 1: Basic DNSPod Update

```bash
# Check every 10 seconds to update both A and AAAA records for home.example.com
./ddns \
--domain "example.com" \
--sub-domain "home" \
--token "YOUR_ID,YOUR_TOKEN" \
--interval 10 \
--ipv6
```

#### Example 2: With HTTP Notifier

This is useful for instantly updating a reverse proxy's backend IP.

```bash
./ddns \
--domain "example.com" \
--sub-domain "home" \
--token "YOUR_ID,YOUR_TOKEN" \
--http-url "https://your-proxy.com/update-ip?ip={IP_ADDRESS}" \
--http-token "YOUR_PROXY_SECRET_TOKEN" \
--interval 60
```

### Using Environment Variables

This is highly recommended for running as a service or in a container.

```bash
# --- Required DNSPod Settings ---
export DNSPOD_DOMAIN="example.com"
export DNSPOD_SUB_DOMAIN="home"
export DNSPOD_TOKEN="YOUR_ID,YOUR_TOKEN"

# --- Optional HTTP Notifier Settings ---
export HTTP_URL="https://your-proxy.com/update-ip?ip={IP_ADDRESS}"
export HTTP_TOKEN="YOUR_PROXY_SECRET_TOKEN"

# --- General Settings ---
export UPDATE_INTERVAL_SECS=600 # Check every 10 minutes
export ENABLE_IPV6=true

# Now you can just run the command without arguments
./ddns
```

---

## 🤖 Running as a Service (Systemd)

For a reliable, always-on setup, running `ddns` as a `systemd` service is the best approach.

1. **Create an environment file for your secrets**:
Create `/etc/default/ddns` and add your secrets to it. This keeps them out of your public service file.
```
# /etc/default/ddns
DNSPOD_TOKEN="YOUR_ID,YOUR_TOKEN"
HTTP_TOKEN="YOUR_PROXY_SECRET_TOKEN"
```
Set strict permissions: `sudo chmod 600 /etc/default/ddns`

2. **Create the systemd service file**:
Create `/etc/systemd/system/ddns.service` with the following content. **Remember to change your domain, sub-domain, and API URL!**

```ini
[Unit]
Description=DDNS Client for DNSPod with HTTP Notifier
After=network-online.target
Wants=network-online.target

[Service]
# --- Configure your domains and API URL here ---
ExecStart=/usr/local/bin/ddns \
--domain example.com \
--sub-domain home \
--http-url "https://your-proxy.com/update-ip?ip={IP_ADDRESS}" \
--ipv6

# --- Load secrets from the environment file ---
EnvironmentFile=-/etc/default/ddns
# Set log level (info, warn, error, debug, trace)
Environment="RUST_LOG=info"

# --- Run as a non-root user for security ---
User=nobody
Group=nogroup
# Alternatively, create a dedicated user:
# sudo useradd --system --no-create-home --shell /bin/false ddns-user
# User=ddns-user
# Group=ddns-user

# --- Service management ---
Type=simple
Restart=on-failure
RestartSec=30s
TimeoutStopSec=30s

[Install]
WantedBy=multi-user.target
```

3. **Enable and start the service**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable ddns.service
sudo systemctl start ddns.service
```

You can check the logs with `journalctl -u ddns.service -f`.

---

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

## 📖 Resources
- [DNSPod API Documentation](https://docs.dnspod.cn/api/record-list/)