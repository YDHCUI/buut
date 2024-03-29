# BUUT 

## 介绍 
一款rust开发的正反向代理隧道工具。 

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
    opts.optopt("p", "proxy_port",      "", "代理监听,端口默认10086");
    opts.optopt("m", "transport",       "", "协议类型,默认TCP");//|UDP|WS|ICMP|DNS
    opts.optopt("k", "key",             "", "加密密钥");
    opts.optopt("n", "channel",         "", "通道数量,默认2");
    opts.optopt("c", "config",          "", "配置文件路径");
    opts.optopt("",  "sockspass",       "", "代理密码,默认不验证,用户名buut"); 
    opts.optflagopt("F", "forward",     "", "是否正向模式");
    opts.optflagopt("S", "server",      "", "是否服务模式");
```

## 特点：
	速度快，使用rust开发，速度、稳定性和安全性都有保证。
	体积小，win编译打包后只有不到400K，相比golang写的程序动辄10多M还是很有优势的。
	无特征，程序使用Noise定制加密，可使用动态密钥保证无任何特征。
	单文件，客户端和服务端使用同样的单文件、多模式自由组合切换。
	多协议支持，目前release 0.3只支持tcp，后续把udp和icmp协议支持加进去。
	


## 使用方法 

### 正向隧道：
	target执行：./buut -F -l 443
	vps执行 ：./buut -F -S -s target:443 -p 10086
  	hacker连接socks5  vps:10086


### 反向隧道
	vps执行：./buut -S -l 443 -p 10086
	target执行：./buut -s vps:443
   	hacker连接socks5  vps:10086 

 
 ## todo
 	增加udp icmp协议支持  
	增加自动设置端口复用模式参数
	
