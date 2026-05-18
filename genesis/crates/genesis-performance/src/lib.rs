//! Genesis Engine — Performance & Optimization System
//!
//! Makes every game run faster automatically:
//!
//! LOD (Level of Detail):
//! - Automatic LOD generation from high-res meshes
//! - Dynamic LOD based on distance + screen coverage
//! - Impostor LOD (billboard replace distant 3D objects)
//! - Hierarchical LOD (HLOD) for cities/dense scenes
//!
//! GPU Optimization:
//! - Frustum culling (don't render off-screen objects)
//! - Occlusion culling (don't render behind walls)
//! - GPU-driven rendering (indirect draw calls)
//! - Mesh instancing (1000 trees = 1 draw call)
//! - Texture streaming (load detail as camera approaches)
//! - Shader permutation pruning
//!
//! CPU Optimization:
//! - Entity streaming (load/unload based on camera)
//! - Spatial partitioning (only update nearby entities)
//! - Job system (parallel processing)
//! - Frame budget allocation per system
//! - AI token budget (balance quality vs speed)
//!
//! Memory:
//! - Asset streaming (load on demand)
//! - Memory pools (pre-allocated, no GC pauses)
//! - Texture compression (BC7, ASTC)
//! - Asset deduplication
//!
//! Network:
//! - Delta compression (only send changes)
//! - Client-side prediction (zero perceived lag)
//! - Bandwidth throttling per connection
//!
//! Profiling:
//! - Frame timeline (see every system's time)
//! - GPU profiler (see render passes)
//! - Memory tracker
//! - Network analyzer
//! - AI cost tracker (tokens, latency, cost per call)

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};

// ─── Frame Budget ─────────────────────────────────────────────────────────────

/// How much time each system gets per frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameBudget {
    pub target_fps: u32,
    pub target_frame_ms: f32,
    pub budgets: HashMap<String, f32>,   // system_name → ms budget
    pub actual_times: HashMap<String, f32>,
    pub over_budget_frames: u32,
    pub current_fps: f32,
    pub frame_times: VecDeque<f32>,     // ring buffer of last 120 frames
    pub mode: PerformanceMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceMode {
    /// Target 60 FPS — best for most games
    Performance60,
    /// Target 30 FPS — better quality settings
    Quality30,
    /// Target 120 FPS — competitive gaming
    Competitive120,
    /// Target 90 FPS — VR
    Vr90,
    /// Unlimited (editor, benchmark)
    Unlimited,
    /// Power saving (mobile, laptop on battery)
    PowerSave,
    /// Custom FPS target
    Custom(u32),
}

impl PerformanceMode {
    pub fn target_fps(&self) -> u32 {
        match self {
            Self::Performance60 => 60,
            Self::Quality30 => 30,
            Self::Competitive120 => 120,
            Self::Vr90 => 90,
            Self::Unlimited => 9999,
            Self::PowerSave => 30,
            Self::Custom(fps) => *fps,
        }
    }

    pub fn frame_budget_ms(&self) -> f32 {
        1000.0 / self.target_fps() as f32
    }
}

impl FrameBudget {
    pub fn new(mode: PerformanceMode) -> Self {
        let target = mode.target_fps();
        let frame_ms = 1000.0 / target as f32;

        let mut budgets = HashMap::new();
        budgets.insert("render".to_string(), frame_ms * 0.45);
        budgets.insert("physics".to_string(), frame_ms * 0.15);
        budgets.insert("ai_agents".to_string(), frame_ms * 0.15);
        budgets.insert("scripts".to_string(), frame_ms * 0.10);
        budgets.insert("audio".to_string(), frame_ms * 0.05);
        budgets.insert("network".to_string(), frame_ms * 0.05);
        budgets.insert("animation".to_string(), frame_ms * 0.05);

        Self {
            target_fps: target,
            target_frame_ms: frame_ms,
            budgets,
            actual_times: HashMap::new(),
            over_budget_frames: 0,
            current_fps: target as f32,
            frame_times: VecDeque::with_capacity(120),
            mode,
        }
    }

    pub fn record_frame(&mut self, frame_ms: f32) {
        if self.frame_times.len() >= 120 { self.frame_times.pop_front(); }
        self.frame_times.push_back(frame_ms);

        // Update FPS
        let avg = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        self.current_fps = 1000.0 / avg.max(0.001);

        if frame_ms > self.target_frame_ms * 1.1 {
            self.over_budget_frames += 1;
        }
    }

    pub fn avg_fps(&self) -> f32 { self.current_fps }
    pub fn is_over_budget(&self) -> bool { self.over_budget_frames > 3 }

    pub fn frame_time_percentile(&self, pct: f32) -> f32 {
        let mut sorted: Vec<f32> = self.frame_times.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((pct / 100.0) * sorted.len() as f32) as usize;
        sorted.get(idx.min(sorted.len()-1)).cloned().unwrap_or(0.0)
    }
}

// ─── LOD System ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodConfig {
    pub enabled: bool,
    pub levels: Vec<LodLevel>,
    pub auto_generate: bool,
    pub bias: f32,             // positive = more aggressive LOD, negative = higher quality
    pub force_level: Option<u32>,   // override for debugging
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodLevel {
    pub level: u32,
    pub mesh_asset_id: String,
    pub screen_size_threshold: f32,   // 0-1 (fraction of screen height)
    pub polygon_count: u32,
    pub shadow_lod: u32,              // which shadow LOD to use
    pub use_impostor: bool,           // replace with billboard at this level
    pub impostor_asset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpostorConfig {
    pub atlas_size: u32,          // texture resolution (1024, 2048)
    pub views: u32,               // number of angles captured (8, 16, 32)
    pub capture_on_import: bool,
    pub update_interval_secs: f32, // for animated objects
}

// ─── Culling ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingConfig {
    pub frustum_culling: bool,
    pub occlusion_culling: OcclusionMethod,
    pub small_feature_culling: bool,
    pub small_feature_threshold: f32,    // pixels — cull objects smaller than this
    pub distance_culling: bool,
    pub max_render_distance: f32,
    pub shadow_max_distance: f32,
    pub detail_objects_distance: f32,    // grass, pebbles max distance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OcclusionMethod {
    None,
    Software { depth_buffer_size: u32 },
    Hardware,
    HizBuffer,    // Hierarchical Z-buffer (best GPU)
}

// ─── GPU Optimization ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuOptimizationConfig {
    pub instanced_rendering: bool,
    pub gpu_driven_rendering: bool,   // indirect draw calls
    pub mesh_shaders: bool,           // DirectX 12 / Vulkan mesh shaders
    pub texture_streaming: bool,
    pub texture_stream_pool_mb: u32,
    pub texture_min_mip: u32,         // minimum mip level always loaded
    pub shader_cache: bool,
    pub pipeline_cache: bool,
    pub async_compute: bool,          // compute on separate GPU queue
    pub dlss_mode: UpscaleMode,
    pub fsr_mode: UpscaleMode,
    pub xess_mode: UpscaleMode,
    pub taa_enabled: bool,
    pub variable_rate_shading: bool,  // lower quality on less important areas
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpscaleMode {
    Off,
    Performance,   // 2x upscale (highest perf)
    Balanced,
    Quality,
    UltraQuality,  // minimal upscale
    Auto,
}

// ─── Memory Management ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub pool_size_mb: u32,              // pre-allocated general pool
    pub asset_cache_mb: u32,
    pub texture_budget_mb: u32,
    pub mesh_budget_mb: u32,
    pub audio_budget_mb: u32,
    pub gc_threshold: f32,              // trigger cleanup at this % usage
    pub streaming_enabled: bool,
    pub streaming_lookahead_m: f32,     // load assets this far ahead of camera
    pub compression: AssetCompression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCompression {
    pub textures: TextureCompression,
    pub meshes: bool,                   // vertex compression
    pub audio: bool,                    // lossless compress audio at rest
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureCompression {
    None,
    BC7,            // DirectX high quality
    ASTC_6x6,      // Mobile
    ASTC_4x4,      // Mobile high quality
    ETC2,          // Android
    DXT5,           // Legacy DX
}

// ─── Anti-Lag System ─────────────────────────────────────────────────────────

/// Automatically reduces settings when FPS drops below target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveQuality {
    pub enabled: bool,
    pub target_fps: f32,
    pub hysteresis_ms: f32,      // dead zone to prevent oscillation
    pub steps: Vec<QualityStep>,
    pub current_step: usize,
    pub hold_time_secs: f32,
    pub step_down_count: u32,
    pub step_up_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStep {
    pub name: String,
    pub resolution_scale: f32,   // 0.5-1.0
    pub shadow_quality: u32,     // 0-4
    pub ao_enabled: bool,
    pub bloom_enabled: bool,
    pub motion_blur: bool,
    pub grass_density: f32,
    pub draw_distance: f32,
    pub particle_budget: u32,
    pub reflections: ReflectionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReflectionQuality { Off, SSR, Probe, Raytraced }

impl AdaptiveQuality {
    pub fn standard() -> Self {
        Self {
            enabled: true,
            target_fps: 60.0,
            hysteresis_ms: 2.0,
            steps: vec![
                QualityStep {
                    name: "Ultra".to_string(), resolution_scale: 1.0, shadow_quality: 4,
                    ao_enabled: true, bloom_enabled: true, motion_blur: true,
                    grass_density: 1.0, draw_distance: 1000.0, particle_budget: 50000,
                    reflections: ReflectionQuality::SSR,
                },
                QualityStep {
                    name: "High".to_string(), resolution_scale: 1.0, shadow_quality: 3,
                    ao_enabled: true, bloom_enabled: true, motion_blur: true,
                    grass_density: 0.7, draw_distance: 800.0, particle_budget: 30000,
                    reflections: ReflectionQuality::Probe,
                },
                QualityStep {
                    name: "Medium".to_string(), resolution_scale: 0.9, shadow_quality: 2,
                    ao_enabled: true, bloom_enabled: true, motion_blur: false,
                    grass_density: 0.5, draw_distance: 600.0, particle_budget: 15000,
                    reflections: ReflectionQuality::Probe,
                },
                QualityStep {
                    name: "Low".to_string(), resolution_scale: 0.75, shadow_quality: 1,
                    ao_enabled: false, bloom_enabled: false, motion_blur: false,
                    grass_density: 0.3, draw_distance: 400.0, particle_budget: 5000,
                    reflections: ReflectionQuality::Off,
                },
                QualityStep {
                    name: "Minimum".to_string(), resolution_scale: 0.6, shadow_quality: 0,
                    ao_enabled: false, bloom_enabled: false, motion_blur: false,
                    grass_density: 0.0, draw_distance: 200.0, particle_budget: 1000,
                    reflections: ReflectionQuality::Off,
                },
            ],
            current_step: 1,
            hold_time_secs: 2.0,
            step_down_count: 0,
            step_up_count: 0,
        }
    }

    pub fn current_quality(&self) -> &QualityStep {
        &self.steps[self.current_step.min(self.steps.len()-1)]
    }

    pub fn tick(&mut self, current_fps: f32, delta: f32) {
        if !self.enabled { return; }
        if current_fps < self.target_fps - self.hysteresis_ms && self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
            self.step_down_count += 1;
            tracing::warn!("Performance: stepped down to '{}'", self.steps[self.current_step].name);
        } else if current_fps > self.target_fps + self.hysteresis_ms && self.current_step > 0 {
            self.current_step -= 1;
            self.step_up_count += 1;
            tracing::info!("Performance: stepped up to '{}'", self.steps[self.current_step].name);
        }
    }
}

// ─── Profiler ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfiler {
    pub enabled: bool,
    pub frame_history: VecDeque<FrameProfile>,
    pub history_size: usize,
    pub current_frame: FrameProfile,
    pub alerts: Vec<PerformanceAlert>,
    pub gpu_profiler_enabled: bool,
    pub network_profiler_enabled: bool,
    pub ai_cost_tracker: AiCostTracker,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameProfile {
    pub frame_number: u64,
    pub total_ms: f32,
    pub render_ms: f32,
    pub physics_ms: f32,
    pub scripts_ms: f32,
    pub audio_ms: f32,
    pub network_ms: f32,
    pub ai_ms: f32,
    pub animation_ms: f32,
    pub ui_ms: f32,
    pub draw_calls: u32,
    pub triangles: u64,
    pub entities_updated: u32,
    pub entities_rendered: u32,
    pub entities_culled: u32,
    pub particles_alive: u32,
    pub vram_used_mb: f64,
    pub ram_used_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub system: String,
    pub message: String,
    pub value: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity { Info, Warning, Critical }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiCostTracker {
    pub session_tokens: u64,
    pub session_cost_usd: f64,
    pub session_requests: u64,
    pub per_agent: HashMap<String, AgentCostEntry>,
    pub daily_budget_usd: f64,
    pub today_cost_usd: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentCostEntry {
    pub tokens: u64,
    pub cost_usd: f64,
    pub requests: u64,
    pub avg_latency_ms: f32,
}

// ─── Fast Launch Optimizer ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchOptimizer {
    pub cold_start_ms: u32,
    pub warm_start_ms: u32,
    pub last_project: Option<String>,
    pub shader_cache_valid: bool,
    pub asset_cache_valid: bool,
    pub ai_model_loaded: bool,
    pub engine_ready_ms: u32,
    pub optimizations_applied: Vec<String>,
}

impl LaunchOptimizer {
    pub fn estimate_startup_ms(&self, is_first_launch: bool) -> u32 {
        if is_first_launch {
            self.cold_start_ms
        } else {
            self.warm_start_ms
        }
    }
}

extern crate tracing;
