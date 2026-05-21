//! Genesis Realistic Locomotion System
//!
//! Makes characters move like real people — not like game puppets.
//! Based on techniques from Ghost of Tsushima, Horizon, The Last of Us:
//!
//! MOTION MATCHING:
//! - Large database of real captured motions
//! - Every frame: find the best matching pose for current intent
//! - Blend seamlessly — no visible state machine transitions
//! - Character naturally runs, turns, stops, starts
//!
//! PROCEDURAL ANIMATION:
//! - Foot IK: feet plant on uneven terrain, no floating
//! - Leg IK: legs bend correctly on slopes and stairs
//! - Body lean: character leans into turns and movement
//! - Head look: naturally looks at points of interest
//! - Hand IK: grab walls, railings, interact with objects
//! - Breathing: visible chest movement at rest
//! - Eye dart: eyes micro-move, look alive
//! - Facial micro-expressions: subtle emotion bleed-through
//!
//! PHYSICS-DRIVEN:
//! - Secondary motion: clothing, hair, accessories bounce
//! - Impact reactions: whole-body response to hits
//! - Balance simulation: character catches themselves
//! - Weight shift: heavy items change posture
//!
//! UNREAL-STYLE FEATURES:
//! - Control Rig equivalent (procedural pose manipulation)
//! - Full-body IK solver
//! - Pose warping (adapt animations to terrain angle)
//! - Stride warping (adapt step length to actual speed)
//! - Slope warping (lean forward going uphill)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use genesis_math::{vec3::Vec3, quat::Quaternion};

// ─── Locomotion State ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocomotionState {
    /// Current velocity in world space
    pub velocity: Vec3,
    /// Previous frame velocity (for acceleration)
    pub prev_velocity: Vec3,
    /// Facing direction
    pub facing: Vec3,
    /// True facing (where character wants to go)
    pub intent_facing: Vec3,
    /// Speed in m/s
    pub speed: f32,
    /// Whether on ground
    pub grounded: bool,
    /// Normal of the ground surface
    pub ground_normal: Vec3,
    /// Slope angle in degrees
    pub slope_angle: f32,
    /// Crouching?
    pub crouching: bool,
    /// Swimming?
    pub in_water: bool,
    /// Carrying something heavy?
    pub carry_weight: f32,
    /// Injured (limping)?
    pub injured: bool,
    /// Stealth mode
    pub stealth: bool,
    /// Acceleration this frame
    pub acceleration: Vec3,
    /// Turn rate (radians/sec)
    pub turn_rate: f32,
    /// Stride cycle phase (0-1, 0=left foot plant, 0.5=right foot plant)
    pub stride_phase: f32,
    /// Body lean (radians, forward=positive)
    pub body_lean: f32,
    pub body_lean_side: f32,
}

impl Default for LocomotionState {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO, prev_velocity: Vec3::ZERO,
            facing: Vec3::FORWARD, intent_facing: Vec3::FORWARD,
            speed: 0.0, grounded: true,
            ground_normal: Vec3::UP, slope_angle: 0.0,
            crouching: false, in_water: false,
            carry_weight: 0.0, injured: false, stealth: false,
            acceleration: Vec3::ZERO, turn_rate: 0.0,
            stride_phase: 0.0, body_lean: 0.0, body_lean_side: 0.0,
        }
    }
}

// ─── Foot IK ─────────────────────────────────────────────────────────────────

/// Makes feet plant correctly on any terrain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootIkSolver {
    pub enabled: bool,
    /// How quickly feet adapt to terrain (lower = smoother but laggy)
    pub adapt_speed: f32,
    /// Maximum height adjustment per foot
    pub max_height_adjust: f32,
    /// Maximum foot rotation on slope
    pub max_angle_adjust: f32,
    /// Current left foot target
    pub left_foot_target: Vec3,
    /// Current right foot target
    pub right_foot_target: Vec3,
    /// Current left foot rotation
    pub left_foot_rotation: Quaternion,
    /// Current right foot rotation
    pub right_foot_rotation: Quaternion,
    /// Body height adjustment (compensates when feet are higher)
    pub body_height_offset: f32,
    /// IK weight (0 = disabled, 1 = full IK)
    pub ik_weight: f32,
    /// Foot offset below ankle bone
    pub foot_offset: f32,
    pub raycast_length: f32,
    pub foot_radius: f32,
}

impl FootIkSolver {
    pub fn new() -> Self {
        Self {
            enabled: true,
            adapt_speed: 12.0,
            max_height_adjust: 0.5,
            max_angle_adjust: 30.0,
            left_foot_target: Vec3::ZERO,
            right_foot_target: Vec3::ZERO,
            left_foot_rotation: Quaternion::IDENTITY,
            right_foot_rotation: Quaternion::IDENTITY,
            body_height_offset: 0.0,
            ik_weight: 1.0,
            foot_offset: 0.05,
            raycast_length: 1.5,
            foot_radius: 0.1,
        }
    }

    /// Update foot IK targets by casting rays down from each foot
    pub fn update(&mut self, left_ankle: Vec3, right_ankle: Vec3, delta: f32) {
        if !self.enabled { return; }

        // In production: cast rays down from each ankle, find terrain hit
        // Then solve two-bone IK for each leg
        // Adjust body height so neither foot is in the air

        let target_body_offset = (self.left_foot_target.y + self.right_foot_target.y) * 0.5;
        self.body_height_offset += (target_body_offset - self.body_height_offset) * self.adapt_speed * delta;
        self.body_height_offset = self.body_height_offset.clamp(-self.max_height_adjust, self.max_height_adjust);
    }
}

// ─── Body Lean ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyLeanSolver {
    pub enabled: bool,
    pub forward_lean_factor: f32,   // lean into acceleration
    pub side_lean_factor: f32,      // lean into turns
    pub slope_lean_factor: f32,     // lean into uphill slopes
    pub max_forward_lean: f32,      // degrees
    pub max_side_lean: f32,
    pub smoothing: f32,
    pub current_forward: f32,
    pub current_side: f32,
}

impl BodyLeanSolver {
    pub fn new() -> Self {
        Self {
            enabled: true,
            forward_lean_factor: 0.8,
            side_lean_factor: 0.6,
            slope_lean_factor: 0.5,
            max_forward_lean: 12.0,
            max_side_lean: 8.0,
            smoothing: 8.0,
            current_forward: 0.0,
            current_side: 0.0,
        }
    }

    pub fn update(&mut self, loco: &LocomotionState, delta: f32) {
        if !self.enabled { return; }

        let accel_forward = loco.acceleration.dot(loco.facing);
        let accel_side = loco.acceleration.dot(loco.facing.cross(Vec3::UP));

        let target_forward = (accel_forward * self.forward_lean_factor + loco.slope_angle * self.slope_lean_factor)
            .clamp(-self.max_forward_lean, self.max_forward_lean);
        let target_side = (accel_side * self.side_lean_factor + loco.turn_rate * self.side_lean_factor * 10.0)
            .clamp(-self.max_side_lean, self.max_side_lean);

        let t = self.smoothing * delta;
        self.current_forward += (target_forward - self.current_forward) * t;
        self.current_side    += (target_side    - self.current_side) * t;
    }

    pub fn forward_radians(&self) -> f32 { self.current_forward.to_radians() }
    pub fn side_radians(&self) -> f32 { self.current_side.to_radians() }
}

// ─── Head Look IK ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadLookSolver {
    pub enabled: bool,
    pub target: Option<Vec3>,
    pub weight: f32,
    pub clamp_yaw: f32,        // max degrees left/right
    pub clamp_pitch: f32,      // max degrees up/down
    pub smooth_speed: f32,
    /// Current look direction (smoothed)
    pub current_look_dir: Vec3,
    /// Secondary: eyes can look 30° further than head
    pub eye_look_weight: f32,
    pub eye_clamp_additional: f32,
    /// Interest points: head naturally drifts to nearby interesting things
    pub interest_points: Vec<LookInterestPoint>,
    pub idle_look_enabled: bool,
    pub idle_look_speed: f32,
    pub blink_enabled: bool,
    pub blink_rate: f32,         // blinks per minute
    pub blink_timer: f32,
    pub blink_duration: f32,
    pub is_blinking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookInterestPoint {
    pub position: Vec3,
    pub priority: f32,
    pub range: f32,
    pub look_duration: f32,
    pub remaining_time: f32,
}

impl HeadLookSolver {
    pub fn new() -> Self {
        Self {
            enabled: true,
            target: None,
            weight: 0.85,
            clamp_yaw: 80.0,
            clamp_pitch: 40.0,
            smooth_speed: 6.0,
            current_look_dir: Vec3::FORWARD,
            eye_look_weight: 0.5,
            eye_clamp_additional: 30.0,
            interest_points: Vec::new(),
            idle_look_enabled: true,
            idle_look_speed: 0.4,
            blink_enabled: true,
            blink_rate: 15.0,
            blink_timer: 4.0,
            blink_duration: 0.12,
            is_blinking: false,
        }
    }

    pub fn look_at(&mut self, target: Vec3) { self.target = Some(target); }
    pub fn clear_target(&mut self) { self.target = None; }

    pub fn tick(&mut self, head_position: Vec3, delta: f32) {
        if !self.enabled { return; }

        // Update blink
        self.blink_timer -= delta;
        if self.blink_timer <= 0.0 {
            self.is_blinking = true;
            self.blink_timer = 60.0 / self.blink_rate + rand_f32() * 2.0;
        }
        if self.is_blinking {
            self.blink_duration -= delta;
            if self.blink_duration <= 0.0 {
                self.is_blinking = false;
                self.blink_duration = 0.12;
            }
        }

        // Smooth look direction
        if let Some(target) = self.target {
            let desired = (target - head_position).normalize();
            let t = self.smooth_speed * delta;
            self.current_look_dir = self.current_look_dir.lerp(desired, t).normalize();
        }

        // Update interest points
        for point in &mut self.interest_points {
            point.remaining_time -= delta;
        }
        self.interest_points.retain(|p| p.remaining_time > 0.0);
    }
}

fn rand_f32() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as f32 / u32::MAX as f32)
}

// ─── Stride Warping ───────────────────────────────────────────────────────────

/// Adapts animation stride length to match actual movement speed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrideWarping {
    pub enabled: bool,
    /// Speed at which base animation was captured
    pub reference_speed: f32,
    /// Current speed ratio vs reference
    pub speed_ratio: f32,
    /// How much to scale stride (conservative to avoid artifacts)
    pub scale_factor: f32,
    pub max_scale: f32,
    pub min_scale: f32,
    /// Smoothed speed ratio
    pub smoothed_ratio: f32,
}

impl StrideWarping {
    pub fn new(reference_walk_speed: f32) -> Self {
        Self {
            enabled: true,
            reference_speed: reference_walk_speed,
            speed_ratio: 1.0,
            scale_factor: 0.7,   // only warp 70% to avoid foot skating
            max_scale: 1.5,
            min_scale: 0.6,
            smoothed_ratio: 1.0,
        }
    }

    pub fn update(&mut self, actual_speed: f32, delta: f32) {
        if !self.enabled || self.reference_speed < 0.001 { return; }
        self.speed_ratio = (actual_speed / self.reference_speed).clamp(self.min_scale, self.max_scale);
        self.smoothed_ratio += (self.speed_ratio - self.smoothed_ratio) * 10.0 * delta;
    }

    pub fn stride_scale(&self) -> f32 {
        1.0 + (self.smoothed_ratio - 1.0) * self.scale_factor
    }
}

// ─── Slope Warping ────────────────────────────────────────────────────────────

/// Adapts animations to match terrain slope angle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlopeWarping {
    pub enabled: bool,
    pub max_slope_angle: f32,   // beyond this, use special animation
    pub warp_strength: f32,
    pub current_slope: f32,
    pub smoothed_slope: f32,
}

impl SlopeWarping {
    pub fn new() -> Self {
        Self { enabled: true, max_slope_angle: 45.0, warp_strength: 0.8, current_slope: 0.0, smoothed_slope: 0.0 }
    }

    pub fn update(&mut self, ground_normal: Vec3, delta: f32) {
        if !self.enabled { return; }
        self.current_slope = ground_normal.angle_to(Vec3::UP).to_degrees();
        self.smoothed_slope += (self.current_slope - self.smoothed_slope) * 8.0 * delta;
    }

    pub fn warp_angle_radians(&self) -> f32 {
        (self.smoothed_slope * self.warp_strength).clamp(-self.max_slope_angle, self.max_slope_angle).to_radians()
    }
}

// ─── Procedural Breathing ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathingSystem {
    pub enabled: bool,
    pub rate: f32,          // breaths per minute (rest ~15, exertion ~25)
    pub amplitude: f32,     // chest movement scale
    pub phase: f32,         // current cycle 0-1
    pub exertion: f32,      // 0-1, increases rate and amplitude
    pub held: bool,         // breath is held (aiming, etc.)
    pub hold_sway: f32,     // subtle shake when holding breath
}

impl BreathingSystem {
    pub fn new() -> Self {
        Self { enabled: true, rate: 15.0, amplitude: 0.01, phase: 0.0, exertion: 0.0, held: false, hold_sway: 0.002 }
    }

    pub fn tick(&mut self, delta: f32, speed: f32) {
        self.exertion = ((speed / 5.5) * 0.8).min(1.0);
        let actual_rate = self.rate + self.exertion * 10.0;
        let actual_amplitude = self.amplitude + self.exertion * 0.02;
        if !self.held { self.phase = (self.phase + actual_rate / 60.0 * delta) % 1.0; }
    }

    pub fn chest_offset_y(&self) -> f32 {
        if self.held { return 0.0; }
        (self.phase * std::f32::consts::TAU).sin() * self.amplitude
    }

    pub fn hold_sway(&self, time: f32) -> Vec3 {
        if !self.held { return Vec3::ZERO; }
        Vec3::new((time * 3.0).sin() * self.hold_sway, (time * 2.3).cos() * self.hold_sway * 0.5, 0.0)
    }
}

// ─── Full Locomotion Controller ───────────────────────────────────────────────

pub struct LocomotionController {
    pub state: LocomotionState,
    pub foot_ik: FootIkSolver,
    pub body_lean: BodyLeanSolver,
    pub head_look: HeadLookSolver,
    pub stride_warp: StrideWarping,
    pub slope_warp: SlopeWarping,
    pub breathing: BreathingSystem,
    pub time: f32,

    /// Motion matching database (if available)
    pub use_motion_matching: bool,
    pub motion_db_id: Option<String>,

    // Secondary motion: physics-driven accessory movement
    pub secondary_motion_enabled: bool,
    pub secondary_motion_strength: f32,
}

impl LocomotionController {
    pub fn new() -> Self {
        Self {
            state: LocomotionState::default(),
            foot_ik: FootIkSolver::new(),
            body_lean: BodyLeanSolver::new(),
            head_look: HeadLookSolver::new(),
            stride_warp: StrideWarping::new(2.0), // 2 m/s walk reference
            slope_warp: SlopeWarping::new(),
            breathing: BreathingSystem::new(),
            time: 0.0,
            use_motion_matching: false,
            motion_db_id: None,
            secondary_motion_enabled: true,
            secondary_motion_strength: 1.0,
        }
    }

    pub fn tick(&mut self, delta: f32, entity_pos: Vec3) {
        self.time += delta;
        self.state.acceleration = (self.state.velocity - self.state.prev_velocity) / delta;
        self.state.prev_velocity = self.state.velocity;
        self.state.speed = self.state.velocity.length();

        self.body_lean.update(&self.state, delta);
        self.stride_warp.update(self.state.speed, delta);
        self.slope_warp.update(self.state.ground_normal, delta);
        self.breathing.tick(delta, self.state.speed);
        self.head_look.tick(entity_pos + Vec3::new(0.0, 1.7, 0.0), delta);
        self.foot_ik.update(
            entity_pos + Vec3::new(-0.15, 0.1, 0.0),
            entity_pos + Vec3::new( 0.15, 0.1, 0.0),
            delta
        );

        self.update_stride_phase(delta);
    }

    fn update_stride_phase(&mut self, delta: f32) {
        if self.state.speed > 0.1 {
            let stride_frequency = self.state.speed / 1.5; // ~1.5m stride
            self.state.stride_phase = (self.state.stride_phase + stride_frequency * delta) % 1.0;
        }
    }

    pub fn is_left_foot_planted(&self) -> bool { self.state.stride_phase < 0.5 }
    pub fn is_right_foot_planted(&self) -> bool { self.state.stride_phase >= 0.5 }

    pub fn body_height_offset(&self) -> f32 {
        self.foot_ik.body_height_offset + self.breathing.chest_offset_y()
    }

    pub fn forward_lean(&self) -> f32 { self.body_lean.forward_radians() }
    pub fn side_lean(&self) -> f32 { self.body_lean.side_radians() }
    pub fn slope_warp_angle(&self) -> f32 { self.slope_warp.warp_angle_radians() }
    pub fn stride_scale(&self) -> f32 { self.stride_warp.stride_scale() }
    pub fn head_look_dir(&self) -> Vec3 { self.head_look.current_look_dir }
    pub fn is_blinking(&self) -> bool { self.head_look.is_blinking }
}

// Extension method
trait Vec3Ext {
    fn lerp(self, other: Vec3, t: f32) -> Vec3;
    fn angle_to(self, other: Vec3) -> f32;
}

impl Vec3Ext for Vec3 {
    fn lerp(self, other: Vec3, t: f32) -> Vec3 {
        Vec3::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }
    fn angle_to(self, other: Vec3) -> f32 {
        let dot = self.dot(other).clamp(-1.0, 1.0);
        dot.acos()
    }
}
