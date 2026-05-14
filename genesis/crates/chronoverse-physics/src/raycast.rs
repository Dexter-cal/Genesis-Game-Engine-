//! ChronoVerse Raycast & Aim System
//!
//! Complete raycasting:
//! - Physics raycasts (collision queries against world geometry)
//! - Screen-to-world raycasts (mouse/crosshair → world point)
//! - Spherecasts, capsulecasts, shape sweeps
//! - Multi-hit raycasts (pierce through objects)
//! - Layered raycasts (filter by collision layer)
//!
//! Aim System:
//! - Crosshair / reticle rendering (100+ styles)
//! - Aim assist (magnetic, bubble, slowdown)
//! - Hit detection and confirmation
//! - Bullet spread / accuracy system
//! - Projectile travel calculation
//! - Lead targeting (predictive aim for moving targets)
//! - Zoom / ADS (Aim Down Sights)
//! - First person weapon sway
//! - Recoil patterns
//! - Headshot detection
//! - Penetration (bullets through materials)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chronoverse_math::{vec3::Vec3, ray::Ray, color::Color};

// ─── Raycast Types ────────────────────────────────────────────────────────────

/// Result of a single raycast hit
#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub entity_id: Option<String>,
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub distance: f32,
    pub surface_material: Option<String>,
    pub layer: u32,
    pub collider_id: u64,
    pub is_trigger: bool,
    /// UV coordinates at hit point (for decal placement)
    pub uv: Option<[f32; 2]>,
    /// Bone name if hit a skinned mesh
    pub bone_hit: Option<String>,
    /// Body part category (head, torso, limb)
    pub body_part: Option<BodyPart>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodyPart {
    Head, Neck, UpperTorso, LowerTorso,
    LeftUpperArm, LeftForearm, LeftHand,
    RightUpperArm, RightForearm, RightHand,
    LeftUpperLeg, LeftLowerLeg, LeftFoot,
    RightUpperLeg, RightLowerLeg, RightFoot,
    Other(String),
}

impl BodyPart {
    pub fn damage_multiplier(&self) -> f32 {
        match self {
            Self::Head           => 3.0,
            Self::Neck           => 2.0,
            Self::UpperTorso     => 1.0,
            Self::LowerTorso     => 0.85,
            Self::LeftUpperArm | Self::RightUpperArm => 0.7,
            Self::LeftForearm | Self::RightForearm => 0.6,
            Self::LeftHand | Self::RightHand => 0.5,
            Self::LeftUpperLeg | Self::RightUpperLeg => 0.75,
            Self::LeftLowerLeg | Self::RightLowerLeg => 0.65,
            Self::LeftFoot | Self::RightFoot => 0.5,
            Self::Other(_) => 1.0,
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(self, Self::Head | Self::Neck)
    }
}

/// Raycast configuration
#[derive(Debug, Clone)]
pub struct RaycastConfig {
    pub max_distance: f32,
    pub layer_mask: u32,
    pub ignore_entities: Vec<String>,
    pub detect_triggers: bool,
    pub detect_navmesh: bool,
    pub precision: RaycastPrecision,
}

impl Default for RaycastConfig {
    fn default() -> Self {
        Self {
            max_distance: 1000.0,
            layer_mask: u32::MAX,
            ignore_entities: Vec::new(),
            detect_triggers: false,
            detect_navmesh: false,
            precision: RaycastPrecision::Standard,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RaycastPrecision {
    /// AABB only (fastest)
    BroadPhase,
    /// Collision mesh (standard)
    Standard,
    /// Triangle-level precision (slowest, most accurate)
    Triangle,
    /// Per-bone for skinned meshes
    Skeletal,
}

/// Multi-hit result (for penetrating shots)
#[derive(Debug, Clone)]
pub struct RaycastMultiHit {
    pub hits: Vec<RaycastHit>,
    pub penetrated_through: Vec<String>,
    pub total_distance: f32,
    pub stopped_at: Option<Vec3>,
}

// ─── Crosshair / Reticle System ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crosshair {
    pub id: String,
    pub name: String,
    pub style: CrosshairStyle,
    pub color: Color,
    pub hit_color: Color,       // flashes when hitting something
    pub crit_color: Color,      // flashes on crit/headshot
    pub size: f32,
    pub thickness: f32,
    pub gap: f32,               // gap in center (for spread indicator)
    pub dynamic_spread: bool,   // expands with movement/firing
    pub opacity: f32,
    pub outline: bool,
    pub outline_color: Color,
    pub outline_thickness: f32,
    pub dot: bool,              // center dot
    pub dot_size: f32,
    pub animated: bool,
    pub animation: CrosshairAnimation,
    /// Current spread (updated each frame)
    pub current_spread: f32,
    pub hit_flash_timer: f32,
    pub crit_flash_timer: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrosshairStyle {
    Default,           // classic 4-line cross
    Circle,            // ring
    CircleDot,         // ring + center dot
    Cross,             // + shape
    CrossDot,
    TSection,          // no top line
    Sniper,            // fine cross for sniper
    Dot,               // single dot
    Triangle,          // triangle pointing up
    DiamondOutline,
    CustomLines { count: u32, angles: Vec<f32> },
    Texture { asset_id: String },
    None,              // hidden (stealth, RTS, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosshairAnimation {
    pub anim_type: CrosshairAnimType,
    pub speed: f32,
    pub amplitude: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrosshairAnimType {
    None, Rotate, Pulse, Bounce, Spin,
}

impl Crosshair {
    pub fn default_fps() -> Self {
        Self {
            id: "default_fps".to_string(),
            name: "FPS Default".to_string(),
            style: CrosshairStyle::CrossDot,
            color: Color::new(0.0, 1.0, 0.0, 0.9),
            hit_color: Color::new(1.0, 0.8, 0.0, 1.0),
            crit_color: Color::new(1.0, 0.2, 0.0, 1.0),
            size: 6.0,
            thickness: 2.0,
            gap: 4.0,
            dynamic_spread: true,
            opacity: 0.9,
            outline: true,
            outline_color: Color::new(0.0, 0.0, 0.0, 0.5),
            outline_thickness: 1.0,
            dot: true,
            dot_size: 2.0,
            animated: false,
            animation: CrosshairAnimation { anim_type: CrosshairAnimType::None, speed: 1.0, amplitude: 1.0 },
            current_spread: 0.0,
            hit_flash_timer: 0.0,
            crit_flash_timer: 0.0,
        }
    }

    pub fn sniper() -> Self {
        Self {
            id: "sniper".to_string(),
            name: "Sniper".to_string(),
            style: CrosshairStyle::Circle,
            color: Color::new(1.0, 1.0, 1.0, 0.6),
            hit_color: Color::new(1.0, 0.5, 0.0, 1.0),
            crit_color: Color::new(1.0, 0.0, 0.0, 1.0),
            size: 15.0,
            thickness: 1.0,
            gap: 0.0,
            dynamic_spread: false,
            opacity: 0.7,
            outline: false,
            outline_color: Color::BLACK,
            outline_thickness: 0.5,
            dot: true,
            dot_size: 1.5,
            animated: false,
            animation: CrosshairAnimation { anim_type: CrosshairAnimType::None, speed: 1.0, amplitude: 1.0 },
            current_spread: 0.0,
            hit_flash_timer: 0.0,
            crit_flash_timer: 0.0,
        }
    }

    pub fn none() -> Self {
        let mut c = Self::default_fps();
        c.style = CrosshairStyle::None;
        c
    }

    pub fn tick(&mut self, delta: f32, is_moving: bool, is_firing: bool) {
        // Decay hit flash
        if self.hit_flash_timer > 0.0 { self.hit_flash_timer -= delta; }
        if self.crit_flash_timer > 0.0 { self.crit_flash_timer -= delta; }

        // Update spread
        if self.dynamic_spread {
            let target_spread = if is_firing { 8.0 } else if is_moving { 4.0 } else { 0.0 };
            let speed = if is_firing { 20.0 } else { 8.0 };
            let diff = target_spread - self.current_spread;
            self.current_spread += diff * delta * speed;
        }
    }

    pub fn on_hit(&mut self, is_crit: bool) {
        if is_crit {
            self.crit_flash_timer = 0.25;
        } else {
            self.hit_flash_timer = 0.15;
        }
    }

    pub fn current_color(&self) -> Color {
        if self.crit_flash_timer > 0.0 { self.crit_color }
        else if self.hit_flash_timer > 0.0 { self.hit_color }
        else { Color::new(self.color.r, self.color.g, self.color.b, self.color.a * self.opacity) }
    }

    pub fn visual_size(&self) -> f32 {
        self.size + self.current_spread
    }
}

// ─── Aim Assist System ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AimAssistConfig {
    pub enabled: bool,
    pub mode: AimAssistMode,
    /// How strongly to pull toward target (0-1)
    pub strength: f32,
    /// Radius around target that triggers assist (screen pixels)
    pub bubble_radius: f32,
    /// Max distance for aim assist
    pub max_distance: f32,
    /// Layers that aim assist targets
    pub target_layers: u32,
    /// Only assists against moving targets
    pub moving_targets_only: bool,
    /// Slow down look speed when near target
    pub slowdown_enabled: bool,
    pub slowdown_factor: f32,
    pub slowdown_radius: f32,
    /// Magnetic: pulls aim to target
    pub magnetic_enabled: bool,
    /// Friction: slows rotation through target
    pub friction_enabled: bool,
    /// Controller only (no mouse aim assist)
    pub controller_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AimAssistMode {
    None,
    Soft,     // slight pull, minimal slowdown
    Medium,   // Fortnite/COD style
    Strong,   // console shooter style
    Aimbot,   // dev/testing only
}

impl Default for AimAssistConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: AimAssistMode::Medium,
            strength: 0.35,
            bubble_radius: 80.0,
            max_distance: 150.0,
            target_layers: 0b110, // player + enemy layers
            moving_targets_only: false,
            slowdown_enabled: true,
            slowdown_factor: 0.4,
            slowdown_radius: 100.0,
            magnetic_enabled: true,
            friction_enabled: true,
            controller_only: true,
        }
    }
}

/// Active aim assist state per player
pub struct AimAssistState {
    pub config: AimAssistConfig,
    pub current_target: Option<String>,
    pub target_screen_pos: Option<[f32; 2]>,
    pub assist_strength_current: f32,
    pub in_slowdown_zone: bool,
    pub target_lock_timer: f32,
    pub candidates: Vec<AimTarget>,
}

#[derive(Debug, Clone)]
pub struct AimTarget {
    pub entity_id: String,
    pub screen_position: [f32; 2],
    pub world_position: Vec3,
    pub distance: f32,
    pub priority: f32,
    pub is_in_bubble: bool,
    pub velocity: Vec3,
}

impl AimAssistState {
    pub fn new(config: AimAssistConfig) -> Self {
        Self {
            config,
            current_target: None,
            target_screen_pos: None,
            assist_strength_current: 0.0,
            in_slowdown_zone: false,
            target_lock_timer: 0.0,
            candidates: Vec::new(),
        }
    }

    /// Compute aim assist adjustment for this frame
    pub fn compute_adjustment(
        &mut self,
        look_delta: [f32; 2],
        cursor_pos: [f32; 2],
        delta: f32,
        is_controller: bool,
    ) -> [f32; 2] {
        if !self.config.enabled { return look_delta; }
        if self.config.controller_only && !is_controller { return look_delta; }

        let mut result = look_delta;

        if let Some(target_pos) = self.target_screen_pos {
            let dx = target_pos[0] - cursor_pos[0];
            let dy = target_pos[1] - cursor_pos[1];
            let dist = (dx*dx + dy*dy).sqrt();

            // Slowdown zone
            if self.config.slowdown_enabled && dist < self.config.slowdown_radius {
                let t = 1.0 - (dist / self.config.slowdown_radius);
                let slowdown = 1.0 - t * (1.0 - self.config.slowdown_factor);
                result[0] *= slowdown;
                result[1] *= slowdown;
                self.in_slowdown_zone = true;
            } else {
                self.in_slowdown_zone = false;
            }

            // Magnetic pull
            if self.config.magnetic_enabled && dist < self.config.bubble_radius {
                let t = 1.0 - (dist / self.config.bubble_radius);
                let pull = t * self.config.strength * delta * 60.0;
                result[0] += dx.signum() * pull.min(dx.abs());
                result[1] += dy.signum() * pull.min(dy.abs());
            }
        }

        result
    }

    /// Lead prediction for moving targets
    pub fn predict_aim_point(&self, target: &AimTarget, projectile_speed: f32) -> Vec3 {
        let time_to_target = target.distance / projectile_speed;
        target.world_position + target.velocity * time_to_target
    }
}

// ─── Weapon Spread & Recoil ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponAccuracy {
    /// Base spread (degrees)
    pub base_spread: f32,
    /// Additional spread while moving
    pub move_spread: f32,
    /// Additional spread while jumping
    pub air_spread: f32,
    /// Spread increase per shot (reset on no fire)
    pub spread_per_shot: f32,
    /// Maximum spread
    pub max_spread: f32,
    /// Spread decay rate (per second)
    pub spread_decay: f32,
    /// Current spread state
    pub current_spread: f32,
    /// Recoil pattern
    pub recoil: RecoilPattern,
    /// ADSing reduces spread by this factor
    pub ads_spread_multiplier: f32,
    /// Crouch spread multiplier
    pub crouch_spread_multiplier: f32,
}

impl Default for WeaponAccuracy {
    fn default() -> Self {
        Self {
            base_spread: 0.5,
            move_spread: 1.5,
            air_spread: 3.0,
            spread_per_shot: 0.3,
            max_spread: 6.0,
            spread_decay: 4.0,
            current_spread: 0.0,
            recoil: RecoilPattern::default(),
            ads_spread_multiplier: 0.3,
            crouch_spread_multiplier: 0.6,
        }
    }
}

impl WeaponAccuracy {
    pub fn fire(&mut self) {
        self.current_spread = (self.current_spread + self.spread_per_shot).min(self.max_spread);
        self.recoil.fire();
    }

    pub fn tick(&mut self, delta: f32, is_moving: bool, is_in_air: bool, is_ads: bool, is_crouched: bool) {
        let mut target = self.base_spread;
        if is_moving  { target += self.move_spread; }
        if is_in_air  { target += self.air_spread; }
        if is_ads     { target *= self.ads_spread_multiplier; }
        if is_crouched { target *= self.crouch_spread_multiplier; }

        // Decay spread toward base
        if self.current_spread > target {
            self.current_spread -= self.spread_decay * delta;
            self.current_spread = self.current_spread.max(target);
        }

        self.recoil.tick(delta);
    }

    pub fn effective_spread(&self, is_ads: bool, is_crouched: bool) -> f32 {
        let mut s = self.current_spread;
        if is_ads { s *= self.ads_spread_multiplier; }
        if is_crouched { s *= self.crouch_spread_multiplier; }
        s
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoilPattern {
    pub pattern: Vec<[f32; 2]>,   // per-shot [pitch, yaw] offsets
    pub current_shot: usize,
    pub recovery_speed: f32,       // how fast aim returns after firing
    pub accumulated: [f32; 2],     // total recoil not yet recovered
    pub recovery_timer: f32,
    pub recovery_delay: f32,       // seconds before recovery starts
}

impl Default for RecoilPattern {
    fn default() -> Self {
        Self {
            // Typical AR pattern: goes up and slightly right
            pattern: vec![
                [0.1, 0.0], [0.15, 0.05], [0.2, 0.1], [0.2, 0.08],
                [0.18, -0.02], [0.15, -0.05], [0.12, -0.08],
                [0.1, -0.05], [0.08, 0.0], [0.06, 0.02],
            ],
            current_shot: 0,
            recovery_speed: 8.0,
            accumulated: [0.0, 0.0],
            recovery_timer: 0.0,
            recovery_delay: 0.15,
        }
    }
}

impl RecoilPattern {
    pub fn fire(&mut self) {
        if self.current_shot < self.pattern.len() {
            let shot = self.pattern[self.current_shot];
            self.accumulated[0] += shot[0];
            self.accumulated[1] += shot[1];
            self.current_shot += 1;
        } else if !self.pattern.is_empty() {
            // Past pattern end: use last shot with randomness
            let last = self.pattern.last().unwrap();
            self.accumulated[0] += last[0];
            self.accumulated[1] += last[1] * (rand_f32() * 2.0 - 1.0);
        }
        self.recovery_timer = self.recovery_delay;
    }

    pub fn tick(&mut self, delta: f32) {
        if self.recovery_timer > 0.0 {
            self.recovery_timer -= delta;
            return;
        }

        // Recover toward zero
        let speed = self.recovery_speed * delta;
        let old_shot = self.current_shot;

        if self.accumulated[0].abs() < speed {
            self.accumulated[0] = 0.0;
        } else {
            self.accumulated[0] -= self.accumulated[0].signum() * speed;
        }

        if self.accumulated[1].abs() < speed {
            self.accumulated[1] = 0.0;
        } else {
            self.accumulated[1] -= self.accumulated[1].signum() * speed;
        }

        if self.accumulated[0] == 0.0 && self.accumulated[1] == 0.0 {
            self.current_shot = 0; // reset pattern
        }
    }

    pub fn current_offset(&self) -> [f32; 2] { self.accumulated }
}

fn rand_f32() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    (n as f32 / u32::MAX as f32)
}

// ─── Penetration System ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialPenetration {
    pub material: String,
    pub thickness_threshold: f32, // max thickness bullet can pass through (cm)
    pub damage_reduction: f32,    // damage multiplier after penetration (0-1)
    pub speed_reduction: f32,     // speed reduction after penetration
    pub sound_on_hit: String,
    pub decal_type: String,       // impact decal style
}

pub struct PenetrationDatabase {
    pub materials: HashMap<String, MaterialPenetration>,
}

impl PenetrationDatabase {
    pub fn standard() -> Self {
        let mut db = Self { materials: HashMap::new() };

        db.materials.insert("glass".to_string(), MaterialPenetration {
            material: "glass".to_string(),
            thickness_threshold: 5.0,
            damage_reduction: 0.95,
            speed_reduction: 0.05,
            sound_on_hit: "impact_glass".to_string(),
            decal_type: "glass_shatter".to_string(),
        });
        db.materials.insert("wood".to_string(), MaterialPenetration {
            material: "wood".to_string(),
            thickness_threshold: 15.0,
            damage_reduction: 0.7,
            speed_reduction: 0.15,
            sound_on_hit: "impact_wood".to_string(),
            decal_type: "wood_splinter".to_string(),
        });
        db.materials.insert("drywall".to_string(), MaterialPenetration {
            material: "drywall".to_string(),
            thickness_threshold: 10.0,
            damage_reduction: 0.85,
            speed_reduction: 0.1,
            sound_on_hit: "impact_drywall".to_string(),
            decal_type: "wall_chunk".to_string(),
        });
        db.materials.insert("steel".to_string(), MaterialPenetration {
            material: "steel".to_string(),
            thickness_threshold: 0.5,
            damage_reduction: 0.0,
            speed_reduction: 1.0,
            sound_on_hit: "impact_metal_heavy".to_string(),
            decal_type: "metal_dent".to_string(),
        });
        db.materials.insert("concrete".to_string(), MaterialPenetration {
            material: "concrete".to_string(),
            thickness_threshold: 2.0,
            damage_reduction: 0.1,
            speed_reduction: 0.9,
            sound_on_hit: "impact_concrete".to_string(),
            decal_type: "concrete_crack".to_string(),
        });

        db
    }

    pub fn can_penetrate(&self, material: &str, thickness_cm: f32, bullet_penetration: f32) -> bool {
        if let Some(mat) = self.materials.get(material) {
            bullet_penetration >= thickness_cm / mat.thickness_threshold
        } else {
            false
        }
    }
}

// ─── Zoom / ADS System ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdsConfig {
    pub enabled: bool,
    /// Field of view when fully aiming
    pub ads_fov: f32,
    /// Normal FOV
    pub hip_fov: f32,
    /// Time to reach ADS (seconds)
    pub ads_time: f32,
    pub current_ads: f32,       // 0 = hip, 1 = full ADS
    pub scope_texture: Option<String>,
    pub scope_overlay: bool,
    pub parallax_correction: bool,
    /// Movement speed penalty while ADSing
    pub move_speed_multiplier: f32,
    /// Weapon sway in ADS
    pub sway_strength: f32,
    pub sway_speed: f32,
    pub sway_offset: [f32; 2],
}

impl Default for AdsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ads_fov: 40.0,
            hip_fov: 75.0,
            ads_time: 0.2,
            current_ads: 0.0,
            scope_texture: None,
            scope_overlay: false,
            parallax_correction: true,
            move_speed_multiplier: 0.75,
            sway_strength: 0.3,
            sway_speed: 1.5,
            sway_offset: [0.0, 0.0],
        }
    }
}

impl AdsConfig {
    pub fn tick(&mut self, delta: f32, is_ads: bool, velocity: Vec3) {
        let target = if is_ads { 1.0 } else { 0.0 };
        let speed = 1.0 / self.ads_time;
        self.current_ads += (target - self.current_ads) * speed * delta;
        self.current_ads = self.current_ads.clamp(0.0, 1.0);

        // Weapon sway
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f32();
        let sway = self.sway_strength * (1.0 - self.current_ads * 0.7);
        let vel_mag = velocity.length();
        self.sway_offset = [
            (t * self.sway_speed).sin() * sway + vel_mag * 0.01,
            (t * self.sway_speed * 0.7).cos() * sway * 0.7,
        ];
    }

    pub fn current_fov(&self) -> f32 {
        self.hip_fov + (self.ads_fov - self.hip_fov) * self.current_ads
    }

    pub fn is_fully_ads(&self) -> bool { self.current_ads >= 0.99 }
    pub fn is_hip_fire(&self) -> bool { self.current_ads <= 0.01 }
}
