


## 更新 

### 1.2.1

1、加入端口映射模式 。
	使用,如将target的80端口映射到vps:81
	target端设置 buut -s vps:1234 -p 81:80   
	或vps端设置  buut -l 1234 -p 81:80   	
	正向时设置   buut -F -s target:443 -p 81:80  

2、 其它优化


### 1.1.3

1、更换序列化类型、方便更新

2、加入内网IP回显功能 。

3、加入窗口隐藏选项。（仅windows）

4、支持配置文件，
	使用方法,将需要的命令行参数全名写成toml就行 一行一个参数 。
	
	比如简单的服务端配置，

	```toml
	listen_addr = "0.0.0.0:443"
	proxy_port = "10086"
	```

	客户端配置
	```toml
	remote_addr = "192.168.93.217:443"
	```

	然后直接./buut -c conf.toml 启动就行


### 1.0.3

1、修复安全问题

2、修复32位长度不一至的bug 。


### 1.0.2

1、支持自定义id,多个相同id的客户端时会自动分配流量负载。

3、支持headers覆写。

4、稳定版本，以后api保持不变，不兼容之前版本。

5、支持多平台，其它优化。


### 0.7.1

1、 仔细研究了下nginx 发现其实只要包含Connection: Upgrade的头 tcp也是能正常穿透的。 
所以整体重构了下 正向直接伪装websocket头 建立连接后使用tcp传输。

2、解决空连接超时的ddos安全问题，提高性能，正向代理情况下使用-f参数不会影响到原有业务运行。

3、修改忽略服务端证书校验，以适应老旧网站。

4、使用so_reuseport优化端口复用，只支持unix。


### 0.6.1

1、修复随机数不随机的安全问题。 不兼容前面版本。

2、去除流量特征。


### 0.6.0

1、实现forward_addr功能， 将非buut流量转发到指定地址。 考虑到流量特征等原因该功能只对正向代理生效。

使用如:  ./buut -F -l 1234 -f 127.0.0.1:8080  

在不影响 buut 代理的使用的情况下 浏览器访问 127.0.0.1:1234 会返回 127.0.0.1:8080 的内容。 

2、不兼容前面版本

### 0.5.1

1、修改支持nginx默认配置下的websocket穿透

2、通过noiseparams参数自定义noise加密方式,默认为 Noise_KK_25519_ChaChaPoly_BLAKE2s 

使用如:  ./buut --noiseparams Noise_KK_25519_ChaChaPoly_BLAKE2s

3、修改版本长度。 所以不兼容前面版本 


### 0.5.0

1、之前NN模式有安全问题，现在更改noiseparams 为 Noise_KK_25519_ChaChaPoly_BLAKE2s 所以可能和以前版本会有不兼容

2、不兼容前面版本

### 0.4.4

1、加入自建sock5代理功能

使用如: ./buut -F -S -l 1234 会在本地启一个socks5 端口为 1234 


### 0.4.3

1、websocket支持tls。

2、正向websocket连接加入自定义header选项参数 

使用如: ./buut -F -m ws -s https://192.168.1.110/svr/wschat --headers Cookie:SessionId=xxxxxxxxxxxxx;\nHost: oauth2.hunnu.edu.cn


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

