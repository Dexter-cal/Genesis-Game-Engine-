//! Component trait and type registry.
//!
//! Every piece of game data is a Component. Components are plain data —
//! no logic. Systems read and write components.

use std::any::{Any, TypeId};
use std::fmt;
use serde::{Serialize, Deserialize};

/// Unique ID for a component type (based on TypeId)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentId(pub u64);

impl ComponentId {
    pub fn of<T: Component>() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// Marker trait for all ECS components
pub trait Component: Send + Sync + 'static {
    fn component_id() -> ComponentId where Self: Sized {
        ComponentId::of::<Self>()
    }
    fn name() -> &'static str where Self: Sized {
        std::any::type_name::<Self>()
    }
}

// ─── Built-in components ─────────────────────────────────────────────────────

/// World-space position, rotation, scale
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Component for Position {}
impl Default for Position {
    fn default() -> Self { Self { x: 0.0, y: 0.0, z: 0.0 } }
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx*dx + dy*dy + dz*dz).sqrt()
    }
    pub fn to_array(&self) -> [f32; 3] { [self.x, self.y, self.z] }
}

/// Rotation as quaternion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Component for Rotation {}
impl Default for Rotation {
    fn default() -> Self { Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 } }
}

/// Non-uniform scale
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Component for Scale {}
impl Default for Scale {
    fn default() -> Self { Self { x: 1.0, y: 1.0, z: 1.0 } }
}

/// Linear velocity (physics-driven or manually set)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Component for Velocity {}

impl Velocity {
    pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn magnitude(&self) -> f32 {
        (self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
    }
    pub fn is_nearly_zero(&self) -> bool { self.magnitude() < 0.001 }
}

/// Angular velocity
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AngularVelocity { pub x: f32, pub y: f32, pub z: f32 }
impl Component for AngularVelocity {}

/// Health component — works for players, NPCs, destructibles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub regen_per_second: f32,
    pub invincible: bool,
}

impl Component for Health {}
impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max, regen_per_second: 0.0, invincible: false }
    }
    pub fn percent(&self) -> f32 { self.current / self.max }
    pub fn is_alive(&self) -> bool { self.current > 0.0 }
    pub fn is_full(&self) -> bool { self.current >= self.max }
    pub fn apply_damage(&mut self, amount: f32) -> f32 {
        if self.invincible { return 0.0; }
        let actual = amount.min(self.current);
        self.current -= actual;
        actual
    }
    pub fn heal(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.max - self.current);
        self.current += actual;
        actual
    }
    pub fn tick_regen(&mut self, delta: f32) {
        if self.regen_per_second > 0.0 && !self.is_full() {
            self.heal(self.regen_per_second * delta);
        }
    }
}

/// Stamina — used for attacks, sprinting, blocking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub regen_per_second: f32,
    pub regen_delay: f32,          // seconds after action before regen starts
    pub regen_delay_remaining: f32,
}

impl Component for Stamina {}
impl Stamina {
    pub fn new(max: f32) -> Self {
        Self { current: max, max, regen_per_second: 15.0, regen_delay: 1.0, regen_delay_remaining: 0.0 }
    }
    pub fn can_afford(&self, cost: f32) -> bool { self.current >= cost }
    pub fn spend(&mut self, cost: f32) -> bool {
        if self.can_afford(cost) {
            self.current -= cost;
            self.regen_delay_remaining = self.regen_delay;
            true
        } else { false }
    }
    pub fn tick(&mut self, delta: f32) {
        if self.regen_delay_remaining > 0.0 {
            self.regen_delay_remaining -= delta;
        } else if self.current < self.max {
            self.current = (self.current + self.regen_per_second * delta).min(self.max);
        }
    }
    pub fn percent(&self) -> f32 { self.current / self.max }
}

/// Mana/energy resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mana {
    pub current: f32,
    pub max: f32,
    pub regen_per_second: f32,
}
impl Component for Mana {}
impl Mana {
    pub fn new(max: f32) -> Self { Self { current: max, max, regen_per_second: 5.0 } }
    pub fn tick(&mut self, delta: f32) {
        self.current = (self.current + self.regen_per_second * delta).min(self.max);
    }
    pub fn spend(&mut self, cost: f32) -> bool {
        if self.current >= cost { self.current -= cost; true } else { false }
    }
}

/// Name label for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);
impl Component for Name {}
impl Name {
    pub fn new(s: &str) -> Self { Self(s.to_string()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Tags — multiple string tags for filtering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tags(pub Vec<String>);
impl Component for Tags {}
impl Tags {
    pub fn has(&self, tag: &str) -> bool { self.0.iter().any(|t| t == tag) }
    pub fn add(&mut self, tag: &str) {
        if !self.has(tag) { self.0.push(tag.to_string()); }
    }
    pub fn remove(&mut self, tag: &str) { self.0.retain(|t| t != tag); }
}

/// Marks entity as active/inactive (inactive = skip most systems)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Active(pub bool);
impl Component for Active {}

/// Marks entity as static (never moves — physics optimization)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Static;
impl Component for Static {}

/// Layer bitmask (for collision layers, render layers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer(pub u32);
impl Component for Layer {}
impl Default for Layer {
    fn default() -> Self { Self(1) }
}

/// The entity's scene/chunk location
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneMembership {
    pub scene_id: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
}
impl Component for SceneMembership {}

/// Mesh reference for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRef(pub String); // asset ID
impl Component for MeshRef {}

/// Material references for rendering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialRefs(pub Vec<String>);
impl Component for MaterialRefs {}

/// Camera component — entities with this get rendered from their POV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub fov_degrees: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub is_active: bool,
    pub camera_mode: CameraMode,
}

impl Component for Camera {}
impl Default for Camera {
    fn default() -> Self {
        Self {
            fov_degrees: 75.0,
            near_clip: 0.1,
            far_clip: 1000.0,
            is_active: false,
            camera_mode: CameraMode::ThirdPerson { distance: 5.0, pitch: -20.0 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson { distance: f32, pitch: f32 },
    TopDown { height: f32 },
    Isometric { angle: f32, zoom: f32 },
    SideScroll { z_locked: f32 },
    OverShoulder { offset: [f32; 3] },
    Cinematic,
    Vr,
    Free,
}

/// Light source component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub light_type: LightType,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub cast_shadows: bool,
    pub enabled: bool,
}

impl Component for Light {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Point,
    Directional { direction: [f32; 3] },
    Spot { direction: [f32; 3], inner_angle: f32, outer_angle: f32 },
    Area { width: f32, height: f32 },
}

/// Audio source attached to an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSource {
    pub clip_id: Option<String>,
    pub volume: f32,
    pub pitch: f32,
    pub loop_audio: bool,
    pub spatial: bool,
    pub min_distance: f32,
    pub max_distance: f32,
    pub playing: bool,
    pub auto_play: bool,
}

impl Component for AudioSource {}
impl Default for AudioSource {
    fn default() -> Self {
        Self {
            clip_id: None, volume: 1.0, pitch: 1.0,
            loop_audio: false, spatial: true,
            min_distance: 1.0, max_distance: 50.0,
            playing: false, auto_play: false,
        }
    }
}

/// Particle system component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystem {
    pub effect_id: String,
    pub playing: bool,
    pub looping: bool,
    pub auto_play: bool,
    pub duration: f32,
    pub elapsed: f32,
}
impl Component for ParticleSystem {}

/// Animation state machine component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animator {
    pub controller_id: String,
    pub current_state: String,
    pub parameters: std::collections::HashMap<String, f32>,
    pub speed: f32,
    pub enabled: bool,
}
impl Component for Animator {}

/// Navigation agent (pathfinding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavAgent {
    pub speed: f32,
    pub angular_speed: f32,
    pub acceleration: f32,
    pub stopping_distance: f32,
    pub radius: f32,
    pub height: f32,
    pub current_path: Vec<[f32; 3]>,
    pub destination: Option<[f32; 3]>,
    pub is_moving: bool,
    pub agent_type: NavAgentType,
}

impl Component for NavAgent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavAgentType {
    Humanoid,
    Quadruped,
    Flying,
    Swimming,
}

/// Interaction target — this entity can be interacted with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interactable {
    pub enabled: bool,
    pub interaction_radius: f32,
    pub prompt_text: String,
    pub interaction_points: Vec<InteractionPoint>,
    pub max_concurrent_users: u32,
    pub current_users: u32,
}

impl Component for Interactable {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPoint {
    pub id: String,
    pub offset: [f32; 3],
    pub approach_direction: [f32; 3],
    pub actions: Vec<String>,
    pub occupied: bool,
}

/// Inventory component — entities that hold items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<InventorySlot>,
    pub max_weight: f32,
    pub current_weight: f32,
    pub gold: u64,
}

impl Component for Inventory {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item_id: String,
    pub quantity: u32,
    pub slot_index: u32,
    pub equipped: bool,
    pub equipped_slot: Option<String>,
}

/// Equipment slots
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub head: Option<String>,
    pub chest: Option<String>,
    pub legs: Option<String>,
    pub feet: Option<String>,
    pub hands: Option<String>,
    pub main_hand: Option<String>,
    pub off_hand: Option<String>,
    pub ring_left: Option<String>,
    pub ring_right: Option<String>,
    pub amulet: Option<String>,
}
impl Component for Equipment {}

/// Character stats for RPG systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub strength: f32,
    pub dexterity: f32,
    pub intelligence: f32,
    pub endurance: f32,
    pub charisma: f32,
    pub perception: f32,
    pub level: u32,
    pub experience: u64,
    pub experience_to_next: u64,
    pub skill_points: u32,
    pub attribute_points: u32,
}

impl Component for Stats {}
impl Default for Stats {
    fn default() -> Self {
        Self {
            strength: 10.0, dexterity: 10.0, intelligence: 10.0,
            endurance: 10.0, charisma: 10.0, perception: 10.0,
            level: 1, experience: 0, experience_to_next: 100,
            skill_points: 0, attribute_points: 0,
        }
    }
}

/// Faction membership
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Faction {
    pub faction_id: String,
    pub reputation: std::collections::HashMap<String, f32>,
}
impl Component for Faction {}

/// Combat state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CombatState {
    pub in_combat: bool,
    pub combat_target: Option<String>,
    pub last_attacker: Option<String>,
    pub combo_count: u32,
    pub last_hit_time: f32,
    pub stagger_remaining: f32,
    pub block_active: bool,
    pub dodge_invincible: bool,
    pub attack_cooldown: f32,
}
impl Component for CombatState {}

/// Status effects on an entity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusEffects {
    pub effects: Vec<ActiveEffect>,
}
impl Component for StatusEffects {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub effect_id: String,
    pub remaining_duration: f32,
    pub stacks: u32,
    pub source_entity: Option<String>,
}

/// Marker: this entity is the player
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Player {
    pub player_id: u8, // 0 = single player, 1-4 = co-op
}
impl Component for Player {}

/// Marker: this entity is an NPC controlled by AI
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NpcMarker {
    pub npc_id: String,
    pub npc_type: String, // "humanoid", "creature", "boss", "merchant"
}
impl Component for NpcMarker {}

/// Child-parent relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parent(pub String); // parent entity ID
impl Component for Parent {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Children(pub Vec<String>);
impl Component for Children {}

/// Zone/trigger volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerZone {
    pub zone_id: String,
    pub zone_type: String,
    pub entities_inside: Vec<String>,
}
impl Component for TriggerZone {}

/// Waypoint for patrol routes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub waypoint_id: String,
    pub next_waypoint: Option<String>,
    pub wait_time: f32,
}
impl Component for Waypoint {}

/// Visibility range for NPCs (sight/hearing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perception {
    pub sight_range: f32,
    pub sight_fov_degrees: f32,
    pub hearing_range: f32,
    pub awareness_level: AwarenessLevel,
    pub awareness_meter: f32, // 0.0 to 1.0
    pub last_known_player_pos: Option<[f32; 3]>,
}
impl Component for Perception {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AwarenessLevel {
    Unaware,
    Suspicious,
    Alerted,
    Combat,
}

/// Bounty / wanted level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wanted {
    pub bounty: u64,
    pub crimes: Vec<String>,
    pub wanted_in: Vec<String>, // faction IDs
}
impl Component for Wanted {}

/// Reputation with all factions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Reputation {
    pub values: std::collections::HashMap<String, f32>,
}
impl Component for Reputation {}

impl Reputation {
    pub fn get(&self, faction: &str) -> f32 {
        *self.values.get(faction).unwrap_or(&0.0)
    }
    pub fn modify(&mut self, faction: &str, delta: f32) {
        let entry = self.values.entry(faction.to_string()).or_insert(0.0);
        *entry = (*entry + delta).clamp(-1000.0, 1000.0);
    }
}
