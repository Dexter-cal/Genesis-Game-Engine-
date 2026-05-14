//! Genesis Cast — Chromecast, AirPlay, Miracast, secondary screen, WebRTC casting
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CastDevice {
    pub id:String, pub name:String, pub kind:DeviceKind,
    pub ip:String, pub port:u16, pub model:Option<String>,
    pub caps:Vec<Capability>, pub last_seen:DateTime<Utc>, pub connected:bool,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DeviceKind {
    Chromecast{gen:u32}, ChromecastUltra, AppleTv{model:String},
    AirPlay, Miracast{vendor:String}, Dlna{model:String},
    GenesisApp{platform:String}, WebRtcPeer, Hdmi{port:u32},
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Capability {
    H264,H265,Av1,Audio51,Hdr10,FourK,LowLatency,SecondScreen,TouchInput,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CastSession {
    pub id:String, pub device_id:String, pub device_name:String,
    pub started:DateTime<Utc>, pub mode:CastMode, pub quality:Quality,
    pub res:[u32;2], pub fps:u32, pub bitrate_kbps:u32, pub codec:String,
    pub latency_ms:u32, pub bytes_sent:u64, pub frames_sent:u64,
    pub frames_dropped:u64, pub active:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CastMode {
    MirrorMain,
    SecondScreen{content:SecondContent},
    GameAudio, ExtendedDesktop,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SecondContent {
    Map{zoom:f32,markers:bool}, Inventory, Stats{layout:String},
    Minimap{size_m:f32}, SpectatorCam{follow:Option<String>},
    Director{auto_secs:f32}, Chat, Achievements, Url{url:String},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Quality { Auto, Low, Medium, High, Lossless }

pub struct CastManager {
    pub enabled:bool, pub devices:Vec<CastDevice>,
    pub sessions:Vec<CastSession>, pub secondary:SecondScreenConfig,
    pub quality:Quality, pub latency_target_ms:u32,
    pub total_cast_secs:f64,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SecondScreenConfig {
    pub enabled:bool, pub content:SecondContent,
    pub update_hz:f32, pub touch_controls:bool,
}

impl CastManager {
    pub fn new()->Self{
        Self{
            enabled:true, devices:Vec::new(), sessions:Vec::new(),
            secondary:SecondScreenConfig{
                enabled:false,
                content:SecondContent::Map{zoom:1.0,markers:true},
                update_hz:30.0, touch_controls:false,
            },
            quality:Quality::Auto, latency_target_ms:80, total_cast_secs:0.0,
        }
    }

    pub fn discover(&mut self){
        tracing::info!("Scanning for cast devices (mDNS/SSDP)...");
        // Real: mDNS scan for _googlecast._tcp, _airplay._tcp, SSDP for DLNA
    }

    pub fn start(&mut self,device_id:&str,mode:CastMode)->Option<String>{
        let dev=self.devices.iter().find(|d|d.id==device_id)?;
        let id=format!("cast_{}", Utc::now().timestamp_millis());
        tracing::info!("Starting cast {} → {}",&id,dev.name);
        self.sessions.push(CastSession{
            id:id.clone(), device_id:device_id.to_string(), device_name:dev.name.clone(),
            started:Utc::now(), mode, quality:self.quality.clone(),
            res:[1920,1080], fps:60, bitrate_kbps:8000, codec:"H264".to_string(),
            latency_ms:0, bytes_sent:0, frames_sent:0, frames_dropped:0, active:true,
        });
        Some(id)
    }

    pub fn stop(&mut self,session_id:&str){
        if let Some(s)=self.sessions.iter_mut().find(|s|s.id==session_id){
            s.active=false;
            let secs=(Utc::now()-s.started).num_seconds() as f64;
            self.total_cast_secs+=secs;
            tracing::info!("Cast session {} ended ({:.0}s)",session_id,secs);
        }
        self.sessions.retain(|s|s.active);
    }

    pub fn active_count(&self)->usize{self.sessions.iter().filter(|s|s.active).count()}
    pub fn device_count(&self)->usize{self.devices.len()}
}
extern crate tracing;
