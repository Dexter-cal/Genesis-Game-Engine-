//! Physics — Rapier3D integration, collision, joints, wind, cloth basics
use serde::{Serialize,Deserialize};
use std::collections::HashMap;

// ═══ RIGID BODY ═══════════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum RbKind { Dynamic, Static, KinematicPos, KinematicVel }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RigidBody {
    pub id:            String,
    pub kind:          RbKind,
    pub position:      [f32;3],
    pub rotation:      [f32;4],
    pub linear_vel:    [f32;3],
    pub angular_vel:   [f32;3],
    pub mass:          f32,
    pub gravity_scale: f32,
    pub linear_damping: f32,
    pub angular_damping:f32,
    pub can_sleep:     bool,
    pub sleeping:      bool,
    pub ccd:           bool,          // continuous collision detection
    pub dominance:     i8,
    pub colliders:     Vec<String>,
    pub entity_id:     String,
}

impl RigidBody {
    pub fn dynamic(id:&str, entity_id:&str, mass:f32, pos:[f32;3]) -> Self {
        Self { id:id.to_string(), kind:RbKind::Dynamic, position:pos, rotation:[0.0,0.0,0.0,1.0],
               linear_vel:[0.0;3], angular_vel:[0.0;3], mass, gravity_scale:1.0,
               linear_damping:0.0, angular_damping:0.0, can_sleep:true, sleeping:false,
               ccd:false, dominance:0, colliders:Vec::new(), entity_id:entity_id.to_string() }
    }
    pub fn static_(id:&str, entity_id:&str, pos:[f32;3]) -> Self {
        let mut rb = Self::dynamic(id, entity_id, f32::MAX, pos);
        rb.kind = RbKind::Static;
        rb
    }
    pub fn speed(&self) -> f32 {
        let v = self.linear_vel;
        (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt()
    }
    pub fn is_moving(&self) -> bool { self.speed() > 0.01 }
}

// ═══ COLLIDERS ═══════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ColliderShape {
    Ball      { radius:f32 },
    Cuboid    { half_extents:[f32;3] },
    Capsule   { half_height:f32, radius:f32 },
    Cylinder  { half_height:f32, radius:f32 },
    Cone      { half_height:f32, radius:f32 },
    Heightfield{ width:u32, height:u32, data:Vec<f32>, scale:[f32;3] },
    TriMesh   { vertices:Vec<[f32;3]>, indices:Vec<[u32;3]> },
    Convex    { points:Vec<[f32;3]> },
    Compound  { shapes:Vec<(ColliderShape,[f32;3],[f32;4])> },
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Collider {
    pub id:            String,
    pub rb_id:         Option<String>,
    pub shape:         ColliderShape,
    pub offset:        [f32;3],
    pub rotation:      [f32;4],
    pub friction:      f32,
    pub restitution:   f32,
    pub density:       f32,
    pub sensor:        bool,
    pub collision_groups: u32,
    pub solver_groups:    u32,
    pub active_events: bool,
}

impl Collider {
    pub fn ball(id:&str, radius:f32) -> Self {
        Self { id:id.to_string(), rb_id:None, shape:ColliderShape::Ball{radius},
               offset:[0.0;3], rotation:[0.0,0.0,0.0,1.0], friction:0.5, restitution:0.0,
               density:1.0, sensor:false, collision_groups:0xFFFF, solver_groups:0xFFFF, active_events:false }
    }
    pub fn cuboid(id:&str, he:[f32;3]) -> Self {
        let mut c = Self::ball(id, 1.0);
        c.shape = ColliderShape::Cuboid{half_extents:he};
        c
    }
    pub fn capsule(id:&str, hh:f32, r:f32) -> Self {
        let mut c = Self::ball(id, 1.0);
        c.shape = ColliderShape::Capsule{half_height:hh, radius:r};
        c
    }
    pub fn as_sensor(mut self) -> Self { self.sensor = true; self }
}

// ═══ COLLISION EVENT ═════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CollisionEvent {
    pub collider_a: String,
    pub collider_b: String,
    pub entity_a:   String,
    pub entity_b:   String,
    pub kind:       CollisionKind,
    pub point:      [f32;3],
    pub normal:     [f32;3],
    pub depth:      f32,
    pub impulse:    f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CollisionKind { Started, Persisting, Stopped }

// ═══ JOINTS ══════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Joint {
    pub id:     String,
    pub body_a: String,
    pub body_b: String,
    pub kind:   JointKind,
    pub anchor_a:[f32;3],
    pub anchor_b:[f32;3],
    pub break_force: Option<f32>,
    pub break_torque:Option<f32>,
    pub broken:  bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum JointKind {
    Fixed,
    Ball  { limits:[f32;2] },
    Revolute { axis:[f32;3], limits:[f32;2], motor:Option<Motor> },
    Prismatic{ axis:[f32;3], limits:[f32;2], motor:Option<Motor> },
    Rope  { max_dist:f32 },
    Spring{ rest_length:f32, stiffness:f32, damping:f32 },
    Custom{ params:serde_json::Value },
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Motor { pub target_vel:f32, pub max_force:f32 }

// ═══ WIND / ENVIRONMENTAL ════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WindConfig {
    pub enabled:     bool,
    pub direction:   [f32;3],
    pub speed:       f32,
    pub turbulence:  f32,
    pub gust_freq:   f32,
    pub gust_strength:f32,
}
impl Default for WindConfig {
    fn default() -> Self {
        Self { enabled:false, direction:[1.0,0.0,0.0], speed:3.0, turbulence:0.3, gust_freq:0.5, gust_strength:2.0 }
    }
}

// ═══ PHYSICS WORLD ═══════════════════════════════════════════════
pub struct PhysicsWorld {
    pub bodies:         HashMap<String,RigidBody>,
    pub colliders:      HashMap<String,Collider>,
    pub joints:         HashMap<String,Joint>,
    pub gravity:        [f32;3],
    pub wind:           WindConfig,
    pub collision_events: Vec<CollisionEvent>,
    pub substeps:       u32,
    pub time_scale:     f32,
    pub max_velocity:   f32,
    pub sleep_threshold_linear:  f32,
    pub sleep_threshold_angular: f32,
    pub total_steps:    u64,
    pub broad_phase:    BroadPhase,
    pub enabled:        bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum BroadPhase { Bvh, SweepAndPrune, Grid }

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies:HashMap::new(), colliders:HashMap::new(), joints:HashMap::new(),
            gravity:[0.0,-9.81,0.0], wind:WindConfig::default(),
            collision_events:Vec::new(), substeps:4, time_scale:1.0,
            max_velocity:100.0, sleep_threshold_linear:0.01,
            sleep_threshold_angular:0.01, total_steps:0,
            broad_phase:BroadPhase::Bvh, enabled:true,
        }
    }

    pub fn add_body(&mut self, rb:RigidBody) -> String {
        let id = rb.id.clone();
        self.bodies.insert(id.clone(), rb);
        id
    }

    pub fn add_collider(&mut self, mut c:Collider, body_id:Option<&str>) -> String {
        let id = c.id.clone();
        c.rb_id = body_id.map(|s|s.to_string());
        if let Some(bid) = body_id {
            if let Some(rb) = self.bodies.get_mut(bid) {
                rb.colliders.push(id.clone());
            }
        }
        self.colliders.insert(id.clone(), c);
        id
    }

    pub fn add_joint(&mut self, j:Joint) -> String {
        let id = j.id.clone();
        self.joints.insert(id.clone(), j);
        id
    }

    pub fn apply_force(&mut self, body_id:&str, force:[f32;3]) {
        if let Some(rb) = self.bodies.get_mut(body_id) {
            if matches!(rb.kind, RbKind::Dynamic) {
                let inv_mass = if rb.mass > 0.0 { 1.0/rb.mass } else { 0.0 };
                rb.linear_vel[0] += force[0]*inv_mass*0.016;
                rb.linear_vel[1] += force[1]*inv_mass*0.016;
                rb.linear_vel[2] += force[2]*inv_mass*0.016;
            }
        }
    }

    pub fn apply_impulse(&mut self, body_id:&str, impulse:[f32;3]) {
        if let Some(rb) = self.bodies.get_mut(body_id) {
            if matches!(rb.kind, RbKind::Dynamic) {
                let inv_mass = if rb.mass > 0.0 { 1.0/rb.mass } else { 0.0 };
                rb.linear_vel[0] += impulse[0]*inv_mass;
                rb.linear_vel[1] += impulse[1]*inv_mass;
                rb.linear_vel[2] += impulse[2]*inv_mass;
                // Wake if sleeping
                rb.sleeping = false;
            }
        }
    }

    pub fn teleport(&mut self, body_id:&str, pos:[f32;3]) {
        if let Some(rb) = self.bodies.get_mut(body_id) {
            rb.position = pos;
            rb.linear_vel  = [0.0;3];
            rb.angular_vel = [0.0;3];
            rb.sleeping = false;
        }
    }

    pub fn step(&mut self, delta:f32) {
        if !self.enabled { return; }
        let dt = delta * self.time_scale / self.substeps as f32;
        for _ in 0..self.substeps {
            self.integrate(dt);
        }
        self.total_steps += 1;
        self.collision_events.clear();
    }

    fn integrate(&mut self, dt:f32) {
        let g = self.gravity;
        for rb in self.bodies.values_mut() {
            if !matches!(rb.kind, RbKind::Dynamic) { continue; }
            if rb.sleeping { continue; }

            // Gravity
            rb.linear_vel[0] += g[0] * rb.gravity_scale * dt;
            rb.linear_vel[1] += g[1] * rb.gravity_scale * dt;
            rb.linear_vel[2] += g[2] * rb.gravity_scale * dt;

            // Damping
            let ld = (1.0 - rb.linear_damping * dt).max(0.0);
            let ad = (1.0 - rb.angular_damping * dt).max(0.0);
            rb.linear_vel[0] *= ld; rb.linear_vel[1] *= ld; rb.linear_vel[2] *= ld;
            rb.angular_vel[0] *= ad; rb.angular_vel[1] *= ad; rb.angular_vel[2] *= ad;

            // Clamp velocity
            let spd = rb.speed();
            if spd > self.max_velocity {
                let f = self.max_velocity/spd;
                rb.linear_vel[0] *= f; rb.linear_vel[1] *= f; rb.linear_vel[2] *= f;
            }

            // Integrate position
            rb.position[0] += rb.linear_vel[0] * dt;
            rb.position[1] += rb.linear_vel[1] * dt;
            rb.position[2] += rb.linear_vel[2] * dt;

            // Floor plane at y=0 (placeholder until real collision detection)
            if rb.position[1] < 0.0 {
                rb.position[1] = 0.0;
                rb.linear_vel[1] = -rb.linear_vel[1] * 0.3;
                if rb.linear_vel[1].abs() < 0.1 { rb.linear_vel[1] = 0.0; }
            }

            // Sleep check
            let still = rb.speed() < self.sleep_threshold_linear;
            let ang_still = (rb.angular_vel[0].powi(2)+rb.angular_vel[1].powi(2)+rb.angular_vel[2].powi(2)).sqrt() < self.sleep_threshold_angular;
            if still && ang_still && rb.can_sleep { rb.sleeping = true; }
        }
    }

    pub fn body_count(&self) -> usize { self.bodies.len() }
    pub fn collider_count(&self) -> usize { self.colliders.len() }
    pub fn joint_count(&self) -> usize { self.joints.len() }
    pub fn sleeping_count(&self) -> usize { self.bodies.values().filter(|rb|rb.sleeping).count() }
    pub fn get_position(&self, id:&str) -> Option<[f32;3]> { self.bodies.get(id).map(|rb|rb.position) }
    pub fn set_gravity(&mut self, g:[f32;3]) { self.gravity = g; }
    pub fn remove_body(&mut self, id:&str) { self.bodies.remove(id); }
    pub fn remove_joint(&mut self, id:&str) { self.joints.remove(id); }
}

impl Default for PhysicsWorld { fn default() -> Self { Self::new() } }
