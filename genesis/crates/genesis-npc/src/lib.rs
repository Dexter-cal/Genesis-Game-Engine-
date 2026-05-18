//! NPC System — lifecycle, schedules, factions, dialogue, memory, relationships
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ═══ NPC DEFINITION ══════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NpcDefinition {
    pub id:          String,
    pub name:        String,
    pub display_name:String,
    pub archetype:   NpcArchetype,
    pub faction:     String,
    pub race:        String,
    pub gender:      String,
    pub age:         u32,
    pub occupation:  String,
    pub personality: Personality,
    pub stats:       NpcStats,
    pub appearance:  NpcAppearance,
    pub voice_id:    Option<String>,
    pub home_scene:  String,
    pub home_pos:    [f32;3],
    pub schedule:    Vec<ScheduleEntry>,
    pub dialogue:    Vec<DialogueTree>,
    pub ai_prompt:   String,   // system prompt for AI dialogue
    pub ai_enabled:  bool,     // use AI for live dialogue
    pub quest_giver: bool,
    pub merchant:    bool,
    pub inventory:   Vec<String>,
    pub loot_table:  Vec<String>,
    pub relationships: HashMap<String,i32>,  // entity_id → attitude (-100 to 100)
    pub known_facts:   Vec<String>,
    pub secrets:       Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum NpcArchetype {
    Civilian, Guard, Merchant, QuestGiver, Companion, Villain, Boss,
    Sage, Artisan, Noble, Criminal, Soldier, Priest, Scholar,
    Innkeeper, Blacksmith, Herbalist, Custom(String),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Personality {
    pub openness:        f32,  // 0-1
    pub conscientiousness:f32,
    pub extraversion:    f32,
    pub agreeableness:   f32,
    pub neuroticism:     f32,
    pub humor:           f32,
    pub loyalty:         f32,
    pub bravery:         f32,
    pub greed:           f32,
    pub curiosity:       f32,
    pub traits:          Vec<String>,
    pub speech_style:    String,
    pub catchphrase:     Option<String>,
}

impl Personality {
    pub fn friendly() -> Self { Self { openness:0.8, conscientiousness:0.7, extraversion:0.8, agreeableness:0.9, neuroticism:0.2, humor:0.6, loyalty:0.8, bravery:0.5, greed:0.1, curiosity:0.7, traits:vec!["warm".to_string(),"talkative".to_string()], speech_style:"warm and welcoming".to_string(), catchphrase:None } }
    pub fn gruff() -> Self { Self { openness:0.3, conscientiousness:0.8, extraversion:0.2, agreeableness:0.3, neuroticism:0.4, humor:0.3, loyalty:0.9, bravery:0.9, greed:0.2, curiosity:0.3, traits:vec!["blunt".to_string(),"stoic".to_string()], speech_style:"short and blunt".to_string(), catchphrase:None } }
    pub fn cunning() -> Self { Self { openness:0.7, conscientiousness:0.9, extraversion:0.6, agreeableness:0.2, neuroticism:0.3, humor:0.5, loyalty:0.2, bravery:0.6, greed:0.8, curiosity:0.8, traits:vec!["scheming".to_string(),"sly".to_string()], speech_style:"clever and evasive".to_string(), catchphrase:None } }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NpcStats {
    pub health:    f32, pub max_health:f32, pub stamina:f32,
    pub strength:  f32, pub agility:f32, pub intelligence:f32,
    pub charisma:  f32, pub perception:f32, pub luck:f32,
    pub level:     u32, pub xp:u64,
    pub threat_level: u32,
}

impl NpcStats {
    pub fn civilian() -> Self { Self { health:80.0, max_health:80.0, stamina:100.0, strength:5.0, agility:5.0, intelligence:8.0, charisma:7.0, perception:6.0, luck:5.0, level:1, xp:0, threat_level:0 } }
    pub fn guard()    -> Self { Self { health:150.0, max_health:150.0, stamina:120.0, strength:12.0, agility:8.0, intelligence:7.0, charisma:6.0, perception:9.0, luck:5.0, level:5, xp:0, threat_level:3 } }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NpcAppearance {
    pub height_cm:  u32,
    pub build:      Build,
    pub hair:       String,
    pub eyes:       String,
    pub skin:       String,
    pub clothing:   Vec<String>,
    pub scars:      Vec<String>,
    pub tattoos:    Vec<String>,
    pub description:String,
    pub locked:     bool,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Build { Slim, Average, Athletic, Heavy, Muscular }

// ═══ SCHEDULE ═════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ScheduleEntry {
    pub hour:       f32,       // 0.0-24.0
    pub day:        Option<u32>,// None=every day, Some(0-6)=specific weekday
    pub activity:   Activity,
    pub location:   [f32;3],
    pub location_id:Option<String>,
    pub duration_h: f32,
    pub interrupt:  Vec<InterruptCondition>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Activity {
    Sleep, Wake, Eat{meal:String}, Work{task:String}, Patrol{route:Vec<[f32;3]>},
    Trade, Socialize{with:Option<String>}, Pray, Train, Wander,
    Guard{post:[f32;3]}, Read, Cook, Craft{item:String},
    Drink{at:String}, Custom(String),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum InterruptCondition {
    PlayerNearby{range:f32}, Attacked, QuestActive{id:String},
    WorldFlag{flag:String,val:bool}, TimeOfDay{hour:f32},
    Raining, Night, Custom(String),
}

// ═══ DIALOGUE ════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DialogueTree {
    pub id:    String,
    pub nodes: Vec<DialogueNode>,
    pub start: String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DialogueNode {
    pub id:          String,
    pub speaker:     String,
    pub text:        String,
    pub conditions:  Vec<String>,
    pub choices:     Vec<DialogueChoice>,
    pub auto_next:   Option<String>,
    pub effects:     Vec<DialogueEffect>,
    pub voice_file:  Option<String>,
    pub emotion:     String,
    pub camera_hint: Option<String>,
    pub translations:HashMap<String,String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DialogueChoice {
    pub id:          String,
    pub text:        String,
    pub next:        Option<String>,
    pub conditions:  Vec<String>,
    pub effects:     Vec<DialogueEffect>,
    pub shown:       bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DialogueEffect {
    SetFlag{flag:String,val:bool}, AddItem{item:String,qty:u32},
    RemoveItem{item:String,qty:u32}, GiveXp(u64), ChangeRelation{target:String,delta:i32},
    StartQuest{id:String}, CompleteQuest{id:String}, OpenShop,
    TransitionScene{id:String}, Custom(String),
}

// ═══ FACTION ══════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Faction {
    pub id:          String,
    pub name:        String,
    pub description: String,
    pub alignment:   Alignment,
    pub leader:      Option<String>,
    pub members:     Vec<String>,
    pub headquarters:[f32;3],
    pub resources:   HashMap<String,f32>,
    pub goals:       Vec<String>,
    pub relations:   HashMap<String,i32>,  // faction_id → attitude
    pub emblems:     Vec<String>,
    pub color:       [f32;4],
    pub hostility_threshold: i32,         // below this → attack on sight
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Alignment { LawfulGood,NeutralGood,ChaoticGood,LawfulNeutral,TrueNeutral,ChaoticNeutral,LawfulEvil,NeutralEvil,ChaoticEvil }

impl Faction {
    pub fn attitude(&self, faction_id:&str) -> i32 { *self.relations.get(faction_id).unwrap_or(&0) }
    pub fn is_hostile(&self, faction_id:&str) -> bool { self.attitude(faction_id) <= self.hostility_threshold }
    pub fn set_relation(&mut self, faction_id:&str, attitude:i32) { self.relations.insert(faction_id.to_string(), attitude.clamp(-100,100)); }
    pub fn shift_relation(&mut self, faction_id:&str, delta:i32) {
        let cur = self.attitude(faction_id);
        self.set_relation(faction_id, cur+delta);
    }
}

// ═══ NPC INSTANCE ════════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum NpcState {
    Idle, Sleeping, Eating, Working{task:String}, Wandering,
    Patrolling{waypoint:usize}, Socializing{with:String},
    InDialogue{with:String}, Shopping, Fleeing{from:String},
    Fighting{target:String}, Dead, Custom(String),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NpcInstance {
    pub instance_id:  String,
    pub def_id:       String,
    pub name:         String,
    pub state:        NpcState,
    pub position:     [f32;3],
    pub rotation:     f32,
    pub health:       f32,
    pub schedule_idx: usize,
    pub attitude:     HashMap<String,i32>,  // player/npc_id → attitude
    pub memory:       Vec<NpcMemory>,
    pub current_activity: Option<String>,
    pub in_dialogue:  bool,
    pub last_seen_player_pos: Option<[f32;3]>,
    pub alert_level:  f32,           // 0=calm, 1=max alert
    pub ai_context:   String,        // injected into AI prompt
    pub barks:        Vec<String>,   // short ambient voice lines
    pub bark_timer:   f32,
    pub quest_state:  HashMap<String,String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NpcMemory {
    pub content:   String,
    pub kind:      String,
    pub ts:        DateTime<Utc>,
    pub relevance: f32,
    pub about:     Option<String>,
}

impl NpcInstance {
    pub fn new(instance_id:&str, def_id:&str, name:&str, pos:[f32;3]) -> Self {
        Self { instance_id:instance_id.to_string(), def_id:def_id.to_string(),
               name:name.to_string(), state:NpcState::Idle, position:pos, rotation:0.0,
               health:100.0, schedule_idx:0, attitude:HashMap::new(),
               memory:Vec::new(), current_activity:None, in_dialogue:false,
               last_seen_player_pos:None, alert_level:0.0, ai_context:String::new(),
               barks:Vec::new(), bark_timer:0.0, quest_state:HashMap::new() }
    }
    pub fn attitude_toward(&self, id:&str) -> i32 { *self.attitude.get(id).unwrap_or(&0) }
    pub fn is_hostile_to(&self, id:&str) -> bool { self.attitude_toward(id) <= -50 }
    pub fn is_friendly_to(&self, id:&str) -> bool { self.attitude_toward(id) >= 25 }
    pub fn remember(&mut self, content:&str, kind:&str, about:Option<&str>) {
        self.memory.push(NpcMemory { content:content.to_string(), kind:kind.to_string(),
            ts:Utc::now(), relevance:1.0, about:about.map(|s|s.to_string()) });
        if self.memory.len() > 50 { self.memory.remove(0); }
    }
    pub fn is_alive(&self) -> bool { self.health > 0.0 && !matches!(self.state, NpcState::Dead) }
}

// ═══ NPC MANAGER ═════════════════════════════════════════════════
pub struct NpcManager {
    pub definitions: HashMap<String,NpcDefinition>,
    pub instances:   HashMap<String,NpcInstance>,
    pub factions:    HashMap<String,Faction>,
    pub active_dialogues: HashMap<String,(String,String)>,  // player_id → (npc_id, node_id)
    pub ai_dialogue_enabled: bool,
    pub schedule_enabled:    bool,
    pub total_spawned:       u64,
    pub dialogue_count:      u64,
}

impl NpcManager {
    pub fn new() -> Self {
        Self { definitions:HashMap::new(), instances:HashMap::new(),
               factions:HashMap::new(), active_dialogues:HashMap::new(),
               ai_dialogue_enabled:true, schedule_enabled:true,
               total_spawned:0, dialogue_count:0 }
    }

    pub fn register_def(&mut self, def:NpcDefinition) {
        tracing::info!("Registered NPC def: {} ({})", def.name, def.id);
        self.definitions.insert(def.id.clone(), def);
    }

    pub fn register_faction(&mut self, faction:Faction) {
        tracing::info!("Registered faction: {}", faction.name);
        self.factions.insert(faction.id.clone(), faction);
    }

    pub fn spawn(&mut self, def_id:&str, pos:[f32;3]) -> Option<String> {
        let def = self.definitions.get(def_id)?;
        let id = format!("npc_{}_{}", def_id, self.total_spawned);
        let mut inst = NpcInstance::new(&id, def_id, &def.display_name, pos);
        // Copy base stats
        inst.health = def.stats.health;
        inst.ai_context = def.ai_prompt.clone();
        inst.barks = vec![
            format!("Hello there, traveller."),
            format!("Nice weather we're having."),
            format!("Watch yourself out there."),
        ];
        self.instances.insert(id.clone(), inst);
        self.total_spawned += 1;
        tracing::debug!("Spawned NPC: {} at {:?}", def.name, pos);
        Some(id)
    }

    pub fn start_dialogue(&mut self, player_id:&str, npc_id:&str) -> Option<&DialogueTree> {
        let inst = self.instances.get_mut(npc_id)?;
        let def = self.definitions.get(&inst.def_id.clone())?;
        inst.state = NpcState::InDialogue { with:player_id.to_string() };
        inst.in_dialogue = true;
        self.active_dialogues.insert(player_id.to_string(), (npc_id.to_string(), "root".to_string()));
        self.dialogue_count += 1;
        tracing::info!("Dialogue: {} ↔ {}", player_id, npc_id);
        def.dialogue.first()
    }

    pub fn end_dialogue(&mut self, player_id:&str) {
        if let Some((npc_id,_)) = self.active_dialogues.remove(player_id) {
            if let Some(inst) = self.instances.get_mut(&npc_id) {
                inst.state = NpcState::Idle;
                inst.in_dialogue = false;
            }
        }
    }

    pub fn set_faction_relation(&mut self, faction_a:&str, faction_b:&str, attitude:i32) {
        if let Some(f) = self.factions.get_mut(faction_a) { f.set_relation(faction_b, attitude); }
        if let Some(f) = self.factions.get_mut(faction_b) { f.set_relation(faction_a, attitude); }
    }

    pub fn npc_attitude_to_player(&self, npc_id:&str, player_id:&str) -> i32 {
        self.instances.get(npc_id).map(|n|n.attitude_toward(player_id)).unwrap_or(0)
    }

    pub fn tick(&mut self, delta:f32, time_of_day:f32) {
        if !self.schedule_enabled { return; }
        for inst in self.instances.values_mut() {
            if !inst.is_alive() || inst.in_dialogue { continue; }
            // Bark timer
            inst.bark_timer -= delta;
            if inst.bark_timer <= 0.0 && !inst.barks.is_empty() {
                inst.bark_timer = 30.0 + pseudo_rand()*60.0;
            }
            // Alert decay
            inst.alert_level = (inst.alert_level - delta*0.1).max(0.0);
        }
    }

    pub fn npc_count(&self) -> usize { self.instances.len() }
    pub fn alive_count(&self) -> usize { self.instances.values().filter(|n|n.is_alive()).count() }
    pub fn faction_count(&self) -> usize { self.factions.len() }
}

fn pseudo_rand() -> f32 {
    (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos() as f32/u32::MAX as f32)
}

impl Default for NpcManager { fn default() -> Self { Self::new() } }
extern crate tracing;
