//! GENESIS Dedicated Game Server
use anyhow::Result;
use clap::Parser;
use tracing::{info,warn};
use chrono::Utc;

#[derive(Parser,Debug)]
#[command(name="genesis-server",version=env!("CARGO_PKG_VERSION"),about="GENESIS Dedicated Server")]
struct Cli {
    #[arg(long,default_value_t=7070)] port:u16,
    #[arg(long,default_value_t=32)]   slots:u32,
    #[arg(long)] mode:Option<String>,
    #[arg(long)] map:Option<String>,
    #[arg(long)] password:Option<String>,
    #[arg(long,default_value_t=20)]   tick_rate:u32,
    #[arg(long)] anti_cheat:bool,
    #[arg(short,long,action=clap::ArgAction::Count)] verbose:u8,
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct ServerState {
    pub id:String, pub port:u16, pub slots:u32, pub connected:u32,
    pub mode:String, pub map:String, pub tick_rate:u32,
    pub anti_cheat:bool, pub uptime_secs:u64, pub total_connections:u64,
    pub peak_players:u32, pub matches:u32, pub avg_tick_ms:f32,
}

impl ServerState {
    pub fn new(port:u16,slots:u32,mode:&str,map:&str,tick_rate:u32,anti_cheat:bool)->Self{
        Self{id:format!("server_{}", Utc::now().timestamp()),port,slots,connected:0,
             mode:mode.to_string(),map:map.to_string(),tick_rate,anti_cheat,
             uptime_secs:0,total_connections:0,peak_players:0,matches:0,avg_tick_ms:0.0}
    }
    pub fn uptime_str(&self)->String{
        let s=self.uptime_secs;
        if s<60{format!("{}s",s)}else if s<3600{format!("{}m{}s",s/60,s%60)}else{format!("{}h{}m",s/3600,(s%3600)/60)}
    }
}

#[tokio::main]
async fn main()->Result<()>{
    let cli=Cli::parse();
    let level=match cli.verbose{0=>"info",1=>"debug",_=>"trace"};
    tracing_subscriber::fmt().with_env_filter(level).with_target(false).compact().init();

    let mode=cli.mode.as_deref().unwrap_or("default");
    let map=cli.map.as_deref().unwrap_or("main");

    info!("╔══════════════════════════════════════════╗");
    info!("║  GENESIS Server v{}                 ║",env!("CARGO_PKG_VERSION"));
    info!("╠══════════════════════════════════════════╣");
    info!("║  Port:  {}   Slots: {}               ║",cli.port,cli.slots);
    info!("║  Mode:  {:<32} ║",mode);
    info!("║  Map:   {:<32} ║",map);
    info!("║  Tick:  {}Hz  AntiCheat: {}          ║",cli.tick_rate,if cli.anti_cheat{"ON"}else{"OFF"});
    info!("╚══════════════════════════════════════════╝");

    let mut state=ServerState::new(cli.port,cli.slots,mode,map,cli.tick_rate,cli.anti_cheat);
    let interval=std::time::Duration::from_secs_f64(1.0/cli.tick_rate as f64);
    let start=std::time::Instant::now();
    let mut tick=0u64;
    let (tx,mut rx)=tokio::sync::oneshot::channel::<()>();
    ctrlc::set_handler(move||{let _=tx.send(());})?;

    info!("Server ready — listening on 0.0.0.0:{}", cli.port);

    loop{
        if rx.try_recv().is_ok(){info!("Shutdown requested");break;}
        let t0=std::time::Instant::now();
        state.uptime_secs=start.elapsed().as_secs();
        state.peak_players=state.peak_players.max(state.connected);
        tick+=1;
        let tm=t0.elapsed().as_secs_f32()*1000.0;
        state.avg_tick_ms=state.avg_tick_ms*0.99+tm*0.01;
        if tick%(cli.tick_rate as u64*5)==0{
            info!("[{}] Players:{}/{} | Uptime:{} | Tick:{:.2}ms",
                state.id,state.connected,state.slots,state.uptime_str(),state.avg_tick_ms);
        }
        if tm>1000.0/cli.tick_rate as f32{warn!("Tick overrun: {:.2}ms",tm);}
        let elapsed=t0.elapsed();
        if elapsed<interval{tokio::time::sleep(interval-elapsed).await;}
    }
    info!("Server exited after {} ticks | Peak:{} | Connections:{}",tick,state.peak_players,state.total_connections);
    Ok(())
}
