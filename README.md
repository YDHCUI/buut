# pytunnel
正反向代理隧道 

# 参数：
	-s 远端地址
	-l 监听地址
	-p 代理地址
	-t 转发地址
	-k 加密密钥
	-r 反向模式
	-u 协议模式 目前只有tcp 后续会有 tcp,udp,icmp 可选
	--log 日志文件路径
 
# 用法：
## 端口转发：
	   target执行 ：./pytunnel -l 0.0.0.0:1234 -t 127.0.0.1:22
   访问 target:1234 会被转发到 target:22 

## 反向隧道：
	vps执行：./pytunnel -l 0.0.0.0:1234 
	target执行：./pytunnel -s vps:1234 -p 0.0.0.0:1080 -r 
   操作机连接 socks5   vps:1080 

## 正向隧道：
	vps执行：./pytunnel -s target:80 -p 0.0.0.0:1080
	target执行 ：./pytunnel -l 0.0.0.0:80 -t 127.0.0.1:8080 
  操作机连接 socks5  vps:1080 
  只有使用同样客户端且密钥一致时才会建立socks5连接 其它流量会被转发到 target:8080 
 
 
 ## todo
 增加udp icmp协议 
 增加通道数 优化速度 目前只有1通道
 转发端口服务端断开连接客户端不会自动断开连接问题 
 解决反向模式时监听端口doss问题 
