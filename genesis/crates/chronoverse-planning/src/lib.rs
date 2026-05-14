//! AI Scene Planner — generate scenes from descriptions, layout plans, dependency graphs
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ScenePlan {
    pub id:           String,
    pub name:         String,
    pub description:  String,
    pub scene_type:   SceneType,
    pub size_m:       [f32;3],
    pub objects:      Vec<PlannedObject>,
    pub lighting:     PlannedLighting,
    pub audio:        PlannedAudio,
    pub spawn_points: Vec<[f32;3]>,
    pub exits:        Vec<PlannedExit>,
    pub npcs:         Vec<PlannedNpc>,
    pub hazards:      Vec<PlannedHazard>,
    pub secrets:      Vec<PlannedSecret>,
    pub mood:         String,
    pub constraints:  Vec<String>,
    pub approved:     bool,
    pub generated_by: String,
    pub created_at:   DateTime<Utc>,
    pub version:      u32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SceneType {
    Interior { building_type:String, floors:u32 },
    Exterior { biome:String, open_world:bool },
    Dungeon  { depth:u32, boss:bool },
    Arena    { pvp:bool, size:String },
    Hub      { fast_travel:bool },
    Cutscene { duration_secs:f32 },
    Tutorial { steps:Vec<String> },
    Custom(String),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlannedObject {
    pub id:          String,
    pub prefab:      String,
    pub position:    [f32;3],
    pub rotation:    [f32;3],
    pub scale:       [f32;3],
    pub purpose:     String,
    pub interactive: bool,
    pub destructible:bool,
    pub variants:    Vec<String>,
    pub tags:        Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlannedLighting {
    pub time_of_day:  f32,
    pub mood:         String,
    pub sun_dir:      [f32;3],
    pub sun_color:    [f32;3],
    pub sun_intensity:f32,
    pub ambient:      [f32;4],
    pub fog:          bool,
    pub fog_density:  f32,
    pub fog_color:    [f32;4],
    pub bloom:        bool,
    pub lut:          Option<String>,
    pub point_lights: Vec<PlannedLight>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlannedLight {
    pub pos:       [f32;3],
    pub color:     [f32;3],
    pub intensity: f32,
    pub range:     f32,
    pub kind:      String,
    pub shadows:   bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlannedAudio {
    pub ambient_loops: Vec<String>,
    pub music_track:   Option<String>,
    pub music_mood:    String,
    pub reverb:        String,
    pub audio_zones:   Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlannedExit {
    pub id:          String,
    pub position:    [f32;3],
    pub destination: String,
    pub label:       String,
    pub locked:      bool,
    pub key_item:    Option<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlannedNpc {
    pub archetype:  String,
    pub position:   [f32;3],
    pub faction:    String,
    pub purpose:    String,
    pub quest_giver:bool,
    pub merchant:   bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlannedHazard {
    pub kind:     String,
    pub position: [f32;3],
    pub radius:   f32,
    pub damage:   f32,
    pub visible:  bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlannedSecret {
    pub description:  String,
    pub position:     [f32;3],
    pub trigger:      String,
    pub reward:       String,
}

// ── Plan Request ──────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlanRequest {
    pub id:          String,
    pub description: String,
    pub scene_type:  Option<String>,
    pub constraints: Vec<String>,
    pub style_ref:   Option<String>,
    pub size_hint:   Option<[f32;3]>,
    pub npc_count:   u32,
    pub secret_count:u32,
    pub submitted:   DateTime<Utc>,
    pub status:      PlanStatus,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum PlanStatus { Pending, Planning, Complete, Failed(String) }

// ── Planner ───────────────────────────────────────────────────────
pub struct ScenePlanner {
    pub queue:     Vec<PlanRequest>,
    pub plans:     HashMap<String, ScenePlan>,
    pub active:    Option<String>,
    pub ai_model:  String,
    pub total_gen: u64,
    pub avg_time_secs: f32,
}

impl ScenePlanner {
    pub fn new() -> Self {
        Self { queue:Vec::new(), plans:HashMap::new(), active:None,
               ai_model:"qwen2.5:7b".to_string(), total_gen:0, avg_time_secs:0.0 }
    }

    pub fn request(&mut self, desc:&str, constraints:Vec<String>) -> String {
        let id = format!("plan_{}", Utc::now().timestamp_millis());
        tracing::info!("Scene plan requested: {}", desc);
        self.queue.push(PlanRequest {
            id:id.clone(), description:desc.to_string(), scene_type:None,
            constraints, style_ref:None, size_hint:None,
            npc_count:3, secret_count:1,
            submitted:Utc::now(), status:PlanStatus::Pending,
        });
        id
    }

    pub fn submit_plan(&mut self, plan:ScenePlan) {
        tracing::info!("Scene plan complete: {} ({} objects)", plan.name, plan.objects.len());
        self.total_gen += 1;
        if let Some(req) = self.queue.iter_mut().find(|r|r.id==plan.id) {
            req.status = PlanStatus::Complete;
        }
        self.plans.insert(plan.id.clone(), plan);
        self.active = None;
    }

    pub fn get(&self, id:&str) -> Option<&ScenePlan> { self.plans.get(id) }
    pub fn pending(&self) -> usize { self.queue.iter().filter(|r|r.status==PlanStatus::Pending).count() }
    pub fn plan_count(&self) -> usize { self.plans.len() }
    pub fn has_active(&self) -> bool { self.active.is_some() }
}

impl Default for ScenePlanner { fn default() -> Self { Self::new() } }
extern crate tracing;
