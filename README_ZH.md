[English](./README.md) | 简体中文

# 🚀 DNSPod DDNS

一个现代、快速且可靠的 DNSPod 动态 DNS 客户端，由 Rust 语言编写。🦀

[![CI](https://github.com/luohoufu/dnspod-ddns/actions/workflows/ci.yml/badge.svg)](https://github.com/luohoufu/dnspod-ddns/actions/workflows/ci.yml)
[![Release](https://github.com/luohoufu/dnspod-ddns/actions/workflows/release.yml/badge.svg)](https://github.com/luohoufu/dnspod-ddns/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`dnspod-ddns` 会自动检测您的公网 IP 地址并更新您的 DNSPod DNS 记录，确保您的域名始终指向您的家庭网络、服务器或任何具有动态 IP 的设备。

---

## ✨ 主要特性

- **极致性能**: 基于 Rust 构建，资源占用极低，性能卓越。
- **智能健壮**: 在网络故障时，能以指数退避方式安静地等待恢复，避免日志刷屏和不必要的 API 请求。
- **IPv4 & IPv6 双栈支持**: 可同时更新 `A` (IPv4) 和 `AAAA` (IPv6) 记录。
- **跨平台**: 为 Linux, macOS, 和 Windows 提供了预编译的二进制文件。
- **配置简单**: 可通过命令行参数或环境变量进行配置。
- **随处运行**: 完美适用于 Docker 容器、树莓派、NAS 或任何服务器。

---

## 📦 安装

您有多种方式来获取 `dnspod-ddns`。

### 1. 从 GitHub Releases 下载 (推荐)

最简单的方式是从 [**GitHub Releases**](https://github.com/luohoufu/dnspod-ddns/releases) 页面下载适用于您系统的预编译二进制文件。

1. 访问 [Releases 页面](https://github.com/luohoufu/dnspod-ddns/releases)。
2. 下载适用于您操作系统和架构的压缩包 (例如, `ddns-x86_64-unknown-linux-musl.tar.gz`)。
3. 解压压缩包，即可开始使用！

```bash
# Linux 系统示例
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

本工具通过命令行参数或环境变量进行配置。

### 命令行参数

运行 `ddns --help` 查看所有可用选项：

```text
一个现代、异步的 DNSPod DDNS 工具，支持 IPv4/IPv6。

用法: ddns [OPTIONS] --domain <DOMAIN> --sub-domain <SUB_DOMAIN> --token <TOKEN>

选项:
-d, --domain <DOMAIN>
主域名，例如 "example.com"
[环境变量: DNSPOD_DOMAIN=]

-s, --sub-domain <SUB_DOMAIN>
子域名，例如 "home"
[环境变量: DNSPOD_SUB_DOMAIN=]

-t, --token <TOKEN>
您的 DNSPod API Token，格式为 "ID,Token"
[环境变量: DNSPOD_TOKEN=]

-i, --interval <INTERVAL>
检查间隔时间（秒）。设置为 0 表示只运行一次。
[环境变量: DNSPOD_INTERVAL=, 默认值: 300]

--ipv6
启用 IPv6 (AAAA 记录) 更新。
[环境变量: DNSPOD_IPV6_ENABLED=]

-h, --help
打印帮助信息

-V, --version
打印版本信息
```

### 快速开始示例

```bash
# 每 10 秒检查一次，为 home.example.com 更新 A 和 AAAA 记录
./ddns \
--domain "example.com" \
--sub-domain "home" \
--token "YOUR_ID,YOUR_TOKEN" \
--interval 10 \
--ipv6
```

### 使用环境变量

强烈建议在作为服务或在容器中运行时使用此方法。

```bash
export DNSPOD_DOMAIN="example.com"
export DNSPOD_SUB_DOMAIN="home"
export DNSPOD_TOKEN="YOUR_ID,YOUR_TOKEN"
export DNSPOD_IPV6_ENABLED=true
export DNSPOD_INTERVAL=600 # 每 10 分钟检查一次

# 现在可以直接运行命令，无需任何参数
./ddns
```

---

## 🤖 作为 Systemd 服务运行

为了实现可靠的、永远在线的设置，将 `dnspod-ddns` 作为 `systemd` 服务运行是最佳实践。

1. **为安全起见，创建专用用户**：
```bash
sudo useradd --system --no-create-home --shell /bin/false ddns-user
```

2. **创建存放 Token 的环境文件**：
创建 `/etc/default/ddns` 文件，并将您的 Token 放入其中。这能让您的密钥与服务文件分离。
```
# /etc/default/ddns
DNSPOD_TOKEN="YOUR_ID,YOUR_TOKEN"
```
设置严格权限: `sudo chmod 600 /etc/default/ddns`

3. **创建 Systemd 服务文件**:
创建 `/etc/systemd/system/ddns.service` 并填入以下内容。**请务必修改您的域名和子域名！**

```ini
[Unit]
Description=DNSPod DDNS Client
After=network-online.target
Wants=network-online.target

[Service]
# 在这里修改您的域名、子域名等参数
ExecStart=/usr/local/bin/ddns --domain example.com --sub-domain home --ipv6

# 从环境文件中加载 Token
EnvironmentFile=-/etc/default/ddns
# 设置日志级别 (info, warn, error, debug, trace)
Environment="RUST_LOG=info"

# 以非 root 用户运行
User=ddns-user
Group=ddns-user

# 服务管理
Type=simple
Restart=on-failure
RestartSec=30s
TimeoutStopSec=30s

[Install]
WantedBy=multi-user.target
```

4. **启用并启动服务**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable ddns.service
sudo systemctl start ddns.service
```

您可以使用 `journalctl -u ddns.service -f` 查看实时日志。

---

## 📄 许可证

本项目基于 **MIT 许可证**。详情请见 [LICENSE](LICENSE) 文件。

## 📖 参考资料
- [DNSPod API 文档](https://docs.dnspod.cn/api/record-list/)