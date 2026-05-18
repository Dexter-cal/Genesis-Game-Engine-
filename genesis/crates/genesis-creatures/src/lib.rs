//! Genesis Creature Ecosystem — 200+ species, genetics, AI behavior, taming, riding
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CreatureDefinition {
    pub id: String, pub name: String, pub class: CreatureClass,
    pub size: SizeClass, pub height_m: f32, pub weight_kg: f32,
    pub health_max: f32, pub stamina_max: f32,
    pub speed_walk: f32, pub speed_run: f32, pub speed_fly: f32, pub speed_swim: f32,
    pub threat: ThreatLevel, pub disposition: Disposition,
    pub diet: DietType, pub territory_m: f32,
    pub natural_weapons: Vec<NaturalWeapon>,
    pub abilities: Vec<String>, pub loot: Vec<LootEntry>,
    pub tameable: bool, pub rideable: bool,
    pub genetics: GeneticProfile, pub ecosystem_role: EcosystemRole,
    pub biomes: Vec<String>, pub pack_size: [u32;2],
    pub intelligence: f32, pub memory_hours: f32,
    pub night_active: bool, pub migrates: bool,
    pub sounds: CreatureSounds,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CreatureClass {
    Mammal, Reptile, Bird, Fish, Amphibian, Insect, Arachnid,
    Dragon{element:String}, Undead, Elemental{element:String},
    Fae, Demon, Celestial, Construct, Alien, Custom(String),
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SizeClass { Tiny, Small, Medium, Large, Huge, Gargantuan, Colossal }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ThreatLevel { Passive, Neutral, Wary, Aggressive, Predator, Apex, Legendary }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Disposition { Friendly, Neutral, Wary, Hostile, Passive }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DietType { Carnivore, Herbivore, Omnivore, Insectivore, Magical, Photosynthetic }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NaturalWeapon {
    pub name: String, pub damage_min: f32, pub damage_max: f32,
    pub damage_type: String, pub reach_m: f32, pub effect: Option<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LootEntry { pub item_id: String, pub chance: f32, pub qty: [u32;2] }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct GeneticProfile {
    pub genes: HashMap<String,f32>, pub mutation_rate: f32,
    pub crossbreed_with: Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EcosystemRole {
    pub trophic_level: u32, pub preys_on: Vec<String>,
    pub prey_of: Vec<String>, pub pop_min: u32, pub pop_max: u32,
    pub respawn_hours: f32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CreatureSounds {
    pub idle: Vec<String>, pub alert: Vec<String>,
    pub attack: Vec<String>, pub death: Vec<String>,
    pub ambient: Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CreatureInstance {
    pub instance_id: String, pub definition_id: String,
    pub name: Option<String>, pub health: f32, pub stamina: f32,
    pub hunger: f32, pub thirst: f32, pub age_days: f32,
    pub state: CreatureState, pub position: [f32;3],
    pub home: [f32;3], pub target: Option<String>,
    pub owner: Option<String>, pub trust: f32,
    pub mood: CreatureMood, pub pack_id: Option<String>,
    pub level: u32, pub experience: u64,
    pub individual_traits: Vec<String>,
    pub known_threats: Vec<String>,
    pub known_food_sources: Vec<[f32;3]>,
    pub spawned_at: DateTime<Utc>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CreatureState {
    Idle, Sleeping, Eating, Grazing, Patrol, Alert{source:String},
    Fleeing{from:String}, Hunting{prey:String}, Fighting{target:String},
    Migrating{dest:[f32;3]}, Mating{partner:String}, Following{entity:String}, Dead,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CreatureMood { Content, Happy, Neutral, Anxious, Aggressive, Playful, Hungry, Scared }

pub struct EcosystemManager {
    pub definitions: HashMap<String,CreatureDefinition>,
    pub instances: HashMap<String,CreatureInstance>,
    pub packs: HashMap<String,Vec<String>>,
    pub enabled: bool, pub simulation_radius_m: f32,
    pub full_ai_radius_m: f32, pub lod_ai_radius_m: f32,
}

impl EcosystemManager {
    pub fn new() -> Self {
        Self { definitions: HashMap::new(), instances: HashMap::new(),
               packs: HashMap::new(), enabled: true,
               simulation_radius_m: 500.0, full_ai_radius_m: 80.0, lod_ai_radius_m: 250.0 }
    }
    pub fn register(&mut self, def: CreatureDefinition) {
        tracing::info!("Registered creature: {}", def.name);
        self.definitions.insert(def.id.clone(), def);
    }
    pub fn spawn(&mut self, def_id: &str, position: [f32;3]) -> Option<String> {
        let def = self.definitions.get(def_id)?;
        let id = format!("{}_{}", def_id, self.instances.len());
        let inst = CreatureInstance {
            instance_id: id.clone(), definition_id: def_id.to_string(),
            name: None, health: def.health_max, stamina: def.stamina_max,
            hunger: 0.0, thirst: 0.0, age_days: 0.0,
            state: CreatureState::Idle, position, home: position,
            target: None, owner: None, trust: 0.0, mood: CreatureMood::Neutral,
            pack_id: None, level: 1, experience: 0,
            individual_traits: Vec::new(), known_threats: Vec::new(),
            known_food_sources: Vec::new(), spawned_at: Utc::now(),
        };
        self.instances.insert(id.clone(), inst);
        Some(id)
    }
    pub fn tick(&mut self, delta: f32, player_pos: [f32;3]) {
        if !self.enabled { return; }
        for inst in self.instances.values_mut() {
            let dx = inst.position[0]-player_pos[0];
            let dz = inst.position[2]-player_pos[2];
            let dist = (dx*dx+dz*dz).sqrt();
            if dist < self.full_ai_radius_m {
                inst.hunger += delta / 3600.0;
                inst.thirst += delta / 1800.0;
                inst.age_days += delta / 86400.0;
            }
        }
    }
    pub fn count(&self) -> usize { self.instances.len() }
    pub fn species_count(&self) -> usize { self.definitions.len() }
}
extern crate tracing;
