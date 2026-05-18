//! Wind simulation system — affects cloth, particles, foliage, projectiles

use serde::{Serialize, Deserialize};
use genesis_math::vec3::Vec3;
use std::collections::HashMap;

/// A wind zone in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindZone {
    pub id: String,
    /// Base wind direction (normalized)
    pub direction: Vec3,
    /// Base wind speed (m/s)
    pub speed: f32,
    /// Turbulence intensity (0-1)
    pub turbulence: f32,
    /// Turbulence frequency (Hz)
    pub turbulence_frequency: f32,
    /// Zone type
    pub zone_type: WindZoneType,
    /// Affects particles
    pub affects_particles: bool,
    /// Affects cloth
    pub affects_cloth: bool,
    /// Affects foliage
    pub affects_foliage: bool,
    /// Affects rigid bodies (light ones)
    pub affects_rigidbodies: bool,
    /// Maximum body mass that wind affects (kg)
    pub max_affected_mass: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindZoneType {
    /// Affects the whole world uniformly
    Global,
    /// Spherical zone (wind blows outward from center)
    Sphere { center: Vec3, radius: f32 },
    /// Box zone
    Box { center: Vec3, half_extents: Vec3 },
    /// Directional cone (e.g. wind tunnel)
    Cone { origin: Vec3, direction: Vec3, angle_degrees: f32, range: f32 },
    /// Vortex / tornado
    Vortex { center: Vec3, radius: f32, height: f32, rotation_speed: f32 },
}

impl WindZone {
    pub fn global(direction: Vec3, speed: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            direction: direction.normalize(),
            speed,
            turbulence: 0.1,
            turbulence_frequency: 0.5,
            zone_type: WindZoneType::Global,
            affects_particles: true,
            affects_cloth: true,
            affects_foliage: true,
            affects_rigidbodies: false,
            max_affected_mass: 5.0,
        }
    }

    pub fn gentle_breeze() -> Self {
        Self::global(Vec3::new(1.0, 0.0, 0.3).normalize(), 2.5)
    }

    pub fn storm() -> Self {
        let mut z = Self::global(Vec3::new(1.0, -0.1, 0.2).normalize(), 18.0);
        z.turbulence = 0.8;
        z.affects_rigidbodies = true;
        z.max_affected_mass = 50.0;
        z
    }
}

/// Wind simulation manager — calculates wind force at any world position
pub struct WindManager {
    pub zones: Vec<WindZone>,
    pub time: f32,
    /// Noise seed for turbulence
    seed: u64,
}

impl WindManager {
    pub fn new() -> Self {
        Self { zones: Vec::new(), time: 0.0, seed: 42 }
    }

    pub fn add_zone(&mut self, zone: WindZone) { self.zones.push(zone); }
    pub fn remove_zone(&mut self, id: &str) { self.zones.retain(|z| z.id != id); }

    pub fn tick(&mut self, delta: f32) { self.time += delta; }

    /// Calculate total wind force vector at a world position
    pub fn wind_at(&self, position: Vec3) -> Vec3 {
        let mut total = Vec3::ZERO;

        for zone in &self.zones {
            if let Some(influence) = self.zone_influence_at(zone, position) {
                let base = zone.direction * zone.speed * influence;

                // Add turbulence using simple hash noise
                let turb = self.turbulence_at(position, zone) * zone.turbulence;
                let turbulence_vec = Vec3::new(
                    turb * (self.time * zone.turbulence_frequency).sin(),
                    turb * 0.1 * (self.time * zone.turbulence_frequency * 1.3).cos(),
                    turb * (self.time * zone.turbulence_frequency * 0.7 + 1.5).sin(),
                ) * zone.speed;

                total += base + turbulence_vec;
            }
        }

        total
    }

    /// Calculate the influence factor (0-1) of a zone at a position
    fn zone_influence_at(&self, zone: &WindZone, pos: Vec3) -> Option<f32> {
        match &zone.zone_type {
            WindZoneType::Global => Some(1.0),

            WindZoneType::Sphere { center, radius } => {
                let dist = pos.distance(*center);
                if dist >= *radius { None }
                else { Some(1.0 - dist / radius) }
            }

            WindZoneType::Box { center, half_extents } => {
                let local = pos - *center;
                if local.x.abs() <= half_extents.x &&
                   local.y.abs() <= half_extents.y &&
                   local.z.abs() <= half_extents.z {
                    // Falloff near edges
                    let fx = 1.0 - (local.x.abs() / half_extents.x);
                    let fy = 1.0 - (local.y.abs() / half_extents.y);
                    let fz = 1.0 - (local.z.abs() / half_extents.z);
                    Some(fx.min(fy).min(fz))
                } else { None }
            }

            WindZoneType::Cone { origin, direction, angle_degrees, range } => {
                let to_pos = pos - *origin;
                let dist = to_pos.length();
                if dist > *range { return None; }
                let dot = to_pos.normalize().dot(*direction);
                let cos_angle = (angle_degrees * std::f32::consts::PI / 180.0).cos();
                if dot >= cos_angle {
                    Some((1.0 - dist / range) * ((dot - cos_angle) / (1.0 - cos_angle)))
                } else { None }
            }

            WindZoneType::Vortex { center, radius, height, .. } => {
                let dist_xz = Vec3::new(pos.x - center.x, 0.0, pos.z - center.z).length();
                let dy = (pos.y - center.y).abs();
                if dist_xz >= *radius || dy >= *height * 0.5 { None }
                else { Some((1.0 - dist_xz / radius) * (1.0 - dy / (height * 0.5))) }
            }
        }
    }

    fn turbulence_at(&self, pos: Vec3, zone: &WindZone) -> f32 {
        // Simple value noise approximation
        let p = pos * zone.turbulence_frequency + Vec3::splat(self.time * 0.1);
        let x = (p.x * 127.1 + p.y * 311.7 + p.z * 74.7).sin() * 43758.5;
        (x - x.floor()) * 2.0 - 1.0
    }

    /// Get wind force as a physics force for an object with given mass
    pub fn force_for_object(&self, position: Vec3, mass: f32, drag_coefficient: f32) -> Vec3 {
        let wind = self.wind_at(position);
        let air_density = 1.225; // kg/m³
        let speed = wind.length();
        if speed < 0.01 { return Vec3::ZERO; }
        // F = 0.5 * ρ * v² * Cd * A  (simplified, A=1m²)
        let force_mag = 0.5 * air_density * speed * speed * drag_coefficient;
        wind.normalize() * force_mag.min(mass * 9.81 * 2.0) // cap force
    }

    pub fn has_zones(&self) -> bool { !self.zones.is_empty() }
}

// ─── Collision Layers ─────────────────────────────────────────────────────────

/// 32 collision layers (bitmask system like Unity/Unreal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollisionLayer(pub u32);

impl CollisionLayer {
    pub const DEFAULT:     Self = Self(1 << 0);
    pub const PLAYER:      Self = Self(1 << 1);
    pub const ENEMY:       Self = Self(1 << 2);
    pub const FRIENDLY:    Self = Self(1 << 3);
    pub const PROJECTILE:  Self = Self(1 << 4);
    pub const TRIGGER:     Self = Self(1 << 5);
    pub const TERRAIN:     Self = Self(1 << 6);
    pub const WATER:       Self = Self(1 << 7);
    pub const ITEM:        Self = Self(1 << 8);
    pub const VEHICLE:     Self = Self(1 << 9);
    pub const INTERACTABLE:Self = Self(1 << 10);
    pub const RAGDOLL:     Self = Self(1 << 11);
    pub const SENSOR:      Self = Self(1 << 12);
    pub const AI_VISION:   Self = Self(1 << 13);
    pub const CUSTOM_1:    Self = Self(1 << 16);
    pub const CUSTOM_2:    Self = Self(1 << 17);
    pub const CUSTOM_3:    Self = Self(1 << 18);
    pub const CUSTOM_4:    Self = Self(1 << 19);
    pub const ALL:         Self = Self(u32::MAX);
    pub const NONE:        Self = Self(0);

    pub fn contains(self, other: Self) -> bool { (self.0 & other.0) != 0 }
    pub fn combine(self, other: Self) -> Self { Self(self.0 | other.0) }
    pub fn exclude(self, other: Self) -> Self { Self(self.0 & !other.0) }
}

/// Layer interaction matrix — which layers collide with each other
pub struct LayerMatrix {
    /// matrix[i] is the bitmask of layers that layer i collides with
    matrix: [u32; 32],
}

impl LayerMatrix {
    pub fn all_collide() -> Self {
        Self { matrix: [u32::MAX; 32] }
    }

    pub fn set_collides(&mut self, layer_a: u32, layer_b: u32, collides: bool) {
        if collides {
            self.matrix[layer_a as usize] |= 1 << layer_b;
            self.matrix[layer_b as usize] |= 1 << layer_a;
        } else {
            self.matrix[layer_a as usize] &= !(1 << layer_b);
            self.matrix[layer_b as usize] &= !(1 << layer_a);
        }
    }

    pub fn does_collide(&self, layer_a: u32, layer_b: u32) -> bool {
        self.matrix[layer_a as usize] & (1 << layer_b) != 0
    }

    /// Standard game setup: player doesn't collide with friendly projectiles
    pub fn standard_game() -> Self {
        let mut m = Self::all_collide();
        // Player vs player projectiles: no collision
        m.set_collides(1, 4, false); // PLAYER vs PROJECTILE
        // Friendly vs enemy: no friendly fire by default
        m.set_collides(3, 4, false);
        m
    }
}

// ─── Character Controller ─────────────────────────────────────────────────────

use rapier3d::prelude::*;

/// Character controller — kinematic body with movement features
pub struct CharacterController {
    /// Rapier's KinematicCharacterController
    pub controller: rapier3d::control::KinematicCharacterController,
    /// Entity ID this controller belongs to
    pub entity_id: String,
    /// Desired velocity (set by scripts/AI)
    pub desired_velocity: Vec3,
    /// Currently grounded?
    pub grounded: bool,
    /// Coyote time remaining (can jump briefly after walking off ledge)
    pub coyote_time: f32,
    pub coyote_time_max: f32,
    /// Jump buffer (jump queued slightly before landing)
    pub jump_buffered: bool,
    pub jump_buffer_time: f32,
    pub jump_buffer_max: f32,
    /// Is the character currently sliding down a steep slope?
    pub sliding: bool,
    /// Vertical velocity (for gravity/jumping)
    pub vertical_velocity: f32,
    /// Maximum slope angle the character can walk on (degrees)
    pub max_slope_angle: f32,
    /// Step height the character can climb
    pub step_height: f32,
    /// Jump strength
    pub jump_speed: f32,
    /// Gravity scale
    pub gravity_scale: f32,
}

impl CharacterController {
    pub fn new(entity_id: &str) -> Self {
        let mut controller = rapier3d::control::KinematicCharacterController::default();
        controller.slide = true;
        controller.autostep = Some(rapier3d::control::CharacterAutostep {
            max_height: rapier3d::control::CharacterLength::Absolute(0.4),
            min_width: rapier3d::control::CharacterLength::Absolute(0.1),
            include_dynamic_bodies: true,
        });
        controller.snap_to_ground = Some(rapier3d::control::CharacterLength::Absolute(0.2));
        controller.max_slope_climb_angle = 45.0_f32.to_radians();
        controller.min_slope_slide_angle = 55.0_f32.to_radians();

        Self {
            controller,
            entity_id: entity_id.to_string(),
            desired_velocity: Vec3::ZERO,
            grounded: false,
            coyote_time: 0.0,
            coyote_time_max: 0.12,
            jump_buffered: false,
            jump_buffer_time: 0.0,
            jump_buffer_max: 0.15,
            sliding: false,
            vertical_velocity: 0.0,
            max_slope_angle: 45.0,
            step_height: 0.4,
            jump_speed: 7.0,
            gravity_scale: 1.0,
        }
    }

    pub fn jump(&mut self) {
        if self.grounded || self.coyote_time > 0.0 {
            self.vertical_velocity = self.jump_speed;
            self.coyote_time = 0.0;
        } else {
            // Buffer the jump for when we land
            self.jump_buffered = true;
            self.jump_buffer_time = self.jump_buffer_max;
        }
    }

    pub fn tick(&mut self, delta: f32) {
        // Coyote time countdown
        if self.grounded {
            self.coyote_time = self.coyote_time_max;
        } else if self.coyote_time > 0.0 {
            self.coyote_time -= delta;
        }

        // Jump buffer countdown
        if self.jump_buffer_time > 0.0 {
            self.jump_buffer_time -= delta;
            if self.jump_buffer_time <= 0.0 {
                self.jump_buffered = false;
            }
        }

        // Handle buffered jump on landing
        if self.grounded && self.jump_buffered {
            self.jump();
            self.jump_buffered = false;
        }

        // Apply gravity
        if !self.grounded {
            self.vertical_velocity -= 9.81 * self.gravity_scale * delta;
            self.vertical_velocity = self.vertical_velocity.max(-50.0); // terminal velocity
        } else {
            self.vertical_velocity = self.vertical_velocity.max(0.0);
        }
    }

    /// Build the complete desired movement for this frame
    pub fn movement_vector(&self, delta: f32) -> Vec3 {
        Vec3::new(
            self.desired_velocity.x * delta,
            self.vertical_velocity * delta,
            self.desired_velocity.z * delta,
        )
    }
}

// ─── Physics Queries ─────────────────────────────────────────────────────────

/// Results from a raycast
#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub entity_id: String,
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub distance: f32,
    pub collider_handle: rapier3d::geometry::ColliderHandle,
}

/// Results from a sphere/shape cast
#[derive(Debug, Clone)]
pub struct ShapeCastHit {
    pub entity_id: String,
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub time_of_impact: f32,
}

/// Result of an overlap test
#[derive(Debug, Clone)]
pub struct OverlapResult {
    pub entity_ids: Vec<String>,
}

// ─── Physics Plugin ───────────────────────────────────────────────────────────

use genesis_core::plugin::Plugin;
use async_trait::async_trait;
use genesis_core::engine::Engine;

pub struct PhysicsPlugin {
    pub physics: PhysicsWorld,
    pub wind:    WindManager,
    pub layers:  LayerMatrix,
}

impl PhysicsPlugin {
    pub fn new(gravity: Vec3, fixed_hz: f64) -> Self {
        Self {
            physics: PhysicsWorld::new(gravity, fixed_hz),
            wind:    WindManager::new(),
            layers:  LayerMatrix::standard_game(),
        }
    }
}

#[async_trait]
impl Plugin for PhysicsPlugin {
    fn name(&self) -> &str { "physics" }
    fn description(&self) -> &str { "Rapier3D physics simulation with wind and cloth" }

    async fn initialize(&mut self, engine: &mut Engine) -> anyhow::Result<()> {
        tracing::info!("Physics plugin initialized (Rapier3D, {}Hz)", 1.0 / self.physics.fixed_dt);
        Ok(())
    }

    async fn update(&mut self, engine: &mut Engine, delta: f32) -> anyhow::Result<()> {
        // Step wind simulation
        self.wind.tick(delta);

        // Step physics
        self.physics.step(delta, &engine.game_bus);

        Ok(())
    }

    async fn shutdown(&mut self, _engine: &mut Engine) -> anyhow::Result<()> {
        tracing::info!("Physics plugin shutdown ({} steps total)", self.physics.step_count);
        Ok(())
    }
}
