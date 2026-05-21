//! Genesis VFX Compositor — Advanced Studio Effects
//!
//! Everything for the virtual production pipeline:
//! - Node-based compositing (like Nuke/After Effects)
//! - Real-time performance capture pipeline
//! - Virtual camera system
//! - Live 3D compositing (actor in game world)
//! - Depth-of-field rendering
//! - Motion blur
//! - Lens effects (flares, aberrations, vignette)
//! - HDR workflows
//! - DeepComp (Z-depth compositing)
//! - Rotoscoping (AI-assisted)
//! - Color science (LUTs, ACES, DCI-P3)
//! - Keying (green screen, blue screen, luma key)
//! - Stabilization (AI-powered)
//! - Noise reduction
//! - Up-scaling (DLSS/FSR/XeSS equivalent)
//! - AI face replacement (one actor, many characters)
//! - AI body replacement (actor body → character model)
//! - Real-time results preview

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// ─── Compositing Node Graph ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompNode {
    pub id: String,
    pub name: String,
    pub node_type: CompNodeType,
    pub position: [f32; 2],
    pub width: f32,
    pub inputs: Vec<CompPin>,
    pub outputs: Vec<CompPin>,
    pub properties: HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub muted: bool,
    pub preview: bool,    // show mini-preview in node header
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompPin {
    pub name: String,
    pub pin_type: CompPinType,
    pub connected: bool,
    pub socket_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompPinType {
    Rgba, Alpha, Depth, Vector, Value, Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompNodeType {
    // ─── Input ──────────────────────────────────────────────────────
    Image        { path: String },
    Movie        { path: String, frame: i32 },
    RenderLayer  { scene_id: String, layer: String },
    GameCapture  { scene_id: String, camera_id: String },
    LiveCapture  { camera_device: String },
    Mask         { mask_id: String },
    ColorRamp,
    ValueNode    { value: f32 },
    RgbNode      { color: [f32; 4] },
    Texture      { texture_id: String },
    TimeNode,    // current frame number
    SceneData    { data_path: String },

    // ─── Output ─────────────────────────────────────────────────────
    Composite,     // final output
    Viewer,        // preview in viewport
    SplitViewer    { factor: f32 },
    FileOutput    { path: String, format: String },

    // ─── Color ──────────────────────────────────────────────────────
    AlphaOver    { use_premultiply: bool },
    AlphaConvert { pre_multiply: bool },
    ColorBalance { shadows: [f32; 4], midtones: [f32; 4], highlights: [f32; 4] },
    Brightness   { brightness: f32, contrast: f32 },
    Gamma        { gamma: f32 },
    HueSaturation { hue: f32, saturation: f32, value: f32 },
    Rgb,
    InvertNode,
    Mix          { blend_type: BlendType, factor: f32 },
    ColorCorrection { master_r: f32, master_g: f32, master_b: f32, master_g2: f32 },
    ExposureNode { exposure: f32 },
    Tonemap      { key_val: f32, offset: f32, gamma: f32, intensity: f32 },

    // ─── Filtering ─────────────────────────────────────────────────
    Blur         { size_x: f32, size_y: f32, filter: String },
    DirectionalBlur { zoom: f32, spin: f32, iterations: u32 },
    Bilateral    { sigma_color: f32, sigma_space: f32 },
    Dilate       { distance: f32, edge_amount: f32 },
    Erode        { distance: f32 },
    InPaint      { distance: u32 },
    Despeckle    { threshold: f32 },
    Sharpen      { strength: f32 },
    AntiAlias,
    Denoiser     { algorithm: DenoiseAlgo },

    // ─── Vector ────────────────────────────────────────────────────
    MapUV,
    Displace     { scale: f32 },
    Flip         { x: bool, y: bool },
    Crop         { x1: f32, y1: f32, x2: f32, y2: f32, relative: bool },
    Scale        { factor: f32, interpolation: String },
    Transform    { x: f32, y: f32, angle: f32, scale: f32 },
    Stabilize2D,

    // ─── Matte/Keying ──────────────────────────────────────────────
    ChromaMatte  { key_color: [f32; 4], tolerance: f32, falloff: f32, spillsup: f32 },
    ColorMatte   { key_color: [f32; 4], h: f32, s: f32, v: f32 },
    DifferMatte  { tolerance: f32, falloff: f32 },
    LumaMatte    { hi: f32, lo: f32 },
    KeyingScreen { tracking_object: String },
    Keying       { key_color: [f32; 4], screen_balance: f32 },

    // ─── Distortion ────────────────────────────────────────────────
    LensDistort  { k1: f32, k2: f32, k3: f32 },
    PixelArt     { pixel_size: u32 },
    Glare        { type_: GlareType, quality: u32, iterations: u32, fade: f32, threshold: f32 },
    VignetteNode { falloff: f32, midpoint: f32, roundness: f32, feather: f32 },

    // ─── Depth ─────────────────────────────────────────────────────
    Defocus      { z_scale: f32, max_blur: f32 },
    BokehBlur    { blur: f32 },
    BokehImage   { angle: f32, flaps: u32, rounding: f32 },
    ZCombine     { use_alpha: bool },

    // ─── AI-Specific ───────────────────────────────────────────────
    AiFaceSwap   { source_capture: String, target_character: String, blend: f32 },
    AiBodyReplace{ actor_capture: String, character_mesh: String },
    AiRotoscope  { threshold: f32, edge_smooth: f32 },
    AiUpscale    { scale: f32, algorithm: UpscaleAlgo },
    AiStabilize  { strength: f32, rolling_shutter: bool },
    AiDeepfake   { source_actor: String, target_actor: String },  // consented only!
    AiBackground { prompt: Option<String>, reference: Option<String> },

    // ─── Nodes ─────────────────────────────────────────────────────
    Group        { tree_id: String },
    Frame        { label: String, color: [f32; 4] },
    Reroute,
    SwitchNode   { threshold: f32 },
    SetAlpha,
    PremulKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendType { Mix, Screen, Overlay, Multiply, Add, Subtract, Divide, Difference, Exclusion, HardLight, SoftLight, Burn, Dodge, Color, Hue, Saturation, Value, Luminosity }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DenoiseAlgo { Optix, OpenImageDenoise, AiNlm }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlareType { Streaks, GhostsFog, FogGlow, SimpleStars, Bloom }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpscaleAlgo { Dlss, Fsr2, Xess, AiSr, BiCubic }

// ─── Compositor Graph ─────────────────────────────────────────────────────────

pub struct CompGraph {
    pub nodes: HashMap<String, CompNode>,
    pub connections: Vec<CompConnection>,
    pub output_node: Option<String>,
    pub preview_node: Option<String>,
    pub use_gpu: bool,
    pub resolution: [u32; 2],
    pub fps: f32,
    pub frame_range: [i32; 2],
    pub current_frame: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompConnection {
    pub id: String,
    pub from_node: String,
    pub from_socket: String,
    pub to_node: String,
    pub to_socket: String,
}

impl CompGraph {
    pub fn new(resolution: [u32; 2], fps: f32) -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            output_node: None,
            preview_node: None,
            use_gpu: true,
            resolution,
            fps,
            frame_range: [1, 250],
            current_frame: 1,
        }
    }

    pub fn add_node(&mut self, node: CompNode) -> String {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        id
    }

    pub fn connect(&mut self, from_node: &str, from_socket: &str, to_node: &str, to_socket: &str) {
        self.connections.push(CompConnection {
            id: uuid::Uuid::new_v4().to_string(),
            from_node: from_node.to_string(),
            from_socket: from_socket.to_string(),
            to_node: to_node.to_string(),
            to_socket: to_socket.to_string(),
        });
    }

    /// Build a default compositing setup for performance capture
    pub fn performance_capture_default(game_camera: &str, actor_camera: &str) -> Self {
        let mut g = Self::new([1920, 1080], 24.0);

        let game_node = CompNode {
            id: "game_capture".to_string(),
            name: "Game Render".to_string(),
            node_type: CompNodeType::GameCapture { scene_id: "current".to_string(), camera_id: game_camera.to_string() },
            position: [0.0, 0.0], width: 200.0,
            inputs: Vec::new(),
            outputs: vec![
                CompPin { name: "Image".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "img".to_string() },
                CompPin { name: "Depth".to_string(), pin_type: CompPinType::Depth, connected: false, socket_id: "depth".to_string() },
            ],
            properties: HashMap::new(),
            enabled: true, muted: false, preview: false,
        };

        let actor_node = CompNode {
            id: "actor_capture".to_string(),
            name: "Actor Camera".to_string(),
            node_type: CompNodeType::LiveCapture { camera_device: actor_camera.to_string() },
            position: [0.0, 250.0], width: 200.0,
            inputs: Vec::new(),
            outputs: vec![
                CompPin { name: "Image".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "img".to_string() },
            ],
            properties: HashMap::new(),
            enabled: true, muted: false, preview: false,
        };

        let key_node = CompNode {
            id: "chroma_key".to_string(),
            name: "Chroma Key".to_string(),
            node_type: CompNodeType::ChromaMatte { key_color: [0.0, 1.0, 0.0, 1.0], tolerance: 0.1, falloff: 0.1, spillsup: 0.5 },
            position: [250.0, 250.0], width: 200.0,
            inputs: vec![
                CompPin { name: "Image".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "img_in".to_string() },
            ],
            outputs: vec![
                CompPin { name: "Image".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "img_out".to_string() },
                CompPin { name: "Matte".to_string(), pin_type: CompPinType::Alpha, connected: false, socket_id: "matte".to_string() },
            ],
            properties: HashMap::new(),
            enabled: true, muted: false, preview: false,
        };

        let ai_body = CompNode {
            id: "ai_body_replace".to_string(),
            name: "AI Body Replace".to_string(),
            node_type: CompNodeType::AiBodyReplace { actor_capture: actor_camera.to_string(), character_mesh: "player_character".to_string() },
            position: [500.0, 250.0], width: 220.0,
            inputs: vec![
                CompPin { name: "Actor".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "actor".to_string() },
            ],
            outputs: vec![
                CompPin { name: "Character".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "char".to_string() },
            ],
            properties: HashMap::new(),
            enabled: false, muted: false, preview: false, // disabled by default
        };

        let alpha_over = CompNode {
            id: "alpha_over".to_string(),
            name: "Alpha Over".to_string(),
            node_type: CompNodeType::AlphaOver { use_premultiply: false },
            position: [750.0, 100.0], width: 200.0,
            inputs: vec![
                CompPin { name: "Image".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "img1".to_string() },
                CompPin { name: "Overlay".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "img2".to_string() },
            ],
            outputs: vec![
                CompPin { name: "Image".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "result".to_string() },
            ],
            properties: HashMap::new(),
            enabled: true, muted: false, preview: false,
        };

        let output = CompNode {
            id: "composite_out".to_string(),
            name: "Composite".to_string(),
            node_type: CompNodeType::Composite,
            position: [1000.0, 100.0], width: 200.0,
            inputs: vec![
                CompPin { name: "Image".to_string(), pin_type: CompPinType::Rgba, connected: false, socket_id: "img".to_string() },
            ],
            outputs: Vec::new(),
            properties: HashMap::new(),
            enabled: true, muted: false, preview: false,
        };

        for node in [game_node, actor_node, key_node, ai_body, alpha_over, output] {
            g.add_node(node);
        }

        // Connect: actor_capture → chroma_key
        g.connect("actor_capture", "img", "chroma_key", "img_in");
        // Connect: game_capture + keyed_actor → alpha_over
        g.connect("game_capture", "img", "alpha_over", "img1");
        g.connect("chroma_key", "img_out", "alpha_over", "img2");
        // Connect: alpha_over → composite
        g.connect("alpha_over", "result", "composite_out", "img");

        g.output_node = Some("composite_out".to_string());
        g
    }
}

// ─── Performance Capture Pipeline ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCaptureSession {
    pub id: String,
    pub name: String,
    pub actors: Vec<ActorCapture>,
    pub comp_graph: Option<String>,    // comp graph ID
    pub game_scene: Option<String>,
    pub output_cutscene: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub recording: bool,
    pub take_number: u32,
    pub recorded_takes: Vec<RecordedTake>,
    pub live_preview_enabled: bool,
    pub monitoring_participants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorCapture {
    pub actor_name: String,
    pub camera_device: String,
    pub character_entity: Option<String>,
    pub tracking_enabled: bool,
    pub face_tracking: bool,
    pub body_tracking: bool,
    pub voice_recording: bool,
    pub replace_with_character: bool,
    pub face_clone_id: Option<String>,
    pub blend_actor_face: bool,
    pub live_costume: Option<String>,  // game costume shown on actor in preview
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedTake {
    pub take_number: u32,
    pub duration_secs: f32,
    pub video_path: String,
    pub tracking_data_path: Option<String>,
    pub audio_path: Option<String>,
    pub notes: String,
    pub rating: u8,  // 1-5
    pub approved: bool,
    pub recorded_at: DateTime<Utc>,
}

// ─── VFX Compositor Manager ───────────────────────────────────────────────────

pub struct VfxCompositor {
    pub comp_graphs: HashMap<String, CompGraph>,
    pub capture_sessions: HashMap<String, PerformanceCaptureSession>,
    pub active_session: Option<String>,
    pub preview_quality: PreviewQuality,
    pub render_engine: CompRenderEngine,
    pub gpu_enabled: bool,
    pub cache_size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewQuality { Quarter, Half, Full }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompRenderEngine { Cpu, GpuVulkan, GpuMetal, GpuDx12, Wgpu }

impl VfxCompositor {
    pub fn new() -> Self {
        Self {
            comp_graphs: HashMap::new(),
            capture_sessions: HashMap::new(),
            active_session: None,
            preview_quality: PreviewQuality::Half,
            render_engine: CompRenderEngine::Wgpu,
            gpu_enabled: true,
            cache_size_mb: 2048,
        }
    }

    pub fn create_capture_session(&mut self, name: &str, scene_id: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let session = PerformanceCaptureSession {
            id: id.clone(),
            name: name.to_string(),
            actors: Vec::new(),
            comp_graph: None,
            game_scene: Some(scene_id.to_string()),
            output_cutscene: None,
            started_at: None,
            recording: false,
            take_number: 1,
            recorded_takes: Vec::new(),
            live_preview_enabled: true,
            monitoring_participants: Vec::new(),
        };
        self.capture_sessions.insert(id.clone(), session);
        self.active_session = Some(id.clone());
        tracing::info!("VFX Capture session created: {}", name);
        id
    }

    pub fn add_actor(&mut self, session_id: &str, actor: ActorCapture) {
        if let Some(session) = self.capture_sessions.get_mut(session_id) {
            tracing::info!("Actor added to session: {}", actor.actor_name);
            session.actors.push(actor);
        }
    }

    pub fn start_recording(&mut self, session_id: &str) {
        if let Some(session) = self.capture_sessions.get_mut(session_id) {
            session.recording = true;
            session.started_at = Some(Utc::now());
            tracing::info!("VFX Recording started: take {}", session.take_number);
        }
    }

    pub fn stop_recording(&mut self, session_id: &str, output_path: &str) {
        if let Some(session) = self.capture_sessions.get_mut(session_id) {
            if session.recording {
                session.recording = false;
                let take = session.take_number;
                session.recorded_takes.push(RecordedTake {
                    take_number: take,
                    duration_secs: 0.0,
                    video_path: output_path.to_string(),
                    tracking_data_path: Some(format!("{}.tracking.bin", output_path)),
                    audio_path: Some(format!("{}.wav", output_path)),
                    notes: String::new(),
                    rating: 0,
                    approved: false,
                    recorded_at: Utc::now(),
                });
                session.take_number += 1;
                tracing::info!("VFX Recording stopped. Take {} saved.", take);
            }
        }
    }

    pub fn create_default_comp(&mut self, session_id: &str) {
        if let Some(session) = self.capture_sessions.get(&session_id.to_string()) {
            let game_cam = "virtual_cam_1";
            let actor_cam = session.actors.first()
                .map(|a| a.camera_device.as_str())
                .unwrap_or("webcam_0");

            let graph = CompGraph::performance_capture_default(game_cam, actor_cam);
            let graph_id = uuid::Uuid::new_v4().to_string();
            self.comp_graphs.insert(graph_id.clone(), graph);

            if let Some(session) = self.capture_sessions.get_mut(session_id) {
                session.comp_graph = Some(graph_id);
            }
        }
    }
}

extern crate uuid;
extern crate tracing;
