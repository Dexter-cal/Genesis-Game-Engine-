//! ChronoVerse Studio — Virtual Production Suite
//!
//! A complete virtual production environment like Unreal's MetaHuman + Live Link:
//!
//! PERFORMANCE CAPTURE:
//! - Phone cameras as mocap cameras (QR code connection)
//! - Webcam face tracking → character face animation
//! - Body tracking → character body animation
//! - Multi-camera capture (sync multiple phones/cameras)
//! - Real-time preview (act and see character in game world)
//! - AI body tracking (MediaPipe/WHAM for markerless)
//!
//! VIDEO EDITING:
//! - Timeline editor (clips, cuts, transitions)
//! - Multi-track (video, audio, effects, subtitles)
//! - Live game rendering as background
//! - Compositing (real actor + game world)
//! - Color grading
//! - Export to game engine (cutscene asset)
//!
//! AUDIO STUDIO:
//! - Multi-track audio editor
//! - Voice recording
//! - Music composition
//! - Foley sound design
//! - Mixing with game audio engine
//!
//! COLLABORATION:
//! - Multiple people can work simultaneously
//! - Director sees all cameras
//! - Actors see their character in real-time
//! - Remote collaboration via the web

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// ─── Studio Session ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioSession {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub scene_id: Option<String>,          // game scene to render in background
    pub participants: Vec<StudioParticipant>,
    pub cameras: Vec<StudioCamera>,
    pub timeline: Timeline,
    pub audio_mixer: StudioMixer,
    pub recording: RecordingState,
    pub render_background: RenderBackground,
    pub ai_director: AiDirectorState,
    pub created_at: DateTime<Utc>,
    pub session_type: SessionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    /// Record cutscene with real actors
    PerformanceCapture,
    /// Motion capture session
    MocapSession,
    /// Audio recording
    AudioRecording,
    /// Video editing
    Editing,
    /// Live preview / playtesting
    LivePreview,
    /// Collaborative game design
    Design,
}

// ─── Participants ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioParticipant {
    pub id: String,
    pub name: String,
    pub role: StudioRole,
    pub connection: ParticipantConnection,
    pub assigned_character: Option<String>,    // entity ID
    pub camera_device: Option<String>,         // camera device ID
    pub microphone_device: Option<String>,
    pub is_recording: bool,
    pub is_tracked: bool,
    pub tracking_data: Option<LiveTrackingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StudioRole {
    Director,
    Actor { character: String },
    CameraOperator,
    SoundDesigner,
    Editor,
    Spectator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantConnection {
    pub device_type: DeviceType,
    pub session_url: Option<String>,     // URL they connected from
    pub latency_ms: u32,
    pub connected: bool,
    pub qr_code_type: QrCodeType,        // which QR they scanned
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Phone,
    Tablet,
    WebBrowser,
    VrHeadset,
    ExternalCamera { model: String, fps: f32, resolution: [u32; 2] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QrCodeType {
    Controller,     // phone as game controller
    StudioCamera,   // phone as studio camera
    Actor,          // phone tracks actor, drives character
    Director,       // director mode
    Spectator,
}

// ─── Studio Cameras ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioCamera {
    pub id: String,
    pub name: String,
    pub camera_type: StudioCameraType,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub fov: f32,
    pub active: bool,
    pub preview_resolution: [u32; 2],
    pub recording_resolution: [u32; 2],
    pub frame_rate: f32,
    pub iso: u32,
    pub aperture: f32,
    pub shutter_speed: f32,
    pub white_balance: u32,       // Kelvin
    pub lens_profile: LensProfile,
    pub stabilization: StabilizationMode,
    pub effects: Vec<CameraEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StudioCameraType {
    /// Virtual camera in the 3D game world
    Virtual { entity_id: String },
    /// Phone camera streaming via QR code
    PhoneCamera { device_session_id: String },
    /// Professional camera (USB/HDMI capture)
    External { device_id: String, capture_card: Option<String> },
    /// Webcam
    Webcam { device_id: String },
    /// AI-controlled cinematic camera
    AiCinematic { style: String },
    /// Drone simulation
    Drone { altitude: f32, speed: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensProfile {
    pub name: String,
    pub focal_length_mm: f32,
    pub distortion_k1: f32,
    pub distortion_k2: f32,
    pub chromatic_aberration: f32,
    pub vignette: f32,
    pub grain: f32,
}

impl LensProfile {
    pub fn clean() -> Self { Self { name: "Clean".to_string(), focal_length_mm: 50.0, distortion_k1: 0.0, distortion_k2: 0.0, chromatic_aberration: 0.0, vignette: 0.0, grain: 0.0 } }
    pub fn anamorphic() -> Self { Self { name: "Anamorphic".to_string(), focal_length_mm: 50.0, distortion_k1: -0.1, distortion_k2: 0.05, chromatic_aberration: 0.03, vignette: 0.3, grain: 0.02 } }
    pub fn phone() -> Self { Self { name: "Phone".to_string(), focal_length_mm: 26.0, distortion_k1: 0.05, distortion_k2: 0.0, chromatic_aberration: 0.01, vignette: 0.1, grain: 0.01 } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StabilizationMode { None, Electronic, Optical, AiSmooth { strength: f32 } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraEffect {
    ColorGrade { lut_id: String, intensity: f32 },
    FilmGrain { intensity: f32, size: f32 },
    Vignette { strength: f32, radius: f32 },
    ChromaticAberration { strength: f32 },
    DepthOfField { focus_dist: f32, aperture: f32 },
    MotionBlur { shutter_angle: f32 },
    Bloom { threshold: f32, intensity: f32 },
    CrtEffect { scanline_intensity: f32, curvature: f32 },
}

// ─── Live Tracking Data ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTrackingData {
    pub face: Option<FaceTrackingData>,
    pub body: Option<BodyTrackingData>,
    pub hands: Option<HandTrackingLive>,
    pub voice: Option<VoiceLiveData>,
    pub source: TrackingSource,
    pub confidence: f32,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceTrackingData {
    /// 468 landmarks from MediaPipe
    pub landmarks: Vec<[f32; 3]>,
    /// Blend shape weights for facial expressions
    pub blend_shapes: HashMap<String, f32>,
    pub head_position: [f32; 3],
    pub head_rotation: [f32; 4],
    pub left_eye_rotation: [f32; 2],  // pitch, yaw
    pub right_eye_rotation: [f32; 2],
    pub jaw_open: f32,
    pub brow_inner_up: f32,
    pub brow_outer_up_left: f32,
    pub brow_outer_up_right: f32,
    pub eye_blink_left: f32,
    pub eye_blink_right: f32,
    pub mouth_smile_left: f32,
    pub mouth_smile_right: f32,
    pub cheek_puff: f32,
    pub nose_sneer_left: f32,
    pub nose_sneer_right: f32,
    pub tongue_out: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyTrackingData {
    /// 33 body landmarks from MediaPipe Pose
    pub landmarks: Vec<[f32; 3]>,
    pub world_landmarks: Vec<[f32; 3]>,
    pub visibility: Vec<f32>,
    /// Mapped bone rotations (ready to drive rig)
    pub bone_rotations: HashMap<String, [f32; 4]>,
    pub root_position: [f32; 3],
    pub root_velocity: [f32; 3],
    pub is_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandTrackingLive {
    pub left_hand: Option<Vec<[f32; 3]>>,
    pub right_hand: Option<Vec<[f32; 3]>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceLiveData {
    pub audio_level_db: f32,
    pub phoneme: Option<String>,
    pub viseme: Option<String>,
    pub is_speaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackingSource {
    MediaPipe,
    Wham,
    OpenPose,
    VisionPro,
    MetaQuest,
    ExternalMarkers,
    AiEstimated,
}

// ─── Timeline / Video Editor ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub id: String,
    pub duration_secs: f32,
    pub fps: f32,
    pub current_time: f32,
    pub tracks: Vec<TimelineTrack>,
    pub markers: Vec<TimelineMarker>,
    pub in_point: f32,
    pub out_point: f32,
    pub loop_playback: bool,
    pub snap_to_frames: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineTrack {
    pub id: String,
    pub name: String,
    pub track_type: TrackType,
    pub clips: Vec<TimelineClip>,
    pub muted: bool,
    pub locked: bool,
    pub solo: bool,
    pub color: [f32; 4],
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackType {
    Video   { layer: i32 },
    Audio   { channel: u8 },
    CameraAnimation,
    CharacterAnimation { entity_id: String },
    SubtitleCaptions,
    Effects,
    GameEvent,    // trigger game events at specific times
    Music,
    Voiceover,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineClip {
    pub id: String,
    pub asset_id: Option<String>,
    pub start_time: f32,
    pub duration: f32,
    pub asset_start: f32,    // source in/out
    pub asset_end: f32,
    pub speed: f32,
    pub reversed: bool,
    pub blend_in: f32,
    pub blend_out: f32,
    pub effects: Vec<ClipEffect>,
    pub color: Option<[f32; 4]>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipEffect {
    ColorGrade { lut: Option<String>, exposure: f32, contrast: f32, saturation: f32, temperature: f32 },
    Transition { trans_type: TransitionType, duration: f32 },
    AudioFade { fade_in: f32, fade_out: f32 },
    SpeedRamp { from_speed: f32, to_speed: f32 },
    Blur { radius: f32 },
    Shake { intensity: f32 },
    ChromaKey { color: [f32; 4], tolerance: f32, spill_suppression: f32 },
    LumaKey { threshold: f32, invert: bool },
    VolumeEnvelope { points: Vec<[f32; 2]> },
    Reverb { preset: String, wet: f32 },
    PitchShift { semitones: f32 },
    NoiseReduction { strength: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionType {
    Cut, Dissolve, Fade, Wipe { direction: String }, Slide { direction: String },
    Push { direction: String }, Zoom { in_out: bool }, Spin, Ripple,
    Glitch, Flash, Custom { shader: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMarker {
    pub time: f32,
    pub label: String,
    pub color: [f32; 4],
    pub chapter: bool,      // chapter marker for navigation
}

// ─── Recording State ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingState {
    pub is_recording: bool,
    pub is_playing: bool,
    pub countdown: Option<f32>,    // recording countdown
    pub recorded_clips: Vec<RecordedClip>,
    pub record_cameras: Vec<String>, // which camera IDs to record
    pub record_audio: bool,
    pub record_game_state: bool,   // record entity states too
    pub output_format: RecordFormat,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedClip {
    pub id: String,
    pub camera_id: String,
    pub file_path: String,
    pub duration: f32,
    pub resolution: [u32; 2],
    pub fps: f32,
    pub recorded_at: DateTime<Utc>,
    pub has_tracking_data: bool,
    pub tracking_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordFormat {
    H264 { crf: u32, preset: String },
    H265 { crf: u32 },
    ProRes { profile: String },
    Av1,
    RawFrames { format: String },
    ChronoNative, // engine's own format, lossless with game state
}

// ─── Background Rendering ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBackground {
    pub mode: BackgroundMode,
    pub scene_id: Option<String>,
    pub camera_id: Option<String>,
    pub time_of_day_override: Option<f32>,
    pub weather_override: Option<String>,
    pub render_resolution: [u32; 2],
    pub hdr: bool,
    pub enable_raytracing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackgroundMode {
    /// Live 3D game scene renders behind actors
    LiveGameScene,
    /// Pre-rendered game scene (HDRI/360 photo)
    StaticRender { hdri_path: String },
    /// Green screen (actor on green, comp in post)
    GreenScreen,
    /// Black (for black screen shots)
    Black,
    /// Custom solid color
    Color([f32; 4]),
    /// Transparent (for overlay/UI work)
    Transparent,
}

// ─── AI Director ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDirectorState {
    pub enabled: bool,
    /// AI agent that guides direction
    pub guidance_style: DirectorStyle,
    /// Current direction notes
    pub current_notes: Vec<DirectorNote>,
    /// Automatic camera cuts
    pub auto_cut: bool,
    pub auto_cut_interval: f32,
    /// AI-suggested camera angles
    pub suggested_shots: Vec<SuggestedShot>,
    pub last_suggestion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectorStyle {
    /// Cinematic film style
    Cinematic,
    /// YouTube/social media style
    Social,
    /// Documentary
    Documentary,
    /// Anime style
    Anime,
    /// Horror
    Horror,
    /// Custom with prompt
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorNote {
    pub note: String,
    pub applies_to: String,  // participant or camera name
    pub priority: u8,
    pub category: NoteCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoteCategory { Blocking, Emotion, Camera, Audio, Continuity, Technical }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedShot {
    pub camera_id: String,
    pub description: String,
    pub duration_secs: f32,
    pub confidence: f32,
    pub reason: String,
}

// ─── Studio Mixer ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioMixer {
    pub channels: Vec<MixerChannel>,
    pub master_volume: f32,
    pub master_limiter: bool,
    pub sample_rate: u32,
    pub bit_depth: u32,
    pub monitoring_enabled: bool,
    pub monitoring_device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerChannel {
    pub name: String,
    pub source: AudioSource,
    pub volume: f32,
    pub pan: f32,          // -1 to 1
    pub muted: bool,
    pub soloed: bool,
    pub effects: Vec<String>,
    pub sends: Vec<SendBus>,
    pub meter_left: f32,   // current level
    pub meter_right: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSource {
    Microphone { device_id: String },
    File { path: String },
    Network { participant_id: String },
    GameEngine { bus: String },
    Synthesized { synth_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendBus {
    pub bus_name: String,
    pub send_level: f32,
    pub pre_fader: bool,
}

// ─── Studio Manager ───────────────────────────────────────────────────────────

pub struct StudioManager {
    pub sessions: HashMap<String, StudioSession>,
    pub active_session: Option<String>,
    pub max_participants: u8,
    pub streaming_enabled: bool,
    pub stream_url: Option<String>,
}

impl StudioManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session: None,
            max_participants: 16,
            streaming_enabled: false,
            stream_url: None,
        }
    }

    pub fn create_session(&mut self, name: &str, project_id: &str, session_type: SessionType) -> String {
        let session = StudioSession {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            project_id: project_id.to_string(),
            scene_id: None,
            participants: Vec::new(),
            cameras: Vec::new(),
            timeline: Timeline {
                id: uuid::Uuid::new_v4().to_string(),
                duration_secs: 300.0,
                fps: 24.0,
                current_time: 0.0,
                tracks: Vec::new(),
                markers: Vec::new(),
                in_point: 0.0,
                out_point: 300.0,
                loop_playback: false,
                snap_to_frames: true,
            },
            audio_mixer: StudioMixer {
                channels: Vec::new(),
                master_volume: 1.0,
                master_limiter: true,
                sample_rate: 48000,
                bit_depth: 24,
                monitoring_enabled: true,
                monitoring_device: None,
            },
            recording: RecordingState {
                is_recording: false,
                is_playing: false,
                countdown: None,
                recorded_clips: Vec::new(),
                record_cameras: Vec::new(),
                record_audio: true,
                record_game_state: true,
                output_format: RecordFormat::H264 { crf: 18, preset: "slow".to_string() },
                output_path: "./recordings".to_string(),
            },
            render_background: RenderBackground {
                mode: BackgroundMode::LiveGameScene,
                scene_id: None,
                camera_id: None,
                time_of_day_override: None,
                weather_override: None,
                render_resolution: [1920, 1080],
                hdr: true,
                enable_raytracing: false,
            },
            ai_director: AiDirectorState {
                enabled: true,
                guidance_style: DirectorStyle::Cinematic,
                current_notes: Vec::new(),
                auto_cut: false,
                auto_cut_interval: 5.0,
                suggested_shots: Vec::new(),
                last_suggestion: None,
            },
            created_at: Utc::now(),
            session_type,
        };

        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.active_session = Some(id.clone());
        tracing::info!("Studio session created: {}", name);
        id
    }

    pub fn join_as_camera(&mut self, session_id: &str, device_session_id: &str) -> Option<String> {
        let camera_id = uuid::Uuid::new_v4().to_string();
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.cameras.push(StudioCamera {
                id: camera_id.clone(),
                name: format!("Phone Camera {}", session.cameras.len() + 1),
                camera_type: StudioCameraType::PhoneCamera { device_session_id: device_session_id.to_string() },
                position: [0.0, 1.5, 3.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                fov: 75.0,
                active: true,
                preview_resolution: [854, 480],
                recording_resolution: [1920, 1080],
                frame_rate: 30.0,
                iso: 400,
                aperture: 2.8,
                shutter_speed: 1.0 / 60.0,
                white_balance: 5500,
                lens_profile: LensProfile::phone(),
                stabilization: StabilizationMode::Electronic,
                effects: Vec::new(),
            });
        }
        Some(camera_id)
    }

    pub fn generate_studio_qr(&self, session_id: &str, role: QrCodeType) -> String {
        format!("https://studio.chronoverse.io/join/{}/{:?}", session_id, role)
    }
}

extern crate uuid;
extern crate tracing;
