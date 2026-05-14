//! ChronoVerse World System
//!
//! Manages the entire game world:
//! - Infinite world streaming via chunk system
//! - Biome system (generates terrain, weather, flora, fauna per biome)
//! - Navigation mesh (pathfinding for NPCs and AI)
//! - Day/night cycle (time of day, sun/moon position)
//! - Weather system (integrates with particles and audio)
//! - World persistence (save/load world state)
//! - Spatial indexing for fast entity queries
//! - Level-of-Detail system for distant objects
//! - Procedural city/dungeon/terrain generation
//! - World events (earthquakes, floods, disasters)

pub mod chunk;
pub mod biome;
pub mod navmesh;
pub mod time_of_day;
pub mod weather;
pub mod spatial_index;
pub mod persistence;
pub mod generator;
pub mod events;

pub use chunk::*;
pub use biome::*;
pub use time_of_day::*;
pub use weather::*;
pub use spatial_index::*;
pub use generator::WorldGenerator;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chronoverse_math::vec3::Vec3;

// ─── Chunk System ─────────────────────────────────────────────────────────────

/// Size of one world chunk in meters
pub const CHUNK_SIZE: f32 = 128.0;
/// How many chunks to keep loaded around the player
pub const LOAD_RADIUS: i32 = 4;
/// Chunks beyond this radius are unloaded
pub const UNLOAD_RADIUS: i32 = 6;

/// A world chunk coordinate
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkCoord { pub x: i32, pub z: i32 }

impl ChunkCoord {
    pub fn from_world_pos(pos: Vec3) -> Self {
        Self {
            x: (pos.x / CHUNK_SIZE).floor() as i32,
            z: (pos.z / CHUNK_SIZE).floor() as i32,
        }
    }

    pub fn world_origin(&self) -> Vec3 {
        Vec3::new(self.x as f32 * CHUNK_SIZE, 0.0, self.z as f32 * CHUNK_SIZE)
    }

    pub fn distance_to(&self, other: ChunkCoord) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        (dx*dx + dz*dz).sqrt()
    }

    /// Get all chunk coords within a radius
    pub fn in_radius(center: ChunkCoord, radius: i32) -> Vec<ChunkCoord> {
        let mut coords = Vec::new();
        for z in -radius..=radius {
            for x in -radius..=radius {
                if x*x + z*z <= radius*radius {
                    coords.push(ChunkCoord { x: center.x + x, z: center.z + z });
                }
            }
        }
        coords
    }
}

/// The loaded state of a chunk
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkState {
    Unloaded,
    Generating,
    Generated,
    Loaded,     // full detail, entities active
    Sleeping,   // loaded but entities not ticking (far away)
    Unloading,
}

/// A world chunk containing terrain + entities
pub struct WorldChunk {
    pub coord: ChunkCoord,
    pub state: ChunkState,
    pub biome: BiomeType,
    pub height_map: Vec<f32>,          // 129x129 heights
    pub entity_ids: Vec<String>,       // entities in this chunk
    pub generated_by: String,          // "world_generator_agent" or "creator"
    pub seed: u64,
    pub last_accessed: std::time::Instant,
}

impl WorldChunk {
    pub fn new(coord: ChunkCoord, biome: BiomeType, seed: u64) -> Self {
        Self {
            coord,
            state: ChunkState::Unloaded,
            biome,
            height_map: vec![0.0; 129 * 129],
            entity_ids: Vec::new(),
            generated_by: "world_generator_agent".to_string(),
            seed,
            last_accessed: std::time::Instant::now(),
        }
    }

    pub fn height_at(&self, local_x: f32, local_z: f32) -> f32 {
        let xi = (local_x.clamp(0.0, CHUNK_SIZE) / CHUNK_SIZE * 128.0) as usize;
        let zi = (local_z.clamp(0.0, CHUNK_SIZE) / CHUNK_SIZE * 128.0) as usize;
        self.height_map.get(zi * 129 + xi).copied().unwrap_or(0.0)
    }
}

// ─── Biome System ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BiomeType {
    // ─── Temperate ───
    TemperateForest,
    TemperateGrassland,
    TemperateShrubland,
    // ─── Tropical ───
    TropicalRainforest,
    Savanna,
    TropicalDryForest,
    // ─── Cold ───
    Taiga,
    Tundra,
    SnowForest,
    Glacier,
    // ─── Arid ───
    Desert,
    DesertScrub,
    HotSpring,
    // ─── Wetlands ───
    Swamp,
    Mangrove,
    Marsh,
    // ─── Aquatic ───
    Ocean, DeepOcean,
    CoralReef,
    Beach,
    // ─── Special / Fantastical ───
    MagicForest,
    CrystalCaverns,
    Lava, Volcanic,
    FloatingIslands,
    Wasteland, PostApocalyptic,
    Frozen, Permafrost,
    Mushroom,
    Sky,
    Underground, Cave,
    // ─── Urban ───
    City, Town, Village, Ruins,
    // ─── Custom ───
    Custom(String),
}

/// Properties of a biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConfig {
    pub biome_type: BiomeType,
    pub display_name: String,
    /// Temperature (Celsius)
    pub temperature_min: f32,
    pub temperature_max: f32,
    /// Annual rainfall (mm)
    pub rainfall: f32,
    /// Terrain roughness 0-1
    pub terrain_roughness: f32,
    /// Average elevation (meters)
    pub elevation: f32,
    /// Height variance
    pub elevation_variance: f32,
    /// Flora spawn rates (entity type → per chunk count)
    pub flora: HashMap<String, f32>,
    /// Fauna spawn rates
    pub fauna: HashMap<String, f32>,
    /// Base ground material
    pub ground_material: String,
    /// Sky color (morning, noon, evening, night)
    pub sky_colors: [chronoverse_math::color::Color; 4],
    /// Fog density
    pub fog_density: f32,
    pub fog_color: chronoverse_math::color::Color,
    /// Preferred weather patterns
    pub weather_weights: HashMap<String, f32>,
    /// NPC factions likely to be here
    pub factions: Vec<String>,
    /// Music track for this biome
    pub music_track: Option<String>,
    /// Ambient sounds
    pub ambient_sounds: Vec<String>,
}

// ─── Time of Day ─────────────────────────────────────────────────────────────

/// The game world's time system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldTime {
    /// Current time in hours (0-24)
    pub hour: f32,
    /// Current day (1-indexed)
    pub day: u32,
    /// Current month (1-12)
    pub month: u32,
    /// Current year
    pub year: u32,
    /// Time scale (1.0 = real time, 60.0 = 1 minute per second)
    pub time_scale: f32,
    pub paused: bool,
    /// Day length in real seconds (default: 20 minutes)
    pub day_duration_secs: f32,
}

impl WorldTime {
    pub fn new() -> Self {
        Self {
            hour: 8.0, // Start at 8 AM
            day: 1, month: 6, year: 1000,
            time_scale: 72.0, // 20 min days (24*60/20 = 72x)
            paused: false,
            day_duration_secs: 1200.0,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        if self.paused { return; }
        let hours_per_second = 24.0 / self.day_duration_secs * self.time_scale;
        self.hour += hours_per_second * delta;
        while self.hour >= 24.0 {
            self.hour -= 24.0;
            self.advance_day();
        }
    }

    fn advance_day(&mut self) {
        self.day += 1;
        let days_in_month = self.days_in_month(self.month, self.year);
        if self.day > days_in_month {
            self.day = 1;
            self.month += 1;
            if self.month > 12 {
                self.month = 1;
                self.year += 1;
            }
        }
    }

    fn days_in_month(&self, month: u32, year: u32) -> u32 {
        match month {
            2 => if year % 4 == 0 { 29 } else { 28 },
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    pub fn is_daytime(&self) -> bool { self.hour >= 6.0 && self.hour < 20.0 }
    pub fn is_night(&self) -> bool { !self.is_daytime() }
    pub fn is_dawn(&self) -> bool { self.hour >= 5.0 && self.hour < 7.5 }
    pub fn is_dusk(&self) -> bool { self.hour >= 18.5 && self.hour < 21.0 }
    pub fn normalized(&self) -> f32 { self.hour / 24.0 }

    /// Sun elevation angle (radians)
    pub fn sun_angle(&self) -> f32 {
        let t = self.normalized();
        (std::f32::consts::PI * 2.0 * (t - 0.25)).sin() * std::f32::consts::HALF_PI
    }

    pub fn formatted(&self) -> String {
        let h = self.hour as u32;
        let m = ((self.hour - h as f32) * 60.0) as u32;
        format!("{:02}:{:02}, Day {} Month {} Year {}", h, m, self.day, self.month, self.year)
    }
}

// ─── Weather System ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WeatherType {
    Clear, PartlyCloudy, Overcast,
    Drizzle, Rain, HeavyRain, Thunderstorm,
    Snow, Blizzard, Sleet,
    Fog, DenseFog,
    Hail,
    Sandstorm, Duststorm,
    Heatwave,
    Hurricane, Tornado,
    AcidicRain, // for sci-fi / post-apocalyptic
    MagicStorm, // for fantasy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub current: WeatherType,
    pub next: Option<WeatherType>,
    pub transition_progress: f32,  // 0-1
    pub transition_duration: f32,  // seconds
    pub wind_direction: Vec3,
    pub wind_speed: f32,           // m/s
    pub wind_gustiness: f32,       // 0-1
    pub temperature: f32,          // Celsius
    pub humidity: f32,             // 0-1
    pub cloud_coverage: f32,       // 0-1
    pub fog_density: f32,
    pub precipitation: f32,        // 0-1
    pub lightning_strikes: Vec<Vec3>, // active lightning positions
}

impl WeatherState {
    pub fn clear() -> Self {
        Self {
            current: WeatherType::Clear,
            next: None,
            transition_progress: 1.0,
            transition_duration: 0.0,
            wind_direction: Vec3::new(1.0, 0.0, 0.3).normalize(),
            wind_speed: 2.0,
            wind_gustiness: 0.1,
            temperature: 20.0,
            humidity: 0.4,
            cloud_coverage: 0.1,
            fog_density: 0.0,
            precipitation: 0.0,
            lightning_strikes: Vec::new(),
        }
    }

    pub fn is_precipitation(&self) -> bool {
        matches!(self.current,
            WeatherType::Drizzle | WeatherType::Rain | WeatherType::HeavyRain |
            WeatherType::Thunderstorm | WeatherType::Snow | WeatherType::Blizzard |
            WeatherType::Sleet | WeatherType::Hail)
    }

    pub fn transition_to(&mut self, weather: WeatherType, over_secs: f32) {
        self.next = Some(weather);
        self.transition_progress = 0.0;
        self.transition_duration = over_secs;
    }

    pub fn tick(&mut self, delta: f32) {
        if let Some(next) = &self.next {
            self.transition_progress += delta / self.transition_duration;
            if self.transition_progress >= 1.0 {
                self.current = next.clone();
                self.next = None;
                self.transition_progress = 1.0;
            }
        }
    }
}

// ─── Spatial Index ────────────────────────────────────────────────────────────

/// Fast spatial queries for entities in the world
pub struct SpatialIndex {
    /// Grid cell size (meters)
    cell_size: f32,
    /// Map from grid cell → entity IDs in that cell
    cells: HashMap<(i32, i32, i32), Vec<String>>,
    /// Map from entity ID → its current cell
    entity_cells: HashMap<String, (i32, i32, i32)>,
}

impl SpatialIndex {
    pub fn new(cell_size: f32) -> Self {
        Self { cell_size, cells: HashMap::new(), entity_cells: HashMap::new() }
    }

    fn world_to_cell(&self, pos: Vec3) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    pub fn insert(&mut self, entity_id: &str, position: Vec3) {
        let cell = self.world_to_cell(position);
        self.cells.entry(cell).or_default().push(entity_id.to_string());
        self.entity_cells.insert(entity_id.to_string(), cell);
    }

    pub fn update(&mut self, entity_id: &str, new_position: Vec3) {
        if let Some(old_cell) = self.entity_cells.get(entity_id).copied() {
            let new_cell = self.world_to_cell(new_position);
            if old_cell != new_cell {
                if let Some(entities) = self.cells.get_mut(&old_cell) {
                    entities.retain(|id| id != entity_id);
                }
                self.cells.entry(new_cell).or_default().push(entity_id.to_string());
                self.entity_cells.insert(entity_id.to_string(), new_cell);
            }
        } else {
            self.insert(entity_id, new_position);
        }
    }

    pub fn remove(&mut self, entity_id: &str) {
        if let Some(cell) = self.entity_cells.remove(entity_id) {
            if let Some(entities) = self.cells.get_mut(&cell) {
                entities.retain(|id| id != entity_id);
            }
        }
    }

    /// Find all entities within radius of a position
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<&str> {
        let cell_radius = (radius / self.cell_size).ceil() as i32 + 1;
        let center_cell = self.world_to_cell(center);
        let r2 = radius * radius;

        let mut results = Vec::new();
        for dz in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                for dx in -cell_radius..=cell_radius {
                    let cell = (center_cell.0 + dx, center_cell.1 + dy, center_cell.2 + dz);
                    if let Some(entities) = self.cells.get(&cell) {
                        results.extend(entities.iter().map(|s| s.as_str()));
                    }
                }
            }
        }
        results
    }

    pub fn entity_count(&self) -> usize { self.entity_cells.len() }
}

// ─── World Generator ─────────────────────────────────────────────────────────

pub struct WorldGenerator {
    pub seed: u64,
    pub config: WorldGenConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldGenConfig {
    pub name: String,
    pub seed: u64,
    pub world_type: WorldType,
    pub biome_scale: f32,     // how large biomes are
    pub sea_level: f32,
    pub mountain_height: f32,
    pub continent_size: f32,
    pub island_frequency: f32,
    pub river_density: f32,
    pub cave_density: f32,
    pub dungeon_frequency: f32,
    pub city_frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldType {
    Fantasy,      // classic RPG world
    SciFi,        // alien worlds, space stations
    PostApocalyptic,
    Historical { era: String },
    Procedural,   // fully random
    Custom { description: String },
}

impl WorldGenerator {
    pub fn new(config: WorldGenConfig) -> Self {
        let seed = config.seed;
        Self { seed, config }
    }

    /// Get the biome type at a world coordinate
    pub fn biome_at(&self, x: f32, z: f32) -> BiomeType {
        // Simple noise-based biome selection
        let nx = x / (self.config.biome_scale * 1000.0);
        let nz = z / (self.config.biome_scale * 1000.0);
        let t = self.noise2d(nx, nz);
        let m = self.noise2d(nx + 100.0, nz + 100.0);

        match (t, m) {
            (t, _) if t < 0.2 => BiomeType::Tundra,
            (t, m) if t < 0.4 && m < 0.3 => BiomeType::TemperateGrassland,
            (t, m) if t < 0.4 && m >= 0.3 => BiomeType::TemperateForest,
            (t, m) if t < 0.6 && m < 0.2 => BiomeType::Desert,
            (t, m) if t < 0.6 && m >= 0.2 => BiomeType::Savanna,
            (_, m) if m > 0.7 => BiomeType::TropicalRainforest,
            _ => BiomeType::TemperateForest,
        }
    }

    /// Get terrain height at world coordinate
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let n1 = self.noise2d(x * 0.001, z * 0.001) * 100.0;
        let n2 = self.noise2d(x * 0.005, z * 0.005) * 30.0;
        let n3 = self.noise2d(x * 0.02, z * 0.02) * 8.0;
        (n1 + n2 + n3).max(self.config.sea_level)
    }

    /// Generate a chunk's height map
    pub fn generate_heightmap(&self, coord: ChunkCoord) -> Vec<f32> {
        let origin = coord.world_origin();
        let step = CHUNK_SIZE / 128.0;
        let mut heights = Vec::with_capacity(129 * 129);

        for z in 0..=128 {
            for x in 0..=128 {
                let wx = origin.x + x as f32 * step;
                let wz = origin.z + z as f32 * step;
                heights.push(self.height_at(wx, wz));
            }
        }
        heights
    }

    /// Simple value noise
    fn noise2d(&self, x: f32, z: f32) -> f32 {
        let xi = x.floor() as i64;
        let zi = z.floor() as i64;
        let fx = x - xi as f32;
        let fz = z - zi as f32;

        let v00 = self.hash2d(xi, zi);
        let v10 = self.hash2d(xi + 1, zi);
        let v01 = self.hash2d(xi, zi + 1);
        let v11 = self.hash2d(xi + 1, zi + 1);

        let sx = fx * fx * (3.0 - 2.0 * fx);
        let sz = fz * fz * (3.0 - 2.0 * fz);

        let t0 = v00 + sx * (v10 - v00);
        let t1 = v01 + sx * (v11 - v01);
        t0 + sz * (t1 - t0)
    }

    fn hash2d(&self, x: i64, z: i64) -> f32 {
        let h = x.wrapping_mul(374761393)
            ^ z.wrapping_mul(668265263)
            ^ self.seed as i64;
        let h = h.wrapping_mul(1274126177);
        ((h & 0x7FFFFFFF) as f32 / 0x7FFFFFFF as f32)
    }
}

pub mod chunk { pub use super::{WorldChunk, ChunkCoord, ChunkState, CHUNK_SIZE}; }
pub mod biome { pub use super::{BiomeType, BiomeConfig}; }
pub mod navmesh { pub struct NavMesh; }
pub mod time_of_day { pub use super::WorldTime; }
pub mod weather { pub use super::{WeatherType, WeatherState}; }
pub mod spatial_index { pub use super::SpatialIndex; }
pub mod persistence { pub struct WorldPersistence; }
pub mod generator { pub use super::{WorldGenerator, WorldGenConfig, WorldType}; }
pub mod events { pub struct WorldEventSystem; }
