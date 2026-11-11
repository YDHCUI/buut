## 更新

### 1.6.1

**P2P 传输 (-m p2p)**

1. 新增应用层 P2P 传输：iroh (公钥身份拨号的 QUIC, noq 栈)，每条连接取一条双向流当字节管道，上层 Snowstorm(Noise)+yamux 照常工作；`--channel N` 多通道各自独立 Endpoint/UDP 端口，与 icmp/kcp/quic 多通道模型一致。
2. 身份零配置：服务端 ed25519 身份与 ALPN 均由 `-k` 派生 (sha256(标签||-k))，客户端用同一 `-k` 推导目标 EndpointId，无需交换公钥/证书；两端 ALPN 不一致握手直接失败，兼起协议识别作用；认证仍由上层 Noise 完成。
3. NAT 穿透：`-s` 可达时直接 UDP 直连；不可达时经 relay 起步并并行打洞，成功后 QUIC 路径迁移为直连 (relay 退出数据路径)，对称 NAT 退化为 relay 中转；`-s` 填错也能靠 relay + 公钥身份找回服务端。
4. relay 策略 (`--relay_server`)：缺省 = n0 公共 relay (打洞信令 + 兜底中继)；自建 relay 填域名或 URL，裸域名自动补 `https://`，无端口默认 443。
5. `--dns_server` 参数更名 `--relay_server`，dns 模式 (上游递归解析器) 与 p2p 模式 (中继策略) 共用。

**修复**

6. 未配置 `--relay_server` 时 p2p 默认 n0 策略不生效：`from_config` 原先仅在 Some 时解析，None 落回 Disabled 纯直连底座 (双端 NAT 后连不上)；改为无条件过 parse，回归测试锁住。
7. dial 目标为 loopback 时同族绑定 loopback 源地址：多网卡环境下路径探测可能选中非 loopback 源，造成回程数据黑洞。
8. dial 失败路径 Endpoint 优雅 close：原先直接 drop 会以 ungracefully 中止连接。

**服务模式与其它**

9. `-S` 服务模式：编译了 p2p 时 udp 链路用 p2p 替换 kcp (同为 UDP 同端口冲突，二选一)，`-S` = tcp + icmp + (p2p 或 kcp)，未编译 p2p 时回落 kcp。
10. `cargo dev` 别名 (`run --features log`)：debug 运行默认带 log feature，配合 `--log info` 调试。
11. vpn 与 p2p 回归默认编译 (1.5.4 曾改为默认不编译)；README 补全 P2P 模式章节。

### 1.5.4

**体积极限优化**

1. 依赖裁剪：移除死依赖 `ctrlc` (1.5.1 起 src/shutdown.rs 已替代)；裁剪 `windows` crate 未用 feature (Security/System_IO/HttpServer)；`tokio-tungstenite` 关闭 default (`connect` 拉入 TLS 栈, ws 传输只用基于流的握手 API)。
2. 报错信息精简：全部 bail/anyhow/context/expect 消息去冗余 ("failed to ..." 类前缀全部精简)，保留可定位的关键信息 (错误对象 + 权限/参数提示)。
3. 无用兼容代码清除：删除从未调用的 `From<String>` 转换 (TransportType/ProxyPort/BuutAddr, 零调用方，与 FromStr 重复)；删除 core.rs 遗留的 STUN 死静态量 ("直连模式" 未实现残留)。

**可选 feature 瘦身**

4. QUIC 改为可选 feature (`--features quic`，默认不编译，quinn+rustls+ring 约 0.55 MB)；未编译时 `-m quic` 启动报错指引，`-h` 协议列表与实际能力一致。
5. TUN(VPN) 改为可选 feature (`--features vpn`，默认不编译，wintun/tokio-tun 驱动层约 0.27 MB)；未编译时 `-T` (命令行或配置文件 `tun = true`) 启动报错指引，`-h` 文案同步；libc 保持非可选 (-X 端口复用的 eBPF/nft 依赖)。
6. 默认产物 Windows x64 实测 2.28 MB (无 quic/vpn，1.5.3 同条件基线 3.31 MB)；`--features vpn` 2.66 MB、约 +0.55 MB 的 quic 按需叠加。测试：默认 27、+vpn 40 全部通过。

**健壮性**

7. 消除三处可达 panic (release `panic=abort` 会直接杀死进程)：`-S -F` 不带 `-l` 从 expect panic 改为报错 `-S -F requires -l <listen addr>`；非法 `--noise_params` 从 NoiseConfig 内 unwrap panic 改为启动时报 `Config Error: invalid noise_params`；`-p` 缺失时的报错文案精简。

### 1.5.3

**服务模式**

1. `-S` 服务模式加入 ICMP 监听：tcp + kcp + icmp 三链路同端口/同进程同时监听，客户端不换参数即可任选链路接入；icmp 无端口概念不与 tcp/kcp 冲突，Windows 服务端因内核不递交 echo request 不加入 (仅非 Windows)。
2. 全量功能测试通过 (VPS CentOS 7 / 内核 3.10 + Windows 11，两端真实公网)：6 链路 (tcp/kcp/ws/icmp/quic/dns) 正反向全通，dns 直连+中继 (`-s 域名`+`--dns_server`) 正反向全通，`-S` 三链路接入、`-Z` 压缩、`--channel 3` 多通道、`-p` 端口映射、`-X` 端口复用 (iptables 兜底后端全周期：规则注册/流量分发/原服务不受影响/退出清理)、TUN (tcp/icmp 的 L3 路由 + socks5 出口 + `-F` 正向)、`-c` 配置文件启动。
3. 文档修正：dns 直连模式示例中 socks5 监听位置描述错误 (反向模式 socks5 在服务端，非本机)。

**参数与配置**

4. 参数名统一：`--sockspass`/`--noiseparams`/`--dnsserver` 改名为 `--socks_pass`/`--noise_params`/`--dns_server`，与配置文件字段名 (snake_case) 一致。
5. 配置文件 (-c) 解析完善：缺失字段回落默认值而非变 None (原先缺 `transport` 启动报错、缺 `socks_pass` 变无认证、缺 `key` 变空串)；取值写法与命令行对齐 —— `transport` 大小写不敏感、`listen_addr`/`forward_addr` 支持纯端口 (自动补 `0.0.0.0`/`127.0.0.1`)、`proxy_port` 支持纯数字与 `"本地:目标"` 映射、`socks_pass = "NOAUTH"` 关闭认证；非法取值 (-m/-p/-s) 返回明确错误不再 panic；补全 `conf.toml` 模板与 README 配置文件章节。

### 1.5.2

**TUN 模式 (VPN)**

1. 移除内层 IPv6 双栈，TUN 仅 IPv4：删除 `--tun_ip6` 参数、三平台 v6 地址配置、`--tun_route` 的 v6 CIDR、Linux 客户端 ip6tables 转发/NAT；外层传输不受影响 (`-s` 仍可指定 v6 服务器地址)。
2. TUN 模式支持 `-F` 正向模式，socks5 方向与普通隧道模式对齐 (默认反向：服务端监听 socks5、客户端出口)；mux 拆分为可对称复用的入口 (`SocksHub`) 与出口 (`ExitStreams`) 两半，正向时客户端入口监听器只建一次、重连后自动切到新通道。
3. 稳定性与性能：删除四处可达的 `todo!()`/`unwrap()` (release `panic=abort` 会打死进程，含恶意握手触发服务端崩溃)；正向客户端拨号/握手失败改为重连；重连等待改异步 sleep 不再阻塞 tokio worker；TCP 服务端 `accept` 也设 `TCP_NODELAY`；release `opt-level` 由 `z` 调整为 `s`；删除无效参数 `-O/--origins` 与 `TcpConfig.proxy` 死字段。

**ICMP 传输**

4. 查明 Windows 上 ICMP 隧道握手必败的根因：Windows 的 TCP/IP 栈不把收到的 ICMP echo request 递交原始套接字 (内核自应答后只分发 reply，已实测确认)，服务端 (-l) 收不到任何请求；现在 Windows 上启动 ICMP 服务端直接报错并给出指引，服务端需跑 Linux/macOS，Windows 只能作客户端 (-s)。
5. 可观测性：各丢包分支 (非本协议/非 echo/标志位或会话不匹配) 加 debug/trace 日志，收包线程致命错误与重试耗尽改 `eprintln` 直接可见；`Reverse/Forward` 服务端 accept 错误不再被静默吞掉 (原先接收器失效后会 100% CPU 空转)，打印错误并限速重试。

**链路传输**

6. 新增 QUIC 传输 (`-m quic`)：quinn 实现 (QUIC v1 + TLS 1.3 + ring 后端)，每条通道一条独立连接/UDP 源端口，内建丢包恢复与拥塞控制，keep-alive 5s 防 NAT 老化；TLS 证书运行期随机自签、客户端跳过验证 (认证仍由上层 Noise 完成，无需配置证书)；普通 UDP 套接字无需特权；`-X` 端口复用与 `-S` 服务模式不适用 (UDP 同端口冲突)。
7. 新增 DNS 传输 (`-m dns`)：数据编码进标准 DNS TXT 查询/应答 (上行 base32 入子域名约 134B/查询、下行 TXT 记录约 900B/应答，UDP 载荷控制在 1232B 内)，半双工 lockstep 模型同 icmp (同 seq 超时重传、应答缓存回放、空闲空帧轮询)，但承载是普通 UDP 套接字 —— 无需特权，Windows 也可当服务端。两种模式：**直连** (`-s ip:port`) 与 **中继** (`--dnsserver 8.8.8.8` 走公共递归解析器，`-s` 填隧道域名，流量路径 `客户端↔解析器↔服务端`，两端无直接连接)；中继带 EDNS0 (udpsize=1232 防 512 截断)、应答 TTL=0 不被缓存、QNAME 大小写不敏感 (兼容 0x20 改写)、空闲轮询 250ms/重传 2.5s；同一服务端二进制同时服务两种模式 (按 QNAME 内容识别)；传输层新增 `Transport::from_config` 默认方法把配置注入传输。

### 1.5.1

1. 新增应用层 ICMP 传输 (`-m icmp`)：Echo Request/Reply 半双工"一收一发"，空闲 20ms 空帧轮询保下行延迟，超时重传、重复请求只回放缓存；`--channel N` 多通道各自独立原始套接字并行，聚合带宽随通道数增长；内核自动回复的 echo 因缺应用层标志位被丢弃，不影响正常 ping。
2. 修复 `--channel>1` 时同对端地址的多条通道互相覆盖 (流量全挤一条)：改为 `CMD_HELLO` 握手交换序号/总数 + 五元组哈希按流固定分流，既并行又不跨通道乱序。
3. `vpn` 目录重构为"只实现 `--tun` 本身"，链路协议由 `-m` 决定 (tcp/udp(kcp)/ws/icmp)；新增 `vpn/frame.rs` 长度前缀分帧，codec 批次上限随传输自适应 (icmp 1200B，tcp/ws 32KB)；原始套接字封装 `transport/rawsock.rs` 合并进 `transport/icmp.rs`。
4. 路由改为**单向拓扑**：服务端最多一条 `--tun_route` (下一跳取对端 TUN 地址，由握手帧带上来)，不配置时一条都不加；客户端不加任何路由，对端无法反向进入服务端网络。删除 `/1` 半路由与"服务器直连例外路由"。
5. Linux 客户端自动做单向转发：`ip_forward` + 只放行 `-i <tun>` 方向的 FORWARD + TUN 网段 SNAT；Windows/macOS 点对点访问无需配置。
6. 修复单向拓扑下服务端主动发往里网的包被丢弃：会话选择原先只按对端 TUN 地址精确匹配，改为匹配不到时退回全部会话按流哈希分流。
7. 服务端同时监听 socks5 (默认 10086，`-p` 自定义)，socks5 协议在客户端处理并从客户端出口，与普通隧道模式用法统一。
8. 支持 macOS utun，Windows(wintun)/Linux(tokio-tun)/macOS 三平台可用；支持 IPv6 双栈 (内层 `--tun_ip6`，1.5.2 已移除)。
9. Linux 端口复用重构为 `platform/linux/{mod,ebpf,nft,iptables}` 三级回退：eBPF `sk_lookup` (5.9+) → `nf_tables` (4.10+，走 netlink) → `iptables` 命令，前两级**不需要任何外部命令**；iptables 后端改用专用链 `BUUT_PRE`，注销只清自己的链，规则添加幂等；新增 macOS 端口复用 (pf `rdr`，独立 anchor `buut`)。
10. 新增 `src/shutdown.rs`：SIGINT + SIGTERM 统一清理 (vpn 路由/防火墙 + `-X` 规则)，去掉 `ctrlc` crate 与 panic hook 临时写法；防火墙规则幂等 (添加前清同规则残留，`kill -9` 后也不堆积)；运行期输出统一为英文 (避免 Windows GBK 乱码)，`-h` 帮助保持中文。
11. 新增 `build.ps1` 一键交叉编译：Windows x64/x86 (MSVC + VC-LTL/YY-Thunks 兼容 Win7/XP)、Linux gnu 2.17/musl 静态 + aarch64、macOS x64/arm64，产物自动落到 `成品/{版本}/`。

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
