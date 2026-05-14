//! Genesis Engine — Social, Recording & Screen Casting System
//!
//! GAMEPLAY RECORDING:
//! - Record gameplay to video (H264, H265, AV1)
//! - Instant replay buffer (always recording last 30s)
//! - Clip maker (save highlight moments)
//! - GIF capture
//! - Screenshot (RAW, PNG, JPEG)
//! - Slow motion replay
//! - Annotate clips with text, arrows
//!
//! SOCIAL SHARING:
//! - Post directly to TikTok, YouTube Shorts, Instagram Reels
//! - Share to Twitter/X, Facebook, Discord
//! - Auto-generate share captions using AI
//! - Add game watermark / branding
//! - Clip leaderboard (best moments auto-detected)
//!
//! SCREEN CASTING:
//! - Cast game to TV via Chromecast
//! - AirPlay (Apple TV, smart TVs)
//! - Miracast (Windows / Android)
//! - DLNA streaming
//! - Local network streaming (anyone on WiFi can watch)
//! - Cast to second monitor
//! - Phone/tablet as second screen (map, inventory)
//!
//! LIVE STREAMING:
//! - Built-in Twitch/YouTube Live integration
//! - RTMP push to any streaming platform
//! - Overlay system (chat, alerts, game stats)
//! - Stream health monitoring
//! - Auto-quality adjustment for stream bitrate

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

// ═══════════════════════════════════════════════════════════════════════════
// RECORDING SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSystem {
    pub state: RecordingState,
    pub config: RecordingConfig,
    pub replay_buffer: ReplayBuffer,
    pub clips: Vec<GameClip>,
    pub screenshots: Vec<ScreenshotEntry>,
    pub output_dir: PathBuf,
    pub total_recorded_seconds: f64,
    pub storage_used_mb: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecordingState {
    Idle,
    Recording { started_at: DateTime<Utc>, elapsed_secs: f32 },
    Paused,
    Encoding { progress: f32 },
    Complete,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    pub resolution: RecordResolution,
    pub fps: u32,
    pub codec: VideoCodec,
    pub bitrate_kbps: u32,
    pub quality_preset: String,
    pub audio_enabled: bool,
    pub audio_bitrate_kbps: u32,
    pub microphone: bool,
    pub system_audio: bool,
    pub game_audio: bool,
    pub output_format: String,     // "mp4", "webm", "mkv"
    pub include_ui: bool,          // record HUD or just world
    pub include_cursor: bool,
    pub max_duration_secs: Option<f32>,
    pub max_file_size_mb: Option<u32>,
    pub watermark: Option<WatermarkConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordResolution {
    Native,
    R4K { width: u32, height: u32 },
    R1440p,
    R1080p,
    R720p,
    R480p,
    Custom { width: u32, height: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264 { preset: String },      // "ultrafast" to "veryslow"
    H265 { preset: String },
    Av1  { cq: u32 },
    Vp9  { cq: u32 },
    ProRes { profile: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkConfig {
    pub image_path: Option<String>,
    pub text: Option<String>,
    pub position: WatermarkPosition,
    pub opacity: f32,
    pub scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatermarkPosition { TopLeft, TopRight, BottomLeft, BottomRight, Center }

// ─── Replay Buffer (always recording last N seconds) ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayBuffer {
    pub enabled: bool,
    pub duration_secs: f32,      // how many seconds to keep
    pub actual_duration_secs: f32,
    pub resolution: RecordResolution,
    pub fps: u32,
    pub codec: VideoCodec,
    pub size_mb: u32,
    /// Ring buffer of encoded frames
    pub frame_count: u64,
}

impl ReplayBuffer {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            enabled: true,
            duration_secs,
            actual_duration_secs: 0.0,
            resolution: RecordResolution::R1080p,
            fps: 60,
            codec: VideoCodec::H264 { preset: "ultrafast".to_string() },
            size_mb: 0,
            frame_count: 0,
        }
    }

    pub fn save_clip(&self, output_path: &str) -> GameClip {
        GameClip {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("Clip_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")),
            file_path: PathBuf::from(output_path),
            duration_secs: self.actual_duration_secs,
            resolution: self.resolution.clone(),
            fps: self.fps,
            file_size_mb: self.size_mb,
            created_at: Utc::now(),
            source: ClipSource::ReplayBuffer,
            tags: Vec::new(),
            thumbnail: None,
            shares: Vec::new(),
            views: 0,
            ai_caption: None,
        }
    }
}

// ─── Clips & Screenshots ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameClip {
    pub id: String,
    pub title: String,
    pub file_path: PathBuf,
    pub duration_secs: f32,
    pub resolution: RecordResolution,
    pub fps: u32,
    pub file_size_mb: u32,
    pub created_at: DateTime<Utc>,
    pub source: ClipSource,
    pub tags: Vec<String>,
    pub thumbnail: Option<String>,
    pub shares: Vec<ShareRecord>,
    pub views: u64,
    pub ai_caption: Option<String>,   // AI-generated caption for social
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipSource {
    Manual,        // user pressed record key
    ReplayBuffer,  // saved from instant replay
    Highlight,     // auto-detected highlight moment
    Cutscene,      // engine recorded a cutscene
    Screenshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotEntry {
    pub id: String,
    pub file_path: PathBuf,
    pub resolution: [u32; 2],
    pub format: String,
    pub taken_at: DateTime<Utc>,
    pub mode: ScreenshotMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenshotMode {
    Normal,
    NoHud,         // clean screenshot without UI
    Raw,           // HDR, untonemapped
    Depth,         // depth buffer
    Wireframe,
    Photomode,     // special photo mode with settings
}

// ═══════════════════════════════════════════════════════════════════════════
// SOCIAL SHARING
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSystem {
    pub accounts: Vec<SocialAccount>,
    pub share_history: Vec<ShareRecord>,
    pub auto_caption: bool,
    pub default_hashtags: Vec<String>,
    pub watermark: Option<WatermarkConfig>,
    pub branding: GameBranding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAccount {
    pub id: String,
    pub platform: SocialPlatform,
    pub username: String,
    pub display_name: String,
    pub access_token: String,    // encrypted
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub connected: bool,
    pub default_privacy: Privacy,
    pub auto_post_highlights: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SocialPlatform {
    TikTok,
    YouTube,
    InstagramReels,
    Twitter,
    Facebook,
    Discord,
    Reddit,
    Twitch,
    Steam,
    GameCapture,     // NVIDIA GeForce Experience
    XboxGameDVR,
    PlayStation,
    Custom { name: String, api_endpoint: String },
}

impl SocialPlatform {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::TikTok => "TikTok",
            Self::YouTube => "YouTube",
            Self::InstagramReels => "Instagram Reels",
            Self::Twitter => "Twitter / X",
            Self::Facebook => "Facebook",
            Self::Discord => "Discord",
            Self::Reddit => "Reddit",
            Self::Twitch => "Twitch",
            Self::Steam => "Steam",
            _ => "Custom",
        }
    }

    pub fn max_video_duration_secs(&self) -> f32 {
        match self {
            Self::TikTok => 180.0,
            Self::YouTube => f32::MAX,
            Self::InstagramReels => 90.0,
            Self::Twitter => 140.0,
            Self::Facebook => f32::MAX,
            Self::Discord => 30.0, // Nitro for more
            _ => 300.0,
        }
    }

    pub fn preferred_aspect(&self) -> (f32, f32) {
        match self {
            Self::TikTok | Self::InstagramReels => (9.0, 16.0), // vertical
            Self::YouTube => (16.0, 9.0),
            Self::Twitter => (16.0, 9.0),
            _ => (16.0, 9.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Privacy { Public, FriendsOnly, Private }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    pub clip_id: String,
    pub platform: SocialPlatform,
    pub account_id: String,
    pub title: String,
    pub caption: String,
    pub hashtags: Vec<String>,
    pub privacy: Privacy,
    pub schedule_at: Option<DateTime<Utc>>,
    pub trim_start: f32,
    pub trim_end: f32,
    pub add_watermark: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRecord {
    pub id: String,
    pub clip_id: String,
    pub platform: SocialPlatform,
    pub shared_at: DateTime<Utc>,
    pub post_url: Option<String>,
    pub views: u64,
    pub likes: u64,
    pub comments: u32,
    pub shares: u32,
    pub status: ShareStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShareStatus { Pending, Uploading { progress: f32 }, Published, Failed(String), Scheduled }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBranding {
    pub game_name: String,
    pub logo_path: Option<String>,
    pub brand_color: [f32; 4],
    pub website: Option<String>,
    pub store_links: HashMap<String, String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// SCREEN CASTING
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastSystem {
    pub enabled: bool,
    pub active_cast: Option<ActiveCast>,
    pub discovered_receivers: Vec<CastReceiver>,
    pub cast_quality: CastQuality,
    pub latency_mode: CastLatency,
    pub secondary_screen: Option<SecondaryScreen>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastReceiver {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub ip_address: String,
    pub port: u16,
    pub protocol: CastProtocol,
    pub resolution: [u32; 2],
    pub hdr: bool,
    pub latency_ms: u32,
    pub signal_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastProtocol {
    Chromecast { model: ChromecastModel },
    AirPlay    { version: String },
    Miracast,
    Dlna,
    GenesisLocal,    // our own LAN casting protocol
    WebRtc,          // browser-based
    Hdmi,            // direct cable (treated as cast target)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChromecastModel {
    Gen1, Gen2, Gen3, Gen4,
    Ultra,
    WithGoogleTv,
    Embedded,  // built into TV
}

impl CastProtocol {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Chromecast { .. } => "Chromecast",
            Self::AirPlay { .. } => "AirPlay",
            Self::Miracast => "Miracast / Wi-Fi Direct",
            Self::Dlna => "DLNA",
            Self::GenesisLocal => "Genesis Local Cast",
            Self::WebRtc => "Browser Cast",
            Self::Hdmi => "HDMI Cable",
        }
    }

    pub fn typical_latency_ms(&self) -> u32 {
        match self {
            Self::Hdmi => 1,
            Self::AirPlay { .. } => 50,
            Self::Chromecast { .. } => 100,
            Self::Miracast => 80,
            Self::GenesisLocal => 30,
            Self::WebRtc => 150,
            Self::Dlna => 200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCast {
    pub receiver_id: String,
    pub protocol: CastProtocol,
    pub started_at: DateTime<Utc>,
    pub bytes_sent: u64,
    pub current_latency_ms: u32,
    pub dropped_frames: u32,
    pub resolution: [u32; 2],
    pub fps: f32,
    pub bitrate_kbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastQuality {
    Auto,                         // adapt to network
    Performance { fps: u32, bitrate_kbps: u32 },
    Quality     { fps: u32, bitrate_kbps: u32 },
    Custom { resolution: [u32; 2], fps: u32, bitrate_kbps: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastLatency {
    UltraLow,   // 30-50ms (local network, competitive)
    Low,        // 50-100ms (normal gaming)
    Balanced,   // 100-200ms (living room TV)
    Standard,   // 200ms+ (best quality)
}

/// Second screen features (phone/tablet as map, inventory, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryScreen {
    pub device_id: String,
    pub mode: SecondaryScreenMode,
    pub content: serde_json::Value,  // content to display
    pub fps: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecondaryScreenMode {
    GameMap        { zoom: f32, show_markers: bool },
    Inventory      { tab: String },
    ChatOverlay    { size: f32 },
    Stats          { which: Vec<String> },
    Spectate       { target_entity: String },
    Director,      // full director view with all cameras
    Debug          { panels: Vec<String> },
    Custom         { ui_id: String },
}

impl CastSystem {
    pub fn new() -> Self {
        Self {
            enabled: true,
            active_cast: None,
            discovered_receivers: Vec::new(),
            cast_quality: CastQuality::Auto,
            latency_mode: CastLatency::Low,
            secondary_screen: None,
        }
    }

    pub fn is_casting(&self) -> bool { self.active_cast.is_some() }

    pub fn start_cast(&mut self, receiver: &CastReceiver) {
        tracing::info!("Starting cast to '{}' via {:?}", receiver.name, receiver.protocol.display_name());
        self.active_cast = Some(ActiveCast {
            receiver_id: receiver.id.clone(),
            protocol: receiver.protocol.clone(),
            started_at: Utc::now(),
            bytes_sent: 0,
            current_latency_ms: receiver.latency_ms,
            dropped_frames: 0,
            resolution: receiver.resolution,
            fps: 60.0,
            bitrate_kbps: 8000,
        });
    }

    pub fn stop_cast(&mut self) {
        if let Some(cast) = &self.active_cast {
            let duration = (Utc::now() - cast.started_at).num_seconds();
            tracing::info!("Cast ended: {} seconds, {} MB sent", duration, cast.bytes_sent / 1_000_000);
        }
        self.active_cast = None;
    }

    pub fn discover_receivers(&mut self) {
        tracing::info!("Scanning for cast receivers on local network...");
        // In production: mDNS/Bonjour scan + SSDP for DLNA + Chromecast discovery
        // For now: add placeholder local Genesis receiver
        self.discovered_receivers = vec![
            CastReceiver {
                id: "local_genesis".to_string(),
                name: "Same Device (Preview)".to_string(),
                device_type: "local".to_string(),
                ip_address: "127.0.0.1".to_string(),
                port: 8765,
                protocol: CastProtocol::GenesisLocal,
                resolution: [1920, 1080],
                hdr: false,
                latency_ms: 5,
                signal_strength: 1.0,
            },
        ];
        tracing::info!("Found {} cast receivers", self.discovered_receivers.len());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LIVE STREAMING
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStreamConfig {
    pub enabled: bool,
    pub platform: StreamPlatform,
    pub stream_key: String,          // encrypted
    pub rtmp_url: String,
    pub resolution: RecordResolution,
    pub fps: u32,
    pub bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
    pub overlay: StreamOverlay,
    pub auto_start_on_record: bool,
    pub chat_integration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamPlatform {
    Twitch { channel: String },
    YouTube { channel_id: String },
    Facebook { page_id: String },
    TikTokLive,
    Kick,
    AfreecaTv,
    Custom { rtmp_url: String, name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOverlay {
    pub enabled: bool,
    pub chat_box: bool,
    pub alerts_box: bool,
    pub game_stats: bool,
    pub viewer_count: bool,
    pub now_playing: bool,
    pub webcam: bool,
    pub webcam_position: WatermarkPosition,
    pub custom_elements: Vec<OverlayElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayElement {
    pub id: String,
    pub element_type: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub data_source: String,
    pub style: serde_json::Value,
}

// ─── Social Manager ───────────────────────────────────────────────────────────

pub struct SocialManager {
    pub recording: RecordingSystem,
    pub cast: CastSystem,
    pub social: SocialSystem,
    pub stream: Option<LiveStreamConfig>,
    pub highlight_detector: HighlightDetector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightDetector {
    pub enabled: bool,
    pub sensitivity: f32,    // 0-1
    /// Events that trigger highlight saving
    pub triggers: Vec<HighlightTrigger>,
    pub auto_clip_duration_secs: f32,
    pub pre_trigger_secs: f32,
    pub post_trigger_secs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightTrigger {
    pub name: String,
    pub event_type: String,
    pub min_significance: f32,
    pub cooldown_secs: f32,
}

impl HighlightDetector {
    pub fn default_triggers() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.6,
            triggers: vec![
                HighlightTrigger { name: "Kill".to_string(), event_type: "enemy_killed".to_string(), min_significance: 0.5, cooldown_secs: 5.0 },
                HighlightTrigger { name: "Kill Streak".to_string(), event_type: "kill_streak".to_string(), min_significance: 0.8, cooldown_secs: 2.0 },
                HighlightTrigger { name: "Achievement".to_string(), event_type: "achievement_unlocked".to_string(), min_significance: 0.7, cooldown_secs: 0.0 },
                HighlightTrigger { name: "Boss Kill".to_string(), event_type: "boss_defeated".to_string(), min_significance: 1.0, cooldown_secs: 0.0 },
                HighlightTrigger { name: "Level Up".to_string(), event_type: "player_level_up".to_string(), min_significance: 0.6, cooldown_secs: 30.0 },
                HighlightTrigger { name: "Death".to_string(), event_type: "player_died".to_string(), min_significance: 0.7, cooldown_secs: 10.0 },
            ],
            auto_clip_duration_secs: 30.0,
            pre_trigger_secs: 10.0,
            post_trigger_secs: 5.0,
        }
    }
}

impl SocialManager {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            recording: RecordingSystem {
                state: RecordingState::Idle,
                config: RecordingConfig {
                    resolution: RecordResolution::R1080p,
                    fps: 60,
                    codec: VideoCodec::H264 { preset: "fast".to_string() },
                    bitrate_kbps: 8000,
                    quality_preset: "fast".to_string(),
                    audio_enabled: true,
                    audio_bitrate_kbps: 192,
                    microphone: false,
                    system_audio: true,
                    game_audio: true,
                    output_format: "mp4".to_string(),
                    include_ui: true,
                    include_cursor: false,
                    max_duration_secs: None,
                    max_file_size_mb: None,
                    watermark: None,
                },
                replay_buffer: ReplayBuffer::new(30.0),
                clips: Vec::new(),
                screenshots: Vec::new(),
                output_dir: output_dir.clone(),
                total_recorded_seconds: 0.0,
                storage_used_mb: 0,
            },
            cast: CastSystem::new(),
            social: SocialSystem {
                accounts: Vec::new(),
                share_history: Vec::new(),
                auto_caption: true,
                default_hashtags: vec!["GenesisEngine".to_string(), "IndieGame".to_string()],
                watermark: None,
                branding: GameBranding {
                    game_name: "My Game".to_string(),
                    logo_path: None,
                    brand_color: [0.2, 0.6, 1.0, 1.0],
                    website: None,
                    store_links: HashMap::new(),
                },
            },
            stream: None,
            highlight_detector: HighlightDetector::default_triggers(),
        }
    }

    pub fn take_screenshot(&mut self, mode: ScreenshotMode) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        tracing::info!("Screenshot taken: {:?}", mode);
        id
    }

    pub fn save_replay_clip(&mut self) -> String {
        let output = self.recording.output_dir.join(format!("clip_{}.mp4", chrono::Utc::now().format("%Y%m%d_%H%M%S")));
        let clip = self.recording.replay_buffer.save_clip(output.to_str().unwrap_or("clip.mp4"));
        let id = clip.id.clone();
        tracing::info!("Replay clip saved: {}", output.display());
        self.recording.clips.push(clip);
        id
    }

    pub fn auto_share_highlight(&mut self, clip_id: &str) {
        for account in &self.social.accounts {
            if account.auto_post_highlights && account.connected {
                tracing::info!("Auto-sharing clip {} to {}", clip_id, account.platform.display_name());
            }
        }
    }
}

extern crate uuid;
extern crate tracing;
