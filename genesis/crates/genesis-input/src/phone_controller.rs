//! Genesis Phone-as-Controller System
//!
//! Allows a phone to act as a full game controller:
//! 1. Player opens game → QR code appears
//! 2. Player scans with phone → opens web page
//! 3. Phone becomes a controller with:
//!    - Virtual buttons (configurable layout)
//!    - Gyroscope/accelerometer input
//!    - Touch joysticks
//!    - Haptic feedback (phone vibration)
//!    - Optional: game viewport streamed to phone
//!    - Optional: buttons overlaid on top of stream (like emulators)
//!    - Adjustable button positions/sizes
//!    - Multiple controller profiles
//!
//! Architecture:
//! - Game runs local WebSocket server
//! - Phone connects via QR-encoded URL
//! - Phone sends input events over WebSocket
//! - Game receives and treats as gamepad input
//! - Optional: game streams JPEG frames to phone

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::net::SocketAddr;

// ─── QR Controller Session ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneControllerSession {
    pub session_id: String,
    pub player_index: u8,
    pub device_name: String,
    pub phone_model: Option<String>,
    pub os: PhoneOs,
    pub connected: bool,
    pub address: Option<SocketAddr>,
    pub layout: ControllerLayout,
    pub viewport_streaming: bool,
    pub stream_quality: StreamQuality,
    pub latency_ms: u32,
    pub created_at: u64,
    pub capabilities: PhoneCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhoneOs { Android, Ios, Unknown }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneCapabilities {
    pub gyroscope: bool,
    pub accelerometer: bool,
    pub haptics: bool,
    pub touch_force: bool,
    pub multi_touch: u8,
    pub screen_width: u32,
    pub screen_height: u32,
    pub dpr: f32, // device pixel ratio
}

// ─── Controller Layout ────────────────────────────────────────────────────────

/// Virtual controller layout on the phone screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerLayout {
    pub id: String,
    pub name: String,
    pub orientation: ScreenOrientation,
    pub elements: Vec<ControllerElement>,
    pub background_opacity: f32,
    pub safe_area_aware: bool,
    /// Mirror game viewport in background
    pub viewport_background: bool,
    /// Opacity of buttons over viewport (0=invisible, 1=opaque)
    pub button_overlay_opacity: f32,
    /// Auto-hide inactive buttons
    pub auto_hide_inactive: bool,
    pub theme: ControllerTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenOrientation { Portrait, Landscape, Auto }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerElement {
    pub id: String,
    pub element_type: ControllerElementType,
    /// Position in normalized screen coordinates (0-1)
    pub position: [f32; 2],
    /// Size in normalized screen coordinates
    pub size: [f32; 2],
    pub label: String,
    pub icon: Option<String>,
    pub color: [f32; 4],
    pub opacity: f32,
    pub haptic_feedback: bool,
    /// Can user drag this to reposition?
    pub repositionable: bool,
    /// Can user resize this?
    pub resizable: bool,
    /// What game input this maps to
    pub maps_to: GameInputMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControllerElementType {
    /// On-screen joystick (floating or fixed)
    Joystick { floating: bool, dead_zone: f32 },
    /// Single button (press)
    Button { shape: ButtonShape },
    /// Trigger (analog 0-1)
    Trigger { vertical: bool },
    /// D-Pad (4 or 8 directions)
    DPad { eight_way: bool },
    /// Touchpad (raw touch coords)
    Touchpad,
    /// Gyro enable toggle
    GyroToggle,
    /// Touch area (swipe gesture)
    TouchArea { tracks_velocity: bool },
    /// Slider (0-1 analog)
    Slider { vertical: bool },
    /// Label display only
    Label,
    /// Battery/latency indicator
    StatusIndicator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ButtonShape { Circle, Square, Rounded, Diamond }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameInputMapping {
    GamepadButton(String),     // "South", "North", etc.
    GamepadAxis(String),       // "LeftStickX", etc.
    Key(String),               // keyboard key name
    MouseButton(String),
    MouseAxis(String),
    Gyro { axis: String },
    Custom(String),            // custom action name
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControllerTheme {
    Dark, Light, Transparent, Neon, Retro, Minimal, Custom(String),
}

impl ControllerLayout {
    /// Standard gamepad layout (Xbox-style)
    pub fn standard_gamepad(viewport_background: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Standard Gamepad".to_string(),
            orientation: ScreenOrientation::Landscape,
            viewport_background,
            button_overlay_opacity: if viewport_background { 0.75 } else { 1.0 },
            background_opacity: if viewport_background { 0.0 } else { 0.15 },
            auto_hide_inactive: false,
            safe_area_aware: true,
            theme: ControllerTheme::Dark,
            elements: vec![
                // Left joystick
                ControllerElement {
                    id: "left_stick".to_string(),
                    element_type: ControllerElementType::Joystick { floating: false, dead_zone: 0.1 },
                    position: [0.15, 0.65],
                    size: [0.2, 0.3],
                    label: "".to_string(), icon: None,
                    color: [0.3, 0.3, 0.3, 0.8],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadAxis("LeftStick".to_string()),
                },
                // Right joystick
                ControllerElement {
                    id: "right_stick".to_string(),
                    element_type: ControllerElementType::Joystick { floating: false, dead_zone: 0.1 },
                    position: [0.65, 0.65],
                    size: [0.2, 0.3],
                    label: "".to_string(), icon: None,
                    color: [0.3, 0.3, 0.3, 0.8],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadAxis("RightStick".to_string()),
                },
                // D-Pad
                ControllerElement {
                    id: "dpad".to_string(),
                    element_type: ControllerElementType::DPad { eight_way: true },
                    position: [0.08, 0.35],
                    size: [0.15, 0.22],
                    label: "".to_string(), icon: None,
                    color: [0.25, 0.25, 0.25, 0.85],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadButton("DPad".to_string()),
                },
                // A button (South)
                ControllerElement {
                    id: "btn_south".to_string(),
                    element_type: ControllerElementType::Button { shape: ButtonShape::Circle },
                    position: [0.9, 0.6],
                    size: [0.07, 0.1],
                    label: "A".to_string(), icon: None,
                    color: [0.2, 0.7, 0.2, 0.9],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadButton("South".to_string()),
                },
                // B button (East)
                ControllerElement {
                    id: "btn_east".to_string(),
                    element_type: ControllerElementType::Button { shape: ButtonShape::Circle },
                    position: [0.94, 0.45],
                    size: [0.07, 0.1],
                    label: "B".to_string(), icon: None,
                    color: [0.8, 0.2, 0.2, 0.9],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadButton("East".to_string()),
                },
                // X button (West)
                ControllerElement {
                    id: "btn_west".to_string(),
                    element_type: ControllerElementType::Button { shape: ButtonShape::Circle },
                    position: [0.86, 0.45],
                    size: [0.07, 0.1],
                    label: "X".to_string(), icon: None,
                    color: [0.2, 0.4, 0.9, 0.9],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadButton("West".to_string()),
                },
                // Y button (North)
                ControllerElement {
                    id: "btn_north".to_string(),
                    element_type: ControllerElementType::Button { shape: ButtonShape::Circle },
                    position: [0.90, 0.3],
                    size: [0.07, 0.1],
                    label: "Y".to_string(), icon: None,
                    color: [0.85, 0.75, 0.1, 0.9],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadButton("North".to_string()),
                },
                // Left trigger
                ControllerElement {
                    id: "left_trigger".to_string(),
                    element_type: ControllerElementType::Trigger { vertical: false },
                    position: [0.0, 0.0],
                    size: [0.2, 0.08],
                    label: "LT".to_string(), icon: None,
                    color: [0.4, 0.4, 0.4, 0.85],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadAxis("LeftTrigger".to_string()),
                },
                // Right trigger
                ControllerElement {
                    id: "right_trigger".to_string(),
                    element_type: ControllerElementType::Trigger { vertical: false },
                    position: [0.8, 0.0],
                    size: [0.2, 0.08],
                    label: "RT".to_string(), icon: None,
                    color: [0.4, 0.4, 0.4, 0.85],
                    opacity: 0.9,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadAxis("RightTrigger".to_string()),
                },
                // Start / Options
                ControllerElement {
                    id: "btn_start".to_string(),
                    element_type: ControllerElementType::Button { shape: ButtonShape::Rounded },
                    position: [0.55, 0.08],
                    size: [0.06, 0.08],
                    label: "≡".to_string(), icon: None,
                    color: [0.5, 0.5, 0.5, 0.7],
                    opacity: 0.8,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadButton("Start".to_string()),
                },
                // Select / Back
                ControllerElement {
                    id: "btn_select".to_string(),
                    element_type: ControllerElementType::Button { shape: ButtonShape::Rounded },
                    position: [0.43, 0.08],
                    size: [0.06, 0.08],
                    label: "⧉".to_string(), icon: None,
                    color: [0.5, 0.5, 0.5, 0.7],
                    opacity: 0.8,
                    haptic_feedback: true, repositionable: true, resizable: true,
                    maps_to: GameInputMapping::GamepadButton("Select".to_string()),
                },
                // Gyro toggle
                ControllerElement {
                    id: "gyro_toggle".to_string(),
                    element_type: ControllerElementType::GyroToggle,
                    position: [0.49, 0.95],
                    size: [0.1, 0.05],
                    label: "Gyro".to_string(), icon: None,
                    color: [0.3, 0.6, 0.9, 0.7],
                    opacity: 0.7,
                    haptic_feedback: false, repositionable: true, resizable: false,
                    maps_to: GameInputMapping::Gyro { axis: "all".to_string() },
                },
                // Latency indicator
                ControllerElement {
                    id: "latency".to_string(),
                    element_type: ControllerElementType::StatusIndicator,
                    position: [0.5, 0.02],
                    size: [0.08, 0.04],
                    label: "".to_string(), icon: None,
                    color: [1.0, 1.0, 1.0, 0.5],
                    opacity: 0.5,
                    haptic_feedback: false, repositionable: false, resizable: false,
                    maps_to: GameInputMapping::None,
                },
            ],
        }
    }

    /// MOBA touch layout
    pub fn moba_layout() -> Self {
        let mut layout = Self::standard_gamepad(false);
        layout.name = "MOBA Touch".to_string();
        layout.elements.retain(|e| e.id == "left_stick" || e.id == "dpad");
        // Add skill buttons etc.
        layout
    }

    /// FPS layout (floating joystick + look area)
    pub fn fps_layout(with_viewport: bool) -> Self {
        let mut layout = Self::standard_gamepad(with_viewport);
        layout.name = "FPS Touch".to_string();
        // Make left stick floating (starts where you first touch)
        for el in &mut layout.elements {
            if el.id == "left_stick" {
                el.element_type = ControllerElementType::Joystick { floating: true, dead_zone: 0.08 };
                el.position = [0.15, 0.5]; // general left area
            }
            if el.id == "right_stick" {
                // Right side becomes a touch area for camera look
                el.element_type = ControllerElementType::TouchArea { tracks_velocity: true };
                el.position = [0.5, 0.2];
                el.size = [0.5, 0.75];
                el.opacity = 0.0; // invisible touch area
                el.label = "".to_string();
            }
        }
        layout
    }
}

// ─── Stream Quality ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQuality {
    pub enabled: bool,
    pub resolution: StreamResolution,
    pub fps: u8,
    pub bitrate_kbps: u32,
    pub codec: StreamCodec,
    pub latency_mode: LatencyMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamResolution {
    Low_480p, Medium_720p, High_1080p, Native,
    Custom { width: u32, height: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamCodec { H264, H265, Vp8, Vp9, Av1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LatencyMode {
    UltraLow,  // 30-50ms, lower quality
    Low,       // 50-100ms, balanced
    Standard,  // 100-200ms, best quality
}

impl StreamQuality {
    pub fn ultra_low_latency() -> Self {
        Self {
            enabled: true,
            resolution: StreamResolution::Low_480p,
            fps: 30,
            bitrate_kbps: 1500,
            codec: StreamCodec::H264,
            latency_mode: LatencyMode::UltraLow,
        }
    }

    pub fn balanced() -> Self {
        Self {
            enabled: true,
            resolution: StreamResolution::Medium_720p,
            fps: 60,
            bitrate_kbps: 4000,
            codec: StreamCodec::H264,
            latency_mode: LatencyMode::Low,
        }
    }
}

// ─── QR Code Generation ───────────────────────────────────────────────────────

/// Generates QR code data for phone controller connection
pub struct QrControllerLink {
    pub url: String,
    pub session_id: String,
    pub qr_data: Vec<u8>, // PNG bytes of QR code
    pub expires_at: u64,
}

impl QrControllerLink {
    pub fn generate(host: &str, port: u16, session_id: &str) -> Self {
        let url = format!("http://{}:{}/controller/{}", host, port, session_id);
        // In production: use a QR library to generate PNG bytes
        // For now: store the URL data
        Self {
            url: url.clone(),
            session_id: session_id.to_string(),
            qr_data: url.as_bytes().to_vec(), // placeholder
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 300,
        }
    }

    pub fn is_expired(&self) -> bool {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() > self.expires_at
    }
}

// ─── Phone Controller Manager ─────────────────────────────────────────────────

/// Manages all phone controller sessions
pub struct PhoneControllerManager {
    pub sessions: HashMap<String, PhoneControllerSession>,
    pub host_address: String,
    pub ws_port: u16,
    pub http_port: u16,
    pub max_controllers: u8,
    pub pending_qr: HashMap<u8, QrControllerLink>, // player_index → QR
    pub default_layout: ControllerLayout,
    pub layouts: HashMap<String, ControllerLayout>,
}

impl PhoneControllerManager {
    pub fn new(host: &str, ws_port: u16, http_port: u16) -> Self {
        let mut layouts = HashMap::new();
        layouts.insert("standard".to_string(), ControllerLayout::standard_gamepad(false));
        layouts.insert("standard_viewport".to_string(), ControllerLayout::standard_gamepad(true));
        layouts.insert("fps_touch".to_string(), ControllerLayout::fps_layout(false));
        layouts.insert("fps_viewport".to_string(), ControllerLayout::fps_layout(true));

        Self {
            sessions: HashMap::new(),
            host_address: host.to_string(),
            ws_port,
            http_port,
            max_controllers: 4,
            pending_qr: HashMap::new(),
            default_layout: ControllerLayout::standard_gamepad(false),
            layouts,
        }
    }

    /// Generate QR code for a player slot
    pub fn generate_qr_for_player(&mut self, player_index: u8) -> &QrControllerLink {
        let session_id = uuid::Uuid::new_v4().to_string();
        let qr = QrControllerLink::generate(&self.host_address, self.http_port, &session_id);
        self.pending_qr.insert(player_index, qr);
        self.pending_qr.get(&player_index).unwrap()
    }

    /// Get the WebSocket URL for a session
    pub fn ws_url(&self, session_id: &str) -> String {
        format!("ws://{}:{}/ws/{}", self.host_address, self.ws_port, session_id)
    }

    /// Process an incoming WebSocket message from a phone
    pub fn process_input(&self, session_id: &str, message: PhoneInputMessage) -> Option<GamepadInputEvent> {
        let session = self.sessions.get(session_id)?;

        match message {
            PhoneInputMessage::ButtonDown { element_id } => {
                let element = session.layout.elements.iter().find(|e| e.id == element_id)?;
                match &element.maps_to {
                    GameInputMapping::GamepadButton(btn) => Some(GamepadInputEvent::ButtonDown(btn.clone())),
                    _ => None,
                }
            }
            PhoneInputMessage::ButtonUp { element_id } => {
                let element = session.layout.elements.iter().find(|e| e.id == element_id)?;
                match &element.maps_to {
                    GameInputMapping::GamepadButton(btn) => Some(GamepadInputEvent::ButtonUp(btn.clone())),
                    _ => None,
                }
            }
            PhoneInputMessage::AxisChange { element_id, x, y } => {
                Some(GamepadInputEvent::AxisChange { axis: element_id, x, y })
            }
            PhoneInputMessage::Gyro { pitch, yaw, roll } => {
                Some(GamepadInputEvent::Gyro { pitch, yaw, roll })
            }
            _ => None,
        }
    }

    pub fn connected_count(&self) -> usize {
        self.sessions.values().filter(|s| s.connected).count()
    }

    pub fn disconnect(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// Update session layout (user customized button positions)
    pub fn update_layout(&mut self, session_id: &str, layout: ControllerLayout) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.layout = layout;
        }
    }
}

/// Messages from the phone → game
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PhoneInputMessage {
    ButtonDown { element_id: String },
    ButtonUp { element_id: String },
    AxisChange { element_id: String, x: f32, y: f32 },
    Gyro { pitch: f32, yaw: f32, roll: f32 },
    Accelerometer { x: f32, y: f32, z: f32 },
    Touch { x: f32, y: f32, pressure: f32 },
    LayoutUpdate { layout: ControllerLayout },
    Disconnect,
    Ping { timestamp: u64 },
}

/// Messages from game → phone
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PhoneOutputMessage {
    Haptic { pattern: HapticPattern },
    LayoutSync { layout: ControllerLayout },
    Frame { data: Vec<u8>, width: u32, height: u32 },
    Pong { timestamp: u64, server_time: u64 },
    GameState { data: serde_json::Value },
    Notification { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticPattern {
    pub duration_ms: u32,
    pub intensity: f32,
    pub pattern: HapticPatternType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HapticPatternType {
    Click, DoubleClick, LightImpact, MediumImpact, HeavyImpact,
    Error, Success, Warning,
    Custom { pulses: Vec<(u32, f32)> }, // (duration_ms, intensity)
}

#[derive(Debug, Clone)]
pub enum GamepadInputEvent {
    ButtonDown(String),
    ButtonUp(String),
    AxisChange { axis: String, x: f32, y: f32 },
    Gyro { pitch: f32, yaw: f32, roll: f32 },
}

extern crate uuid;
