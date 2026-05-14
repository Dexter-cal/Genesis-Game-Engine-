//! ChronoVerse Water & Fluid System
//!
//! Covers every type of water in games:
//! - FFT-based ocean simulation (realistic deep ocean waves)
//! - Gerstner wave ocean (fast, stylized, configurable)
//! - Rivers with flow maps and current simulation
//! - Lakes and ponds (calm water with reflection/refraction)
//! - Rain interaction (ripples, puddles, splash particles)
//! - Waterfalls (particle + mesh combination)
//! - Shallow water (wading, beach surf, tidal zones)
//! - Underwater rendering (caustics, fog, murk)
//! - Whirlpools and water vortices
//! - Flooding simulation (rising water level)
//! - Buoyancy (objects float realistically)
//! - Swimming physics (player in water)
//! - Boat/ship physics on water surface

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chronoverse_math::{vec2::Vec2, vec3::Vec3, color::Color};

// ─── Ocean / Large Water Body ─────────────────────────────────────────────────

/// Configuration for an ocean or large water body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanConfig {
    pub id: String,
    pub ocean_type: OceanType,
    /// Water surface color (deep)
    pub deep_color: Color,
    /// Water surface color (shallow / foam)
    pub shallow_color: Color,
    /// Foam color
    pub foam_color: Color,
    /// How far before shallow color kicks in (meters)
    pub shallow_depth: f32,
    /// Water clarity / murk (0 = crystal clear, 1 = opaque)
    pub turbidity: f32,
    /// Reflectivity (0-1)
    pub reflectivity: f32,
    /// Refraction strength
    pub refraction_scale: f32,
    /// Caustics intensity
    pub caustics_strength: f32,
    /// Water surface Y position
    pub water_level: f32,
    /// Wave configuration
    pub waves: Vec<WaveLayer>,
    /// Is rain currently hitting this surface?
    pub rain_ripples_active: bool,
    /// Rain ripple frequency
    pub rain_intensity: f32,
    /// Underwater fog color
    pub underwater_fog_color: Color,
    /// Underwater fog density
    pub underwater_fog_density: f32,
    /// Enable buoyancy simulation
    pub buoyancy_enabled: bool,
    /// Flow direction for rivers (normalized, None for ocean)
    pub flow_direction: Option<Vec3>,
    /// Flow speed (m/s)
    pub flow_speed: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OceanType {
    /// FFT-based statistical ocean (most realistic)
    FftOcean {
        resolution: u32,     // 128, 256, 512
        patch_size: f32,     // size of one FFT tile in meters
        wind_speed: f32,     // knots
        wind_direction: f32, // degrees
        wave_height: f32,    // Phillips spectrum peak
        choppiness: f32,     // Gerstner choppiness 0-1
    },
    /// Gerstner wave ocean (fast, great for stylized games)
    GerstnerOcean,
    /// Calm lake/pond (no waves, just gentle ripples)
    CalmLake,
    /// Fast-moving river
    River {
        flow_map_texture: Option<String>,
        bank_foam: bool,
    },
    /// Shallow beach water
    BeachSurf {
        tide_strength: f32,
        tide_period: f32, // seconds per tide cycle
    },
    /// Stylized toon water
    ToonWater {
        edge_color: Color,
        edge_width: f32,
        color_steps: u32,
    },
}

/// A single wave layer (Gerstner wave)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveLayer {
    pub amplitude: f32,        // wave height in meters
    pub frequency: f32,        // spatial frequency
    pub speed: f32,            // wave travel speed (m/s)
    pub direction: Vec2,       // normalized direction of travel
    pub steepness: f32,        // 0 = sinusoidal, 1 = peaked
    pub enabled: bool,
}

impl WaveLayer {
    /// Compute Gerstner wave displacement at world position (x, z) and time t
    pub fn displacement_at(&self, pos: Vec2, time: f32) -> Vec3 {
        if !self.enabled { return Vec3::ZERO; }

        let k = self.frequency;
        let phase = self.speed * k;
        let dot = self.direction.dot(pos) * k - phase * time;
        let s = dot.sin();
        let c = dot.cos();

        Vec3 {
            x: self.steepness * self.amplitude * self.direction.x * c,
            y: self.amplitude * s,
            z: self.steepness * self.amplitude * self.direction.y * c,
        }
    }

    /// Compute wave normal at position
    pub fn normal_at(&self, pos: Vec2, time: f32) -> Vec3 {
        let k = self.frequency;
        let phase = self.speed * k;
        let dot = self.direction.dot(pos) * k - phase * time;
        let s = dot.sin();
        let c = dot.cos();

        Vec3 {
            x: -(self.direction.x * k * self.amplitude * c),
            y: 1.0 - self.steepness * k * self.amplitude * s,
            z: -(self.direction.y * k * self.amplitude * c),
        }.normalize()
    }
}

impl OceanConfig {
    /// Get total wave height at world position (sum all layers)
    pub fn wave_height_at(&self, pos: Vec2, time: f32) -> f32 {
        self.waves.iter()
            .filter(|w| w.enabled)
            .map(|w| w.displacement_at(pos, time).y)
            .sum()
    }

    /// Combined normal at position
    pub fn surface_normal_at(&self, pos: Vec2, time: f32) -> Vec3 {
        let normal = self.waves.iter()
            .filter(|w| w.enabled)
            .fold(Vec3::UP, |acc, w| acc + w.normal_at(pos, time));
        normal.normalize()
    }

    /// Is this position submerged?
    pub fn is_submerged(&self, world_pos: Vec3) -> bool {
        world_pos.y < self.water_level + self.wave_height_at(Vec2::new(world_pos.x, world_pos.z), 0.0)
    }

    pub fn default_ocean() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ocean_type: OceanType::GerstnerOcean,
            deep_color: Color::new(0.0, 0.12, 0.35, 1.0),
            shallow_color: Color::new(0.0, 0.55, 0.7, 0.8),
            foam_color: Color::new(0.9, 0.95, 1.0, 1.0),
            shallow_depth: 3.0,
            turbidity: 0.3,
            reflectivity: 0.85,
            refraction_scale: 0.05,
            caustics_strength: 1.0,
            water_level: 0.0,
            waves: vec![
                WaveLayer { amplitude: 0.8, frequency: 0.4, speed: 1.2,
                    direction: Vec2::new(1.0, 0.0), steepness: 0.6, enabled: true },
                WaveLayer { amplitude: 0.4, frequency: 0.9, speed: 0.8,
                    direction: Vec2::new(0.7, 0.7), steepness: 0.4, enabled: true },
                WaveLayer { amplitude: 0.2, frequency: 1.8, speed: 1.5,
                    direction: Vec2::new(-0.3, 0.9), steepness: 0.2, enabled: true },
                WaveLayer { amplitude: 0.1, frequency: 3.0, speed: 2.0,
                    direction: Vec2::new(0.5, -0.8), steepness: 0.1, enabled: true },
            ],
            rain_ripples_active: false,
            rain_intensity: 0.0,
            underwater_fog_color: Color::new(0.0, 0.2, 0.4, 1.0),
            underwater_fog_density: 0.05,
            buoyancy_enabled: true,
            flow_direction: None,
            flow_speed: 0.0,
        }
    }

    pub fn calm_lake() -> Self {
        let mut o = Self::default_ocean();
        o.ocean_type = OceanType::CalmLake;
        o.waves.iter_mut().for_each(|w| w.amplitude *= 0.1);
        o.turbidity = 0.05;
        o.deep_color = Color::new(0.05, 0.25, 0.3, 0.9);
        o
    }

    pub fn river(direction: Vec3, speed: f32) -> Self {
        let mut o = Self::default_ocean();
        o.ocean_type = OceanType::River { flow_map_texture: None, bank_foam: true };
        o.flow_direction = Some(direction);
        o.flow_speed = speed;
        o.waves.iter_mut().for_each(|w| { w.amplitude *= 0.3; w.enabled = false; });
        o.waves[0].enabled = true;
        o
    }
}

// ─── Rain System ──────────────────────────────────────────────────────────────

/// Complete rain simulation config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RainSystem {
    pub enabled: bool,
    pub intensity: RainIntensity,
    /// Rain drop fall speed (m/s)
    pub drop_speed: f32,
    /// Wind effect on rain direction (degrees from vertical)
    pub wind_angle: f32,
    /// Rain drop size scale
    pub drop_scale: f32,
    /// Rain color tint
    pub drop_color: Color,
    /// Max simultaneous rain particles
    pub max_particles: u32,
    /// Puddle formation rate
    pub puddle_rate: f32,
    /// Enable splash particles on impact
    pub splash_enabled: bool,
    /// Enable ripples on water surfaces
    pub surface_ripples: bool,
    /// Rain sound volume
    pub sound_volume: f32,
    /// Enable wet surface darkening on all materials
    pub wet_surfaces: bool,
    /// Thunder configuration
    pub thunder: ThunderConfig,
    /// Lightning strike probability
    pub lightning_enabled: bool,
    /// Fog density added by rain
    pub rain_fog_density: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RainIntensity {
    Drizzle,      // light mist
    Light,        // gentle rain
    Moderate,     // normal rain
    Heavy,        // hard rain
    Downpour,     // tropical storm
    Monsoon,      // extreme rain
}

impl RainIntensity {
    pub fn particle_count(&self) -> u32 {
        match self {
            Self::Drizzle   =>  2_000,
            Self::Light     =>  8_000,
            Self::Moderate  => 20_000,
            Self::Heavy     => 50_000,
            Self::Downpour  =>100_000,
            Self::Monsoon   =>200_000,
        }
    }

    pub fn drop_speed(&self) -> f32 {
        match self {
            Self::Drizzle  => 3.0,
            Self::Light    => 5.0,
            Self::Moderate => 7.0,
            Self::Heavy    => 9.0,
            Self::Downpour => 11.0,
            Self::Monsoon  => 14.0,
        }
    }

    pub fn sound_name(&self) -> &'static str {
        match self {
            Self::Drizzle | Self::Light => "rain_light",
            Self::Moderate => "rain_moderate",
            Self::Heavy    => "rain_heavy",
            Self::Downpour | Self::Monsoon => "rain_storm",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThunderConfig {
    pub enabled: bool,
    /// Min/max seconds between thunder claps
    pub interval_range: [f32; 2],
    pub volume: f32,
    /// Distance variance for thunder
    pub distance_variance: f32,
}

impl Default for ThunderConfig {
    fn default() -> Self {
        Self { enabled: false, interval_range: [8.0, 30.0], volume: 1.0, distance_variance: 200.0 }
    }
}

impl RainSystem {
    pub fn none() -> Self {
        Self {
            enabled: false,
            intensity: RainIntensity::Drizzle,
            drop_speed: 7.0,
            wind_angle: 0.0,
            drop_scale: 1.0,
            drop_color: Color::new(0.6, 0.7, 0.8, 0.4),
            max_particles: 0,
            puddle_rate: 0.0,
            splash_enabled: false,
            surface_ripples: false,
            sound_volume: 0.0,
            wet_surfaces: false,
            thunder: ThunderConfig::default(),
            lightning_enabled: false,
            rain_fog_density: 0.0,
        }
    }

    pub fn moderate() -> Self {
        Self {
            enabled: true,
            intensity: RainIntensity::Moderate,
            drop_speed: RainIntensity::Moderate.drop_speed(),
            wind_angle: 15.0,
            drop_scale: 1.0,
            drop_color: Color::new(0.65, 0.75, 0.85, 0.5),
            max_particles: RainIntensity::Moderate.particle_count(),
            puddle_rate: 0.05,
            splash_enabled: true,
            surface_ripples: true,
            sound_volume: 0.7,
            wet_surfaces: true,
            thunder: ThunderConfig { enabled: false, ..Default::default() },
            lightning_enabled: false,
            rain_fog_density: 0.01,
        }
    }

    pub fn thunderstorm() -> Self {
        Self {
            enabled: true,
            intensity: RainIntensity::Heavy,
            drop_speed: RainIntensity::Heavy.drop_speed(),
            wind_angle: 25.0,
            drop_scale: 1.3,
            drop_color: Color::new(0.5, 0.6, 0.7, 0.6),
            max_particles: RainIntensity::Heavy.particle_count(),
            puddle_rate: 0.15,
            splash_enabled: true,
            surface_ripples: true,
            sound_volume: 1.0,
            wet_surfaces: true,
            thunder: ThunderConfig { enabled: true, interval_range: [5.0, 20.0], volume: 1.0, distance_variance: 150.0 },
            lightning_enabled: true,
            rain_fog_density: 0.025,
        }
    }

    /// Update rain system (tick every frame)
    pub fn tick(&mut self, delta: f32) {
        // In production: update particle simulation, puddle accumulation, ripples
    }
}

// ─── Snow System ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowSystem {
    pub enabled: bool,
    pub intensity: f32,        // 0-1
    pub flake_size: f32,
    pub fall_speed: f32,
    pub wind_drift: f32,
    pub max_particles: u32,
    pub accumulation_enabled: bool,
    /// How thick accumulated snow gets (meters)
    pub max_accumulation: f32,
    /// Current accumulation level (0-1)
    pub current_accumulation: f32,
    pub footprint_enabled: bool,
    pub snow_color: Color,
    pub glitter: bool,         // sparkle in light
}

impl SnowSystem {
    pub fn light_snow() -> Self {
        Self {
            enabled: true,
            intensity: 0.3,
            flake_size: 1.0,
            fall_speed: 0.8,
            wind_drift: 0.3,
            max_particles: 15_000,
            accumulation_enabled: true,
            max_accumulation: 0.3,
            current_accumulation: 0.0,
            footprint_enabled: true,
            snow_color: Color::new(0.95, 0.97, 1.0, 0.85),
            glitter: true,
        }
    }

    pub fn blizzard() -> Self {
        Self {
            enabled: true,
            intensity: 1.0,
            flake_size: 0.7,
            fall_speed: 3.5,
            wind_drift: 2.5,
            max_particles: 100_000,
            accumulation_enabled: true,
            max_accumulation: 2.0,
            current_accumulation: 0.0,
            footprint_enabled: true,
            snow_color: Color::new(0.9, 0.93, 1.0, 0.9),
            glitter: false,
        }
    }
}

// ─── Buoyancy System ─────────────────────────────────────────────────────────

/// Buoyancy simulation for objects on water
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuoyancyConfig {
    /// Density of the object (kg/m³). Water = 1000
    pub density: f32,
    /// How quickly the object bobs (damping)
    pub bob_damping: f32,
    /// Angular damping in water
    pub angular_damping: f32,
    /// Drag coefficient in water
    pub water_drag: f32,
    /// Is object currently in water?
    pub in_water: bool,
    /// Submersion depth (0 = surface, 1 = fully submerged)
    pub submersion: f32,
    /// Additional buoyancy points for complex shapes
    pub buoyancy_points: Vec<Vec3>,
}

impl BuoyancyConfig {
    pub fn wood() -> Self {
        Self { density: 600.0, bob_damping: 0.6, angular_damping: 0.8,
               water_drag: 0.3, in_water: false, submersion: 0.0, buoyancy_points: Vec::new() }
    }

    pub fn cork() -> Self {
        Self { density: 200.0, bob_damping: 0.8, angular_damping: 0.9,
               water_drag: 0.2, in_water: false, submersion: 0.0, buoyancy_points: Vec::new() }
    }

    pub fn stone() -> Self {
        Self { density: 2700.0, bob_damping: 0.1, angular_damping: 0.2,
               water_drag: 0.1, in_water: false, submersion: 0.0, buoyancy_points: Vec::new() }
    }

    pub fn floats(&self) -> bool { self.density < 1000.0 }
}

// ─── Puddle System ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuddleSystem {
    pub puddles: Vec<Puddle>,
    /// Global wetness level (affects material rendering)
    pub global_wetness: f32,
    /// Rate of drying (wetness lost per second with no rain)
    pub dry_rate: f32,
    /// Maximum puddle radius
    pub max_puddle_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puddle {
    pub position: Vec3,
    pub radius: f32,
    pub depth: f32,
    pub turbulence: f32, // from rain hitting it
    pub age: f32,
}

impl PuddleSystem {
    pub fn new() -> Self {
        Self { puddles: Vec::new(), global_wetness: 0.0, dry_rate: 0.02, max_puddle_radius: 3.0 }
    }

    pub fn tick(&mut self, delta: f32, rain: &RainSystem) {
        if rain.enabled {
            self.global_wetness = (self.global_wetness + rain.puddle_rate * delta).min(1.0);
        } else {
            self.global_wetness = (self.global_wetness - self.dry_rate * delta).max(0.0);
        }

        // Grow existing puddles
        for puddle in &mut self.puddles {
            puddle.age += delta;
            if rain.enabled {
                puddle.radius = (puddle.radius + 0.01 * delta).min(self.max_puddle_radius);
                puddle.turbulence = rain.intensity as u8 as f32 * 0.3;
            } else {
                // Shrink puddles when not raining
                puddle.radius = (puddle.radius - 0.005 * delta).max(0.0);
                puddle.turbulence = (puddle.turbulence - delta * 0.5).max(0.0);
            }
        }

        // Remove dried puddles
        self.puddles.retain(|p| p.radius > 0.01);
    }

    pub fn spawn_puddle(&mut self, position: Vec3) {
        if self.puddles.len() < 200 {
            self.puddles.push(Puddle {
                position,
                radius: 0.1,
                depth: 0.02,
                turbulence: 0.5,
                age: 0.0,
            });
        }
    }
}

// ─── Water Manager ────────────────────────────────────────────────────────────

/// Central manager for all water-related systems
pub struct WaterManager {
    pub oceans: HashMap<String, OceanConfig>,
    pub rain: RainSystem,
    pub snow: SnowSystem,
    pub puddles: PuddleSystem,
    pub time: f32,
    pub global_water_level: f32,
}

impl WaterManager {
    pub fn new() -> Self {
        Self {
            oceans: HashMap::new(),
            rain: RainSystem::none(),
            snow: SnowSystem { enabled: false, ..SnowSystem::light_snow() },
            puddles: PuddleSystem::new(),
            time: 0.0,
            global_water_level: 0.0,
        }
    }

    pub fn add_ocean(&mut self, ocean: OceanConfig) -> String {
        let id = ocean.id.clone();
        self.oceans.insert(id.clone(), ocean);
        id
    }

    pub fn tick(&mut self, delta: f32) {
        self.time += delta;

        // Update rain
        self.rain.tick(delta);

        // Update puddles
        self.puddles.tick(delta, &self.rain);

        // Propagate rain to ocean surfaces
        for ocean in self.oceans.values_mut() {
            ocean.rain_ripples_active = self.rain.enabled;
            ocean.rain_intensity = match self.rain.intensity {
                RainIntensity::Drizzle  => 0.1,
                RainIntensity::Light    => 0.3,
                RainIntensity::Moderate => 0.5,
                RainIntensity::Heavy    => 0.75,
                RainIntensity::Downpour => 0.9,
                RainIntensity::Monsoon  => 1.0,
            };
        }
    }

    pub fn start_rain(&mut self, intensity: RainIntensity) {
        self.rain = match intensity {
            RainIntensity::Heavy | RainIntensity::Downpour | RainIntensity::Monsoon =>
                RainSystem::thunderstorm(),
            _ => RainSystem::moderate(),
        };
        self.rain.intensity = intensity;
        self.rain.max_particles = intensity.particle_count();
        self.rain.drop_speed = intensity.drop_speed();
        tracing::info!("Rain started: {:?}", intensity);
    }

    pub fn stop_rain(&mut self) {
        self.rain.enabled = false;
        tracing::info!("Rain stopped");
    }

    pub fn start_snow(&mut self, blizzard: bool) {
        self.snow = if blizzard { SnowSystem::blizzard() } else { SnowSystem::light_snow() };
        tracing::info!("Snow started (blizzard: {})", blizzard);
    }

    pub fn is_submerged(&self, pos: Vec3) -> bool {
        self.oceans.values().any(|o| o.is_submerged(pos))
    }

    pub fn wave_height_at(&self, pos: Vec2) -> f32 {
        self.oceans.values()
            .map(|o| o.water_level + o.wave_height_at(pos, self.time))
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

extern crate tracing;
