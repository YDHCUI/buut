//#![allow(unused_imports, unused_mut, unused_variables, dead_code)]

//#![cfg_attr(all(not(debug_assertions), target_os = "windows"),windows_subsystem = "windows")]

mod config;
mod core;
mod transport;
mod noise;
mod utils;

use crate::core::run_main;
use crate::transport::TransportType;
use crate::utils;

use getopts::Matches;
use url::Url;
use anyhow::Result;
use serde::{Serialize,Deserialize};
use serde::Deserializer;

use std::net::SocketAddr;

pub const BUUT_VERSION: &str = "buut-01.02.01";
pub const HANDSHAKE_TIMEOUT: u64 = 5; // Timeout for transport handshake
pub const MAX_HEADERS_SIZE: usize = 10240; // read headers max bufsize

// 这个结构体不能修改 否则会造成版本不兼容 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SynConfig {
    pub version: String,
    pub name:    String,
    pub host:    String,
    pub pass:    Option<String>,
    pub port:    Option<ProxyPort>,
}

impl Default for SynConfig {
    fn default() 
    -> SynConfig {
        SynConfig{
            version: BUUT_VERSION.into(),
            name:    String::from(""),
            host:    String::from(""),
            pass:    None,
            port:    None, 
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProxyPort{
    Socks5(u16),
    Mapping((u16,u16))
}

impl From<String> for ProxyPort {
    fn from(s: String) 
    -> ProxyPort {
        if let Some((sport,aport)) = s.split_once(":") { 
            ProxyPort::Mapping((
                sport.parse::<u16>().unwrap(),
                aport.parse::<u16>().unwrap()
            ))
        }else{
            ProxyPort::Socks5(s.parse::<u16>().unwrap())
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BuutConfig {
    pub config:         Option<String>, 
    pub remote_addr:    Option<BuutAddr>,
    pub listen_addr:    Option<SocketAddr>,
    pub forward_addr:   Option<SocketAddr>,
    pub proxy_port:     Option<ProxyPort>,
    pub transport:      Option<TransportType>,
    pub key:            Option<String>,
    pub name:           Option<String>,
    pub channel:        Option<usize>,
    pub socks_pass:     Option<String>,
    pub headers:        Option<String>,
    pub noise_params:   Option<String>,
    pub forward:        Option<bool>,
    pub service:        Option<bool>,
    pub soreuse:        Option<bool>,
    pub origins:        Option<bool>,
    pub compres:        Option<bool>,
    pub hidecmd:        Option<bool>,
}

impl Default for BuutConfig {
    fn default() 
    -> BuutConfig {
        BuutConfig{
            config:         None,
            remote_addr:    None,
            listen_addr:    None,
            forward_addr:   None,
            proxy_port:     Some(ProxyPort::Socks5(10086)),
            transport:      Some(TransportType::from(String::from("tcp"))),
            key:            Some(String::from("0xffff")),
            name:           Some(utils::get_rnd(8)),
            channel:        Some(1),
            socks_pass:     Some("buut".into()),
            headers:        None,
            noise_params:   Some("Noise_KK_25519_ChaChaPoly_BLAKE2s".into()),
            forward:        Some(false),
            service:        Some(false),
            soreuse:        Some(false),
            origins:        Some(false),
            compres:        Some(false),
            hidecmd:        Some(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuutAddr(pub Url);

impl BuutAddr{
    pub fn is_tls(&self) 
    -> bool {
        match self.0.scheme().to_lowercase().as_str(){
            "https"|"wss"|"tls" => true,
            _ => false,
        }
    }

    pub fn host_str(&self) 
    -> &str {
        let host_str = self.0.host_str().unwrap(); // 调用的地方都是客户端 暂时没大问题
        host_str
    }

    pub fn path(&self) 
    -> &str {
        self.0.path()
    }

    pub fn host(&self) 
    -> String {
        let host = self.host_str();
        if let Some(port) = self.0.port() {
            format!("{}:{}",host,port)
        } else {
            host.into()
        }
    }

    pub fn get_socketaddr(&self) 
    -> Result<SocketAddr>{
        let port = if let Some(port) = self.0.port() {
            port
        } else {
            if self.is_tls(){443}else{80}
        };
        utils::to_addr((self.host_str(),port))
    }
}

impl From<String> for BuutAddr {
    fn from(s: String) 
    -> BuutAddr {
        let uri = if s.contains("://"){
            Url::parse(&s).expect("地址错误")
        }else{
            Url::parse(&format!("http://{}",s)).expect("地址错误")
        };
        BuutAddr(uri)
    }
}

impl<'de> Deserialize<'de> for BuutAddr {
    fn deserialize<D>(deserializer: D) 
    -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(BuutAddr::from(String::deserialize(deserializer)?))
    }
}

pub fn parse(matches: Matches) 
-> Result<BuutConfig> {
    let mut config = BuutConfig::default();

    if let Some(s) = matches.opt_str("config") {
        let data = std::fs::read_to_string(s)?;
        config = toml::from_str(&data)?;
    }; 

    if let Some(s) = matches.opt_str("remote_addr") {
        config.remote_addr = Some(BuutAddr::from(s));
        if let Ok(a) = std::env::var("BUUT") {
            config.remote_addr = Some(BuutAddr::from(a));
        };
    }; 

    if let Some(s) = matches.opt_str("listen_addr") {
        let x = if let Ok(a) = s.parse::<u16>() {
            format!("0.0.0.0:{}", a)
        } else {
            s
        };
        config.listen_addr = Some(utils::to_addr(x)?);
    };

    if let Some(s) = matches.opt_str("forward_addr") {
        config.forward_addr = Some(utils::to_addr(s)?);
    };

    if let Some(s) = matches.opt_str("channel") {
        config.channel = Some(s.parse::<usize>()?);
    };

    if let Some(s) = matches.opt_str("proxy_port") {
        config.proxy_port = Some(ProxyPort::from(s));
    };

    if let Some(s) = matches.opt_str("transport") {
        config.transport = Some(TransportType::from(s.to_lowercase()));
    };
    
    if let Some(s) = matches.opt_str("key"){
        config.key = Some(s);
    };

    if let Some(s) = matches.opt_str("name"){
        config.name = Some(s);
    };

    if let Some(s) = matches.opt_str("sockspass"){
        if s.as_str() == "NOAUTH" {
            config.socks_pass = None;
        }else{ 
            config.socks_pass = Some(s);
        }
    };

    if let Some(s) = matches.opt_str("headers"){
        config.headers = Some(s);
    };

    if let Some(s) = matches.opt_str("noiseparams"){
        config.noise_params = Some(s);
    };

    config.forward = Some(matches.opt_present("forward"));
    config.service = Some(matches.opt_present("service"));
    config.soreuse = Some(matches.opt_present("soreuse"));
    config.origins = Some(matches.opt_present("origins"));
    config.compres = Some(matches.opt_present("compres"));
    config.hidecmd = Some(matches.opt_present("hidecmd"));

    Ok(config)
}


fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optopt("k", "key",             "", "          加密密钥");
    opts.optopt("l", "listen_addr",     "", "  监听地址");
    opts.optopt("s", "remote_addr",     "", "  远程地址");
    opts.optopt("f", "forward_addr",    "", " 转发地址,只支持正向");
    opts.optopt("p", "proxy_port",      "", "   代理端口,默认10086 或得端口转发模式:本地端口:目标端口,如80:81");
    opts.optopt("m", "transport",       "", "    协议类型,默认TCP,支持<TCP|KCP>");
    opts.optopt("c", "config",          "", "       配置文件");
    opts.optopt("n", "name",            "", "         客户端id");
    opts.optopt("", "channel",          "", "      通道数量,默认1");
    opts.optopt("", "sockspass",        "", "    代理密码,默认buut/buut");
    opts.optopt("", "headers",          "", "      连接配置,连接服务所需的一些其它配置如cookie之类的");
    opts.optopt("", "noiseparams",      "", "  加密方式,noise默认Noise_KK_25519_ChaChaPoly_BLAKE2s");
    opts.optflagopt("F", "forward",     "", "是否正向模式");
    opts.optflagopt("S", "service",     "", "是否服务模式");
    opts.optflagopt("X", "soreuse",     "", "是否端口复用");
    opts.optflagopt("O", "origins",     "", "是否流量加密,默认启用");
    opts.optflagopt("Z", "compres",     "", "是否流量压缩"); 
    opts.optflagopt("H", "hidecmd",     "", "是否隐藏窗口"); // 只对windows生效    
    //opts.optopt("",  "p2p",             "", "P2P模式,未实现");
    #[cfg(feature = "log")]
    opts.optopt("",  "log",             "", "日志等级,默认不开"); 

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("{}", opts.usage("BUUT: 一款高性能隧道代理工具"));
            println!("版本: {}",config::BUUT_VERSION);
            println!("更多: https://github.com/YDHCUI/buut");
            return;
        }
    };

    
    #[cfg(feature = "log")]
    if let Some(s) = matches.opt_str("log"){
        std::env::set_var("RUST_LOG", s);
        env_logger::init();
    }
    
    let config = config::parse(matches).unwrap();
    log::info!("{:?}", config);
    
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let config = set_iptables(config).unwrap();
    
    rt.block_on(async {
        run_main(config).await;
    });
}


pub fn set_iptables(mut config: BuutConfig) -> Result<BuutConfig> {
    let match_str = utils::calc_websocket_key(config.key.clone().unwrap_or_default().as_bytes());
    if config.soreuse.unwrap_or_default() { 
        if let Some(listen_addr) = config.listen_addr { 
            let target_port = listen_addr.port();
            let target_listen = utils::to_listen(target_port)?;
            let redirect_port = target_listen.port();
            if target_port != redirect_port {
                if let Some(match_ip) = &config.src_ip { 
                    if let Err(e) = platform::register_filter(target_port,redirect_port,&match_ip,match_str){
                        println!("RegFilter Err {:?}", e);
                    } else {
                        config.listen_addr = Some(target_listen);
                        if let None = config.forward_addr { 
                            config.forward_addr = Some(listen_addr);
                        }
                        let listenaddress = match_ip.clone();
                        let _ = ctrlc::set_handler(move || {
                            if let Err(e) = platform::unregister_filter(&listenaddress,target_port,redirect_port){
                                println!("UnFilter Err {:?}", e);
                            }
                            std::process::exit(1);
                        });                        
                        let listenaddress = match_ip.clone();
                        std::panic::set_hook(Box::new(move |_| {
                            if let Err(e) = platform::unregister_filter(&listenaddress,target_port,redirect_port){
                                println!("UnFilter Err {:?}", e);
                            }
                            std::process::exit(1);
                        }));
                    }
                }
            }
        }
    }
    Ok(config)
}

async fn run_once<T: Transport + Clone + 'static>(
    config: BuutConfig,
    mut reverse: Reverse<T>,
    mut forward: Forward<T>,
) {
    if config.hidewin.unwrap_or_default() { //隐藏cmd窗口
        #[cfg(windows)]
        utils::hide_console();
    }
    if config.server_listen.is_some() {
        if config.forward.unwrap_or_default() {
            if let Err(e) = forward.agent().await {
                println!("{:?}", e);
            };
        }else{
            if let Err(e) = reverse.server().await {
                println!("{:?}", e);
            };
        }
    };
    if config.remote_addr.is_some() {
        if config.forward.unwrap_or_default() {
            if let Err(e) = forward.server().await {
                println!("{:?}", e);
            };
        }else{
            if let Err(e) = reverse.agent().await {
                println!("{:?}", e);
            };
        }
    };
}

async fn run_server(mut config: BuutConfig) {
    let mut handles = Vec::new();
    let mut config1 = config.clone();
    handles.push(tokio::spawn(async move {
        config1.transport = TransportType::TCP;
        let forward: Forward<TcpTransport> = Forward::new(config1.clone());
        let reverse: Reverse<TcpTransport> = Reverse::new(config1.clone());
        run_once(config1, reverse, forward).await;
    }));
    #[cfg(feature = "kcp")]
    handles.push(tokio::spawn(async move {
        config.transport = TransportType::KCP;
        let forward: Forward<KcpTransport> = Forward::new(config.clone());
        let reverse: Reverse<KcpTransport> = Reverse::new(config.clone());
        run_once(config, reverse, forward).await;
    }));
    futures::future::join_all(handles).await;
}

async fn run_transport(config: BuutConfig) {
    match config.transport {
        TransportType::TCP => {
            let forward: Forward<TcpTransport> = Forward::new(config.clone());
            let reverse: Reverse<TcpTransport> = Reverse::new(config.clone());
            run_once(config, reverse, forward).await;
        }
        #[cfg(feature = "kcp")]
        TransportType::KCP => {
            let forward: Forward<KcpTransport> = Forward::new(config.clone());
            let reverse: Reverse<KcpTransport> = Reverse::new(config.clone());
            run_once(config, reverse, forward).await;
        }
        #[cfg(feature = "ws")]
        TransportType::WS => {
            let forward: Forward<WebsocketTransport> = Forward::new(config.clone());
            let reverse: Reverse<WebsocketTransport> = Reverse::new(config.clone());
            run_once(config, reverse, forward).await;
        }
    }
}

pub async fn run_main(config: BuutConfig){
    if config.service.unwrap_or_default() {
        if config.forward.unwrap_or_default() {
            let mut forward: Forward<TcpTransport> = Forward::new(config.clone());
            if let Err(e) = forward.socks5server().await {
                println!("{:?}", e);
            }
        }
        run_server(config.clone()).await;
    }
    run_transport(config).await;
}


/*
todo
    // todo P2P 模式 
    pub async fn punchhole_s(&self) -> Result<()> {
        /* A:agent S:server B:hacker
        1、B 先发个udp包给S，S返回A、B对应的NAT地址
        2、B 将自己的NAT地址打包成socks5流量发给A对应的NAT地址
        3、A 解析出C的NAT地址后 给该地址发一个消息
        4、打洞成功。
        5、B用kcp正向连接到A对应NAT地址。
        */
        Ok(())
    }

    // todo P2P 模式 
    pub async fn punchhole_b(&self) -> Result<()> {
        /* A:agent S:server B:hacker
        1、B 先发个udp包给S，S返回A、B对应的NAT地址
        2、B 将自己的NAT地址打包成socks5流量发给A对应的NAT地址
        3、A 解析出C的NAT地址后 给该地址发一个消息
        4、打洞成功。
        5、B用kcp正向连接到A对应NAT地址。
        */
        Ok(())
    }
    // todo P2P 模式 
    pub async fn punchhole_a(&self) -> Result<()> {
        /* A:agent S:server B:hacker
        1、B 先发个udp包给S，S返回A、B对应的NAT地址
        2、B 将自己的NAT地址打包成socks5流量发给A对应的NAT地址
        3、A 解析出C的NAT地址后 给该地址发一个消息
        4、打洞成功。
        5、B用kcp正向连接到A对应NAT地址。
        */
        Ok(())
    }
*/

