## 更新

### 1.5.1

**链路与传输**

1. 新增应用层 ICMP 传输 (`-m icmp`)：Echo Request/Reply 半双工"一收一发"，空闲 20ms 空帧轮询保下行延迟，超时重传、重复请求只回放缓存。`--channel N` 多通道各自独立原始套接字并行工作，聚合带宽随通道数增长。内核自动回复的 echo 因缺应用层标志位被丢弃，不影响目标正常 ping；需 root / 管理员权限。
2. 修复 `--channel>1` 时同对端地址的多条通道互相覆盖（TURN 以地址字符串为 key，流量全挤一条），改为 `CMD_HELLO` 握手交换序号/总数 + 五元组哈希按流固定分流，既并行又不跨通道乱序。
3. 原始套接字封装 `transport/rawsock.rs` 合并进 `transport/icmp.rs`（改为私有内联）。

**TUN 模式 (VPN)**

4. `vpn` 目录重构为"只实现 `--tun` 本身"，链路协议由 `-m` 决定 (tcp/udp(kcp)/ws/icmp)，不再直接操作原始套接字；新增 `vpn/frame.rs` 长度前缀分帧，codec 批次上限随传输自适应 (icmp 1200B，tcp/ws 32KB)。
5. 路由改为**单向拓扑**：服务端最多一条 `--tun_route`（下一跳取对端 TUN 地址，由握手帧带上来），不配置时一条都不加；客户端不加任何路由，对端无法反向进入服务端网络。删除 `/1` 半路由与"服务器直连例外路由"。
6. Linux 客户端自动做单向转发：`ip_forward` + 只放行 `-i <tun>` 方向的 FORWARD + TUN 网段 SNAT；Windows/macOS 做点对点访问无需配置。
7. 修复单向拓扑下服务端主动发往里网的包被丢弃：会话选择原先只按对端 TUN 地址精确匹配，改为匹配不到时退回全部会话按流哈希分流。
8. 服务端同时监听 socks5 (默认 10086，`-p` 自定义)，socks5 协议在客户端处理并从客户端出口，与普通隧道模式用法统一。
9. 支持 IPv6 双栈 (内层 `--tun_ip6`，外层指定 v6 服务器地址即走 ICMPv6) 与 macOS utun；Windows(wintun)、Linux(tokio-tun)、macOS 三平台可用。
10. 去掉两端的周期性流量统计打印，日志只保留生命周期事件 (监听/连接/路由/退出)。

**平台与端口复用**

11. `platform/linux*.rs` 重构为 `platform/linux/{mod,ebpf,nft,iptables}`，按内核版本与可用性三级回退：eBPF `sk_lookup` (5.9+) → `nf_tables` (4.10+，走 netlink，不依赖 `nft` 命令) → `iptables` 命令。前两级**不需要任何外部命令**，没有 iptables 的精简镜像/容器也能生效。
12. iptables 后端改用专用链 `BUUT_PRE`（PREROUTING 只加一条跳转），注销只清自己的链，不再 `-F PREROUTING` 连用户自己的 NAT 规则一起清掉；规则添加幂等。
13. 实现 macOS 端口复用：pf(4) `rdr` 重定向到 `127.0.0.1:<redirect>`（v6 用 `::1`），规则装在独立 anchor `buut`，注销只清该 anchor。

**清理与稳定性**

14. 新增 `src/shutdown.rs`：SIGINT + SIGTERM，watch 通道避免丢唤醒；vpn 的路由/防火墙清理与 `-X` 的规则清理统一到一处，去掉 `ctrlc` crate 与 panic hook 的临时写法。
15. 防火墙规则幂等：添加前先清除同规则残留副本（`kill -9` 无法执行退出清理），反复启停规则表不再无限堆积。
16. 运行期输出统一为英文，避免 Windows 控制台 (GBK 代码页) 乱码；`-h` 帮助信息保持中文。

**构建**

17. 新增 `build.ps1` 一键交叉编译：Windows x64/x86 (MSVC + VC-LTL/YY-Thunks 兼容 Win7/XP)、Linux x86_64 (gnu 2.17 / musl 静态) 与 aarch64 musl、macOS x64/arm64，产物自动落到 `成品/{版本}/`。

### 1.4.1

1. 加入 TUN 隧道模式 (VPN)，双平台 SOCK_RAW + TUN 虚拟网卡；Windows 需 wintun.dll 与管理员权限，Linux 需 root。
2. 加入 LZ4 流量压缩 (`-Z/--compres`)，两端需同时开启；隧道模式每包独立压缩，VPN 模式每批次独立压缩。
3. 修复 `-S/-O/-X/-Z/-H` 参数相互覆盖的解析 bug；修复编译告警并升级 vendored 依赖的过时 API。
4. vendor wintun 0.5.1 到 `libs/wintun`：创建/打开失败时携带 `GetLastError` 错误码，调用约定改 `extern "system"`。

### 1.3.0

1. 新增 `--srcip` 参数，端口复用支持"只把指定来源的连接改送到 buut"。
2. `-f` 可只写端口（如 `-f 8080`），默认转发到 `127.0.0.1:8080`。

### 1.2.1

1. 加入端口映射模式：`-p 81:80` 把本端 81 映射到对端 80，正向/反向均可。
2. 其它优化。

### 1.1.3 及更早

| 版本 | 主要内容 |
|---|---|
| 1.1.3 | 更换序列化类型；内网 IP 回显；Windows 窗口隐藏 (`-H`)；支持 toml 配置文件 (`-c`)。 |
| 1.0.3 | 修复安全问题；修复 32 位长度不一致。 |
| 1.0.2 | 支持自定义客户端 id（相同 id 自动负载均衡）；支持 `--headers` 覆写；稳定 API。 |
| 0.7.1 | 正向流量伪装 WebSocket 头后仍走 TCP 传输；空连接超时防护；忽略服务端证书校验；Unix 下 `SO_REUSEPORT` 端口复用。 |
| 0.6.x | `forward_addr` 非 buut 流量转发（仅正向）；修复随机数不随机；去除流量特征。 |
| 0.5.x | 支持 nginx 默认配置下的 WebSocket 穿透；`--noiseparams` 自定义 Noise 套件（默认 `Noise_KK_25519_ChaChaPoly_BLAKE2s`）。 |
| 0.4.x | 加入 WebSocket / KCP 协议与 TLS；自建 socks5 (`-F -S`)；`SO_REUSEADDR` 端口复用；KCP 也走 Noise 加密。 |
| 0.3.1 | 加入 Windows 支持，优化体积。 |
