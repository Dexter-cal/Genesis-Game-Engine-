//! ChronoVerse Camera Modes — Universal Camera OS
//!
//! 40+ camera mode combinations from one system.
//! Works with any input driver — webcam only, gamepad only, or hybrid.
//!
//! Modes:
//!   First Person, Second Person, Third Close, Third Far,
//!   Drone/Free, Cockpit, Strategy/Top-Down, Isometric,
//!   Side-Scroller, Follow, Orbit, Cinematic, VR,
//!   3D Window (head-tracked parallax), Off-Axis Projection
//!
//! Input Drivers (8):
//!   Classic (mouse/keyboard/gamepad)
//!   Head Tracking (webcam face detection, 6DOF)
//!   Eye Gaze (IR or webcam gaze estimation)
//!   Hand Gestures (MediaPipe hands)
//!   Full Body Pose (MediaPipe pose / depth camera)
//!   Voice Commands
//!   Audio Reactive (mic amplitude)
//!   Hybrid (any combination)
//!
//! Off-Axis Projection:
//!   Track viewer head position → generate asymmetric frustum
//!   → "3D Window" effect where game world appears BEHIND screen
//!   → Depth parallax makes brain perceive real 3D without VR
//!   → Lean in to inspect detail, lean back to widen view
//!   → Look around corners by moving head left/right

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chronoverse_math::{vec3::Vec3, vec2::Vec2, mat4::Mat4, quat::Quaternion};

// ─── Camera Mode ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CameraMode {
    /// Classic FPS — camera AT player eye level
    FirstPerson {
        fov: f32,
        head_bob: bool,
        bob_amplitude: f32,
        bob_frequency: f32,
        lean_enabled: bool,
        lean_angle: f32,
    },
    /// "You see yourself from NPC's POV" — unique narrative tool
    SecondPerson {
        target_entity: String,
        fov: f32,
        offset: Vec3,
    },
    /// Over-shoulder close
    ThirdClose {
        distance: f32,
        height: f32,
        shoulder_offset: f32,   // positive = right, negative = left
        fov: f32,
        collision_avoidance: bool,
        auto_shoulder_swap: bool,
    },
    /// Full character + environment visible
    ThirdFar {
        distance: f32,
        height: f32,
        fov: f32,
        min_distance: f32,
        max_distance: f32,
        auto_height: bool,
    },
    /// Free-fly spectator / editor camera
    Drone {
        speed: f32,
        fast_speed_multiplier: f32,
        roll_enabled: bool,
        height_lock: Option<f32>,
        gaze_lock_entity: Option<String>,
        orbit_target: Option<Vec3>,
    },
    /// Fixed seat, head-only rotation
    Cockpit {
        head_range_yaw: f32,
        head_range_pitch: f32,
        head_roll: bool,
        lean_sensitivity: f32,
        eye_relief: f32,        // distance to windshield in meters
    },
    /// Top-down God view
    Strategy {
        height: f32,
        min_height: f32,
        max_height: f32,
        edge_pan: bool,
        edge_pan_speed: f32,
        min_zoom: f32,
        max_zoom: f32,
        rotation_enabled: bool,
    },
    /// Fixed angle isometric
    Isometric {
        angle: f32,             // degrees from vertical
        zoom: f32,
        rotation_steps: u32,    // snap to N angles (0 = free)
        tile_aligned: bool,
    },
    /// 2D side-scroller
    SideScroller {
        z_locked: f32,
        parallax_layers: Vec<ParallaxLayer>,
        look_ahead: f32,        // camera leads player by this amount
        look_up: f32,
    },
    /// Follow a target smoothly
    Follow {
        target_entity: String,
        offset: Vec3,
        lag_strength: f32,
        look_ahead: f32,
        predict_motion: bool,
    },
    /// Orbit around a point or entity
    Orbit {
        target: OrbitTarget,
        distance: f32,
        min_distance: f32,
        max_distance: f32,
        azimuth: f32,
        elevation: f32,
        min_elevation: f32,
        max_elevation: f32,
        auto_distance: bool,
        invert_x: bool,
        invert_y: bool,
    },
    /// Cinematic — path-following, scripted
    Cinematic {
        cutscene_id: String,
        allow_skip: bool,
        letterbox: bool,
        letterbox_ratio: f32,
    },
    /// Virtual Reality
    Vr {
        tracking_origin: VrTrackingOrigin,
        ipd: f32,
        foveated_rendering: bool,
        passthrough: bool,
    },
    /// 3D Window / Off-Axis Projection (head-tracked glasses-free 3D)
    Window3D {
        screen_width_cm: f32,
        screen_height_cm: f32,
        viewer_distance_cm: f32,
        strength: f32,              // how much head movement affects frustum
        max_offset_cm: f32,         // safety limit
        smooth_factor: f32,         // low-pass filter strength
        depth_zoom_enabled: bool,   // lean in = zoom in
        depth_zoom_sensitivity: f32,
        parallax_separation: f32,   // visual near/far layer separation
        show_debug: bool,
    },
}

impl CameraMode {
    pub fn default_fov(&self) -> f32 {
        match self {
            Self::FirstPerson { fov, .. } => *fov,
            Self::ThirdClose { fov, .. } => *fov,
            Self::ThirdFar   { fov, .. } => *fov,
            _ => 75.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::FirstPerson  { .. } => "First Person",
            Self::SecondPerson { .. } => "Second Person",
            Self::ThirdClose   { .. } => "Third Person (Close)",
            Self::ThirdFar     { .. } => "Third Person (Far)",
            Self::Drone        { .. } => "Drone / Free Camera",
            Self::Cockpit      { .. } => "Cockpit",
            Self::Strategy     { .. } => "Strategy",
            Self::Isometric    { .. } => "Isometric",
            Self::SideScroller { .. } => "Side-Scroller",
            Self::Follow       { .. } => "Follow Camera",
            Self::Orbit        { .. } => "Orbit Camera",
            Self::Cinematic    { .. } => "Cinematic",
            Self::Vr           { .. } => "Virtual Reality",
            Self::Window3D     { .. } => "3D Window (Off-Axis)",
        }
    }

    pub fn supports_head_tracking(&self) -> bool {
        !matches!(self, Self::Cinematic { .. } | Self::Vr { .. })
    }

    pub fn transition_duration(&self) -> f32 { 0.3 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallaxLayer {
    pub z_depth: f32,
    pub scroll_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrbitTarget {
    Entity(String),
    Point(Vec3),
    Cursor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VrTrackingOrigin { Floor, Eye, Seated }

// ─── Input Driver ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputDriver {
    Classic,
    HeadTracking {
        webcam_device: String,
        model: HeadTrackModel,
        smooth: f32,
        x_sensitivity: f32,
        y_sensitivity: f32,
        z_sensitivity: f32,
    },
    EyeGaze {
        device: EyeGazeDevice,
        smooth: f32,
        deadzone: f32,
        aim_assist: bool,
    },
    HandGestures {
        handedness: Handedness,
        sensitivity: f32,
    },
    FullBodyPose {
        device: PoseDevice,
        mirror: bool,
    },
    Voice {
        language: String,
        commands: HashMap<String, String>,  // spoken → action
        push_to_talk: bool,
    },
    AudioReactive {
        device: String,
        amplitude_action: AudioAction,
        pitch_action: AudioAction,
        beat_action: AudioAction,
    },
    Hybrid(HybridConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeadTrackModel {
    OpenCvDnn,
    MediaPipeFaceMesh,
    Dlib,
    Onnx { model_path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EyeGazeDevice {
    Webcam { model: String },
    Tobii { serial: String },
    VrBuiltIn,
    Integrated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Handedness { Left, Right, Both }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoseDevice {
    Webcam,
    DepthCamera { device_id: String },
    MotionCapture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioAction { None, Zoom, Speed, Action(String) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridConfig {
    pub look_driver: String,    // driver name for camera look
    pub aim_driver: String,
    pub action_driver: String,
    pub movement_driver: String,
    pub fallback: String,
}

// ─── Off-Axis Projection ──────────────────────────────────────────────────────

/// Head position in physical space (from webcam tracking)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadPosition {
    /// Position in centimeters (from screen center)
    pub x_cm: f32,
    pub y_cm: f32,
    pub z_cm: f32,              // distance from screen
    /// Euler angles (degrees)
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub confidence: f32,        // 0-1, low = tracking lost
    pub last_valid: std::time::Instant,
}

impl HeadPosition {
    pub fn is_valid(&self) -> bool { self.confidence > 0.3 }

    /// Smoothly interpolate toward target
    pub fn lerp_to(&self, target: &HeadPosition, t: f32) -> HeadPosition {
        HeadPosition {
            x_cm: self.x_cm + (target.x_cm - self.x_cm) * t,
            y_cm: self.y_cm + (target.y_cm - self.y_cm) * t,
            z_cm: self.z_cm + (target.z_cm - self.z_cm) * t,
            pitch: self.pitch + (target.pitch - self.pitch) * t,
            yaw:   self.yaw   + (target.yaw   - self.yaw) * t,
            roll:  self.roll  + (target.roll  - self.roll) * t,
            confidence: target.confidence,
            last_valid: target.last_valid,
        }
    }
}

/// Off-axis projection — generates asymmetric frustum from head position
/// This is how we create the "3D window" / glasses-free 3D effect
pub struct OffAxisProjection {
    /// Physical screen dimensions in cm
    pub screen_width_cm: f32,
    pub screen_height_cm: f32,
    /// Default viewing distance
    pub default_distance_cm: f32,
    pub near: f32,
    pub far: f32,
    /// Strength multiplier
    pub strength: f32,
    /// Head position from tracker
    pub head: HeadPosition,
    pub head_smoothed: HeadPosition,
    pub smooth_factor: f32,
    pub depth_zoom_enabled: bool,
    pub depth_zoom_sensitivity: f32,
}

impl OffAxisProjection {
    pub fn new(screen_w: f32, screen_h: f32, view_dist: f32) -> Self {
        Self {
            screen_width_cm: screen_w,
            screen_height_cm: screen_h,
            default_distance_cm: view_dist,
            near: 0.1,
            far: 1000.0,
            strength: 1.0,
            head: HeadPosition::default(),
            head_smoothed: HeadPosition::default(),
            smooth_factor: 0.15,
            depth_zoom_enabled: true,
            depth_zoom_sensitivity: 0.5,
        }
    }

    pub fn update_head(&mut self, new_head: HeadPosition, delta: f32) {
        let t = 1.0 - (1.0 - self.smooth_factor).powf(delta * 60.0);
        self.head_smoothed = self.head_smoothed.lerp_to(&new_head, t);
        self.head = new_head;
    }

    /// Compute the asymmetric projection matrix for the current head position
    /// This is the CORE of the 3D window effect
    pub fn compute_projection_matrix(&self) -> Mat4 {
        let h = &self.head_smoothed;

        // Viewer distance (with depth zoom)
        let viewer_z = if self.depth_zoom_enabled {
            (self.default_distance_cm + h.z_cm * self.depth_zoom_sensitivity).max(10.0)
        } else {
            self.default_distance_cm
        };

        // Normalize head position to NDC space
        let eye_x = -(h.x_cm / self.screen_width_cm)  * 2.0 * self.strength;
        let eye_y =  (h.y_cm / self.screen_height_cm) * 2.0 * self.strength;

        // Asymmetric frustum — this is the magic!
        // Standard: left=-1, right=1
        // Off-axis: shifted by eye position
        let l = -1.0 + eye_x;
        let r =  1.0 + eye_x;
        let b = -1.0 + eye_y;
        let t =  1.0 + eye_y;

        // Scale to match viewer distance
        let scale = self.default_distance_cm / viewer_z;
        let near = self.near;
        let far = self.far;

        // Build frustum matrix
        let frustum_l = l * near * scale;
        let frustum_r = r * near * scale;
        let frustum_b = b * near * scale;
        let frustum_t = t * near * scale;

        // Perspective matrix from frustum
        let two_n = 2.0 * near;
        let rml = frustum_r - frustum_l;
        let tmb = frustum_t - frustum_b;
        let fmn = far - near;

        let mut cols = [[0.0f32; 4]; 4];
        cols[0][0] = two_n / rml;
        cols[1][1] = two_n / tmb;
        cols[2][0] = (frustum_r + frustum_l) / rml;
        cols[2][1] = (frustum_t + frustum_b) / tmb;
        cols[2][2] = -(far + near) / fmn;
        cols[2][3] = -1.0;
        cols[3][2] = -(two_n * far) / fmn;

        Mat4 { cols }
    }

    /// Compute how much the virtual camera position should shift
    /// to create the parallax "side peek" effect
    pub fn compute_camera_offset(&self) -> Vec3 {
        let h = &self.head_smoothed;
        Vec3::new(
            h.x_cm / self.screen_width_cm  * self.strength * 0.5,
            h.y_cm / self.screen_height_cm * self.strength * 0.5,
            (self.default_distance_cm - h.z_cm) / self.default_distance_cm * 0.1,
        )
    }

    /// Parallax multiplier for near objects (creates depth separation)
    pub fn near_parallax(&self) -> f32 { 1.5 }
    pub fn far_parallax(&self) -> f32  { 0.3 }
}

// ─── Camera Shake ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraShake {
    pub trauma: f32,              // 0-1, decays over time
    pub max_angle: f32,           // max rotation in degrees
    pub max_offset: f32,          // max position offset in meters
    pub frequency: f32,           // shake frequency Hz
    pub decay_rate: f32,          // trauma loss per second
    pub noise_seed: u64,
    pub rumble_low: f32,          // gamepad low-freq rumble
    pub rumble_high: f32,         // gamepad high-freq rumble
}

impl CameraShake {
    pub fn new() -> Self {
        Self {
            trauma: 0.0, max_angle: 10.0, max_offset: 0.1,
            frequency: 20.0, decay_rate: 1.5, noise_seed: 42,
            rumble_low: 0.0, rumble_high: 0.0,
        }
    }

    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
        // Scale rumble to trauma
        self.rumble_low  = self.trauma * 0.6;
        self.rumble_high = self.trauma * 0.3;
    }

    pub fn tick(&mut self, delta: f32) {
        self.trauma = (self.trauma - self.decay_rate * delta).max(0.0);
        self.rumble_low  = self.trauma * 0.6;
        self.rumble_high = self.trauma * 0.3;
    }

    /// Get shake intensity (trauma²)
    pub fn intensity(&self) -> f32 { self.trauma * self.trauma }

    /// Get rotation offset (degrees)
    pub fn rotation_offset(&self, time: f32) -> [f32; 3] {
        let i = self.intensity();
        let t = time * self.frequency;
        [
            i * self.max_angle * smooth_noise(t + 0.0, self.noise_seed),
            i * self.max_angle * smooth_noise(t + 100.0, self.noise_seed),
            i * self.max_angle * 0.3 * smooth_noise(t + 200.0, self.noise_seed),
        ]
    }

    /// Get position offset
    pub fn position_offset(&self, time: f32) -> Vec3 {
        let i = self.intensity();
        let t = time * self.frequency;
        Vec3::new(
            i * self.max_offset * smooth_noise(t + 300.0, self.noise_seed),
            i * self.max_offset * smooth_noise(t + 400.0, self.noise_seed),
            i * self.max_offset * 0.5 * smooth_noise(t + 500.0, self.noise_seed),
        )
    }
}

fn smooth_noise(t: f32, seed: u64) -> f32 {
    let s = (t.sin() * (seed as f32 * 0.001 + 1.0)).sin();
    s * 2.0 - 1.0
}

// ─── Multi-Camera System ─────────────────────────────────────────────────────

/// Multiple active cameras with compositing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraRig {
    pub id: String,
    pub name: String,
    pub cameras: Vec<RigCamera>,
    pub active_camera: String,
    pub transition: Option<CameraTransition>,
    pub global_shake: CameraShake,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigCamera {
    pub id: String,
    pub name: String,
    pub mode: CameraMode,
    pub driver: InputDriver,
    pub viewport: [f32; 4],         // normalized: x, y, w, h
    pub render_order: i32,
    pub clear_color: [f32; 4],
    pub active: bool,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub post_process: Vec<String>,
    pub shake: CameraShake,
    pub off_axis: Option<OffAxisProjection>,
    pub follow_target: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraTransition {
    pub from_camera: String,
    pub to_camera: String,
    pub transition_type: TransitionType,
    pub duration: f32,
    pub elapsed: f32,
    pub easing: EasingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionType {
    Cut,
    Blend,
    Zoom { zoom_in: bool },
    Pan  { arc: bool },
    Dolly,
    FadeBlack,
    FadeWhite,
    Wipe { direction: [f32; 2] },
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EasingType {
    Linear, EaseIn, EaseOut, EaseInOut, Spring,
    Bounce, Elastic, CubicBezier { p1: [f32; 2], p2: [f32; 2] },
}

// ─── Camera Controller ────────────────────────────────────────────────────────

/// Manages all cameras for a player/session
pub struct CameraController {
    pub rig: CameraRig,
    pub head_position: HeadPosition,
    pub gaze_point: Option<Vec2>,         // screen-space gaze point
    pub voice_active: bool,
    pub gesture_active: bool,
    pub body_pose_active: bool,
    pub current_mode_idx: usize,
    pub available_modes: Vec<CameraMode>,
    pub mode_history: Vec<String>,
    pub sensitivities: CameraSensitivities,
    pub time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSensitivities {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub gamepad_x: f32,
    pub gamepad_y: f32,
    pub head_x: f32,
    pub head_y: f32,
    pub head_z: f32,           // depth zoom sensitivity
    pub gaze_x: f32,
    pub gaze_y: f32,
    pub invert_x: bool,
    pub invert_y: bool,
    pub smooth: f32,
    pub acceleration: f32,
}

impl Default for CameraSensitivities {
    fn default() -> Self {
        Self {
            mouse_x: 1.0, mouse_y: 1.0,
            gamepad_x: 2.0, gamepad_y: 2.0,
            head_x: 0.5, head_y: 0.5, head_z: 0.3,
            gaze_x: 1.5, gaze_y: 1.5,
            invert_x: false, invert_y: false,
            smooth: 0.1, acceleration: 0.0,
        }
    }
}

impl CameraController {
    pub fn new() -> Self {
        let default_camera = RigCamera {
            id: "main".to_string(),
            name: "Main Camera".to_string(),
            mode: CameraMode::ThirdClose {
                distance: 5.0, height: 1.5, shoulder_offset: 0.5,
                fov: 75.0, collision_avoidance: true, auto_shoulder_swap: false,
            },
            driver: InputDriver::Classic,
            viewport: [0.0, 0.0, 1.0, 1.0],
            render_order: 0,
            clear_color: [0.05, 0.05, 0.08, 1.0],
            active: true,
            fov: 75.0, near: 0.1, far: 1000.0,
            post_process: Vec::new(),
            shake: CameraShake::new(),
            off_axis: None,
            follow_target: None,
            tags: Vec::new(),
        };

        let rig = CameraRig {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Player Camera Rig".to_string(),
            cameras: vec![default_camera],
            active_camera: "main".to_string(),
            transition: None,
            global_shake: CameraShake::new(),
        };

        Self {
            rig,
            head_position: HeadPosition::default(),
            gaze_point: None,
            voice_active: false,
            gesture_active: false,
            body_pose_active: false,
            current_mode_idx: 0,
            available_modes: vec![
                CameraMode::ThirdClose { distance: 5.0, height: 1.5, shoulder_offset: 0.5, fov: 75.0, collision_avoidance: true, auto_shoulder_swap: false },
                CameraMode::FirstPerson { fov: 90.0, head_bob: true, bob_amplitude: 0.05, bob_frequency: 2.0, lean_enabled: true, lean_angle: 15.0 },
                CameraMode::ThirdFar { distance: 12.0, height: 4.0, fov: 65.0, min_distance: 5.0, max_distance: 25.0, auto_height: true },
                CameraMode::Drone { speed: 15.0, fast_speed_multiplier: 4.0, roll_enabled: false, height_lock: None, gaze_lock_entity: None, orbit_target: None },
                CameraMode::Strategy { height: 25.0, min_height: 10.0, max_height: 60.0, edge_pan: true, edge_pan_speed: 20.0, min_zoom: 10.0, max_zoom: 60.0, rotation_enabled: true },
            ],
            mode_history: Vec::new(),
            sensitivities: CameraSensitivities::default(),
            time: 0.0,
        }
    }

    /// Cycle to the next camera mode
    pub fn cycle_mode(&mut self) {
        self.current_mode_idx = (self.current_mode_idx + 1) % self.available_modes.len();
        let next_mode = self.available_modes[self.current_mode_idx].clone();
        tracing::info!("Camera mode: {}", next_mode.name());
        if let Some(cam) = self.rig.cameras.get_mut(0) {
            cam.mode = next_mode;
        }
    }

    /// Enable 3D window mode with head tracking
    pub fn enable_3d_window(&mut self, screen_w_cm: f32, screen_h_cm: f32, dist_cm: f32) {
        let off_axis = OffAxisProjection::new(screen_w_cm, screen_h_cm, dist_cm);
        if let Some(cam) = self.rig.cameras.get_mut(0) {
            cam.mode = CameraMode::Window3D {
                screen_width_cm: screen_w_cm,
                screen_height_cm: screen_h_cm,
                viewer_distance_cm: dist_cm,
                strength: 1.0,
                max_offset_cm: 30.0,
                smooth_factor: 0.15,
                depth_zoom_enabled: true,
                depth_zoom_sensitivity: 0.5,
                parallax_separation: 0.3,
                show_debug: false,
            };
            cam.off_axis = Some(off_axis);
            cam.driver = InputDriver::HeadTracking {
                webcam_device: "default".to_string(),
                model: HeadTrackModel::MediaPipeFaceMesh,
                smooth: 0.85,
                x_sensitivity: 1.0, y_sensitivity: 1.0, z_sensitivity: 0.5,
            };
        }
        tracing::info!("3D Window mode enabled ({}×{} cm screen, {}cm viewing distance)", screen_w_cm, screen_h_cm, dist_cm);
    }

    /// Add camera shake (e.g. explosion nearby)
    pub fn shake(&mut self, trauma: f32) {
        self.rig.global_shake.add_trauma(trauma);
        for cam in &mut self.rig.cameras {
            cam.shake.add_trauma(trauma * 0.8);
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.time += delta;
        self.rig.global_shake.tick(delta);
        for cam in &mut self.rig.cameras {
            cam.shake.tick(delta);
        }
        if let Some(transition) = &mut self.rig.transition {
            transition.elapsed += delta;
            if transition.elapsed >= transition.duration {
                self.rig.transition = None;
            }
        }
    }

    /// Transition to another camera smoothly
    pub fn transition_to(&mut self, camera_id: &str, transition: TransitionType, duration: f32) {
        let from = self.rig.active_camera.clone();
        self.rig.transition = Some(CameraTransition {
            from_camera: from,
            to_camera: camera_id.to_string(),
            transition_type: transition,
            duration,
            elapsed: 0.0,
            easing: EasingType::EaseInOut,
        });
        self.rig.active_camera = camera_id.to_string();
    }

    pub fn add_camera(&mut self, camera: RigCamera) -> String {
        let id = camera.id.clone();
        self.rig.cameras.push(camera);
        id
    }
}

extern crate uuid;
extern crate tracing;
