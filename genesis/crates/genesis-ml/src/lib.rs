//! Genesis ML — Player modeling, adaptive difficulty, NPC learning, anomaly detection
use serde::{Serialize,Deserialize};
use std::collections::{HashMap,VecDeque};
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlayerModel {
    pub player_id: String, pub session_count: u64, pub total_hours: f64,
    pub skill: SkillProfile, pub playstyle: PlaystyleProfile,
    pub engagement: EngagementModel, pub preferences: PreferenceModel,
    pub session: SessionMetrics, pub last_updated: DateTime<Utc>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SkillProfile {
    pub mechanical: f32, pub strategic: f32, pub spatial: f32,
    pub pattern_recognition: f32, pub adaptation_rate: f32,
    pub consistency: f32, pub rating: f32, pub percentile: f32,
}

impl Default for SkillProfile {
    fn default() -> Self {
        Self { mechanical:0.5, strategic:0.5, spatial:0.5,
               pattern_recognition:0.5, adaptation_rate:0.5,
               consistency:0.5, rating:1000.0, percentile:50.0 }
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlaystyleProfile {
    pub combat_style: CombatStyle, pub risk_tolerance: f32,
    pub exploration_drive: f32, pub narrative_engagement: f32,
    pub optimizer: bool, pub completionist: f32, pub speedrunner: f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CombatStyle { Aggressive, Defensive, Tactical, Stealth, Magical, Mixed, Avoidant }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EngagementModel {
    pub state: EngagementState, pub flow: f32, pub frustration: f32,
    pub boredom: f32, pub excitement: f32,
    pub recent_deaths: u32, pub win_streak: u32, pub lose_streak: u32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum EngagementState { Flow, Frustrated, Bored, Anxious, Excited, Engaged, AboutToQuit, Disengaged }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PreferenceModel {
    pub preferred_difficulty: f32, pub preferred_session_mins: f32,
    pub churn_risk: f32, pub likes_action: f32, pub likes_story: f32,
    pub likes_exploration: f32, pub likes_crafting: f32,
}

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct SessionMetrics {
    pub session_id: String, pub deaths: u32, pub kills: u32,
    pub quests_done: u32, pub items_found: u32, pub gold_earned: i64,
    pub distance_m: f64, pub playtime_secs: u64, pub current_zone: String,
}

// Adaptive difficulty
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AdaptiveParams {
    pub enemy_hp_mult: f32, pub enemy_dmg_mult: f32, pub enemy_speed_mult: f32,
    pub enemy_accuracy: f32, pub enemy_aggression: f32, pub enemy_mistake_rate: f32,
    pub loot_quality_bonus: f32, pub aim_assist: f32,
    pub iframes_mult: f32, pub hint_frequency: f32,
}
impl Default for AdaptiveParams {
    fn default() -> Self {
        Self { enemy_hp_mult:1.0, enemy_dmg_mult:1.0, enemy_speed_mult:1.0,
               enemy_accuracy:0.75, enemy_aggression:0.7, enemy_mistake_rate:0.15,
               loot_quality_bonus:0.0, aim_assist:0.0, iframes_mult:1.0, hint_frequency:0.3 }
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DifficultyIntervention {
    SubtleNerf{amount:f32}, NpcHint{hint:String}, LootBoost{quality:f32},
    DamageShield{secs:f32,reduction:f32}, IncreaseChallenge{mult:f32},
    TriggerEvent{event_id:String}, FastRespawn,
}

// Lightweight neural net
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TinyNet {
    pub id: String, pub layers: Vec<NetLayer>, pub training_samples: u64, pub accuracy: f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NetLayer { pub weights: Vec<Vec<f32>>, pub biases: Vec<f32>, pub activation: Activation }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Activation { ReLU, Sigmoid, Tanh, Softmax, Linear }

impl TinyNet {
    pub fn new(id: &str, sizes: &[usize]) -> Self {
        let mut layers = Vec::new();
        for i in 0..sizes.len()-1 {
            let scale = (2.0/sizes[i] as f32).sqrt();
            let w = (0..sizes[i+1]).map(|_|
                (0..sizes[i]).map(|_| (pseudo_rand() * 2.0 - 1.0) * scale).collect()
            ).collect();
            layers.push(NetLayer { weights:w, biases:vec![0.0;sizes[i+1]], activation:Activation::ReLU });
        }
        Self { id: id.to_string(), layers, training_samples:0, accuracy:0.0 }
    }
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut cur = input.to_vec();
        for layer in &self.layers {
            let mut next = vec![0.0f32; layer.biases.len()];
            for (o, (ws, b)) in layer.weights.iter().zip(layer.biases.iter()).enumerate() {
                let s: f32 = ws.iter().zip(cur.iter()).map(|(w,x)| w*x).sum::<f32>() + b;
                next[o] = match layer.activation { Activation::ReLU=>s.max(0.0), Activation::Sigmoid=>1.0/(1.0+(-s).exp()), Activation::Tanh=>s.tanh(), _=>s };
            }
            cur = next;
        }
        cur
    }
}

fn pseudo_rand() -> f32 {
    (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos() as f32 / u32::MAX as f32)
}

// ML Runtime
pub struct MlRuntime {
    pub models: HashMap<String,PlayerModel>,
    pub params: HashMap<String,AdaptiveParams>,
    pub nets: HashMap<String,TinyNet>,
    pub enabled: bool, pub timer: f32, pub update_interval: f32,
}

impl MlRuntime {
    pub fn new() -> Self {
        Self { models:HashMap::new(), params:HashMap::new(), nets:HashMap::new(),
               enabled:true, timer:0.0, update_interval:5.0 }
    }
    pub fn get_or_create(&mut self, player_id: &str) -> &mut PlayerModel {
        self.models.entry(player_id.to_string()).or_insert_with(|| PlayerModel {
            player_id: player_id.to_string(), session_count:0, total_hours:0.0,
            skill: SkillProfile::default(),
            playstyle: PlaystyleProfile { combat_style:CombatStyle::Mixed, risk_tolerance:0.5,
                exploration_drive:0.5, narrative_engagement:0.5, optimizer:false, completionist:0.0, speedrunner:0.0 },
            engagement: EngagementModel { state:EngagementState::Engaged, flow:0.5, frustration:0.0,
                boredom:0.0, excitement:0.5, recent_deaths:0, win_streak:0, lose_streak:0 },
            preferences: PreferenceModel { preferred_difficulty:0.5, preferred_session_mins:60.0,
                churn_risk:0.0, likes_action:0.5, likes_story:0.5, likes_exploration:0.5, likes_crafting:0.5 },
            session: SessionMetrics::default(), last_updated: Utc::now(),
        })
    }
    pub fn record_death(&mut self, player_id: &str) {
        let m = self.get_or_create(player_id);
        m.session.deaths += 1; m.engagement.lose_streak += 1; m.engagement.win_streak = 0;
        m.engagement.recent_deaths = (m.engagement.recent_deaths + 1).min(20);
        m.engagement.frustration = (m.engagement.frustration + 0.1).min(1.0);
        if m.engagement.frustration > 0.7 { m.engagement.state = EngagementState::Frustrated; }
        let params = self.params.entry(player_id.to_string()).or_default();
        if m.engagement.frustration > 0.6 {
            params.enemy_hp_mult = (params.enemy_hp_mult - 0.08).max(0.5);
            params.enemy_dmg_mult = (params.enemy_dmg_mult - 0.06).max(0.4);
        }
    }
    pub fn record_kill(&mut self, player_id: &str) {
        let m = self.get_or_create(player_id);
        m.session.kills += 1; m.engagement.win_streak += 1; m.engagement.lose_streak = 0;
        m.engagement.frustration = (m.engagement.frustration - 0.12).max(0.0);
        m.engagement.excitement = (m.engagement.excitement + 0.08).min(1.0);
        if m.engagement.win_streak > 5 { m.engagement.boredom = (m.engagement.boredom + 0.05).min(1.0); }
        if m.engagement.excitement > 0.7 && m.engagement.frustration < 0.2 {
            m.engagement.state = EngagementState::Flow;
        }
        let params = self.params.entry(player_id.to_string()).or_default();
        if m.engagement.boredom > 0.6 {
            params.enemy_hp_mult = (params.enemy_hp_mult + 0.08).min(2.0);
        }
    }
    pub fn get_params(&self, player_id: &str) -> AdaptiveParams {
        self.params.get(player_id).cloned().unwrap_or_default()
    }
    pub fn tick(&mut self, delta: f32) {
        if !self.enabled { return; }
        self.timer += delta;
        if self.timer >= self.update_interval { self.timer = 0.0; }
    }
    pub fn player_count(&self) -> usize { self.models.len() }
}
