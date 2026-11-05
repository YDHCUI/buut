# BUUT

https://github.com/YDHCUI/buut

用 Rust 写的高性能正反向隧道代理工具。**一个单文件二进制**，同时提供反向 socks5、
正向隧道、端口映射、端口复用与三层 TUN(VPN)；链路可在 **TCP / KCP(UDP) / WebSocket / ICMP**
之间切换，`-m` 一个参数即可决定，两端共用同一套加密与复用层。

## 项目统计

| 指标 | 数值 |
|---|---|
| 当前版本 | 1.5.1 |
| 主程序 | 26 个 Rust 文件 / 8,782 行 |
| 内置依赖 (vendored) | 9 个库 / 17,112 行：yamux、snow(Noise)、KCP、fast-socks5、wintun 等 |
| 链路协议 | 4 种：TCP / KCP over UDP / WebSocket / ICMP echo |
| 运行形态 | 6 种：反向隧道 / 正向隧道 / 自建 socks5 / 端口映射 / 端口复用 / TUN |
| 支持平台 | Windows x64(Win7+)·x86(XP+)、Linux x64·arm64(glibc 2.17+ 或纯静态 musl)、macOS x64·arm64 |
| 产物体积 | Windows x64 ≈ 2.4 MB，Linux x64 ≈ 3.0 MB (strip + LTO + opt-level=z) |
| 单元测试 | 23 个 (`cargo test --release`) |
| 运行时外部依赖 | 0 (仅 Windows 的 TUN 模式需同目录放置 wintun.dll) |

## 核心优势

| 优势 | 说明 |
|---|---|
| 协议可插拔 | 传输层抽象成 4 种链路，`-m tcp/udp/ws/icmp` 任意切换；隧道、端口映射、TUN 等业务层与链路完全解耦，换协议不换用法。 |
| 弱网与封锁场景 | 应用层 ICMP 隧道只依赖 ping；WS 可穿过仅放行 HTTP 的中间设备；Noise 握手 + 双向独立 ChaCha20Poly1305 密钥，无固定流量特征。 |
| 多通道聚合 | `--channel N` 建立 N 条独立连接/会话，按五元组哈希把每条流固定到一条通道：既并行提速，又不会跨通道乱序。 |
| 单文件多形态 | 客户端与服务端是同一个二进制，靠参数组合出 6 种形态，投放与维护成本低。 |
| 单向 VPN 拓扑 | 服务端最多加一条 `--tun_route` 即可访问对端内网，客户端不加任何路由 —— 对端无法反向进入服务端网络 (`--tun` 与 `-m icmp` 可组合)。 |
| 端口复用零依赖 | Linux 上按 eBPF `sk_lookup` → `nf_tables`(netlink) → `iptables` 三级回退，**前两级不需要任何外部命令**，精简镜像/容器里也能用；macOS 用 pf 的独立 anchor。 |
| 干净退出 | SIGINT/SIGTERM 统一清理路由、防火墙与端口复用规则；即使被 `kill -9`，下次启动会先清掉同规则的残留副本，不会越跑越脏。 |
| 小体积跨平台 | 一份源码交叉编译 7 个目标，Windows 侧用 VC-LTL / YY-Thunks 兼容 Win7 与 XP。 |
| 可选压缩 | `-Z` 基于 LZ4 分块压缩隧道内流量，两端独立开启，对上层透明。 |

## 工作原理

```text
  socks5 客户端 ┐
  端口映射/转发  ├──> [ 业务层 ]  反向隧道·正向隧道·端口映射·TUN
  TUN (IP 包)   ┘         │        yamux 多路复用 + ChaCha20Poly1305
                          │
                          │  4 字节长度前缀分帧 (vpn/frame.rs)
                          v
                 [ 传输层 ]  -m tcp | udp(kcp) | ws | icmp
                          │
                          │  1..N 条通道 (--channel，按流哈希固定分流)
                          v
                     公网  <════════════════>  对端 (同一份二进制)
```

隧道内的每一帧都是「4 字节长度前缀 + ChaCha20Poly1305 密文」：业务层只管往里塞
字节流，具体走哪种链路、几条通道、是否压缩，全部由 `-m` / `--channel` / `-Z` 在启动时决定。

## 参数介绍

| 参数 | 说明 |
|---|---|
| `-k, --key` | 加密密钥 |
| `-l, --listen_addr` | 监听地址 (服务端) |
| `-s, --remote_addr` | 远程地址 (客户端) |
| `-f, --forward_addr` | 转发地址，只支持正向 |
| `-p, --proxy_port` | 代理端口，默认 10086；或端口映射 `本地:目标`，如 `80:81` |
| `-m, --transport` | 协议类型，默认 TCP，支持 `<TCP\|KCP\|WS\|ICMP>` |
| `-c, --config` | 配置文件 (toml) |
| `-n, --name` | 客户端 id |
| `--channel` | 通道数量，默认 1 |
| `--headers` | 连接服务所需的其它配置，如 cookie |
| `--sockspass` | 代理密码，默认 `buut:buut` |
| `--noiseparams` | 加密方式，默认 `Noise_KK_25519_ChaChaPoly_BLAKE2s` |
| `--srcip` | 来源 IP，端口复用时使用 |
| `--tun_name` / `--tun_mtu` | TUN 网卡名 (默认 `buut0`) / MTU (默认 1380) |
| `--tun_ip` / `--tun_ip6` | TUN 本端地址，服务端默认 `10.0.7.1` / `fd00:7::1`，客户端 `10.0.7.2` / `fd00:7::2` |
| `--tun_route` | 服务端访问对端内网的网段，逗号分隔 CIDR，如 `192.168.1.0/24`；不写则不加路由 |
| `-F, --forward` | 正向模式 |
| `-S, --service` | 服务模式 |
| `-X, --soreuse` | 端口复用 |
| `-O, --origins` | 流量加密，默认启用 |
| `-Z, --compres` | LZ4 流量压缩，两端需同时开启 |
| `-H, --hidecmd` | 隐藏窗口，仅 Windows |
| `-T, --tun` | TUN(VPN) 模式，需管理员权限，链路协议由 `-m` 决定，配合 `-l` 或 `-s` 使用 |

`./buut -h` 可查看完整的帮助信息。

## 使用方法 

### 反向隧道

默认使用tcp连接反向隧道
```bash
    vps执行：          ./buut -l 443 
    target执行：       ./buut -s vps:443
    hacker连接socks5   vps:10086 


    vps:
    [root@localhost]# ./buut -l 443
    Reverse Server Listen [tcp://0.0.0.0:443]
    Client Handshake From 127.0.0.1:29505
    Agent ID [rmSyYaLX] Proxy Listen [0.0.0.0:10086]

    target:
    [root@localhost]# ./buut -s 127.0.0.1:443
    Reverse Agent ID [rmSyYaLX] Channel [1] Connect Suss

```

使用-S 参数启用服务模式，会同时监听tcp和kcp端口
```bash
    vps执行：          ./buut -S -l 443 
    target执行：       ./buut -s vps:443
    hacker连接socks5   vps:10086 
```


### 正向隧道

使用-F 参数连接tcp正向隧道
```bash
    target执行：       ./buut -F -l 443 
    vps执行：          ./buut -F -s target:443 
    hacker连接socks5   vps:10086 

    target:
    [root@localhost]# ./buut -F -l 443
    Forward Agent ID [3Yj2LLAg] Listen On [tcp://0.0.0.0:443]

    vps:
    [root@localhost]# ./buut -F -s 127.0.0.1:443
    Forward Server [tcp://127.0.0.1:443] Connect Suss
    Agent ID [3Yj2LLAg] Proxy Listen [0.0.0.0:10086]
```

如果服务需要登录才能访问 则使用headers参数设置cookie等请求头。
```bash
    target执行：       ./buut -F -l 1234 
    vps执行：          ./buut -F -s https://target:1234/xx --headers Cookie:Session=xxxxxx;
    hacker连接socks5   vps:10086 
```
起正向代理时建议同时使用-f参数设置forward_addr转发 这样buut识别到非buut流量时会把流量转发到对应地址。


### 自建代理

通过以下参数 直接起一个socks5服务。 应用场景是在内网机器里面做二级代理 缺点是流量没有加密。
```bash
    vps执行：          ./buut -F -S -l 10086
    user连接socks5     vps:10086 

    vps:
    [root@localhost]# ./buut -S -F -l 10086
    Socks5 Server ID [FPamyVyY] Listen On [tcp://0.0.0.0:10086]

```

透明端口转发,将2222端口的流量转发到forward_addr对应地址 不做任何处理
```bash
    [root@localhost]# ./buut -F -S -l 2222 -f 11.22.33.44:22
    Agent ID [nfg94w2k] Port Forwarding On [0.0.0.0:2222 <-> 11.22.33.44:22]

```

### 端口映射

```bash
    vps执行：          ./buut -l 443
    target执行         ./buut -s vps:443 -p 81:80  

    vps:
    [root@localhost]# ./buut -l 443
    Reverse Server Listen [tcp://0.0.0.0:443]
    Agent ID [rmSyYaLX] Port Mapping On [0.0.0.0:81 <-> 192.168.1.21:80]

    target:
    [root@localhost]# ./buut -s 127.0.0.1:443 -p 81:80 
    Reverse Agent ID [rmSyYaLX] Channel [1] Connect Suss

```


### 端口复用

1、假设现在有一个web服务运行在8080端口，正常情况下直接监听8080是会报如下错误
    
```bash 
    buut.exe -F -l 8080
    通常每个套接字地址(协议/网络地址/端口)只允许使用一次。 (os error 10048)
```
2、现在使用 -X 参数设置SO_REUSEADDR则可以成功监听  
```bash 
    buut.exe -X -F -l 8080
    Agent Forward ID [SCrLfzxa] Listen On [tcp://0.0.0.0:8080]

```
3、使用SO_REUSEADDR能成功的前提是原服务也启用了SO_REUSEADDR。如果原服务没有启用SO_REUSEADDR则不能成功。这时候可以使用 --srcip参数设置源IP 通过转发来实现端口复用。
```bash 
    [root@localhost buut]# ./buut -X -F -l 32001
    Address already in use (os error 98)

    [root@localhost buut]# ./buut -X -F -l 32001 --srcip 192.168.2.122
    Rule src:192.168.2.122 port:32001 redirect:32002 applied Succ!
    Agent Forward ID [EJ1TROJd] Listen On [tcp://0.0.0.0:32002]
    Rule src:192.168.2.122 port:32001 redirect:32002 Delete Succ!

```
4、客户端正常连接web端口就行
```bash 
    [root@localhost buut]# ./buut -F -s 123.151.152.153:32001
    GET / HTTP/1.1
    Upgrade: websocket
    Origin: http://123.151.152.153:32001/
    Sec-WebSocket-Version: 13
    Connection: Upgrade
    Sec-WebSocket-Key: ogq+DC9z2RVSsf86PpMizw==
    Host: 123.151.152.153:32001


    HTTP/1.1 101 Switching Protocols
    Sec-Websocket-Accept: o99zKAbgLJAkj09CtFiz7MMFpVU=
    Upgrade: websocket
    Connection: upgrade


    Server forward [tcp://123.151.152.153:32001] Connect Suss
    Client [123.151.152.153:32001]-[172.16.16.12] Join [D212ts4C]
    Agent ID [D212ts4C] Proxy Listen On [socks5://buut:buut@0.0.0.0:10086]
```

### Tips

1、设置BUUT变量隐藏vps。 如： 原本的 ./buut -s vps:1234 可改成 export BUUT=vps:1234 && ./buut -s 1

2、buut默认启用密码校验，默认用户名密码buut:buut 。

3、如果需要修改sockspass则需要为每个agent端单独设置，如 ./buut -s 127.0.0.1:443 --sockspass 123456

4、 使用带log的版本进行调试。 ./buut -l 1234 --log info 

5、 目前只支持对tcp做端口复用

5.1、 `--srcip` 的端口复用 (把"某来源访问原端口"的连接改送到 buut 的端口) 在 Linux 上有三种实现，
     按可用性自动降级，**前两种不依赖任何外部命令**，没有 `iptables` 的机器 (精简镜像/容器) 也能生效：
     - `eBPF sk_lookup` (内核 5.9+)：不改包、不经过 netfilter，进程退出/崩溃时内核自动卸载；
     - `nf_tables` (内核 4.10+)：直接走 netlink，不需要 `nft` 命令；
     - `iptables` 命令：兜底，规则放在专用链 `BUUT_PRE` 里，注销只清自己的链
       (旧版本 `-F PREROUTING` 会连用户自己的 NAT 规则一起清掉)；
     启动日志会打印跳过了哪些后端以及最终使用哪个。macOS 用 `pf` 的 `rdr` (anchor `buut`) 实现。
     所有后端都支持 `-X` 规则在 Ctrl+C / SIGTERM 时自动清理。

6、 -Z 流量压缩两端需同时开启，只压缩隧道内流量，对上层完全透明；VPN/ICMP 模式下每包独立压缩，两端可各自独立开启。


### ICMP 隧道 (应用层, -m icmp)

以 ICMP Echo Request/Reply 承载数据，半双工"一收一发"：一个请求带走一段上行数据，
对端必定回一个应答并顺带下行数据；没有数据时发空帧轮询（默认 20ms），保证下行延迟。
服务端只被动应答；内核自动回复的 echo 因缺少应用层标志位会被客户端丢弃，不影响普通 ping。

* 需要原始套接字权限：Linux/macOS 用 root (或 CAP_NET_RAW)，Windows 需管理员。
* 报文载荷 1400 字节/包，单通道吞吐约为 `1400B / RTT`；`--channel N` 开 N 条独立会话
  （各自独立原始套接字 + 独立收包线程），多连接聚合带宽随通道数线性增长。
* 超时重传：同一序号重发，服务端缓存上次应答只回放不重复投递；连续失败自动断开重连。

```bash
    # VPS (正向服务端, 出口在 VPS)
    vps执行：    ./buut -m icmp -l 10086 -F -k 密码
    # 本机 (socks5 监听在本地 1080, 流量经 ICMP 隧道从 VPS 出去)
    本机执行：   ./buut -m icmp -s vps:10086 -F -k 密码 -p 1080 --channel 4
    使用代理：   curl -x socks5://buut:buut@127.0.0.1:1080 https://ipinfo.io/ip

    # 反向隧道同理，去掉 -F 即可：vps 上监听 socks5，出口在本机
```

## TUN 模式 (VPN)

使用 `--tun` 开启 VPN 模式，在两端建立三层隧道，支持 Windows / Linux / macOS 三平台与
IPv4/IPv6 双栈。

`vpn` 目录只实现 TUN 本身(网卡、IP 包收发、路由、socks5 复用)，**链路协议由 `-m` 决定**，
与普通隧道模式共用同一套传输层：

| `-m` 取值 | 底层链路 | 说明 |
|---|---|---|
| `tcp` (默认) | TCP | 通用性最好，穿大部分防火墙 |
| `udp` / `kcp` | KCP over UDP | 低延迟，弱网抗丢包 |
| `ws` / `wss` | WebSocket | 可穿过仅放行 HTTP 的中间设备 |
| `icmp` / `ping` | ICMP echo (raw socket) | 只放行 ping 的网络可用，需 root/管理员 |

```rust
  公网 VPS(服务端)                 隧道                     对端(客户端/内网)
 +------------------+    +---------------------+    +--------------------+
 | --tun_route 一条  |    | 链路协议 (-m)        |    | TUN buut0          |
 | 10.0.7.1/24      |--->| tcp/udp/ws/icmp     |--->| 单向转发 (SNAT)     |
 |                  |    | AEAD + 长度分帧      |<---| 10.0.7.2/24        |
 +------------------+    +---------------------+    +--------------------+
        ^                                                    |
        | socks5 vps:10086 <── 流帧 ── 同一条隧道 ───────────┘
        |
   单向: 服务端 -> 对端内网 可通 ; 对端 -> 服务端网络 无路由, 不可达
```

### 平台支持

| 平台    | TUN 实现              | 内网侧转发 (客户端)                          | 权限   |
|---------|-----------------------|----------------------------------------------|--------|
| Windows | wintun 驱动           | 点对点无需配置；进内网需自建 ICS/NAT          | 管理员 |
| Linux   | tokio-tun             | 自动: 单向 FORWARD + SNAT，退出恢复           | root   |
| macOS   | utun (控制套接字)     | 点对点无需配置；进内网需自行 pf/NAT           | root   |

### 使用方法

```bash
    # TCP 链路 (默认)，服务端要访问对端内网 192.168.1.0/24
    vps执行(服务端):      ./buut --tun -l 4433 -k mykey --tun_route 192.168.1.0/24
    target执行(客户端):   ./buut --tun -s vps_ip:4433 -k mykey

    # ICMP 链路 (需 root / 管理员；-l 的端口仅作角色标记)
    vps执行(服务端):      ./buut --tun -m icmp -l 4433 -k mykey --tun_route 192.168.1.0/24
    target执行(客户端):   ./buut --tun -m icmp -s vps_ip:4433 -k mykey

    # KCP(UDP) / WebSocket
    ./buut --tun -m udp  -l 4433 -k mykey
    ./buut --tun -m ws   -s vps_ip:4433 -k mykey
```

`-l` 与 `-S` 均可进入服务端模式；两端 `-m` 必须一致。

`--tun_route` 只在服务端有效；不写时服务端一条路由都不加 (只能访问对端 TUN 地址)。

1. 路由是单向的: 起 socks5 的一端 (服务端) 能访问对端，对端访问不到服务端网络
   - 服务端默认**不加任何路由** —— TUN 网卡的直连路由 (10.0.7.0/24 dev buut0)
     已经能到对端 TUN 地址；
   - 要访问对端内网网段时用 `--tun_route` 指定，服务端**只加这一条**，
     下一跳是对端 TUN 地址 (对端在通道握手帧里告知，不需要手工填)；
   - 客户端**不加任何路由**，内网流量永远不会进入隧道，也就无法反向访问服务端网络。

2. 内网侧转发: 服务端发往里网的包从客户端 TUN 进来后，要由客户端转发出去
   - Linux 客户端自动完成: 开启 `ip_forward`、只放行 `-i buut0` 方向的 FORWARD、
     对 TUN 网段做 SNAT (只允许"隧道 -> 内网"单向，退出时恢复)；
     Ctrl+C 与 SIGTERM (systemd stop / docker stop) 都会触发清理；进程被 `kill -9`
     时下次启动会先清掉同规则的残留副本，规则表不会无限堆积；
   - Windows/macOS 客户端若只做点对点 (访问对端本机) 无需任何配置；
     要进入对端内网需自行开转发 + NAT (Windows: ICS / `New-NetNat`)；
   - 只开了 `ip_forward` 还不够: FORWARD 策略为 DROP 的环境 (装了 Docker 的主机很常见)
     会静默丢弃转发包，表现为"隧道能通但内网全超时"，这也是自动加 FORWARD 规则的原因。

3. 服务端同时监听 socks5 端口 (默认 10086，可用 `-p` 自定义)，流量经隧道从客户端出口。

### 服务端 socks5 (客户端出口)

TUN 模式下服务端同时开启一个 socks5 代理端口 (与普通隧道模式的用法统一)，
hacker 连接该端口后，流量经 ICMP 隧道转发到客户端、由客户端发起访问:

```
    hacker ──socks5──> vps:10086 ──流帧(AEAD加密)──> client ──socks5处理──> 目标
```

- 端口默认 10086，`-p` 自定义 (端口映射语义 `-p 本地:目标` 在 TUN 模式下取前者);
- socks5 协议在客户端处理，认证密码取客户端的 `--sockspass` (默认 buut:buut);
- 用途与普通模式一致: 访问客户端所能访问的网络 (如客户端所在内网);
- 客户端的出站路由遵循其本机路由表: 访问内网走本地网卡、从客户端出口;
  若客户端配了默认全流量路由，则公网目标会经隧道从服务端出口 ——
  需要客户端公网出口时用 `--tun_route` 收窄隧道路由网段。

### IPv6

- 外层: 客户端 `-s` 指定 v6 服务器地址即走 ICMPv6，服务端同时监听 v4/v6。
- 内层: TUN 双栈，默认 v6 地址服务端 `fd00:7::1/64`、客户端 `fd00:7::2/64`，
  可用 `--tun_ip6` 自定义(建议服务端尾字节为 1)。
- ICMPv6 校验和由内核计算 (RFC 3542)。

### 路由 (--tun_route，仅服务端)

```bash
    # 访问对端内网的两个网段 (v4/v6 均可)
    ./buut --tun -l 4433 -k mykey --tun_route 192.168.1.0/24,2001:db8::/32
```

- 不写 `--tun_route` 时一条路由都不加: 服务端只多了 TUN 网卡自带的直连路由；
- 路由只在隧道握手完成、知道对端 TUN 地址后才添加 (下一跳必须是对端地址)；
  对端地址与路由地址族不一致时跳过该条并提示；
- 退出时自动删除；不持久化，重启后自动失效。

### 说明

- 服务端默认 TUN 地址 10.0.7.1/24，客户端默认 10.0.7.2/24，可用 `--tun_ip` 自定义(建议服务端为 x.x.x.1)。
- 默认 MTU 1380，可用 `--tun_mtu` 自定义，网卡名可用 `--tun_name` 自定义。
- 双方向使用独立密钥(ChaCha20Poly1305 AEAD)，内核自动回复的 echo 无法通过认证会被丢弃，服务端无需修改 sysctl，仍可正常响应 ping，更加隐蔽。
- 服务器管理员权限: windows 需要 wintun.dll (见 bin 目录，放到 exe 同目录) + 管理员；linux/macOS 需要 root。
  注意: 账户在管理员组不等于进程已提权(UAC 过滤令牌)，需“以管理员身份运行”终端；
  未提权时报错携带 os error 5(拒绝访问)，可据此判断。
- wintun Rust 绑定库已 vendored 到 libs/wintun (0.5.1)，修改点： adapter create/open 失败时携带 GetLastError 错误码、extern "system" 调用约定消除告警。
- 客户端空闲时每 5 秒发送加密空批次保活。
- 日志只保留生命周期事件 (监听/连接/路由/退出)，不再周期打印流量统计。
- 排障可临时抓包: 服务端 `tcpdump -ni buut0` 看是否有去往里网的包、
  客户端 `tcpdump -ni buut0` 看包是否进来 (进来但没转发 = 客户端转发/NAT 没生效)。
- 只转发 IP 层路由过的流量，DNS 若走内网服务器需确认解析路径。


## 编译

### 依赖

- Windows 目标: 本机 Rust MSVC 工具链 (x86_64 + i686-pc-windows-msvc，脚本会自动 `rustup target add`)。
  VC-LTL / YY-Thunks 由 `.cargo/config.toml` 的 rustflags 自动应用
  (素材位于 `D:\CodeProject\hw_src\obj`)，产物兼容 Win7(x64) / XP(x86) 老系统。
- Linux / macOS 目标: 在 WSL 默认发行版内编译，需要:
  - `cargo-zigbuild` + zig (Linux 目标用系统 zig，macOS 交叉编译用 `/root/zig11` 的 zig 0.11)
  - macOS SDK: `D:\CodeProject\hw_src\obj\MacOSX11.3.sdk`
  - rustup targets: `x86_64-unknown-linux-gnu` / `x86_64-unknown-linux-musl` /
    `aarch64-unknown-linux-musl` / `x86_64-apple-darwin` / `aarch64-apple-darwin`
  - mac 素材 (zig11/SDK) 缺失时该段自动跳过，不影响其它目标。

### 用法 (build.ps1)

仓库根目录 PowerShell 执行:
```powershell
    .\build.ps1                 # 完整构建: Windows + Linux + macOS
    .\build.ps1 -WindowsOnly    # 只构建 Windows (x64 + x86)
    .\build.ps1 -LinuxOnly      # 只构建 Linux (gnu + musl)
    .\build.ps1 -MacOnly        # 只构建 macOS
    .\build.ps1 -SkipX86 -SkipMusl -SkipAarch64    # 跳过对应目标
    .\build.ps1 -Zip            # 构建完成后打包 zip
```

| 参数 | 说明 |
|------|------|
| -WindowsOnly / -LinuxOnly / -MacOnly | 只构建对应平台 |
| -SkipX86 | 跳过 Windows i686 |
| -SkipMusl | 跳过 Linux musl 静态目标 (保留 gnu) |
| -SkipAarch64 | 跳过 ARM64 (linux aarch64-musl + mac arm64) |
| -Zip | 构建完成后打包 zip |
| -TargetDir | 自定义 cargo target 目录 (默认 `../@target`) |
| -OutDir | 自定义产物目录 (默认 `成品/{版本}`) |

产物自动复制到 `成品/{版本}/`，命名与 Release 一致:

| 产物 | 平台 |
|------|------|
| buut.x86_64-pc-windows-msvc.exe | Windows x64 (Win7+) |
| buut.i686-pc-windows-msvc.exe | Windows x86 (XP+) |
| buut.x86_64-unknown-linux-gnu.elf | Linux x64 (glibc 2.17+) |
| buut.x86_64-unknown-linux-musl.elf | Linux x64 纯静态 |
| buut.aarch64-unknown-linux-musl.elf | Linux arm64 纯静态 |
| buut.x86_64-apple-darwin.osx | macOS x64 |
| buut.aarch64-apple-darwin.osx | macOS arm64 (M 系列) |
| wintun_amd64.dll / wintun_x86.dll | Windows TUN 模式所需 (拷自 bin/) |
| conf.toml / README.md | 配置模板与说明 |

说明:
- Linux gnu 目标链接 glibc 2.17 (CentOS 7+)，musl 目标纯静态可直接跑任意发行版。
- Windows TUN 模式需把对应 wintun dll 与 exe 放同一目录。
- 端口复用 (--srcip) 在 Linux 下按 eBPF (sk_lookup) → nf_tables → iptables 逐级回退，需 root，musl 静态版同样内置。
- 不方便本地构建时可用 GitHub Actions (.github/workflows) CI，产物命名与本地脚本一致。


## todo

1、 驱动过滤 WinDivert 、wpf  


## 交流 

加V拉你进群

![a8e5625b211ad3b3c435e9403ebae9f](https://github.com/YDHCUI/buut/assets/46884495/6c667bb1-7eae-464f-afbd-3f0d67cbcbcb)
