//! Asset Pipeline — import, process, stream, cache all game assets
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum AssetKind {
    Mesh3D{verts:u32,polys:u32,rigged:bool},
    Texture{res:[u32;2],format:String,mips:u32},
    Material{kind:String}, Audio{secs:f32,channels:u8,hz:u32},
    Animation{clips:u32,fps:f32}, Font{glyphs:u32},
    Script{lang:String}, Scene{objects:u32},
    Prefab{components:u32}, Shader{target:String},
    VfxEffect{realtime:bool}, Video{w:u32,h:u32,fps:f32,secs:f32},
    Data{format:String}, Unknown,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum AssetState { Unloaded, Loading, Loaded, Streaming, Error(String), Evicted }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AssetHandle {
    pub id:       String,
    pub path:     String,
    pub kind:     AssetKind,
    pub state:    AssetState,
    pub size_bytes: u64,
    pub version:  u32,
    pub checksum: String,
    pub imported: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub tags:     Vec<String>,
    pub lod_paths: Vec<String>,
    pub streaming: bool,
    pub priority:  u32,
    pub ref_count: u32,
    pub memory_bytes: u64,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ImportSettings {
    pub source_path:     String,
    pub generate_lods:   bool,
    pub lod_levels:      u32,
    pub compress_textures: bool,
    pub texture_format:  String,
    pub max_tex_res:     u32,
    pub generate_mips:   bool,
    pub optimize_mesh:   bool,
    pub target_polys:    Option<u32>,
    pub auto_rig:        bool,
    pub normalize_scale: bool,
    pub embed_textures:  bool,
    pub audio_normalize: bool,
    pub audio_format:    String,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self { source_path:String::new(), generate_lods:true, lod_levels:4,
               compress_textures:true, texture_format:"BC7".to_string(),
               max_tex_res:4096, generate_mips:true, optimize_mesh:true,
               target_polys:None, auto_rig:false, normalize_scale:true,
               embed_textures:false, audio_normalize:true, audio_format:"OGG".to_string() }
    }
}

pub struct AssetPipeline {
    pub registry:    HashMap<String,AssetHandle>,
    pub import_queue: Vec<ImportSettings>,
    pub cache_dir:   String,
    pub max_memory_mb: u64,
    pub used_memory_bytes: u64,
    pub hot_reload:  bool,
    pub streaming_budget_ms: f32,
    pub total_imported: u64,
    pub total_errors:   u32,
}

impl AssetPipeline {
    pub fn new(cache_dir:&str) -> Self {
        Self { registry:HashMap::new(), import_queue:Vec::new(),
               cache_dir:cache_dir.to_string(), max_memory_mb:2048,
               used_memory_bytes:0, hot_reload:true,
               streaming_budget_ms:4.0, total_imported:0, total_errors:0 }
    }

    pub fn queue_import(&mut self, settings:ImportSettings) {
        tracing::info!("Queued import: {}", settings.source_path);
        self.import_queue.push(settings);
    }

    pub fn get(&self, id:&str) -> Option<&AssetHandle> { self.registry.get(id) }
    pub fn is_loaded(&self, id:&str) -> bool {
        self.registry.get(id).map(|a| a.state == AssetState::Loaded).unwrap_or(false)
    }

    pub fn retain(&mut self, id:&str) {
        if let Some(a) = self.registry.get_mut(id) { a.ref_count += 1; }
    }
    pub fn release(&mut self, id:&str) {
        if let Some(a) = self.registry.get_mut(id) {
            a.ref_count = a.ref_count.saturating_sub(1);
            if a.ref_count == 0 && !a.streaming {
                a.state = AssetState::Evicted;
                self.used_memory_bytes = self.used_memory_bytes.saturating_sub(a.memory_bytes);
            }
        }
    }

    pub fn count(&self) -> usize { self.registry.len() }
    pub fn memory_mb(&self) -> f64 { self.used_memory_bytes as f64 / 1_048_576.0 }
    pub fn pending_imports(&self) -> usize { self.import_queue.len() }
}
extern crate tracing;
