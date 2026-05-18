//! ChronoVerse AI Rigging & Motion Capture System
//!
//! Three ways to create animation rigs and animations:
//!
//! 1. AUTO-RIG FROM MESH (UniRig / HumanRig / RigNet)
//!    - Upload mesh → AI predicts skeleton → rig applied
//!    - Fallback: send to Mixamo API, get back rigged GLB
//!
//! 2. MOTION CAPTURE FROM VIDEO (like Rokoko / MovNet)
//!    - Record video of someone moving
//!    - AI extracts 3D pose per frame (MediaPipe / WHAM / SMPL)
//!    - Clean + retarget to game skeleton
//!    - Works with: phone camera, webcam, YouTube video
//!
//! 3. AI ANIMATION GENERATION (text → motion)
//!    - "Run and jump over obstacle" → animation clip
//!    - Uses MDM / MotionDiffuse / T2M-GPT
//!    - Retarget to any rig
//!
//! Motion Matching (Ubisoft-style):
//! - Large database of motion clips
//! - Match current character state to best clip
//! - Blend seamlessly between matched poses
//! - Results in Horizon/AC-quality movement

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use genesis_math::vec3::Vec3;

// ─── Auto Rigging ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRigRequest {
    pub id: String,
    pub mesh_asset_id: String,
    pub rig_type: RigType,
    pub method: AutoRigMethod,
    pub generate_animations: Vec<String>, // animation names to generate
    pub retarget_animations_from: Vec<String>, // existing animation IDs to retarget
    pub status: RigStatus,
    pub result_rig_id: Option<String>,
    pub result_mesh_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RigType {
    Humanoid,        // bipedal humanoid (most common)
    Quadruped,       // 4-legged animals
    Fish,            // swimming creatures
    Bird,            // winged flyers
    Snake,           // elongated bodies
    SpiderLike,      // 8 limbs
    Custom(String),  // from description
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoRigMethod {
    /// Local neural network (UniRig-style)
    Local { model_path: String },
    /// Mixamo online API (sends mesh, gets rigged GLB back)
    Mixamo { email: String },
    /// HumanRig (fast, humanoid only)
    HumanRig,
    /// RigNet (graph neural network, any mesh)
    RigNet,
    /// Send to GPU lab for heavy processing
    GpuLab { job_type: String },
    /// Manual rig from template
    Template { template_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RigStatus {
    Pending,
    Uploading,
    Processing { stage: String, progress: f32 },
    Downloading,
    Complete,
    Failed { reason: String },
}

impl AutoRigRequest {
    pub fn new(mesh_id: &str, rig_type: RigType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            mesh_asset_id: mesh_id.to_string(),
            rig_type,
            method: AutoRigMethod::Mixamo { email: String::new() },
            generate_animations: Vec::new(),
            retarget_animations_from: Vec::new(),
            status: RigStatus::Pending,
            result_rig_id: None,
            result_mesh_id: None,
        }
    }

    pub fn with_animations(mut self, animations: Vec<&str>) -> Self {
        self.generate_animations = animations.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn via_mixamo(mut self, email: &str) -> Self {
        self.method = AutoRigMethod::Mixamo { email: email.to_string() };
        self
    }

    pub fn via_local(mut self, model_path: &str) -> Self {
        self.method = AutoRigMethod::Local { model_path: model_path.to_string() };
        self
    }
}

// ─── Video Motion Capture ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMocapRequest {
    pub id: String,
    pub source: VideoSource,
    pub extraction_model: PoseExtractionModel,
    pub target_rig_id: Option<String>,      // retarget to this rig
    pub target_skeleton: Option<String>,
    pub cleanup_settings: MocapCleanupSettings,
    pub output_clips: Vec<MocapOutputClip>,
    pub status: MocapStatus,
    pub fps_detected: Option<f32>,
    pub frame_count: Option<u32>,
    pub person_count: Option<u8>,
    pub quality_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoSource {
    /// Local video file
    File { path: String },
    /// Phone camera (real-time)
    PhoneCamera { session_id: String },
    /// Webcam (real-time)
    Webcam { device_id: String },
    /// YouTube URL
    YouTube { url: String, start_sec: f32, end_sec: f32 },
    /// Already uploaded asset
    Asset { asset_id: String },
    /// Multiple camera angles (better reconstruction)
    MultiCamera { sources: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoseExtractionModel {
    /// Google MediaPipe Pose (fast, runs local)
    MediaPipe { complexity: u8 },
    /// WHAM (World-grounded Human pose and shape estimation)
    Wham,
    /// SMPL body model fitting
    SmplFit { gender: String },
    /// 4DHumans (video understanding)
    FourDHumans,
    /// OpenPose (classic, good accuracy)
    OpenPose,
    /// AlphaPose (multi-person)
    AlphaPose { max_persons: u8 },
    /// Cloud-based (best quality)
    CloudMocap { provider: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MocapCleanupSettings {
    /// Remove noise/jitter from joints
    pub denoise: bool,
    pub denoise_strength: f32,
    /// Fix feet sliding on ground
    pub foot_contact_fix: bool,
    /// Fix hands penetrating body
    pub body_penetration_fix: bool,
    /// Smooth transitions between frames
    pub temporal_smoothing: f32,
    /// Fill in frames where tracking was lost
    pub fill_gaps: bool,
    /// Normalize to T-pose at start
    pub normalize_to_tpose: bool,
    /// Root motion extraction
    pub extract_root_motion: bool,
    /// Hip height normalization
    pub normalize_hip_height: bool,
}

impl Default for MocapCleanupSettings {
    fn default() -> Self {
        Self {
            denoise: true, denoise_strength: 0.5,
            foot_contact_fix: true,
            body_penetration_fix: true,
            temporal_smoothing: 0.3,
            fill_gaps: true,
            normalize_to_tpose: true,
            extract_root_motion: true,
            normalize_hip_height: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MocapOutputClip {
    pub name: String,
    pub start_frame: u32,
    pub end_frame: u32,
    pub looping: bool,
    pub tags: Vec<String>,
    pub result_asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MocapStatus {
    Pending,
    Downloading,     // for YouTube
    Extracting { frame: u32, total: u32 },
    Cleaning,
    Retargeting,
    Exporting,
    Complete,
    Failed { reason: String },
}

// ─── AI Animation Generation ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextToMotionRequest {
    pub id: String,
    pub prompt: String,           // "Person runs and jumps over a table"
    pub duration_secs: f32,
    pub model: TextToMotionModel,
    pub target_rig_id: Option<String>,
    pub style_reference: Option<String>, // optional: reference animation ID
    pub variations: u8,           // how many variations to generate
    pub result_clips: Vec<String>,
    pub status: RigStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextToMotionModel {
    /// MDM - Motion Diffusion Model (Meta Research)
    Mdm,
    /// T2M-GPT
    T2mGpt,
    /// MotionDiffuse
    MotionDiffuse,
    /// MOTION-X (large dataset)
    MotionX,
    /// GPU Lab job
    GpuLab,
}

impl TextToMotionRequest {
    pub fn new(prompt: &str, duration: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            prompt: prompt.to_string(),
            duration_secs: duration,
            model: TextToMotionModel::GpuLab,
            target_rig_id: None,
            style_reference: None,
            variations: 3,
            result_clips: Vec::new(),
            status: RigStatus::Pending,
        }
    }
}

// ─── Motion Matching System ───────────────────────────────────────────────────

/// Motion matching database entry (one frame of animation)
#[derive(Debug, Clone)]
pub struct MotionFrame {
    pub clip_id: String,
    pub frame: u32,
    pub time: f32,
    /// Pose features for matching (joint positions/velocities)
    pub pose_features: PoseFeatures,
    /// Trajectory features (where character was/is going)
    pub trajectory: TrajectoryFeatures,
    pub tags: Vec<String>,
    pub contact_state: ContactState,
}

/// Compact pose representation for fast matching
#[derive(Debug, Clone)]
pub struct PoseFeatures {
    /// Normalized joint positions relative to hips (key joints only)
    pub joint_positions: Vec<[f32; 3]>, // ~20 joints
    /// Joint velocities
    pub joint_velocities: Vec<[f32; 3]>,
    pub root_velocity: Vec3,
    pub root_angular_velocity: Vec3,
}

/// Past and future trajectory of the character
#[derive(Debug, Clone)]
pub struct TrajectoryFeatures {
    /// Positions in the past (t-0.2s, t-0.4s)
    pub past_positions: Vec<Vec3>,
    /// Future positions (t+0.2s, t+0.4s, t+0.6s)
    pub future_positions: Vec<Vec3>,
    /// Future facing directions
    pub future_directions: Vec<Vec3>,
}

#[derive(Debug, Clone)]
pub struct ContactState {
    pub left_foot_down: bool,
    pub right_foot_down: bool,
    pub left_hand_down: bool,
    pub right_hand_down: bool,
}

/// The motion matching database
pub struct MotionDatabase {
    pub frames: Vec<MotionFrame>,
    /// KD-tree or ball-tree for fast nearest-neighbor search
    /// In production: use faiss or similar
    pub feature_dim: usize,
    /// Feature normalization parameters
    pub mean: Vec<f32>,
    pub std_dev: Vec<f32>,
}

impl MotionDatabase {
    pub fn new() -> Self {
        Self { frames: Vec::new(), feature_dim: 0, mean: Vec::new(), std_dev: Vec::new() }
    }

    /// Add animation clip to database
    pub fn add_clip(&mut self, clip_id: &str, frames: Vec<MotionFrame>) {
        self.frames.extend(frames);
        tracing::info!("Motion DB: added clip '{}', total frames: {}", clip_id, self.frames.len());
    }

    /// Find best matching frame for current state
    pub fn query(&self, query: &MotionFrame, k: usize) -> Vec<(usize, f32)> {
        // Simplified linear search — in production: KD-tree
        let mut distances: Vec<(usize, f32)> = self.frames.iter().enumerate()
            .map(|(i, frame)| (i, self.pose_distance(&query.pose_features, &frame.pose_features)))
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        distances.truncate(k);
        distances
    }

    fn pose_distance(&self, a: &PoseFeatures, b: &PoseFeatures) -> f32 {
        let pos_dist: f32 = a.joint_positions.iter().zip(&b.joint_positions)
            .map(|(pa, pb)| {
                let d = Vec3::from(*pa) - Vec3::from(*pb);
                d.length_squared()
            })
            .sum();

        let vel_dist: f32 = a.joint_velocities.iter().zip(&b.joint_velocities)
            .map(|(pa, pb)| {
                let d = Vec3::from(*pa) - Vec3::from(*pb);
                d.length_squared() * 0.5 // velocity matters less
            })
            .sum();

        (pos_dist + vel_dist).sqrt()
    }

    pub fn frame_count(&self) -> usize { self.frames.len() }
}

/// Motion matching controller for a character
pub struct MotionMatchingController {
    pub database: MotionDatabase,
    pub current_frame_idx: usize,
    pub blend_frames: u32,         // frames to blend when switching
    pub blend_remaining: u32,
    pub prev_frame_idx: Option<usize>,
    pub query_interval: u32,       // how often to query (frames)
    pub frame_counter: u32,
    pub trajectory_weight: f32,
    pub pose_weight: f32,
    pub inertia_weight: f32,       // cost of switching (prefer continuity)
}

impl MotionMatchingController {
    pub fn new(database: MotionDatabase) -> Self {
        Self {
            database,
            current_frame_idx: 0,
            blend_frames: 15,
            blend_remaining: 0,
            prev_frame_idx: None,
            query_interval: 6,  // query every 6 frames (~10Hz at 60fps)
            frame_counter: 0,
            trajectory_weight: 0.7,
            pose_weight: 1.0,
            inertia_weight: 0.3,
        }
    }

    pub fn tick(&mut self, current_state: &MotionFrame, delta: f32) -> usize {
        self.frame_counter += 1;
        self.current_frame_idx += 1;

        if self.frame_counter >= self.query_interval {
            self.frame_counter = 0;
            let candidates = self.database.query(current_state, 5);
            if let Some((best_idx, _)) = candidates.first() {
                if *best_idx != self.current_frame_idx {
                    self.prev_frame_idx = Some(self.current_frame_idx);
                    self.current_frame_idx = *best_idx;
                    self.blend_remaining = self.blend_frames;
                }
            }
        }

        if self.blend_remaining > 0 { self.blend_remaining -= 1; }

        self.current_frame_idx
    }

    pub fn blend_alpha(&self) -> f32 {
        if self.blend_frames == 0 { return 1.0; }
        1.0 - (self.blend_remaining as f32 / self.blend_frames as f32)
    }
}

// ─── Model Download System ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadRequest {
    pub id: String,
    pub model_name: String,
    pub source: ModelSource,
    pub target_format: ModelFormat,
    pub quantization: Option<QuantizationLevel>,
    pub auto_load: bool,
    pub status: DownloadStatus,
    pub progress: f32,
    pub size_bytes: Option<u64>,
    pub downloaded_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSource {
    HuggingFace { repo: String, filename: Option<String> },
    CivitAI { model_id: u64, version_id: Option<u64> },
    OllamaHub { model_name: String },
    DirectUrl { url: String },
    LocalPath { path: String },
    GpuLab { job_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,    // llama.cpp compatible
    Safetensors, // HuggingFace native
    Onnx,   // cross-platform inference
    Pt,     // PyTorch
    Bin,    // generic binary
    Auto,   // detect from source
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantizationLevel {
    Q2_K, Q3_K_M, Q4_0, Q4_K_M, Q5_K_M, Q6_K, Q8_0,
    F16, F32,
    Auto, // choose best for available VRAM
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Connecting,
    Downloading { speed_mbps: f32 },
    Converting,
    Quantizing,
    Verifying,
    Loading,
    Complete,
    Failed { reason: String },
    Cancelled,
}

impl ModelDownloadRequest {
    pub fn from_huggingface(repo: &str, filename: Option<&str>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            model_name: repo.split('/').last().unwrap_or(repo).to_string(),
            source: ModelSource::HuggingFace {
                repo: repo.to_string(),
                filename: filename.map(|s| s.to_string()),
            },
            target_format: ModelFormat::Gguf,
            quantization: Some(QuantizationLevel::Q4_K_M),
            auto_load: true,
            status: DownloadStatus::Queued,
            progress: 0.0,
            size_bytes: None,
            downloaded_bytes: 0,
        }
    }

    pub fn from_ollama(model_name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            model_name: model_name.to_string(),
            source: ModelSource::OllamaHub { model_name: model_name.to_string() },
            target_format: ModelFormat::Gguf,
            quantization: None, // Ollama handles this
            auto_load: true,
            status: DownloadStatus::Queued,
            progress: 0.0,
            size_bytes: None,
            downloaded_bytes: 0,
        }
    }
}

/// Known good models for different tasks
pub struct ModelRegistry;
impl ModelRegistry {
    pub const NPC_DIALOGUE_FAST: &'static str = "Qwen/Qwen2.5-0.5B-Instruct-GGUF";
    pub const NPC_DIALOGUE_QUALITY: &'static str = "Qwen/Qwen2.5-7B-Instruct-GGUF";
    pub const NPC_STORY: &'static str = "mistralai/Mistral-7B-Instruct-v0.3-GGUF";
    pub const CODE_GENERATION: &'static str = "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF";
    pub const EMBEDDING: &'static str = "nomic-ai/nomic-embed-text-v1.5-GGUF";
    pub const TTS_FAST: &'static str = "hexgrad/Kokoro-82M";
    pub const TTS_QUALITY: &'static str = "coqui/XTTS-v2";
    pub const VOICE_CLONE: &'static str = "myshell-ai/OpenVoice-v2";
    pub const FACE_RECON: &'static str = "deepinsight/insightface";
    pub const POSE_ESTIMATION: &'static str = "google/mediapipe-pose-landmarker";
    pub const TEXT_TO_MOTION: &'static str = "SeanChenxy/SMPL-MDM";
    pub const IMAGE_GEN: &'static str = "black-forest-labs/FLUX.1-schnell";
    pub const VISION: &'static str = "Qwen/Qwen2.5-VL-7B-Instruct-GGUF";

    pub fn recommended_for(task: &str, quality: &str) -> Option<&'static str> {
        match (task, quality) {
            ("npc_dialogue", "fast") => Some(Self::NPC_DIALOGUE_FAST),
            ("npc_dialogue", "quality") => Some(Self::NPC_DIALOGUE_QUALITY),
            ("code", _) => Some(Self::CODE_GENERATION),
            ("tts", "fast") => Some(Self::TTS_FAST),
            ("tts", "quality") => Some(Self::TTS_QUALITY),
            ("voice_clone", _) => Some(Self::VOICE_CLONE),
            ("pose", _) => Some(Self::POSE_ESTIMATION),
            ("motion_gen", _) => Some(Self::TEXT_TO_MOTION),
            ("vision", _) => Some(Self::VISION),
            _ => None,
        }
    }
}

pub struct ModelManager {
    pub downloads: HashMap<String, ModelDownloadRequest>,
    pub loaded_models: HashMap<String, LoadedModel>,
    pub download_queue: Vec<String>,
    pub models_dir: std::path::PathBuf,
    pub max_concurrent_downloads: u8,
    pub active_downloads: u8,
    pub total_vram_mb: f64,
    pub used_vram_mb: f64,
}

#[derive(Debug, Clone)]
pub struct LoadedModel {
    pub id: String,
    pub name: String,
    pub path: String,
    pub format: ModelFormat,
    pub size_mb: f64,
    pub vram_usage_mb: f64,
    pub backend: InferenceBackend,
    pub loaded_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum InferenceBackend {
    LlamaCpp { ctx_size: u32, threads: u32, n_gpu_layers: i32 },
    Onnx { providers: Vec<String> },
    Safetensors { device: String },
    Ollama { endpoint: String },
    External { endpoint: String },
}

impl ModelManager {
    pub fn new(models_dir: std::path::PathBuf) -> Self {
        Self {
            downloads: HashMap::new(),
            loaded_models: HashMap::new(),
            download_queue: Vec::new(),
            models_dir,
            max_concurrent_downloads: 2,
            active_downloads: 0,
            total_vram_mb: 0.0,
            used_vram_mb: 0.0,
        }
    }

    pub fn queue_download(&mut self, request: ModelDownloadRequest) -> String {
        let id = request.id.clone();
        self.download_queue.push(id.clone());
        self.downloads.insert(id.clone(), request);
        tracing::info!("Queued model download: {}", id);
        id
    }

    pub fn download_recommended(&mut self, task: &str, quality: &str) -> Option<String> {
        let model_id = ModelRegistry::recommended_for(task, quality)?;
        let request = ModelDownloadRequest::from_huggingface(model_id, None);
        Some(self.queue_download(request))
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded_models.contains_key(name)
    }

    pub fn available_vram_mb(&self) -> f64 {
        self.total_vram_mb - self.used_vram_mb
    }
}

extern crate uuid;
extern crate tracing;
