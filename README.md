# 🚀 DNSPod DDNS

A modern, fast, and reliable DDNS (Dynamic DNS) client for [DNSPod](https://www.dnspod.cn/), written in Rust. 🦀

[![CI](https://github.com/luohoufu/dnspod-ddns/actions/workflows/ci.yml/badge.svg)](https://github.com/luohoufu/dnspod-ddns/actions/workflows/ci.yml)
[![Release](https://github.com/luohoufu/dnspod-ddns/actions/workflows/release.yml/badge.svg)](https://github.com/luohoufu/dnspod-ddns/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`dnspod-ddns` automatically checks your public IP address and updates your DNSPod DNS records, ensuring your domain always points to your home network, server, or any device with a dynamic IP.

---

## ✨ Features

- **Blazing Fast**: Built with Rust for minimal resource usage and high performance.
- **IPv4 & IPv6 Ready**: Simultaneously updates both `A` (IPv4) and `AAAA` (IPv6) records.
- **Efficient**: Uses the official `Record.Ddns` API to minimize API calls.
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
tar -xzvf ddns-x86_64-unknown-linux-musl.tar.gz
sudo mv ddns /usr/local/bin/
```

### 2. Using Cargo

If you have the Rust toolchain installed, you can install `dnspod-ddns` directly from crates.io.

```bash
cargo install ddns
```

### 3. From Source

```bash
git clone https://github.com/luohoufu/dnspod-ddns.git
cd dnspod-ddns
cargo build --release
# The binary will be in ./target/release/ddns
```

---

## 🚀 Usage

The tool is configured via command-line arguments or environment variables.

### Command-Line Arguments

Run `ddns --help` to see all available options:

```text
A modern, async DDNS tool for DNSPod with IPv4/IPv6 support.

Usage: ddns [OPTIONS] --domain <DOMAIN> --sub-domain <SUB_DOMAIN> --token <TOKEN>

Options:
-d, --domain <DOMAIN>
The domain name, e.g., "example.com"
[env: DNSPOD_DOMAIN=]

-s, --sub-domain <SUB_DOMAIN>
The sub-domain name, e.g., "home"
[env: DNSPOD_SUB_DOMAIN=]

-t, --token <TOKEN>
Your DNSPod API token, in "ID,Token" format
[env: DNSPOD_TOKEN=]

-i, --interval <INTERVAL>
Check interval in seconds. Set to 0 to run only once.
[env: DNSPOD_INTERVAL=, default: 300]

--ipv6
Enable IPv6 (AAAA record) update.
[env: DNSPOD_IPV6_ENABLED=]

-h, --help
Print help information (use `-h` for a summary)

-V, --version
Print version information
```

### Quick Start Example

```bash
# Run once to update both A and AAAA records for home.example.com
./ddns \
--domain "example.com" \
--sub-domain "home" \
--token "YOUR_ID,YOUR_TOKEN" \
--interval 10
```

### Using Environment Variables

This is highly recommended for running as a service or in a container.

```bash
export DNSPOD_DOMAIN="example.com"
export DNSPOD_SUB_DOMAIN="home"
export DNSPOD_TOKEN="YOUR_ID,YOUR_TOKEN"
export DNSPOD_IPV6_ENABLED=false
export DNSPOD_INTERVAL=10 # Check every 10 seconds

# Now you can just run the command without arguments
./ddns
```

---

## 🤖 Running as a Service (Systemd)

For a reliable, always-on setup, running `ddns` as a `systemd` service is the best approach.

1. **Create a dedicated user (for security)**:
```bash
sudo useradd --system --no-create-home --shell /bin/false ddns-user
```

2. **Create an environment file for your token**:
Create `/etc/default/ddns` and add your token to it. This keeps secrets out of your service file.
```
# /etc/default/ddns
DNSPOD_TOKEN="YOUR_ID,YOUR_TOKEN"
```
Set strict permissions: `sudo chmod 600 /etc/default/ddns`

3. **Create the systemd service file**:
Create `/etc/systemd/system/ddns.service` with the following content. **Remember to change your domain and sub-domain!**

```ini
[Unit]
Description=DNSPod DDNS Client
After=network-online.target
Wants=network-online.target

[Service]
# Adjust your domain and sub-domain here
ExecStart=/usr/local/bin/ddns --domain example.com --sub-domain home --ipv6

# Load token from the environment file
EnvironmentFile=-/etc/default/ddns
# Set log level
Environment="RUST_LOG=info"

# Run as a non-root user
User=ddns-user
Group=ddns-user

# Service management
Type=simple
Restart=on-failure
RestartSec=30s
TimeoutStopSec=30s

[Install]
WantedBy=multi-user.target
```

4. **Enable and start the service**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable ddns.service
sudo systemctl start ddns.service
```

You can check the logs with `journalctl -u ddns.service -f`.

---

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

## 📖 Resource
DNSPod documentation https://docs.dnspod.cn/api/record-list/