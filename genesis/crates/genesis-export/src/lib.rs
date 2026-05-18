//! Export Pipeline — build targets, platform packaging, signing, distribution
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ExportTarget {
    Windows  { arch:Arch, installer:bool, portable:bool },
    MacOs    { arch:Arch, notarize:bool, dmg:bool },
    Linux    { arch:Arch, appimage:bool, deb:bool, rpm:bool },
    Web      { wasm:bool, progressive_web_app:bool, embed_assets:bool },
    Android  { api_min:u32, abi:Vec<String>, aab:bool, apk:bool },
    Ios      { min_version:String, bitcode:bool },
    Ps5      { sdk_version:String },
    Xbox     { gdk_version:String },
    Switch   { sdk_version:String },
    SteamDeck{ verify:bool },
    Custom   { name:String, script:String },
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Arch { X64, Arm64, Universal, X86 }

impl ExportTarget {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Windows{..}=>"Windows", Self::MacOs{..}=>"macOS",
            Self::Linux{..}=>"Linux", Self::Web{..}=>"Web",
            Self::Android{..}=>"Android", Self::Ios{..}=>"iOS",
            Self::Ps5{..}=>"PS5", Self::Xbox{..}=>"Xbox",
            Self::Switch{..}=>"Switch", Self::SteamDeck{..}=>"Steam Deck",
            Self::Custom{name,..}=>name,
        }
    }
    pub fn is_console(&self)->bool{ matches!(self,Self::Ps5{..}|Self::Xbox{..}|Self::Switch{..}) }
    pub fn is_mobile(&self)->bool{ matches!(self,Self::Android{..}|Self::Ios{..}) }
    pub fn needs_gpu(&self)->bool{ !matches!(self,Self::Web{..}) }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ExportConfig {
    pub target:           ExportTarget,
    pub output_dir:       String,
    pub build_type:       BuildType,
    pub version:          String,
    pub build_number:     u64,
    pub app_name:         String,
    pub bundle_id:        String,    // com.studio.game
    pub icon_path:        String,
    pub splash_path:      Option<String>,
    pub compress_assets:  bool,
    pub encrypt_assets:   bool,
    pub strip_debug:      bool,
    pub include_pdb:      bool,
    pub sign:             bool,
    pub sign_identity:    Option<String>,
    pub notarize:         bool,
    pub steam_app_id:     Option<u64>,
    pub feature_flags:    HashMap<String,bool>,
    pub env_vars:         HashMap<String,String>,
    pub custom_scripts:   Vec<String>,
    pub delta_updates:    bool,
    pub max_texture_res:  u32,
    pub target_fps:       u32,
    pub locales:          Vec<String>,
    pub upx_compress:     bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum BuildType { Debug, Release, RelWithDebInfo, MinSizeRel }

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum ExportStatus { Idle, Preparing, Building, Packaging, Signing, Done, Failed(String) }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ExportJob {
    pub id:          String,
    pub config:      ExportConfig,
    pub status:      ExportStatus,
    pub progress:    f32,
    pub stage:       String,
    pub started_at:  Option<DateTime<Utc>>,
    pub completed_at:Option<DateTime<Utc>>,
    pub output_path: Option<String>,
    pub size_bytes:  u64,
    pub log:         Vec<String>,
    pub warnings:    Vec<String>,
    pub errors:      Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ExportStats {
    pub total_exports:   u64,
    pub successful:      u64,
    pub failed:          u64,
    pub total_size_bytes:u64,
    pub last_export:     Option<DateTime<Utc>>,
    pub per_target:      HashMap<String,u32>,
}

pub struct ExportPipeline {
    pub jobs:    Vec<ExportJob>,
    pub history: Vec<ExportJob>,
    pub stats:   ExportStats,
    pub active:  Option<String>,
    pub auto_upload_steam:   bool,
    pub auto_upload_itch:    bool,
    pub cdn_url: Option<String>,
}

impl ExportPipeline {
    pub fn new() -> Self {
        Self {
            jobs:Vec::new(), history:Vec::new(),
            stats:ExportStats{total_exports:0,successful:0,failed:0,total_size_bytes:0,last_export:None,per_target:HashMap::new()},
            active:None, auto_upload_steam:false, auto_upload_itch:false, cdn_url:None,
        }
    }

    pub fn queue(&mut self, config:ExportConfig) -> String {
        let id = format!("export_{}", Utc::now().timestamp_millis());
        let target_name = config.target.name().to_string();
        tracing::info!("Export queued: {} → {} ({})", config.app_name, target_name, id);
        let job = ExportJob {
            id:id.clone(), config, status:ExportStatus::Idle, progress:0.0,
            stage:"Queued".to_string(), started_at:None, completed_at:None,
            output_path:None, size_bytes:0, log:Vec::new(), warnings:Vec::new(), errors:Vec::new(),
        };
        self.jobs.push(job);
        self.stats.total_exports += 1;
        *self.stats.per_target.entry(target_name).or_insert(0) += 1;
        id
    }

    pub fn start(&mut self, id:&str) -> bool {
        if self.active.is_some() { return false; }
        if let Some(job) = self.jobs.iter_mut().find(|j|j.id==id) {
            job.status = ExportStatus::Preparing;
            job.started_at = Some(Utc::now());
            job.stage = "Preparing assets".to_string();
            self.active = Some(id.to_string());
            tracing::info!("Export started: {}", id);
            true
        } else { false }
    }

    pub fn complete(&mut self, id:&str, path:&str, size:u64) {
        if let Some(job) = self.jobs.iter_mut().find(|j|j.id==id) {
            job.status = ExportStatus::Done;
            job.progress = 1.0;
            job.stage = "Complete".to_string();
            job.completed_at = Some(Utc::now());
            job.output_path = Some(path.to_string());
            job.size_bytes = size;
            self.stats.successful += 1;
            self.stats.total_size_bytes += size;
            self.stats.last_export = Some(Utc::now());
            tracing::info!("Export complete: {} → {} ({:.1}MB)", id, path, size as f32/1_048_576.0);
        }
        if self.active.as_deref() == Some(id) { self.active = None; }
    }

    pub fn fail(&mut self, id:&str, error:&str) {
        if let Some(job) = self.jobs.iter_mut().find(|j|j.id==id) {
            job.status = ExportStatus::Failed(error.to_string());
            job.errors.push(error.to_string());
            job.completed_at = Some(Utc::now());
            self.stats.failed += 1;
            tracing::error!("Export failed {}: {}", id, error);
        }
        if self.active.as_deref() == Some(id) { self.active = None; }
    }

    pub fn active_job(&self) -> Option<&ExportJob> {
        self.active.as_ref().and_then(|id| self.jobs.iter().find(|j|&j.id==id))
    }
    pub fn is_busy(&self) -> bool { self.active.is_some() }
    pub fn pending_count(&self) -> usize { self.jobs.iter().filter(|j| j.status==ExportStatus::Idle).count() }
}
extern crate tracing;
