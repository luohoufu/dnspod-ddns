[English](./README.md) | 简体中文

# 🚀 DNSPod DDNS

一个为 [DNSPod](https://www.dnspod.cn/) 设计的现代化、快速且可靠的 DDNS (动态域名解析) 客户端，由 Rust 编写。🦀

[![CI](https://github.com/luohoufu/dnspod-ddns/actions/workflows/ci.yml/badge.svg)](https://github.com/luohoufu/dnspod-ddns/actions/workflows/ci.yml)
[![Release](https://github.com/luohoufu/dnspod-ddns/actions/workflows/release.yml/badge.svg)](https://github.com/luohoufu/dnspod-ddns/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`dnspod-ddns` 会自动检查您的公网 IP 地址并更新您的 DNSPod 域名解析记录，确保您的域名始终指向您的家庭网络、服务器或任何具有动态 IP 的设备。

此外，它还可以向您的反向代理（如 OpenResty/Nginx）等服务发送一个**即时的 HTTP 通知**，使其能够立即更新其后端 IP，而无需等待 DNS 缓存刷新。

---

## ✨ 功能特性

- **极致性能**: 基于 Rust 构建，确保最小的资源占用和最高的运行效率。
- **智能稳定**: 在网络中断期间，会通过指数退避策略安静地等待网络恢复，避免日志刷屏和不必要的 API 请求。
- **支持 IPv4 & IPv6**: 可同时更新 `A` (IPv4) 和 `AAAA` (IPv6) 记录。
- **即时 HTTP 通知 (可选)**: 在 IP 更新成功后，可以立即通知一个可配置的 HTTP 端点（如反向代理），从而绕过 DNS TTL 延迟。
- **跨平台**: 为 Linux、macOS 和 Windows 提供了预编译的二进制文件。
- **配置简单**: 可通过命令行参数或环境变量进行配置。
- **随处运行**: 完美适用于 Docker 容器、树莓派、NAS 或任何服务器。

---

## 📦 安装

您有多种方式来获取 `dnspod-ddns`。

### 1. 从 GitHub Releases 下载 (推荐)

最简单的方式是从 [**GitHub Releases**](https://github.com/luohoufu/dnspod-ddns/releases) 页面下载适用于您系统的预编译二进制文件。

1. 前往 [Releases 页面](https://github.com/luohoufu/dnspod-ddns/releases)。
2. 下载适用于您操作系统和架构的压缩包 (例如, `ddns-x86_64-unknown-linux-musl.tar.gz`)。
3. 解压压缩包，即可开始使用！

```bash
# Linux 示例
wget https://github.com/luohoufu/dnspod-ddns/releases/latest/download/ddns-x86_64-unknown-linux-musl.tar.gz
tar -xzvf ddns-x86_64-unknown-linux-musl.tar.gz
sudo mv ddns /usr/local/bin/
```

### 2. 从源码编译

```bash
git clone https://github.com/luohoufu/dnspod-ddns.git
cd dnspod-ddns
cargo build --release
# 编译后的二进制文件位于 ./target/release/ddns
```

---

## 🚀 使用方法

本工具通过命令行参数或环境变量进行配置。所有与 DNSPod 相关的参数都是必需的。

### 命令行参数

运行 `ddns --help` 查看所有可用选项：

```text
一个为 DNSPod 设计的、支持 IPv4/IPv6 和可选 HTTP 通知的现代化异步 DDNS 工具。

Usage: ddns [OPTIONS] --token <TOKEN> --domain <DOMAIN> --sub-domain <SUB_DOMAIN>

Options:
--token <TOKEN>
您的 DNSPod API 令牌，格式为 "ID,Token"
[env: DNSPOD_TOKEN=]

--domain <DOMAIN>
您的主域名，例如 "example.com"
[env: DNSPOD_DOMAIN=]

--sub-domain <SUB_DOMAIN>
您的子域名，例如 "home"
[env: DNSPOD_SUB_DOMAIN=]

--http-url <HTTP_URL>
(可选) 用于 HTTP GET 通知的 URL。请使用 ?ip={IP_ADDRESS} 作为 IP 占位符
[env: HTTP_URL=]

--http-token <HTTP_TOKEN>
(可选) 用于 HTTP API 认证的 Bearer 令牌
[env: HTTP_TOKEN=]

-i, --interval <INTERVAL>
检查间隔（秒）。设置为 0 表示只运行一次
[env: UPDATE_INTERVAL_SECS=, default: 300]

--ipv6
启用 IPv6 (AAAA 记录) 更新
[env: ENABLE_IPV6=]

-h, --help
打印帮助信息

-V, --version
打印版本信息
```

### 快速开始示例

#### 示例 1: 基本的 DNSPod 更新

```bash
# 每隔10秒检查一次，为 home.example.com 更新 A 和 AAAA 记录
./ddns \
--domain "example.com" \
--sub-domain "home" \
--token "你的ID,你的TOKEN" \
--interval 10 \
--ipv6
```

#### 示例 2: 带有 HTTP 通知功能

这个功能对于即时更新反向代理的后端 IP 非常有用。

```bash
./ddns \
--domain "example.com" \
--sub-domain "home" \
--token "你的ID,你的TOKEN" \
--http-url "https://你的代理服务器.com/update-ip?ip={IP_ADDRESS}" \
--http-token "你的代理服务器密钥" \
--interval 60
```

### 使用环境变量

强烈建议在作为服务或在容器中运行时使用此方法。

```bash
# --- 必需的 DNSPod 配置 ---
export DNSPOD_DOMAIN="example.com"
export DNSPOD_SUB_DOMAIN="home"
export DNSPOD_TOKEN="你的ID,你的TOKEN"

# --- 可选的 HTTP 通知配置 ---
export HTTP_URL="https://你的代理服务器.com/update-ip?ip={IP_ADDRESS}"
export HTTP_TOKEN="你的代理服务器密钥"

# --- 通用配置 ---
export UPDATE_INTERVAL_SECS=600 # 每10分钟检查一次
export ENABLE_IPV6=true

# 现在你可以直接运行命令，无需任何参数
./ddns
```

---

## 🤖 作为服务运行 (Systemd)

为了实现一个可靠的、永远在线的部署，将 `ddns` 作为 `systemd` 服务运行是最佳方式。

1. **为密钥创建环境文件**:
在 `/etc/default/ddns` 创建一个文件来存放您的密钥，这能让密钥与公开的服务配置文件分离。
```
# /etc/default/ddns
DNSPOD_TOKEN="你的ID,你的TOKEN"
HTTP_TOKEN="你的代理服务器密钥"
```
设置严格的权限: `sudo chmod 600 /etc/default/ddns`

2. **创建 systemd 服务文件**:
创建 `/etc/systemd/system/ddns.service` 文件，并填入以下内容。**请务必修改您的域名、子域名和 API URL！**

```ini
[Unit]
Description=DDNS Client for DNSPod with HTTP Notifier
After=network-online.target
Wants=network-online.target

[Service]
# --- 在这里配置你的域名和 API URL ---
ExecStart=/usr/local/bin/ddns \
--domain example.com \
--sub-domain home \
--http-url "https://你的代理服务器.com/update-ip?ip={IP_ADDRESS}" \
--ipv6

# --- 从环境文件中加载密钥 ---
EnvironmentFile=-/etc/default/ddns
# 设置日志级别 (info, warn, error, debug, trace)
Environment="RUST_LOG=info"

# --- 为了安全，使用非 root 用户运行 ---
User=nobody
Group=nogroup
# 或者，创建一个专用用户：
# sudo useradd --system --no-create-home --shell /bin/false ddns-user
# User=ddns-user
# Group=ddns-user

# --- 服务管理 ---
Type=simple
Restart=on-failure
RestartSec=30s
TimeoutStopSec=30s

[Install]
WantedBy=multi-user.target
```

3. **启用并启动服务**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable ddns.service
sudo systemctl start ddns.service
```

您可以使用 `journalctl -u ddns.service -f` 来查看日志。

---

## 📄 许可证

本项目基于 **MIT 许可证**。详情请见 [LICENSE](LICENSE) 文件。

## 📖 相关资源
- [DNSPod API 文档](https://docs.dnspod.cn/api/record-list/)