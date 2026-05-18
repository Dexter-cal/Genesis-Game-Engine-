//! Genesis Advanced Physics — destruction, fluid sim, rope, vehicles, force fields
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ── Destruction ──────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DestructibleObject {
    pub id:String, pub mesh_id:String, pub material:PhysMaterial,
    pub health:f32, pub max_health:f32, pub broken:bool,
    pub fragments:u32, pub fracture:FractureMethod,
    pub structural:bool, pub cascade_delay:f32,
    pub cracks:Vec<CrackInfo>, pub damage_pct:f32,
    pub last_impact:Option<ImpactRecord>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PhysMaterial {
    pub kind:MatKind, pub hardness:f32, pub brittleness:f32,
    pub density:f32, pub yield_mpa:f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MatKind { Glass{tempered:bool}, Wood, Stone, Metal, Concrete, Ice, Ceramic, Crystal, Custom(String) }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum FractureMethod { Voronoi{sites:u32,noise:f32}, Radial{r:u32,a:u32}, BrickPattern{w:f32,h:f32} }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CrackInfo { pub pos:[f32;3], pub dir:[f32;3], pub width:f32, pub length:f32 }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ImpactRecord {
    pub pos:[f32;3], pub dir:[f32;3], pub force:f32,
    pub kind:ImpactKind, pub ts:DateTime<Utc>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ImpactKind { Bullet{caliber:f32,vel:f32}, Blunt{mass:f32,vel:f32}, Explosive{yield_j:f32,dist:f32}, Cutting, Fire{temp:f32}, Magic(String) }

// ── Fluid Simulation ─────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FluidSim {
    pub id:String, pub fluid:FluidType,
    pub particle_radius:f32, pub max_particles:u32, pub active_particles:u32,
    pub positions:Vec<[f32;3]>, pub velocities:Vec<[f32;3]>,
    pub densities:Vec<f32>, pub temperatures:Vec<f32>,
    pub bounds:[f32;6], pub gravity:[f32;3],
    pub surface_tension:f32, pub gpu:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum FluidType {
    Water{temp_c:f32}, Oil{viscosity:f32}, Lava{temp_c:f32}, Blood,
    Honey{viscosity:f32}, Acid{concentration:f32}, Slime{elasticity:f32},
    Mud{solid:f32}, MagicFluid{color:[f32;4],effect:String},
    Custom{viscosity:f32,density:f32,color:[f32;4]},
}

impl FluidType {
    pub fn viscosity(&self)->f32 {
        match self { Self::Water{..}=>0.001, Self::Oil{viscosity}=>*viscosity,
            Self::Honey{viscosity}=>*viscosity, Self::Lava{..}=>100.0,
            Self::Blood=>0.003, Self::Mud{solid}=>1.0+solid*5.0,
            Self::Custom{viscosity,..}=>*viscosity, _=>0.001 }
    }
    pub fn density(&self)->f32 {
        match self { Self::Water{..}=>1000.0, Self::Oil{..}=>900.0,
            Self::Lava{..}=>2800.0, Self::Blood=>1060.0,
            Self::Honey{..}=>1400.0, Self::Custom{density,..}=>*density, _=>1000.0 }
    }
}

// ── Rope ─────────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RopeSim {
    pub id:String, pub kind:RopeKind, pub length_m:f32, pub segments:u32,
    pub mass_per_m:f32, pub stiffness:f32, pub damping:f32,
    pub break_force:f32, pub broken:bool,
    pub particles:Vec<RopeParticle>, pub attach_a:RopeAttach, pub attach_b:RopeAttach,
    pub wind_influence:f32, pub collision:bool,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum RopeKind { Rope, Chain, Wire, Cable, Vine, Web{stickiness:f32}, Custom(String) }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RopeParticle { pub pos:[f32;3], pub prev:[f32;3], pub vel:[f32;3], pub inv_mass:f32 }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum RopeAttach { Fixed{pos:[f32;3]}, Entity{id:String,bone:Option<String>}, Free }

// ── Vehicles ─────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VehiclePhysics {
    pub id:String, pub kind:VehicleKind,
    pub mass_kg:f32, pub com:[f32;3],
    pub engine:Engine, pub transmission:Transmission,
    pub wheels:Vec<Wheel>, pub drag_coeff:f32, pub frontal_area:f32,
    pub state:VehicleState,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum VehicleKind { Car{seats:u32}, Truck{payload:f32}, Motorcycle, Boat{hull:String},
    Tank, Helicopter, Plane, Hovercraft, Mech{legs:u32}, Custom(String) }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Engine {
    pub max_torque:f32, pub max_power_kw:f32, pub redline_rpm:f32,
    pub idle_rpm:f32, pub current_rpm:f32, pub throttle:f32,
    pub running:bool, pub temperature:f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Transmission {
    pub auto:bool, pub current_gear:i32, pub gear_ratios:Vec<f32>,
    pub final_drive:f32, pub clutch:f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Wheel {
    pub id:String, pub pos:[f32;3], pub radius:f32, pub mass:f32,
    pub spring_rate:f32, pub damper:f32, pub grip:f32,
    pub driven:bool, pub steered:bool, pub grounded:bool,
    pub current_rpm:f32, pub slip:f32, pub wear:f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VehicleState {
    pub speed_mps:f32, pub accel:f32, pub yaw_rate:f32,
    pub pitch:f32, pub roll:f32, pub odometer:f64,
    pub airborne:bool, pub flipped:bool, pub on_fire:bool,
    pub engine_damage:f32, pub body_damage:f32,
}

// ── Force Fields ─────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ForceField {
    pub id:String, pub kind:FieldKind, pub shape:FieldShape,
    pub strength:f32, pub falloff:FieldFalloff,
    pub affects_bodies:bool, pub affects_particles:bool,
    pub affects_cloth:bool, pub active:bool,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum FieldKind {
    Gravity{dir:[f32;3]}, Wind{dir:[f32;3],turb:f32},
    Magnet{north:bool}, Vortex{axis:[f32;3]},
    Repulsion, Attraction, Drag{coeff:f32},
    Explosion{pos:[f32;3],yield_j:f32}, Custom(String),
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum FieldShape { Sphere{r:f32}, Box{he:[f32;3]}, Capsule{r:f32,h:f32}, Global }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum FieldFalloff { None, Linear, InvSquare, Exponential{rate:f32} }

// ── Manager ──────────────────────────────────────────────────────
pub struct AdvancedPhysicsManager {
    pub destructibles:HashMap<String,DestructibleObject>,
    pub fluids:HashMap<String,FluidSim>,
    pub ropes:HashMap<String,RopeSim>,
    pub vehicles:HashMap<String,VehiclePhysics>,
    pub force_fields:HashMap<String,ForceField>,
    pub gravity:[f32;3], pub air_density:f32, pub time_scale:f32,
}

impl AdvancedPhysicsManager {
    pub fn new() -> Self {
        Self { destructibles:HashMap::new(), fluids:HashMap::new(), ropes:HashMap::new(),
               vehicles:HashMap::new(), force_fields:HashMap::new(),
               gravity:[0.0,-9.81,0.0], air_density:1.225, time_scale:1.0 }
    }

    pub fn apply_damage(&mut self, id:&str, impact:ImpactRecord) {
        if let Some(obj) = self.destructibles.get_mut(id) {
            let dmg = impact.force / (obj.material.hardness * 10000.0 + 1.0);
            obj.health = (obj.health - dmg).max(0.0);
            obj.damage_pct = 1.0 - obj.health / obj.max_health;
            if obj.damage_pct > 0.3 && obj.cracks.len() < 8 {
                obj.cracks.push(CrackInfo{pos:impact.pos,dir:impact.dir,width:dmg*0.001,length:dmg*0.05});
            }
            if obj.health <= 0.0 && !obj.broken { obj.broken = true; tracing::info!("{} destroyed",id); }
            obj.last_impact = Some(impact);
        }
    }

    pub fn tick(&mut self, delta:f32) {
        let dt = delta * self.time_scale;
        // Tick fluids
        for fluid in self.fluids.values_mut() {
            let visc = fluid.fluid.viscosity();
            let g = self.gravity;
            for i in 0..fluid.active_particles as usize {
                if i >= fluid.positions.len() { break; }
                fluid.velocities[i][0] += g[0]*dt;
                fluid.velocities[i][1] += g[1]*dt;
                fluid.velocities[i][2] += g[2]*dt;
                fluid.velocities[i][0] *= 1.0-visc*dt;
                fluid.velocities[i][1] *= 1.0-visc*dt;
                fluid.velocities[i][2] *= 1.0-visc*dt;
                for ax in 0..3 {
                    fluid.positions[i][ax] += fluid.velocities[i][ax]*dt;
                    if fluid.positions[i][ax] < fluid.bounds[ax] {
                        fluid.positions[i][ax] = fluid.bounds[ax];
                        fluid.velocities[i][ax] *= -0.3;
                    }
                    if fluid.positions[i][ax] > fluid.bounds[ax+3] {
                        fluid.positions[i][ax] = fluid.bounds[ax+3];
                        fluid.velocities[i][ax] *= -0.3;
                    }
                }
            }
        }
        // Tick ropes — PBD
        for rope in self.ropes.values_mut() {
            if rope.broken { continue; }
            let g = self.gravity;
            for p in &mut rope.particles {
                if p.inv_mass <= 0.0 { continue; }
                p.vel[0]+=g[0]*dt; p.vel[1]+=g[1]*dt; p.vel[2]+=g[2]*dt;
                p.prev=p.pos;
                p.pos[0]+=p.vel[0]*dt; p.pos[1]+=p.vel[1]*dt; p.pos[2]+=p.vel[2]*dt;
            }
        }
    }

    pub fn destructible_count(&self)->usize{self.destructibles.len()}
    pub fn fluid_count(&self)->usize{self.fluids.len()}
    pub fn rope_count(&self)->usize{self.ropes.len()}
    pub fn vehicle_count(&self)->usize{self.vehicles.len()}
}
extern crate tracing;
