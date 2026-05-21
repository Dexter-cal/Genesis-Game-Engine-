//! Genesis Mechanics System
//!
//! Pure game mechanics that don't fit neatly elsewhere:
//! - Gear systems (interlocking gears, ratio calculations)
//! - Lever mechanics (fulcrum, load, effort)
//! - Pulley systems (single, compound, movable)
//! - Rope/chain physics (catenary, dynamic hanging)
//! - Pressure plates and weight triggers
//! - Lock & key mechanisms (tumbler, combination)
//! - Clock/timer mechanisms
//! - Hydraulics (pistons, fluid pressure)
//! - Electrical systems (circuits, switches, power)
//! - Conveyor belts
//! - Escalators and moving platforms
//! - Portals and teleporters
//! - Anti-gravity zones
//! - Sticky surfaces
//! - Ice physics (friction reduction)
//! - Bounce pads / jump pads
//! - Wind tunnels (directional force zones)
//! - Damage zones (lava, acid, electric)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use genesis_math::vec3::Vec3;

// ─── Gear System ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gear {
    pub id: String,
    pub entity_id: String,
    pub teeth: u32,
    pub radius: f32,
    pub current_angle_degrees: f32,
    pub angular_velocity: f32,  // degrees/second
    pub connected_gears: Vec<GearConnection>,
    pub locked: bool,
    pub drive_source: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearConnection {
    pub target_gear_id: String,
    pub mesh_type: GearMeshType,
    pub ratio_override: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GearMeshType {
    External, // counter-rotation (standard)
    Internal, // co-rotation (ring gear)
    Bevel,    // 90-degree axis change
    Worm,     // large ratio reduction, non-reversible
}

impl Gear {
    pub fn gear_ratio_to(&self, other: &Gear) -> f32 {
        self.teeth as f32 / other.teeth as f32
    }

    pub fn tick(&mut self, delta: f32) {
        if !self.locked {
            self.current_angle_degrees += self.angular_velocity * delta;
            self.current_angle_degrees %= 360.0;
        }
    }
}

// ─── Lever System ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lever {
    pub id: String,
    pub entity_id: String,
    pub fulcrum_position: Vec3,
    pub effort_arm_length: f32,
    pub load_arm_length: f32,
    pub current_angle: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub connected_mechanism: Option<String>,
    pub locked: bool,
}

impl Lever {
    /// Mechanical advantage: effort_arm / load_arm
    pub fn mechanical_advantage(&self) -> f32 {
        self.load_arm_length / self.effort_arm_length
    }

    /// How much load force can be lifted with given effort force?
    pub fn output_force(&self, effort_force: f32) -> f32 {
        effort_force * self.mechanical_advantage()
    }

    /// Get normalized position (0 = min angle, 1 = max angle)
    pub fn normalized_position(&self) -> f32 {
        (self.current_angle - self.min_angle) / (self.max_angle - self.min_angle)
    }

    pub fn is_at_min(&self) -> bool { self.current_angle <= self.min_angle + 0.01 }
    pub fn is_at_max(&self) -> bool { self.current_angle >= self.max_angle - 0.01 }
}

// ─── Pulley System ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulleySystem {
    pub id: String,
    pub pulley_type: PulleyType,
    pub pulleys: Vec<PulleyNode>,
    pub rope_length: f32,
    pub load_mass: f32,
    pub current_load_height: f32,
    pub min_height: f32,
    pub max_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PulleyType {
    Fixed,    // single fixed pulley (changes direction, no MA)
    Movable,  // single movable pulley (2:1 MA)
    Compound, // fixed + movable (2:1 MA)
    BlockAndTackle { strands: u32 }, // MA = strands
}

impl PulleyType {
    pub fn mechanical_advantage(&self) -> f32 {
        match self {
            Self::Fixed => 1.0,
            Self::Movable => 2.0,
            Self::Compound => 2.0,
            Self::BlockAndTackle { strands } => *strands as f32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulleyNode {
    pub position: Vec3,
    pub radius: f32,
    pub fixed: bool,
}

// ─── Electrical System ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectricalNode {
    pub id: String,
    pub entity_id: String,
    pub node_type: ElectricalNodeType,
    pub power: f32,        // watts
    pub voltage: f32,
    pub connections: Vec<String>,
    pub active: bool,
    pub can_short_circuit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElectricalNodeType {
    PowerSource { max_power: f32 },
    Switch { open: bool },
    Light { lux: f32 },
    Motor { torque: f32, rpm: f32 },
    Sensor { sensor_type: String },
    Battery { charge: f32, capacity: f32 },
    Wire,
    Capacitor { capacitance: f32, charge: f32 },
    Resistor { resistance: f32 },
    Fuse { max_amperage: f32, blown: bool },
}

/// A simple circuit simulation
pub struct ElectricalCircuit {
    pub nodes: HashMap<String, ElectricalNode>,
    pub total_power_draw: f32,
    pub short_circuited: bool,
}

impl ElectricalCircuit {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), total_power_draw: 0.0, short_circuited: false }
    }

    pub fn add_node(&mut self, node: ElectricalNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn flip_switch(&mut self, switch_id: &str) {
        if let Some(node) = self.nodes.get_mut(switch_id) {
            if let ElectricalNodeType::Switch { open } = &mut node.node_type {
                *open = !*open;
            }
        }
        self.recalculate();
    }

    fn recalculate(&mut self) {
        self.total_power_draw = 0.0;
        self.short_circuited = false;
        for node in self.nodes.values() {
            if node.active {
                self.total_power_draw += node.power;
            }
        }
    }
}

// ─── Special Zones ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanicsZone {
    pub id: String,
    pub entity_id: String,
    pub zone_type: MechanicsZoneType,
    pub shape: ZoneShape,
    pub active: bool,
    pub entities_inside: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MechanicsZoneType {
    /// Reverse gravity direction
    AntiGravity { gravity_scale: f32, gravity_dir: Vec3 },
    /// Sticky — objects stick to surfaces in this zone
    Sticky { friction: f32 },
    /// Ice — very low friction
    Ice { friction: f32 },
    /// Jump pad — launch entities upward
    JumpPad { force: f32, direction: Vec3 },
    /// Wind tunnel — push entities in a direction
    WindTunnel { direction: Vec3, strength: f32 },
    /// Damage zone — continuous damage to anything inside
    DamageZone { damage_per_second: f32, damage_type: String, visual_effect: String },
    /// Slow zone — time dilation for entities inside
    SlowField { time_scale: f32 },
    /// Teleporter — move entities to a destination
    Teleporter { destination: Vec3, destination_rotation: Option<[f32; 4]> },
    /// Portal — linked to another portal
    Portal { linked_portal_id: String },
    /// Silence zone — no audio inside
    SilenceZone,
    /// No-clip zone — entities pass through geometry
    NoClip,
    /// Force field — blocks certain entity types
    ForceField { blocks: Vec<String>, color: genesis_math::color::Color },
    /// Healing zone — heal entities inside
    HealZone { heal_per_second: f32 },
    /// Pressure zone — crushes entities below threshold
    PressureZone { crush_threshold: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZoneShape {
    Sphere { center: Vec3, radius: f32 },
    Box { center: Vec3, half_extents: Vec3 },
    Capsule { base: Vec3, tip: Vec3, radius: f32 },
    Cylinder { center: Vec3, radius: f32, height: f32 },
    Custom { mesh_id: String },
}

impl ZoneShape {
    pub fn contains(&self, point: Vec3) -> bool {
        match self {
            Self::Sphere { center, radius } =>
                point.distance(*center) <= *radius,
            Self::Box { center, half_extents } => {
                let d = point - *center;
                d.x.abs() <= half_extents.x && d.y.abs() <= half_extents.y && d.z.abs() <= half_extents.z
            }
            Self::Cylinder { center, radius, height } => {
                let dx = point.x - center.x;
                let dz = point.z - center.z;
                let dy = (point.y - center.y).abs();
                (dx*dx + dz*dz).sqrt() <= *radius && dy <= height * 0.5
            }
            _ => false,
        }
    }
}

// ─── Conveyor Belt ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConveyorBelt {
    pub id: String,
    pub entity_id: String,
    pub direction: Vec3,
    pub speed: f32,         // m/s
    pub active: bool,
    pub belt_material: String,
}

// ─── Rope / Chain ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RopeConfig {
    pub id: String,
    pub start_entity: String,
    pub end_entity: Option<String>,
    pub length: f32,
    pub segments: u32,
    pub stiffness: f32,
    pub damping: f32,
    pub mass_per_meter: f32,
    pub breakable: bool,
    pub break_force: f32,
    pub current_tension: f32,
    pub broken: bool,
    pub rope_type: RopeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RopeType {
    Rope, Chain, Cable, Vine, Wire, Tentacle,
}

// ─── Lock & Key ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockMechanism {
    pub id: String,
    pub entity_id: String,
    pub lock_type: LockType,
    pub locked: bool,
    pub required_item: Option<String>,
    pub combination: Option<Vec<u32>>,
    pub current_input: Vec<u32>,
    pub attempts: u32,
    pub max_attempts: Option<u32>,
    pub cooldown: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockType {
    KeyLock { key_id: String },
    CombinationLock { digits: u32 },
    Puzzle { solution: serde_json::Value },
    Biometric { entity_ids: Vec<String> },
    TimedLock { unlock_time: f32, lock_time: f32 },
    MagicLock { spell_id: String },
}

impl LockMechanism {
    pub fn try_key(&mut self, item_id: &str) -> bool {
        if let LockType::KeyLock { key_id } = &self.lock_type {
            if key_id == item_id {
                self.locked = false;
                return true;
            }
        }
        false
    }

    pub fn try_combination(&mut self, input: Vec<u32>) -> bool {
        if let Some(combo) = &self.combination {
            if &input == combo {
                self.locked = false;
                return true;
            }
            self.attempts += 1;
        }
        false
    }
}

// ─── Mechanics Manager ────────────────────────────────────────────────────────

pub struct MechanicsManager {
    pub gears: HashMap<String, Gear>,
    pub levers: HashMap<String, Lever>,
    pub pulleys: HashMap<String, PulleySystem>,
    pub zones: HashMap<String, MechanicsZone>,
    pub circuits: HashMap<String, ElectricalCircuit>,
    pub ropes: HashMap<String, RopeConfig>,
    pub locks: HashMap<String, LockMechanism>,
    pub conveyors: HashMap<String, ConveyorBelt>,
}

impl MechanicsManager {
    pub fn new() -> Self {
        Self {
            gears: HashMap::new(),
            levers: HashMap::new(),
            pulleys: HashMap::new(),
            zones: HashMap::new(),
            circuits: HashMap::new(),
            ropes: HashMap::new(),
            locks: HashMap::new(),
            conveyors: HashMap::new(),
        }
    }

    pub fn tick(&mut self, delta: f32) {
        // Tick all gears
        for gear in self.gears.values_mut() {
            gear.tick(delta);
        }

        // Propagate gear connections
        let gear_ids: Vec<String> = self.gears.keys().cloned().collect();
        for id in &gear_ids {
            let connections: Vec<GearConnection> = self.gears[id].connections.clone();
            let source_velocity = self.gears[id].angular_velocity;
            let source_teeth = self.gears[id].teeth;

            for conn in &connections {
                if let Some(target) = self.gears.get_mut(&conn.target_gear_id) {
                    let ratio = conn.ratio_override.unwrap_or(source_teeth as f32 / target.teeth as f32);
                    let direction = match conn.mesh_type {
                        GearMeshType::External => -1.0,
                        _ => 1.0,
                    };
                    target.angular_velocity = source_velocity * ratio * direction;
                }
            }
        }
    }

    /// Check if any entity is inside a zone this frame
    pub fn check_zone(&self, zone_id: &str, entity_pos: Vec3) -> bool {
        self.zones.get(zone_id)
            .map(|z| z.active && z.shape.contains(entity_pos))
            .unwrap_or(false)
    }

    /// Get the force applied to an entity in any zones it overlaps
    pub fn get_zone_forces(&self, entity_pos: Vec3, entity_id: &str) -> Vec3 {
        let mut total = Vec3::ZERO;
        for zone in self.zones.values() {
            if !zone.active || !zone.shape.contains(entity_pos) { continue; }
            match &zone.zone_type {
                MechanicsZoneType::JumpPad { force, direction } =>
                    total += *direction * *force,
                MechanicsZoneType::WindTunnel { direction, strength } =>
                    total += *direction * *strength,
                MechanicsZoneType::AntiGravity { gravity_scale, gravity_dir } =>
                    total += *gravity_dir * 9.81 * (1.0 - gravity_scale),
                _ => {}
            }
        }
        total
    }
}
