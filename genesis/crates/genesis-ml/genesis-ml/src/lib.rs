//! ML Bridge — difficulty controller, player model bus, ECS integration
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ── Events from ECS ───────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MlEvent {
    Death      { cause:String, zone:String },
    Kill       { enemy_type:String, method:String },
    QuestDone  { id:String, time_secs:f32 },
    ItemFound  { id:String, rarity:String },
    Idle       { secs:f32 },
    Achievement{ id:String },
    ZoneEntered{ id:String },
    LevelUp    { new_level:u32 },
    BossKill   { id:String, attempts:u32 },
    Rage       ,   // rage-quit behavior detected
    SessionStart,
    SessionEnd { secs:f64 },
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MlEventMsg {
    pub player_id: String,
    pub event:     MlEvent,
    pub value:     f32,
    pub context:   String,
    pub ts:        DateTime<Utc>,
}

// ── ML Event Bus ──────────────────────────────────────────────────
#[derive(Debug,Default)]
pub struct MlBus {
    pub pending:    Vec<MlEventMsg>,
    pub processed:  u64,
    pub dropped:    u64,
    pub capacity:   usize,
}

impl MlBus {
    pub fn new(capacity:usize) -> Self { Self { pending:Vec::new(), processed:0, dropped:0, capacity } }
    pub fn push(&mut self, msg:MlEventMsg) {
        if self.pending.len() >= self.capacity { self.dropped += 1; return; }
        self.pending.push(msg);
    }
    pub fn push_event(&mut self, player_id:&str, event:MlEvent, value:f32) {
        self.push(MlEventMsg { player_id:player_id.to_string(), event, value, context:String::new(), ts:Utc::now() });
    }
    pub fn drain(&mut self) -> Vec<MlEventMsg> {
        self.processed += self.pending.len() as u64;
        std::mem::take(&mut self.pending)
    }
    pub fn len(&self) -> usize { self.pending.len() }
    pub fn is_empty(&self) -> bool { self.pending.is_empty() }
}

// ── Adaptive Parameters ───────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AdaptiveParams {
    pub enemy_hp_mult:      f32,
    pub enemy_dmg_mult:     f32,
    pub enemy_speed_mult:   f32,
    pub enemy_accuracy:     f32,
    pub enemy_aggression:   f32,
    pub enemy_mistake_rate: f32,
    pub loot_quality_bonus: f32,
    pub aim_assist:         f32,
    pub iframes_mult:       f32,
    pub hint_frequency:     f32,
    pub respawn_time_mult:  f32,
    pub resource_abundance: f32,
}

impl Default for AdaptiveParams {
    fn default() -> Self {
        Self { enemy_hp_mult:1.0, enemy_dmg_mult:1.0, enemy_speed_mult:1.0,
               enemy_accuracy:0.75, enemy_aggression:0.7, enemy_mistake_rate:0.15,
               loot_quality_bonus:0.0, aim_assist:0.0, iframes_mult:1.0,
               hint_frequency:0.3, respawn_time_mult:1.0, resource_abundance:1.0 }
    }
}

impl AdaptiveParams {
    /// Move all params gently toward easier
    pub fn ease(&mut self, amount:f32) {
        let a = amount.clamp(0.0,1.0);
        self.enemy_hp_mult      = (self.enemy_hp_mult      - a*0.08).max(0.5);
        self.enemy_dmg_mult     = (self.enemy_dmg_mult     - a*0.06).max(0.4);
        self.enemy_speed_mult   = (self.enemy_speed_mult   - a*0.04).max(0.6);
        self.enemy_mistake_rate = (self.enemy_mistake_rate + a*0.05).min(0.4);
        self.loot_quality_bonus = (self.loot_quality_bonus + a*0.1 ).min(0.5);
        self.hint_frequency     = (self.hint_frequency     + a*0.1 ).min(1.0);
    }
    /// Move all params gently toward harder
    pub fn harden(&mut self, amount:f32) {
        let a = amount.clamp(0.0,1.0);
        self.enemy_hp_mult      = (self.enemy_hp_mult      + a*0.08).min(2.0);
        self.enemy_dmg_mult     = (self.enemy_dmg_mult     + a*0.06).min(1.8);
        self.enemy_speed_mult   = (self.enemy_speed_mult   + a*0.04).min(1.5);
        self.enemy_mistake_rate = (self.enemy_mistake_rate - a*0.04).max(0.05);
        self.hint_frequency     = (self.hint_frequency     - a*0.05).max(0.0);
    }
    pub fn reset(&mut self) { *self = Self::default(); }
}

// ── Engagement State ──────────────────────────────────────────────
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum EngagementState { Flow, Frustrated, Bored, Anxious, Excited, Engaged, AboutToQuit, Disengaged }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EngagementMetrics {
    pub state:       EngagementState,
    pub flow:        f32,          // 0-1
    pub frustration: f32,          // 0-1
    pub boredom:     f32,          // 0-1
    pub excitement:  f32,          // 0-1
    pub deaths_recent: u32,
    pub win_streak:  u32,
    pub lose_streak: u32,
    pub idle_secs:   f32,
}

impl Default for EngagementMetrics {
    fn default() -> Self {
        Self { state:EngagementState::Engaged, flow:0.5, frustration:0.0, boredom:0.0,
               excitement:0.5, deaths_recent:0, win_streak:0, lose_streak:0, idle_secs:0.0 }
    }
}

impl EngagementMetrics {
    pub fn update_state(&mut self) {
        self.state = if self.frustration > 0.75      { EngagementState::AboutToQuit }
            else if self.frustration > 0.5           { EngagementState::Frustrated }
            else if self.boredom > 0.7               { EngagementState::Bored }
            else if self.excitement > 0.7 && self.frustration < 0.2 { EngagementState::Excited }
            else if self.flow > 0.6 && self.frustration < 0.3 && self.boredom < 0.3 { EngagementState::Flow }
            else if self.idle_secs > 120.0           { EngagementState::Disengaged }
            else                                     { EngagementState::Engaged };
    }
}

// ── Per-player Difficulty Controller ──────────────────────────────
pub struct DifficultyController {
    pub player_id:   String,
    pub params:      AdaptiveParams,
    pub engagement:  EngagementMetrics,
    pub session_deaths: u32,
    pub session_kills:  u32,
    pub total_deaths:   u64,
    pub total_kills:    u64,
    pub interventions:  u64,
    pub session_start:  DateTime<Utc>,
}

impl DifficultyController {
    pub fn new(player_id:&str) -> Self {
        Self { player_id:player_id.to_string(), params:AdaptiveParams::default(),
               engagement:EngagementMetrics::default(),
               session_deaths:0, session_kills:0, total_deaths:0, total_kills:0,
               interventions:0, session_start:Utc::now() }
    }

    pub fn record_death(&mut self) {
        self.session_deaths += 1;
        self.total_deaths += 1;
        self.engagement.deaths_recent += 1;
        self.engagement.lose_streak += 1;
        self.engagement.win_streak = 0;
        self.engagement.frustration = (self.engagement.frustration + 0.1).min(1.0);
        self.engagement.excitement  = (self.engagement.excitement  - 0.05).max(0.0);
        self.engagement.update_state();
        // Invisible interventions
        if self.engagement.frustration > 0.6 {
            self.params.ease(0.5);
            self.interventions += 1;
            tracing::debug!("[ML] Easing difficulty for {} (frustration:{:.2})", self.player_id, self.engagement.frustration);
        }
    }

    pub fn record_kill(&mut self) {
        self.session_kills += 1;
        self.total_kills += 1;
        self.engagement.win_streak += 1;
        self.engagement.lose_streak = 0;
        self.engagement.deaths_recent = self.engagement.deaths_recent.saturating_sub(1);
        self.engagement.frustration = (self.engagement.frustration - 0.12).max(0.0);
        self.engagement.excitement  = (self.engagement.excitement  + 0.08).min(1.0);
        if self.engagement.win_streak > 5 {
            self.engagement.boredom = (self.engagement.boredom + 0.04).min(1.0);
        }
        self.engagement.update_state();
        // Harden if bored
        if self.engagement.boredom > 0.65 {
            self.params.harden(0.4);
            self.interventions += 1;
            tracing::debug!("[ML] Hardening difficulty for {} (boredom:{:.2})", self.player_id, self.engagement.boredom);
        }
    }

    pub fn record_idle(&mut self, secs:f32) {
        self.engagement.idle_secs += secs;
        self.engagement.update_state();
    }

    pub fn record_boss_kill(&mut self, attempts:u32) {
        self.engagement.frustration = (self.engagement.frustration - 0.3).max(0.0);
        self.engagement.excitement  = (self.engagement.excitement  + 0.5).min(1.0);
        self.engagement.boredom     = 0.0;
        self.engagement.win_streak += 5;
        self.engagement.update_state();
    }

    pub fn tick(&mut self, delta:f32) {
        // Natural recovery — frustration decays, boredom decays after action
        self.engagement.frustration = (self.engagement.frustration - delta*0.01).max(0.0);
        self.engagement.boredom     = (self.engagement.boredom     - delta*0.005).max(0.0);
        if !matches!(self.engagement.state, EngagementState::Disengaged) {
            self.engagement.idle_secs  = (self.engagement.idle_secs - delta).max(0.0);
        }
    }

    pub fn in_flow(&self) -> bool { self.engagement.state == EngagementState::Flow }
    pub fn is_frustrated(&self) -> bool { matches!(self.engagement.state, EngagementState::Frustrated|EngagementState::AboutToQuit) }
    pub fn kdr(&self) -> f32 { if self.total_deaths==0 {self.total_kills as f32} else {self.total_kills as f32/self.total_deaths as f32} }
}

// ── Global ML Runtime ─────────────────────────────────────────────
pub struct MlRuntime {
    pub controllers: HashMap<String,DifficultyController>,
    pub bus:         MlBus,
    pub enabled:     bool,
    pub update_interval: f32,
    pub timer:       f32,
}

impl MlRuntime {
    pub fn new() -> Self {
        Self { controllers:HashMap::new(), bus:MlBus::new(1000), enabled:true, update_interval:5.0, timer:0.0 }
    }
    pub fn get_or_create(&mut self, player_id:&str) -> &mut DifficultyController {
        self.controllers.entry(player_id.to_string()).or_insert_with(||DifficultyController::new(player_id))
    }
    pub fn process_bus(&mut self) {
        let msgs = self.bus.drain();
        for msg in msgs {
            let ctrl = self.get_or_create(&msg.player_id.clone());
            match &msg.event {
                MlEvent::Death{..}          => ctrl.record_death(),
                MlEvent::Kill{..}           => ctrl.record_kill(),
                MlEvent::BossKill{attempts,..}=>ctrl.record_boss_kill(*attempts),
                MlEvent::Idle{secs}         => ctrl.record_idle(*secs),
                _                           => {},
            }
        }
    }
    pub fn tick(&mut self, delta:f32) {
        if !self.enabled { return; }
        self.timer += delta;
        if self.timer >= self.update_interval { self.timer = 0.0; self.process_bus(); }
        for ctrl in self.controllers.values_mut() { ctrl.tick(delta); }
    }
    pub fn params_for(&self, player_id:&str) -> AdaptiveParams {
        self.controllers.get(player_id).map(|c|c.params.clone()).unwrap_or_default()
    }
    pub fn player_count(&self) -> usize { self.controllers.len() }
}
extern crate tracing;
