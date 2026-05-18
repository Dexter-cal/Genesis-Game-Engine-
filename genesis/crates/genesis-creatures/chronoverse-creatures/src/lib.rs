//! Genesis Creatures — ECS bridge to genesis-creatures, LOD AI, spawn registry
use serde::{Serialize,Deserialize};
use std::collections::HashMap;

// ── LOD System ────────────────────────────────────────────────────
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
pub enum CreatureLod {
    Full,    // <80m  — full AI, animations, physics
    Medium,  // <250m — simplified AI, LOD mesh
    Crowd,   // <500m — billboard, no AI
    Static,  // <1000m — static sprite
    Culled,  //  >1000m — removed from simulation
}

impl CreatureLod {
    pub fn from_dist(d:f32) -> Self {
        match d as u32 { 0..=80=>Self::Full, 81..=250=>Self::Medium,
                          251..=500=>Self::Crowd, 501..=1000=>Self::Static, _=>Self::Culled }
    }
    pub fn update_hz(&self) -> f32 {
        match self { Self::Full=>60.0, Self::Medium=>10.0, Self::Crowd=>2.0, Self::Static=>0.1, Self::Culled=>0.0 }
    }
    pub fn needs_physics(&self) -> bool { matches!(self,Self::Full|Self::Medium) }
    pub fn needs_ai(&self) -> bool { matches!(self,Self::Full|Self::Medium) }
    pub fn needs_animation(&self) -> bool { matches!(self,Self::Full) }
}

// ── Spawn Request ─────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SpawnRequest {
    pub def_id:  String,
    pub pos:     [f32;3],
    pub tamed:   bool,
    pub owner:   Option<String>,
    pub level:   Option<u32>,
    pub pack_id: Option<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SpawnResult {
    Ok    { instance_id:String },
    Failed{ reason:String },
}

// ── Creature Registry (ECS Bridge) ───────────────────────────────
pub struct CreatureRegistry {
    pub pending:     Vec<SpawnRequest>,
    pub active:      HashMap<String,CreatureEntry>,
    pub by_species:  HashMap<String,Vec<String>>,
    pub by_pack:     HashMap<String,Vec<String>>,
    pub lod_map:     HashMap<String,CreatureLod>,
    pub total_spawned: u64,
    pub total_killed:  u64,
    pub simulation_radius_m: f32,
    pub max_creatures: u32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CreatureEntry {
    pub instance_id: String,
    pub def_id:      String,
    pub pos:         [f32;3],
    pub health:      f32,
    pub alive:       bool,
    pub tamed:       bool,
    pub owner:       Option<String>,
    pub pack_id:     Option<String>,
    pub level:       u32,
    pub lod:         CreatureLod,
    pub tick_timer:  f32,
    pub last_updated:f32,
}

impl CreatureRegistry {
    pub fn new(max:u32, radius:f32) -> Self {
        Self { pending:Vec::new(), active:HashMap::new(), by_species:HashMap::new(),
               by_pack:HashMap::new(), lod_map:HashMap::new(),
               total_spawned:0, total_killed:0,
               simulation_radius_m:radius, max_creatures:max }
    }

    pub fn queue_spawn(&mut self, req:SpawnRequest) -> bool {
        if self.active.len() as u32 >= self.max_creatures { return false; }
        self.pending.push(req);
        true
    }

    pub fn process_pending(&mut self) -> Vec<SpawnResult> {
        let mut results = Vec::new();
        let reqs:Vec<_> = self.pending.drain(..).collect();
        for req in reqs {
            let id = format!("{}_{}",req.def_id, self.total_spawned);
            let entry = CreatureEntry {
                instance_id:id.clone(), def_id:req.def_id.clone(),
                pos:req.pos, health:100.0, alive:true,
                tamed:req.tamed, owner:req.owner, pack_id:req.pack_id.clone(),
                level:req.level.unwrap_or(1), lod:CreatureLod::Full,
                tick_timer:0.0, last_updated:0.0,
            };
            self.by_species.entry(req.def_id.clone()).or_default().push(id.clone());
            if let Some(pack) = &req.pack_id {
                self.by_pack.entry(pack.clone()).or_default().push(id.clone());
            }
            self.lod_map.insert(id.clone(), CreatureLod::Full);
            self.active.insert(id.clone(), entry);
            self.total_spawned += 1;
            results.push(SpawnResult::Ok{instance_id:id});
        }
        results
    }

    pub fn kill(&mut self, id:&str) {
        if let Some(e) = self.active.get_mut(id) { e.alive = false; self.total_killed += 1; }
        self.active.retain(|_,e|e.alive);
    }

    pub fn update_lod(&mut self, player_pos:[f32;3]) {
        for (id, entry) in &mut self.active {
            let dx = entry.pos[0]-player_pos[0];
            let dz = entry.pos[2]-player_pos[2];
            let dist = (dx*dx+dz*dz).sqrt();
            let lod = CreatureLod::from_dist(dist);
            entry.lod = lod;
            self.lod_map.insert(id.clone(), lod);
        }
    }

    pub fn tick(&mut self, delta:f32, player_pos:[f32;3]) {
        self.update_lod(player_pos);
        for entry in self.active.values_mut() {
            if entry.lod == CreatureLod::Culled { continue; }
            entry.tick_timer += delta;
            let hz = entry.lod.update_hz();
            if hz > 0.0 && entry.tick_timer >= 1.0/hz {
                entry.tick_timer = 0.0;
                entry.last_updated += delta;
            }
        }
    }

    pub fn count(&self) -> usize { self.active.len() }
    pub fn alive_count(&self) -> usize { self.active.values().filter(|e|e.alive).count() }
    pub fn species_count(&self) -> usize { self.by_species.len() }
    pub fn in_range(&self, player_pos:[f32;3], range:f32) -> Vec<&CreatureEntry> {
        self.active.values().filter(|e| {
            let dx=e.pos[0]-player_pos[0]; let dz=e.pos[2]-player_pos[2];
            (dx*dx+dz*dz).sqrt() <= range
        }).collect()
    }
    pub fn full_ai_count(&self) -> usize { self.lod_map.values().filter(|&&l|l==CreatureLod::Full).count() }
}

impl Default for CreatureRegistry {
    fn default() -> Self { Self::new(2000, 1000.0) }
}
