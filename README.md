# BUUT 

https://github.com/YDHCUI/buut

## 提醒

目前项目正处理开发阶段，各自版本的接口可能会不兼容。 请谨慎使用！

## 介绍 
一款使用rust开发的高性能正反向隧道代理工具，基于yamux多路复用技术。


## 工作原理
```rust
+------+      +--------------+      +-----+      +-----------+      +------------+      +----------+      +------+
|hacker| <--> | Socks5 stream| <--> | VPS | <--> |Yamux frame| <--> |Noise stream| <--> |TCP stream| <--> |TARGET|
+------+      +--------------+      +-----+      +-----------+      +------------+      +----------+      +------+
```

## 参数介绍 
```rust
    opts.optopt("k", "key", "", "加密密钥");
    opts.optopt("l", "server_listen", "", "监听地址");
    opts.optopt("s", "remote_addr", "", "远程地址");
    opts.optopt("f", "forward_addr", "", "转发地址,只对正向代理生效");
    opts.optopt("p", "proxy_port", "", "代理端口,默认10086");
    opts.optopt("m", "transport", "", "协议类型,默认TCP,支持<TCP|KCP|WS>");
    opts.optopt("n", "channel", "", "通道数量,默认1");
    opts.optopt("c", "config", "", "配置文件,默认路径./conf.toml");
    opts.optflagopt("F", "forward", "", "是否正向模式");
    opts.optflagopt("S", "server", "", "是否服务模式,同时监听tcp和kcp");
    opts.optflagopt("X", "reuse", "", "是否端口复用");
    opts.optopt("", "sockspass", "", "代理密码,默认不验证,用户名buut");
    opts.optopt("", "headers",   "", "连接服务所需的一些其它配置如cookie之类的");
    opts.optopt("",  "noiseparams",   "", "noise加密方式,默认Noise_KK_25519_ChaChaPoly_BLAKE2s");
    #[cfg(feature = "log")]
    opts.optopt("",  "log",             "", "日志等级,默认不开");
```

## 特点：

速度快，使用多路复用技术 将带宽利用到极致。rust开发，速度、稳定性和安全性都有保证。

体积小，编译打包后只有不到几百k, 相比golang写的程序动辄10多M还是很有优势的。

无特征，程序使用Noise定制加密，可使用动态密钥保证无任何特征。

单文件，客户端和服务端使用同样的单文件、多模式自由组合切换。

多协议，目前release 0.4已支持tcp、kcp、websocket，后续把icmp、dns协议支持加进去。



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

如果中间有nginx之类的负载设备，tcp没法直接连接，则可以使用websocket连接来穿透
```bash
    target执行：       ./buut -F -m ws -l 443 
    vps执行：          ./buut -F -m ws -s https://target:443 --headers Cookie:Session=xxxxxx;
    hacker连接socks5   vps:10086 
```

### 自建代理

```bash
    vps执行：          ./buut -F -S -l 10086
    user连接socks5     vps:10086 

    vps:
    [root@localhost]# ./buut -S -F -l 10086
    Socks5 Server ID [FPamyVyY] Listen On [tcp://0.0.0.0:10086]

```


### Tips

1、设置BUUT变量隐藏vps。 如： 原本的 ./buut -s vps:1234 可改成 export BUUT=vps:1234 && ./buut 

2、使用websocket协议时,连接地址应设为http://xxx格式 如: ./buut -m ws -s http://target:8081/xx

3、sockspass是每个agent端单独设置，所以应该在agent端设置，如 ./buut -s 127.0.0.1:443 --sockspass 123456


## 更新 

### 0.6.0

1、实现forward_addr功能， 将非buut流量转发到指定地址。 考虑到流量特征等原因该功能只对正向代理生效。

使用如:  ./buut -F -l 1234 -f 127.0.0.1:8080  

在不影响 buut 代理的使用的情况下 浏览器访问 127.0.0.1:1234 会返回 127.0.0.1:8080 的内容。 

2、不兼容前面版本

### 0.5.1

1、修改支持nginx默认配置下的websocket穿透

2、通过noiseparams参数自定义noise加密方式,默认为 Noise_KK_25519_ChaChaPoly_BLAKE2s 

使用如:  ./buut --noiseparams Noise_KK_25519_ChaChaPoly_BLAKE2s

3、不兼容前面版本 


### 0.5.0

1、之前NN模式有安全问题，现在更改noiseparams 为 Noise_KK_25519_ChaChaPoly_BLAKE2s 所以可能和以前版本会有不兼容


### 0.4.4

1、加入自建sock5代理功能

使用如: ./buut -F -S -l 1234 会在本地启一个socks5 端口为 1234 


### 0.4.3

1、websocket支持tls。

2、正向websocket连接加入自定义header选项参数 

使用如: ./buut -F -m ws -s https://192.168.1.110/svr/wschat --headers Cookie:SessionId=xxxxxxxxxxxxx;


### 0.4.2

1、简单实现端口复用

通过调用 SO_REUSEADDR 实现端口复用 使用如: ./buut -F -X -l 192.168.1.110:443  

2、去除默认参数，优化使用

3、修复握手包超时的的bug。


### 0.4.1

1、修改使KCP协议也使用noise加密。

2、修复正向代理不能使用kcp协议的问题。


### 0.4

1、加入websocket, kcp 协议支持。

2、修复client端不会自动重连的bug。

3、优化参数配置。


### 0.3.1 

1、加入windows支持。

2、修改默认参数。

3、优化体积。


## todo

1、实现icmp、dns协议支持  

2、链式代理支持

3、端口映射模式

4、加入tun模式 

