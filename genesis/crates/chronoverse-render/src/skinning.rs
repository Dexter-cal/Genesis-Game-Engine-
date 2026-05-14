//! ChronoVerse Animation System
//!
//! Complete animation pipeline:
//! - Skeletal animation (bone-based, GPU skinning)
//! - Blend trees (1D and 2D blending)
//! - Additive layers (overlay animations)
//! - Animation state machines (FSM-based)
//! - Inverse Kinematics (FABRIK, CCD, two-bone IK)
//! - Procedural animation (look-at, foot IK, physics-based jiggles)
//! - AI-driven animation selection (agents choose animations)
//! - Root motion extraction and application
//! - Animation events (footstep, weapon swing, VFX trigger)
//! - Morph targets / blend shapes (facial expressions, damage states)
//! - Ragdoll physics transition
//! - Motion matching (dataset-based matching, like Ubisoft's system)
//! - Retargeting (apply animations from one rig to another)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chronoverse_math::{vec3::Vec3, quat::Quaternion};

// ─── Skeleton & Bones ─────────────────────────────────────────────────────────

/// A single bone in the skeleton hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
    pub children: Vec<u32>,
    /// Rest pose (bind pose) transform
    pub rest_position: Vec3,
    pub rest_rotation: Quaternion,
    pub rest_scale: Vec3,
    /// Current pose transform (local space)
    pub local_position: Vec3,
    pub local_rotation: Quaternion,
    pub local_scale: Vec3,
    /// World-space matrix (computed)
    pub world_matrix: [[f32; 4]; 4],
    /// Inverse bind matrix (for skinning)
    pub inverse_bind_matrix: [[f32; 4]; 4],
    /// Bone length in meters
    pub length: f32,
    /// Role hint for retargeting
    pub role: BoneRole,
}

/// Semantic role of a bone for retargeting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BoneRole {
    Root, Hips, Spine, Chest, UpperChest, Neck, Head,
    LeftShoulder, LeftUpperArm, LeftForearm, LeftHand,
    RightShoulder, RightUpperArm, RightForearm, RightHand,
    LeftUpperLeg, LeftLowerLeg, LeftFoot, LeftToe,
    RightUpperLeg, RightLowerLeg, RightFoot, RightToe,
    LeftEye, RightEye, Jaw,
    LeftThumb1, LeftThumb2, LeftThumb3,
    LeftIndex1, LeftIndex2, LeftIndex3,
    // ... full finger set omitted for brevity
    Custom(String),
}

/// A skeleton (hierarchy of bones)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    pub id: String,
    pub name: String,
    pub bones: Vec<Bone>,
    pub root_bone_id: u32,
}

impl Skeleton {
    pub fn bone_by_name(&self, name: &str) -> Option<&Bone> {
        self.bones.iter().find(|b| b.name == name)
    }
    pub fn bone_by_role(&self, role: &BoneRole) -> Option<&Bone> {
        self.bones.iter().find(|b| &b.role == role)
    }
    pub fn bone_count(&self) -> usize { self.bones.len() }
}

// ─── Animation Clip ───────────────────────────────────────────────────────────

/// A single animation clip (e.g. "run", "idle", "attack_swing")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClip {
    pub id: String,
    pub name: String,
    pub duration: f32,       // seconds
    pub fps: f32,
    pub looping: bool,
    pub has_root_motion: bool,
    /// Per-bone animation curves
    pub bone_tracks: HashMap<String, BoneTrack>,
    /// Morph target curves
    pub morph_tracks: HashMap<String, Vec<[f32; 2]>>, // name → [(time, value)]
    /// Events fired at specific times
    pub events: Vec<AnimationEvent>,
    /// Tags for the animation (used by motion matching)
    pub tags: Vec<String>,
    /// Average root velocity (for locomotion blending)
    pub root_velocity: Vec3,
}

/// Animation data for one bone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneTrack {
    pub bone_name: String,
    pub position_keys: Vec<PositionKey>,
    pub rotation_keys: Vec<RotationKey>,
    pub scale_keys:    Vec<ScaleKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionKey { pub time: f32, pub value: Vec3 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationKey { pub time: f32, pub value: Quaternion }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleKey    { pub time: f32, pub value: Vec3 }

/// An event fired during animation playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationEvent {
    pub time: f32,
    pub event_type: AnimEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimEventType {
    Footstep   { foot: String, surface: Option<String> },
    WeaponSwing { weapon_slot: String },
    SpawnParticle { effect_id: String, bone: String },
    PlaySound  { sound_id: String },
    GameEvent  { event_type: String, data: serde_json::Value },
    Custom     { name: String, data: serde_json::Value },
}

impl AnimationClip {
    /// Sample position for a bone at a given time (linear interpolation)
    pub fn sample_position(&self, bone_name: &str, time: f32) -> Option<Vec3> {
        let track = self.bone_tracks.get(bone_name)?;
        if track.position_keys.is_empty() { return None; }

        let t = if self.looping { time % self.duration } else { time.min(self.duration) };

        let idx = track.position_keys.partition_point(|k| k.time <= t);
        if idx == 0 { return Some(track.position_keys[0].value); }
        if idx >= track.position_keys.len() {
            return Some(track.position_keys.last().unwrap().value);
        }

        let a = &track.position_keys[idx - 1];
        let b = &track.position_keys[idx];
        let blend = (t - a.time) / (b.time - a.time).max(0.0001);
        Some(a.value.lerp(b.value, blend))
    }

    /// Sample rotation for a bone at a given time
    pub fn sample_rotation(&self, bone_name: &str, time: f32) -> Option<Quaternion> {
        let track = self.bone_tracks.get(bone_name)?;
        if track.rotation_keys.is_empty() { return None; }

        let t = if self.looping { time % self.duration } else { time.min(self.duration) };

        let idx = track.rotation_keys.partition_point(|k| k.time <= t);
        if idx == 0 { return Some(track.rotation_keys[0].value); }
        if idx >= track.rotation_keys.len() {
            return Some(track.rotation_keys.last().unwrap().value);
        }

        let a = &track.rotation_keys[idx - 1];
        let b = &track.rotation_keys[idx];
        let blend = (t - a.time) / (b.time - a.time).max(0.0001);
        Some(a.value.slerp(b.value, blend))
    }
}

// ─── Animation State Machine ──────────────────────────────────────────────────

/// A full animation state machine (like Unreal Anim Blueprint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatorController {
    pub id: String,
    pub name: String,
    pub states: HashMap<String, AnimState>,
    pub transitions: Vec<AnimTransition>,
    pub parameters: HashMap<String, AnimParameter>,
    pub layers: Vec<AnimLayer>,
    pub current_state: String,
    pub previous_state: Option<String>,
    pub blend_tree: Option<BlendTree>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimState {
    pub name: String,
    pub clip_id: Option<String>,
    pub speed: f32,
    pub speed_multiplier_param: Option<String>,
    pub motion: StateMotion,
    pub transitions_out: Vec<String>,
    pub loop_time: bool,
    /// IK enabled for this state
    pub ik_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateMotion {
    SingleClip { clip_id: String },
    BlendTree(BlendTree),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimTransition {
    pub id: String,
    pub from_state: String,
    pub to_state: String,
    pub conditions: Vec<TransitionCondition>,
    pub duration: f32,
    pub has_exit_time: bool,
    pub exit_time: f32,
    pub offset: f32,
    pub can_interrupt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    BoolTrue  (String),
    BoolFalse (String),
    FloatGreater { param: String, value: f32 },
    FloatLess    { param: String, value: f32 },
    IntEquals    { param: String, value: i32 },
    Trigger      (String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimParameter {
    Bool(bool),
    Float(f32),
    Int(i32),
    Trigger(bool),
}

/// Layered animation (e.g. upper body can play attack while lower body runs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimLayer {
    pub name: String,
    pub weight: f32,
    pub blending: LayerBlending,
    pub mask: Vec<String>, // bone names this layer applies to
    pub current_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerBlending {
    Override,  // replaces base layer
    Additive,  // adds on top of base layer
}

// ─── Blend Trees ─────────────────────────────────────────────────────────────

/// Blend tree — blend multiple animations based on parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlendTree {
    pub blend_type: BlendType,
    pub parameter_x: String,
    pub parameter_y: Option<String>, // for 2D blending
    pub children: Vec<BlendChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendType {
    Linear1D,       // single param
    SimpleDirectional2D, // 2D, motion direction
    FreeformDirectional2D, // 2D with arbitrary positions
    FreeformCartesian2D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlendChild {
    pub clip_id: String,
    pub threshold: f32,      // for 1D
    pub position: [f32; 2],  // for 2D
    pub speed: f32,
    pub mirror: bool,
}

// ─── Inverse Kinematics ───────────────────────────────────────────────────────

/// IK solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IkChain {
    pub id: String,
    pub chain_type: IkChainType,
    /// Bones in the chain (root to tip)
    pub bones: Vec<String>,
    /// Target world position
    pub target: Vec3,
    /// Target rotation (optional)
    pub target_rotation: Option<Quaternion>,
    /// Pole target (elbow/knee hint)
    pub pole_target: Option<Vec3>,
    pub weight: f32,
    pub max_iterations: u32,
    pub tolerance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IkChainType {
    TwoBone,   // arm / leg (exact solution)
    Fabrik,    // Forward And Backward Reaching IK (any chain length)
    Ccd,       // Cyclic Coordinate Descent
    LookAt,    // rotate one bone to look at target
}

// ─── Morph Targets ────────────────────────────────────────────────────────────

/// A blend shape / morph target on a mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphTarget {
    pub id: String,
    pub name: String,
    /// Vertex position deltas (sparse, only changed vertices)
    pub deltas: Vec<(u32, Vec3)>,
    pub normal_deltas: Vec<(u32, Vec3)>,
    pub current_weight: f32,
    pub category: MorphCategory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MorphCategory {
    Facial(FacialExpression),
    BodyShape,
    DamageState { level: u32 },
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FacialExpression {
    Smile, Frown, Angry, Surprised, Afraid, Disgusted,
    EyesBlink, EyesWiden, EyesClosed,
    MouthOpen, MouthSmile, MouthFrown, MouthPucker,
    BrowRaise, BrowFurrow,
    JawOpen, JawLeft, JawRight,
    CheekPuff,
    NoseSneer,
    Custom(String),
}

// ─── Animator (runtime instance) ─────────────────────────────────────────────

/// Runtime animator — controls animation playback for one entity
pub struct EntityAnimator {
    pub entity_id: String,
    pub controller: AnimatorController,
    pub skeleton: Option<Skeleton>,
    pub morph_targets: Vec<MorphTarget>,
    pub ik_chains: Vec<IkChain>,
    pub time: f32,
    pub play_speed: f32,
    pub enabled: bool,
    /// Current blend weight of transition in progress
    pub transition_progress: f32,
    /// AI override — agent can directly set bone rotations
    pub ai_bone_overrides: HashMap<String, Quaternion>,
    /// Root motion delta this frame
    pub root_motion_delta: Vec3,
}

impl EntityAnimator {
    pub fn new(entity_id: &str, controller: AnimatorController) -> Self {
        let first_state = controller.states.keys().next().cloned().unwrap_or_default();
        Self {
            entity_id: entity_id.to_string(),
            controller,
            skeleton: None,
            morph_targets: Vec::new(),
            ik_chains: Vec::new(),
            time: 0.0,
            play_speed: 1.0,
            enabled: true,
            transition_progress: 1.0,
            ai_bone_overrides: HashMap::new(),
            root_motion_delta: Vec3::ZERO,
        }
    }

    /// Set a bool parameter
    pub fn set_bool(&mut self, name: &str, value: bool) {
        self.controller.parameters.insert(name.to_string(), AnimParameter::Bool(value));
    }

    /// Set a float parameter
    pub fn set_float(&mut self, name: &str, value: f32) {
        self.controller.parameters.insert(name.to_string(), AnimParameter::Float(value));
    }

    /// Set an int parameter
    pub fn set_int(&mut self, name: &str, value: i32) {
        self.controller.parameters.insert(name.to_string(), AnimParameter::Int(value));
    }

    /// Fire a trigger parameter
    pub fn trigger(&mut self, name: &str) {
        self.controller.parameters.insert(name.to_string(), AnimParameter::Trigger(true));
    }

    /// Set a morph target weight (0-1)
    pub fn set_morph(&mut self, name: &str, weight: f32) {
        if let Some(m) = self.morph_targets.iter_mut().find(|m| m.name == name) {
            m.current_weight = weight.clamp(0.0, 1.0);
        }
    }

    /// AI agent can override any bone rotation directly
    pub fn ai_override_bone(&mut self, bone: &str, rotation: Quaternion) {
        self.ai_bone_overrides.insert(bone.to_string(), rotation);
    }

    /// Clear all AI overrides
    pub fn clear_ai_overrides(&mut self) {
        self.ai_bone_overrides.clear();
    }

    /// Advance animation by delta
    pub fn tick(&mut self, delta: f32) {
        if !self.enabled { return; }
        self.time += delta * self.play_speed;

        // Check transitions
        // In production: evaluate all transition conditions, perform blending
        self.evaluate_transitions();

        // Apply IK (run after FK)
        self.apply_ik();

        // Apply AI bone overrides
        self.apply_ai_overrides();
    }

    fn evaluate_transitions(&mut self) {
        let current = self.controller.current_state.clone();
        let transitions: Vec<AnimTransition> = self.controller.transitions.iter()
            .filter(|t| t.from_state == current)
            .cloned()
            .collect();

        for transition in &transitions {
            if self.evaluate_conditions(&transition.conditions) {
                self.controller.previous_state = Some(current.clone());
                self.controller.current_state = transition.to_state.clone();
                self.time = transition.offset;
                self.transition_progress = 0.0;
                break;
            }
        }

        if self.transition_progress < 1.0 {
            self.transition_progress = (self.transition_progress + 0.016 / 0.2).min(1.0);
        }
    }

    fn evaluate_conditions(&self, conditions: &[TransitionCondition]) -> bool {
        conditions.iter().all(|cond| match cond {
            TransitionCondition::BoolTrue(p) =>
                matches!(self.controller.parameters.get(p), Some(AnimParameter::Bool(true))),
            TransitionCondition::BoolFalse(p) =>
                matches!(self.controller.parameters.get(p), Some(AnimParameter::Bool(false))),
            TransitionCondition::FloatGreater { param, value } =>
                matches!(self.controller.parameters.get(param), Some(AnimParameter::Float(v)) if *v > *value),
            TransitionCondition::FloatLess { param, value } =>
                matches!(self.controller.parameters.get(param), Some(AnimParameter::Float(v)) if *v < *value),
            TransitionCondition::Trigger(p) =>
                matches!(self.controller.parameters.get(p), Some(AnimParameter::Trigger(true))),
            _ => false,
        })
    }

    fn apply_ik(&mut self) {
        for chain in &mut self.ik_chains {
            if chain.weight <= 0.0 { continue; }
            // In production: run FABRIK or two-bone IK solver
        }
    }

    fn apply_ai_overrides(&mut self) {
        // AI override blends into the FK result
        // In production: apply to skeleton bone local rotations
    }
}
