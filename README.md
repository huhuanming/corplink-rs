# 飞连 CLI（Feilian CLI / CorpLink CLI）

[![npm version](https://img.shields.io/npm/v/feilian-cli?logo=npm&label=npm)](https://www.npmjs.com/package/feilian-cli)
[![npm monthly downloads](https://img.shields.io/npm/dm/feilian-cli?logo=npm&label=downloads%2Fmonth)](https://www.npmjs.com/package/feilian-cli)
[![npm total downloads](https://img.shields.io/npm/dt/feilian-cli?logo=npm&label=downloads)](https://www.npmjs.com/package/feilian-cli)
[![npm package size](https://img.shields.io/npm/unpacked-size/feilian-cli?logo=npm&label=package%20size)](https://www.npmjs.com/package/feilian-cli)
[![Node.js](https://img.shields.io/node/v/feilian-cli?logo=node.js&label=node)](https://www.npmjs.com/package/feilian-cli)
[![GitHub release](https://img.shields.io/github/v/release/huhuanming/corplink-rs?label=release)](https://github.com/huhuanming/corplink-rs/releases/latest)
[![platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)](#安装--install)
[![license](https://img.shields.io/npm/l/feilian-cli?label=license)](./license.txt)

**feilian-cli** 是面向飞连（Feilian / CorpLink）企业租户的非官方跨平台命令行客户端，支持 macOS、Linux 和 Windows。它提供飞连二维码登录、手机 Push MFA、WireGuard 企业 VPN、SOCKS5 本地代理，以及 Clash / Mihomo 企业内网分流。

An unofficial cross-platform **Feilian CLI / CorpLink CLI** with QR-code login, mobile push MFA, WireGuard enterprise VPN, a local SOCKS5 proxy, and Clash/Mihomo split routing.

```bash
npm install --global feilian-cli@latest
```

> 本项目与飞连官方无隶属关系，也不会绕过企业认证、设备合规、安全策略或访问控制。能否使用取决于企业租户和管理员策略。

## 适合这些场景 / Use cases

- 在 macOS、Linux、Windows 或无图形界面的服务器上使用飞连企业 VPN。
- 使用官方飞连 App 扫码登录，无需在终端输入企业密码。
- 在手机上确认 VPN Push MFA（二次验证）。
- 为 Clash、Mihomo、Stash 或单个应用提供本地 SOCKS5 飞连代理。
- 仅让指定企业域名和内网 CIDR 经过飞连，其他流量继续使用原有代理规则。
- 使用 npm 自动安装当前系统的原生二进制，并在后续启动时自动更新。

## 安装 / Install

需要 Node.js 16 或更高版本。

```bash
npm install --global feilian-cli@latest
feilian-cli
```

| 系统 | 架构 |
| --- | --- |
| macOS | Apple Silicon (`arm64`)、Intel (`x64`) |
| Linux | `arm64`、`x64` |
| Windows | `x64` |

npm 只会安装当前平台对应的原生包。

## 二维码登录 / QR login

首次运行会交互式询问企业标识和可选账号，然后创建：

```text
~/feilian-cli.config.json
```

启动流程：

1. 输入企业提供的飞连标识，例如公司专属登录地址中的企业短名。
2. 使用官方飞连 App 扫描终端二维码并确认登录。
3. 如果企业要求 VPN 二次验证，在手机推送中点击确认。
4. CLI 建立飞连 VPN 或启动本地 SOCKS5 服务。

TUN 模式出现的 `Password:` 是本机管理员密码，用于创建虚拟网卡和路由，不是企业账号密码。

常用命令：

```bash
feilian-cli                          # 使用默认配置连接
feilian-cli /path/to/config.json     # 使用指定配置
feilian-cli --version                # 查看版本
feilian-cli --check-update           # 只检查更新，不安装
```

正常启动时发现新版本会自动执行 npm 更新；成功后直接启动新版，失败则继续运行当前版本。

## Clash 企业内网分流 / Clash split routing

推荐使用 SOCKS5/netstack 模式：飞连只监听本机代理端口，不创建系统 TUN，不修改 macOS/Linux 系统路由和 DNS，也不需要管理员权限。

### 1. 启用飞连 SOCKS5

编辑 `~/feilian-cli.config.json`：

```json
{
  "company_name": "your-enterprise-id",
  "username": "you@example.com",
  "platform": "feilian_qr",
  "vpn_mfa_type": "push",
  "socks5_listen": "127.0.0.1:11080",
  "auto_setup_routes": false,
  "use_vpn_dns": false
}
```

然后运行：

```bash
feilian-cli
```

可用以下命令验证 SOCKS5 内网访问；`--socks5-hostname` 会把域名交给飞连隧道内的 DNS 解析：

```bash
curl --socks5-hostname 127.0.0.1:11080 https://portal.corp.example/
```

### 2. 添加 Clash / Mihomo 规则

将占位域名和网段替换为企业管理员提供的实际范围，并把企业规则放在普通代理规则之前：

```yaml
proxies:
  - name: Feilian-Enterprise
    type: socks5
    server: 127.0.0.1
    port: 11080
    udp: false

rules:
  - DOMAIN-SUFFIX,corp.example,Feilian-Enterprise
  - DOMAIN,portal.corp.example,Feilian-Enterprise
  - IP-CIDR,10.20.0.0/16,Feilian-Enterprise,no-resolve
  - MATCH,Your-Existing-Policy
```

流量路径：

```text
企业域名/IP → Clash 规则 → 127.0.0.1:11080 → 飞连 WireGuard → 企业内网
其他流量   → 原有 Clash 规则
```

不要把本地 SOCKS5 地址、飞连租户服务或 VPN 网关再次转发到 `Feilian-Enterprise`，否则会形成环路。企业规则也不要配置公网 fallback；飞连断开时应让内网请求失败，避免把私有地址误发到公网。

## 系统 VPN / TUN 模式

不设置 `socks5_listen` 时，CLI 使用系统 TUN 模式，并根据飞连服务端返回的路由连接企业网络。

常用配置：

| 字段 | 用途 |
| --- | --- |
| `route_mode` | `split` 仅走企业路由；`full` 使用全隧道 |
| `vpn_additional_domains` | 为额外企业域名解析并添加主机路由 |
| `vpn_additional_routes` | 添加额外 CIDR 路由 |
| `vpn_allowed_routes` | 限制允许进入飞连的 CIDR |
| `vpn_disallowed_routes` | 排除本地网络或指定 CIDR |
| `use_vpn_dns` | 在 TUN 模式下使用服务端下发的 DNS |
| `vpn_server_name` | 指定飞连 VPN 节点名称 |

macOS 的 TUN 接口名称必须匹配 `utun[0-9]*`；首次生成的配置会使用有效名称。

## 常见问题 / FAQ

### 支持第三方企业飞连账号吗？

支持通过企业标识自动发现飞连租户，不写死任何公司、账号或内网信息。企业必须允许相应登录和 VPN 认证方式。

### 支持 SSO、邮箱验证码或密码登录吗？

本项目默认并重点支持二维码登录和手机 Push MFA。SSO、证书、设备合规及其他登录方式取决于租户策略，不保证可用，也不会绕过管理员限制。

### 飞连手机推送有什么作用？

部分企业在 VPN 连接阶段要求额外确认。CLI 会发送 Push MFA，并通过飞连长连接接收确认结果，然后继续请求 VPN 配置。

### Clash、Mihomo 和 Stash 都能使用吗？

只要客户端能连接标准 SOCKS5 节点并按域名/CIDR 配置规则，就能使用 `127.0.0.1:11080` 进行飞连内网分流。不同客户端的配置语法可能略有差异。

### 更新失败会影响连接吗？

不会。自动检查或 npm 安装失败时，CLI 会继续启动当前版本。`--check-update` 始终是只读命令。

## 安全说明 / Security

- 配置、Cookie、会话和 WireGuard 密钥仅保存在本机；不要提交到 GitHub。
- SOCKS5 建议只监听 `127.0.0.1`，不要直接暴露到局域网或公网。
- 不要在日志、Issue 或截图中公开企业账号、Token、Cookie、验证码、证书和内网地址。
- 本工具不提供认证绕过；所有访问权限仍由企业飞连服务端决定。

## 致谢 / Acknowledgements

本项目基于 [PinkD/corplink-rs](https://github.com/PinkD/corplink-rs) 开发，感谢原作者和贡献者提供的 Rust、WireGuard 与跨平台基础。

## License

[GPL-2.0-or-later](./license.txt)
