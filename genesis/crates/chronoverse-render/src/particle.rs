//! ChronoVerse GPU Particle System
//!
//! Handles ALL particle-based effects:
//! - Fire, smoke, sparks
//! - Magic spells and abilities
//! - Explosions, debris
//! - Rain, snow, hail (also connected to weather system)
//! - Dust, footprints
//! - Bubble trails, water splashes
//! - Leaf scatter, pollen
//! - Blood, wounds (gore toggle)
//! - Crowd simulation (billboards)
//! - Boid flocking (birds, fish schools)
//!
//! All particles run on GPU via compute shaders for max performance.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chronoverse_math::{vec3::Vec3, vec2::Vec2, color::Color};

// ─── Particle Emitter ─────────────────────────────────────────────────────────

/// A particle emitter — the source of particles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitter {
    pub id: String,
    pub name: String,
    pub emitter_type: EmitterType,
    pub emission: EmissionConfig,
    pub particle: ParticleConfig,
    pub forces: Vec<ParticleForce>,
    pub collisions: ParticleCollisionConfig,
    pub renderer: ParticleRendererConfig,
    pub position: Vec3,
    pub rotation: [f32; 4],
    pub scale: f32,
    pub playing: bool,
    pub looping: bool,
    pub duration: f32,
    pub elapsed: f32,
    /// Number of currently alive particles
    pub alive_count: u32,
    /// Follow the parent entity?
    pub world_space: bool,
    /// Attach to agent (agent controls parameters)
    pub agent_controlled: bool,
}

/// Shape of the particle emitter volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmitterType {
    /// All particles from a single point
    Point,
    /// Particles from a line segment
    Line { length: f32 },
    /// Particles from a sphere surface or volume
    Sphere { radius: f32, emit_from_shell: bool },
    /// Particles from a box
    Box { half_extents: Vec3, emit_from_shell: bool },
    /// Particles from a cone (classic fire/smoke)
    Cone { angle_degrees: f32, radius: f32, emit_from_base: bool },
    /// Particles from a ring
    Ring { radius: f32, arc_degrees: f32 },
    /// Particles from a mesh surface
    Mesh { mesh_id: String, from_triangles: bool },
    /// Burst at position (explosion)
    Burst { radius: f32 },
    /// Trail following a moving object
    Trail { length: f32, min_vertex_distance: f32 },
}

/// How many particles are emitted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionConfig {
    /// Particles emitted per second (rate)
    pub rate: f32,
    /// Burst emission events
    pub bursts: Vec<BurstEvent>,
    /// Max particles alive at once
    pub max_particles: u32,
    /// Spawn rate multiplier curve over lifetime
    pub rate_over_lifetime: CurveF32,
    /// Spawn direction variation (0 = directional, 1 = spherical random)
    pub direction_randomness: f32,
    /// Initial spread cone angle
    pub spread_angle: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstEvent {
    /// Time in emitter lifetime to burst
    pub time: f32,
    /// Number of particles in burst
    pub count: u32,
    pub count_variance: u32,
    /// How often to repeat this burst (0 = once)
    pub repeat_interval: f32,
    pub repeat_count: i32, // -1 = infinite
}

/// Per-particle properties (initial and over-lifetime)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleConfig {
    /// Lifetime range (seconds)
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    /// Initial speed range (m/s)
    pub speed_min: f32,
    pub speed_max: f32,
    /// Initial size range
    pub size_start_min: f32,
    pub size_start_max: f32,
    /// Size at end of lifetime
    pub size_end_min: f32,
    pub size_end_max: f32,
    /// Color gradient over lifetime
    pub color_gradient: Vec<GradientKey>,
    /// Alpha gradient over lifetime
    pub alpha_gradient: Vec<GradientKey>,
    /// Rotation at start (degrees)
    pub rotation_start: f32,
    pub rotation_random: bool,
    /// Rotation speed (degrees/second)
    pub rotation_speed_min: f32,
    pub rotation_speed_max: f32,
    /// Gravity scale (negative = float up)
    pub gravity_scale: f32,
    /// Drag / air resistance
    pub drag: f32,
    /// Noise displacement strength
    pub noise_strength: f32,
    pub noise_frequency: f32,
    /// Sub-emitter: spawns new emitter when particle dies
    pub death_emitter: Option<String>,
    /// Texture atlas animation
    pub atlas_frames: u32,
    pub atlas_columns: u32,
    pub atlas_rows: u32,
    pub atlas_fps: f32,
}

/// A color/value key in a gradient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientKey {
    pub time: f32,    // 0-1
    pub value: f32,   // for alpha/size gradients
    pub color: Option<Color>, // for color gradients
}

/// Simple float curve (control points)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveF32 {
    pub keys: Vec<[f32; 2]>, // [time, value]
}

impl CurveF32 {
    pub fn constant(v: f32) -> Self {
        Self { keys: vec![[0.0, v], [1.0, v]] }
    }

    pub fn evaluate(&self, t: f32) -> f32 {
        if self.keys.is_empty() { return 1.0; }
        if t <= self.keys[0][0] { return self.keys[0][1]; }
        if t >= self.keys.last().unwrap()[0] { return self.keys.last().unwrap()[1]; }

        for i in 0..self.keys.len() - 1 {
            let a = self.keys[i];
            let b = self.keys[i + 1];
            if t >= a[0] && t <= b[0] {
                let local_t = (t - a[0]) / (b[0] - a[0]);
                return a[1] + (b[1] - a[1]) * local_t;
            }
        }
        1.0
    }
}

/// Forces that act on particles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleForce {
    /// Constant directional force (gravity, wind)
    Directional { force: Vec3 },
    /// Point attractor/repeller
    Point { position: Vec3, strength: f32, radius: f32 },
    /// Vortex rotation around an axis
    Vortex { position: Vec3, axis: Vec3, strength: f32, radius: f32 },
    /// Turbulence via noise field
    Turbulence { strength: f32, frequency: f32, octaves: u32 },
    /// Drag force opposing velocity
    Drag { coefficient: f32 },
    /// Curl noise (realistic smoke/cloud movement)
    CurlNoise { strength: f32, frequency: f32, scroll_speed: Vec3 },
}

/// How particles interact with world geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleCollisionConfig {
    pub enabled: bool,
    pub collision_mode: CollisionMode,
    pub bounce: f32,      // 0 = stick, 1 = perfect bounce
    pub friction: f32,
    pub lifetime_loss: f32, // 0-1 fraction of lifetime lost on collision
    pub die_on_collision: bool,
    pub spawn_event: Option<String>, // game event to fire on collision
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionMode {
    None,
    Planes,  // just floor plane
    World,   // full world geometry collision (expensive)
    Gpu,     // depth-buffer reprojection (cheapest GPU method)
}

/// How particles are rendered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleRendererConfig {
    pub render_mode: ParticleRenderMode,
    pub texture_id: Option<String>,
    pub material_id: Option<String>,
    pub blend_mode: ParticleBlend,
    pub sort_mode: ParticleSortMode,
    pub shadow_casting: bool,
    pub receive_shadows: bool,
    pub lighting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleRenderMode {
    /// Camera-facing billboard (default)
    Billboard,
    /// Billboard stretched along velocity
    StretchedBillboard { velocity_scale: f32, speed_scale: f32 },
    /// Always vertical billboard (trees, grass)
    VerticalBillboard,
    /// Full 3D mesh particle
    Mesh { mesh_id: String },
    /// Line from previous position to current
    Trail { width: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleBlend {
    Additive,    // fire, magic, glows
    Alpha,       // smoke, dust, standard
    Multiply,    // dark smoke, shadows
    Subtractive,
    Opaque,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleSortMode {
    None,            // fastest
    ByDistance,      // needed for alpha blend correctness
    OldestFirst,
    YoungestFirst,
}

impl ParticleEmitter {
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            emitter_type: EmitterType::Cone { angle_degrees: 25.0, radius: 0.1, emit_from_base: true },
            emission: EmissionConfig {
                rate: 30.0,
                bursts: Vec::new(),
                max_particles: 500,
                rate_over_lifetime: CurveF32::constant(1.0),
                direction_randomness: 0.1,
                spread_angle: 20.0,
            },
            particle: ParticleConfig {
                lifetime_min: 1.0, lifetime_max: 2.5,
                speed_min: 1.0, speed_max: 3.0,
                size_start_min: 0.1, size_start_max: 0.3,
                size_end_min: 0.0, size_end_max: 0.1,
                color_gradient: vec![
                    GradientKey { time: 0.0, value: 1.0, color: Some(Color::WHITE) },
                    GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(0.5,0.5,0.5,0.0)) },
                ],
                alpha_gradient: Vec::new(),
                rotation_start: 0.0, rotation_random: true,
                rotation_speed_min: -45.0, rotation_speed_max: 45.0,
                gravity_scale: 0.0,
                drag: 0.1,
                noise_strength: 0.0, noise_frequency: 1.0,
                death_emitter: None,
                atlas_frames: 1, atlas_columns: 1, atlas_rows: 1, atlas_fps: 8.0,
            },
            forces: Vec::new(),
            collisions: ParticleCollisionConfig {
                enabled: false, collision_mode: CollisionMode::None,
                bounce: 0.3, friction: 0.5, lifetime_loss: 0.0,
                die_on_collision: false, spawn_event: None,
            },
            renderer: ParticleRendererConfig {
                render_mode: ParticleRenderMode::Billboard,
                texture_id: None, material_id: None,
                blend_mode: ParticleBlend::Alpha,
                sort_mode: ParticleSortMode::None,
                shadow_casting: false, receive_shadows: false, lighting: false,
            },
            position: Vec3::ZERO,
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: 1.0,
            playing: false,
            looping: false,
            duration: 5.0,
            elapsed: 0.0,
            alive_count: 0,
            world_space: true,
            agent_controlled: false,
        }
    }

    // ─── Preset Effects ────────────────────────────────────────────────────

    pub fn fire() -> Self {
        let mut e = Self::new("Fire");
        e.emitter_type = EmitterType::Sphere { radius: 0.2, emit_from_shell: false };
        e.emission.rate = 80.0;
        e.emission.max_particles = 300;
        e.particle.lifetime_min = 0.5;
        e.particle.lifetime_max = 1.5;
        e.particle.speed_min = 1.5;
        e.particle.speed_max = 3.5;
        e.particle.size_start_min = 0.3;
        e.particle.size_start_max = 0.8;
        e.particle.size_end_min = 0.0;
        e.particle.size_end_max = 0.05;
        e.particle.gravity_scale = -0.5; // rises
        e.particle.noise_strength = 0.3;
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 1.0, color: Some(Color::new(1.0, 0.9, 0.1, 1.0)) },
            GradientKey { time: 0.3, value: 1.0, color: Some(Color::new(1.0, 0.4, 0.0, 1.0)) },
            GradientKey { time: 0.7, value: 0.6, color: Some(Color::new(0.4, 0.1, 0.0, 0.5)) },
            GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(0.1, 0.1, 0.1, 0.0)) },
        ];
        e.renderer.blend_mode = ParticleBlend::Additive;
        e.looping = true;
        e.playing = true;
        e
    }

    pub fn smoke() -> Self {
        let mut e = Self::new("Smoke");
        e.emitter_type = EmitterType::Cone { angle_degrees: 15.0, radius: 0.1, emit_from_base: true };
        e.emission.rate = 15.0;
        e.emission.max_particles = 200;
        e.particle.lifetime_min = 3.0;
        e.particle.lifetime_max = 6.0;
        e.particle.speed_min = 0.3;
        e.particle.speed_max = 0.8;
        e.particle.size_start_min = 0.4;
        e.particle.size_start_max = 0.8;
        e.particle.size_end_min = 2.0;
        e.particle.size_end_max = 4.0;
        e.particle.gravity_scale = -0.2;
        e.particle.noise_strength = 0.6;
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 0.0, color: Some(Color::new(0.4,0.4,0.4,0.0)) },
            GradientKey { time: 0.1, value: 0.5, color: Some(Color::new(0.5,0.5,0.5,0.6)) },
            GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(0.7,0.7,0.7,0.0)) },
        ];
        e.renderer.blend_mode = ParticleBlend::Alpha;
        e.looping = true;
        e.playing = true;
        e
    }

    pub fn magic_sparkles(color: Color) -> Self {
        let mut e = Self::new("Magic Sparkles");
        e.emitter_type = EmitterType::Sphere { radius: 0.5, emit_from_shell: true };
        e.emission.rate = 50.0;
        e.emission.max_particles = 150;
        e.particle.lifetime_min = 0.3;
        e.particle.lifetime_max = 0.8;
        e.particle.speed_min = 0.5;
        e.particle.speed_max = 2.0;
        e.particle.size_start_min = 0.03;
        e.particle.size_start_max = 0.1;
        e.particle.size_end_min = 0.0;
        e.particle.size_end_max = 0.0;
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 1.0, color: Some(color) },
            GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(color.r, color.g, color.b, 0.0)) },
        ];
        e.renderer.blend_mode = ParticleBlend::Additive;
        e.forces.push(ParticleForce::Turbulence { strength: 0.5, frequency: 2.0, octaves: 2 });
        e.looping = true;
        e.playing = true;
        e
    }

    pub fn explosion(radius: f32) -> Self {
        let mut e = Self::new("Explosion");
        e.emitter_type = EmitterType::Burst { radius };
        e.emission.rate = 0.0;
        e.emission.bursts = vec![
            BurstEvent { time: 0.0, count: 80, count_variance: 20, repeat_interval: 0.0, repeat_count: 0 },
        ];
        e.emission.max_particles = 120;
        e.particle.lifetime_min = 0.5;
        e.particle.lifetime_max = 1.5;
        e.particle.speed_min = 3.0;
        e.particle.speed_max = 12.0;
        e.particle.size_start_min = 0.2;
        e.particle.size_start_max = 0.8;
        e.particle.size_end_min = 0.0;
        e.particle.size_end_max = 0.0;
        e.particle.gravity_scale = 0.3;
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 1.0, color: Some(Color::new(1.0, 0.8, 0.2, 1.0)) },
            GradientKey { time: 0.2, value: 1.0, color: Some(Color::new(1.0, 0.3, 0.0, 1.0)) },
            GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(0.2, 0.2, 0.2, 0.0)) },
        ];
        e.renderer.blend_mode = ParticleBlend::Additive;
        e.collisions.enabled = true;
        e.collisions.collision_mode = CollisionMode::Planes;
        e.collisions.bounce = 0.2;
        e.looping = false;
        e.playing = true;
        e.duration = 2.0;
        e
    }

    pub fn water_splash() -> Self {
        let mut e = Self::new("Water Splash");
        e.emitter_type = EmitterType::Point;
        e.emission.rate = 0.0;
        e.emission.bursts = vec![
            BurstEvent { time: 0.0, count: 20, count_variance: 10, repeat_interval: 0.0, repeat_count: 0 },
        ];
        e.emission.max_particles = 40;
        e.emission.spread_angle = 60.0;
        e.particle.lifetime_min = 0.3;
        e.particle.lifetime_max = 0.8;
        e.particle.speed_min = 1.0;
        e.particle.speed_max = 4.0;
        e.particle.size_start_min = 0.05;
        e.particle.size_start_max = 0.15;
        e.particle.size_end_min = 0.0;
        e.particle.size_end_max = 0.0;
        e.particle.gravity_scale = 1.2;
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 0.7, color: Some(Color::new(0.7, 0.85, 1.0, 0.7)) },
            GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(0.8, 0.9, 1.0, 0.0)) },
        ];
        e.renderer.blend_mode = ParticleBlend::Alpha;
        e.collisions.enabled = true;
        e.collisions.collision_mode = CollisionMode::Planes;
        e.collisions.die_on_collision = true;
        e.looping = false;
        e.playing = true;
        e.duration = 1.0;
        e
    }

    pub fn footdust() -> Self {
        let mut e = Self::new("Foot Dust");
        e.emitter_type = EmitterType::Point;
        e.emission.rate = 0.0;
        e.emission.bursts = vec![
            BurstEvent { time: 0.0, count: 5, count_variance: 3, repeat_interval: 0.0, repeat_count: 0 },
        ];
        e.emission.max_particles = 20;
        e.particle.lifetime_min = 0.5;
        e.particle.lifetime_max = 1.2;
        e.particle.speed_min = 0.2;
        e.particle.speed_max = 0.8;
        e.particle.size_start_min = 0.1;
        e.particle.size_start_max = 0.3;
        e.particle.size_end_min = 0.3;
        e.particle.size_end_max = 0.6;
        e.particle.gravity_scale = -0.05;
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 0.0, color: Some(Color::new(0.7,0.6,0.4,0.0)) },
            GradientKey { time: 0.1, value: 0.3, color: Some(Color::new(0.7,0.6,0.4,0.3)) },
            GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(0.8,0.7,0.5,0.0)) },
        ];
        e.renderer.blend_mode = ParticleBlend::Alpha;
        e.looping = false;
        e.playing = true;
        e.duration = 1.5;
        e
    }

    pub fn leaves_scatter() -> Self {
        let mut e = Self::new("Leaf Scatter");
        e.emitter_type = EmitterType::Box { half_extents: Vec3::new(3.0, 0.5, 3.0), emit_from_shell: false };
        e.emission.rate = 5.0;
        e.emission.max_particles = 80;
        e.particle.lifetime_min = 4.0;
        e.particle.lifetime_max = 8.0;
        e.particle.speed_min = 0.2;
        e.particle.speed_max = 1.0;
        e.particle.size_start_min = 0.05;
        e.particle.size_start_max = 0.15;
        e.particle.size_end_min = 0.05;
        e.particle.size_end_max = 0.15;
        e.particle.gravity_scale = 0.3;
        e.particle.rotation_random = true;
        e.particle.rotation_speed_min = -90.0;
        e.particle.rotation_speed_max = 90.0;
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 1.0, color: Some(Color::new(0.5, 0.35, 0.05, 1.0)) },
            GradientKey { time: 0.5, value: 1.0, color: Some(Color::new(0.7, 0.45, 0.1, 1.0)) },
            GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(0.5, 0.3, 0.05, 0.0)) },
        ];
        e.forces.push(ParticleForce::Turbulence { strength: 0.8, frequency: 0.5, octaves: 2 });
        e.forces.push(ParticleForce::Directional { force: Vec3::new(0.5, 0.0, 0.2) }); // breeze
        e.renderer.blend_mode = ParticleBlend::Alpha;
        e.looping = true;
        e.playing = true;
        e
    }

    pub fn blood_splatter() -> Self {
        let mut e = Self::new("Blood Splatter");
        e.emitter_type = EmitterType::Burst { radius: 0.1 };
        e.emission.max_particles = 30;
        e.emission.bursts = vec![
            BurstEvent { time: 0.0, count: 15, count_variance: 5, repeat_interval: 0.0, repeat_count: 0 },
        ];
        e.particle.lifetime_min = 0.3;
        e.particle.lifetime_max = 1.0;
        e.particle.speed_min = 1.0;
        e.particle.speed_max = 6.0;
        e.particle.size_start_min = 0.02;
        e.particle.size_start_max = 0.08;
        e.particle.size_end_min = 0.02;
        e.particle.size_end_max = 0.05;
        e.particle.gravity_scale = 2.0;
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 1.0, color: Some(Color::new(0.7, 0.0, 0.0, 1.0)) },
            GradientKey { time: 1.0, value: 1.0, color: Some(Color::new(0.4, 0.0, 0.0, 1.0)) },
        ];
        e.collisions.enabled = true;
        e.collisions.collision_mode = CollisionMode::Planes;
        e.collisions.bounce = 0.05;
        e.collisions.die_on_collision = false;
        e.collisions.lifetime_loss = 0.6;
        e.renderer.blend_mode = ParticleBlend::Alpha;
        e.looping = false;
        e.playing = true;
        e.duration = 1.5;
        e
    }

    pub fn bubble_trail() -> Self {
        let mut e = Self::new("Bubbles");
        e.emitter_type = EmitterType::Sphere { radius: 0.1, emit_from_shell: false };
        e.emission.rate = 20.0;
        e.emission.max_particles = 100;
        e.particle.lifetime_min = 1.0;
        e.particle.lifetime_max = 3.0;
        e.particle.speed_min = 0.3;
        e.particle.speed_max = 1.0;
        e.particle.size_start_min = 0.03;
        e.particle.size_start_max = 0.1;
        e.particle.gravity_scale = -0.8; // float up
        e.particle.color_gradient = vec![
            GradientKey { time: 0.0, value: 0.3, color: Some(Color::new(0.8,0.9,1.0,0.3)) },
            GradientKey { time: 0.8, value: 0.4, color: Some(Color::new(0.9,1.0,1.0,0.4)) },
            GradientKey { time: 1.0, value: 0.0, color: Some(Color::new(1.0,1.0,1.0,0.0)) },
        ];
        e.renderer.blend_mode = ParticleBlend::Alpha;
        e.looping = true;
        e.playing = true;
        e
    }

    /// Play the emitter
    pub fn play(&mut self) { self.playing = true; self.elapsed = 0.0; }
    pub fn stop(&mut self) { self.playing = false; }

    /// Tick the emitter
    pub fn tick(&mut self, delta: f32) {
        if !self.playing { return; }
        self.elapsed += delta;
        if !self.looping && self.elapsed >= self.duration {
            self.playing = false;
        }
    }
}

/// Central particle system manager
pub struct ParticleManager {
    pub emitters: HashMap<String, ParticleEmitter>,
    pub global_particle_limit: u32,
    pub total_alive: u32,
    pub time: f32,
    pub wind: Option<Vec3>,
}

impl ParticleManager {
    pub fn new() -> Self {
        Self {
            emitters: HashMap::new(),
            global_particle_limit: 500_000,
            total_alive: 0,
            time: 0.0,
            wind: None,
        }
    }

    pub fn spawn(&mut self, emitter: ParticleEmitter) -> String {
        let id = emitter.id.clone();
        self.emitters.insert(id.clone(), emitter);
        id
    }

    pub fn spawn_at(&mut self, mut emitter: ParticleEmitter, position: Vec3) -> String {
        emitter.position = position;
        emitter.playing = true;
        self.spawn(emitter)
    }

    pub fn fire_at(&mut self, position: Vec3) -> String {
        self.spawn_at(ParticleEmitter::fire(), position)
    }

    pub fn explosion_at(&mut self, position: Vec3, radius: f32) -> String {
        self.spawn_at(ParticleEmitter::explosion(radius), position)
    }

    pub fn stop(&mut self, id: &str) {
        if let Some(e) = self.emitters.get_mut(id) { e.stop(); }
    }

    pub fn remove(&mut self, id: &str) { self.emitters.remove(id); }

    pub fn tick(&mut self, delta: f32) {
        self.time += delta;
        let mut to_remove = Vec::new();

        for (id, emitter) in &mut self.emitters {
            emitter.tick(delta);
            if !emitter.playing && !emitter.looping && emitter.alive_count == 0 {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove { self.emitters.remove(&id); }
    }

    pub fn set_wind(&mut self, wind: Option<Vec3>) { self.wind = wind; }
}
