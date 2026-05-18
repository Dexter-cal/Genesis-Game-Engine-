//! Runtime — save/load, accounts, achievements, leaderboards, telemetry, crash reporting
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ── Save System ───────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SaveSlot {
    pub slot_id:    u32,
    pub name:       String,
    pub timestamp:  DateTime<Utc>,
    pub playtime_secs: u64,
    pub screenshot: Option<String>,
    pub location:   String,
    pub level:      u32,
    pub completion_pct: f32,
    pub flags:      HashMap<String,bool>,
    pub vars:       HashMap<String,serde_json::Value>,
    pub version:    u32,
    pub encrypted:  bool,
    pub cloud_synced: bool,
    pub checksum:   String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SaveResult { Ok, CorruptData(String), VersionMismatch{got:u32,expected:u32}, IoError(String) }

pub struct SaveSystem {
    pub slots:      Vec<SaveSlot>,
    pub autosave:   AutosaveConfig,
    pub save_dir:   String,
    pub max_slots:  u32,
    pub cloud_sync: bool,
    pub total_saves: u64,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AutosaveConfig {
    pub enabled:        bool,
    pub interval_secs:  f32,
    pub timer:          f32,
    pub slot:           u32,
    pub on_zone_change: bool,
    pub on_boss_enter:  bool,
    pub max_autosaves:  u32,
}

impl SaveSystem {
    pub fn new(save_dir:&str) -> Self {
        Self { slots:Vec::new(), autosave:AutosaveConfig { enabled:true, interval_secs:300.0,
               timer:0.0, slot:0, on_zone_change:true, on_boss_enter:true, max_autosaves:3 },
               save_dir:save_dir.to_string(), max_slots:10, cloud_sync:false, total_saves:0 }
    }

    pub fn tick(&mut self, delta:f32) -> bool {
        if !self.autosave.enabled { return false; }
        self.autosave.timer += delta;
        if self.autosave.timer >= self.autosave.interval_secs {
            self.autosave.timer = 0.0;
            tracing::info!("Autosave triggered (slot {})", self.autosave.slot);
            return true;
        }
        false
    }

    pub fn slot_count(&self) -> usize { self.slots.len() }
    pub fn has_save(&self) -> bool { !self.slots.is_empty() }
}

// ── Achievements ──────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Achievement {
    pub id:          String,
    pub title:       String,
    pub description: String,
    pub icon:        String,
    pub hidden:      bool,
    pub points:      u32,
    pub rarity:      AchievementRarity,
    pub progress:    f32,
    pub target:      f32,
    pub unlocked:    bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub unlock_pct:  f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum AchievementRarity { Common, Uncommon, Rare, Epic, Legendary }

pub struct AchievementSystem {
    pub achievements: HashMap<String,Achievement>,
    pub pending_unlock: Vec<String>,
    pub platform_sync: bool,
    pub toast_queue:   Vec<String>,
}

impl AchievementSystem {
    pub fn new() -> Self {
        Self { achievements:HashMap::new(), pending_unlock:Vec::new(),
               platform_sync:false, toast_queue:Vec::new() }
    }

    pub fn register(&mut self, a:Achievement) { self.achievements.insert(a.id.clone(), a); }

    pub fn progress(&mut self, id:&str, delta:f32) -> bool {
        if let Some(a) = self.achievements.get_mut(id) {
            if a.unlocked { return false; }
            a.progress = (a.progress + delta).min(a.target);
            if a.progress >= a.target {
                a.unlocked = true;
                a.unlock_pct = 0.0;
                a.unlocked_at = Some(Utc::now());
                self.toast_queue.push(id.to_string());
                tracing::info!("Achievement unlocked: {} — {}", a.title, a.description);
                return true;
            }
        }
        false
    }

    pub fn unlock(&mut self, id:&str) -> bool { self.progress(id, f32::MAX) }
    pub fn is_unlocked(&self, id:&str) -> bool { self.achievements.get(id).map(|a|a.unlocked).unwrap_or(false) }
    pub fn total(&self) -> usize { self.achievements.len() }
    pub fn unlocked_count(&self) -> usize { self.achievements.values().filter(|a|a.unlocked).count() }
    pub fn points(&self) -> u32 { self.achievements.values().filter(|a|a.unlocked).map(|a|a.points).sum() }
}

// ── Leaderboard ───────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LeaderboardEntry { pub player_id:String, pub display_name:String, pub score:f64, pub rank:u32, pub ts:DateTime<Utc>, pub extra:HashMap<String,serde_json::Value> }

pub struct Leaderboard {
    pub id:       String,
    pub name:     String,
    pub entries:  Vec<LeaderboardEntry>,
    pub max_entries: u32,
    pub ascending:   bool,
}

impl Leaderboard {
    pub fn new(id:&str,name:&str,ascending:bool) -> Self {
        Self { id:id.to_string(), name:name.to_string(), entries:Vec::new(), max_entries:1000, ascending }
    }
    pub fn submit(&mut self, entry:LeaderboardEntry) {
        self.entries.push(entry);
        if self.ascending { self.entries.sort_by(|a,b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal)); }
        else { self.entries.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)); }
        for (i,e) in self.entries.iter_mut().enumerate() { e.rank = (i+1) as u32; }
        self.entries.truncate(self.max_entries as usize);
    }
    pub fn rank_of(&self, player_id:&str) -> Option<u32> {
        self.entries.iter().find(|e|e.player_id==player_id).map(|e|e.rank)
    }
    pub fn top(&self, n:usize) -> &[LeaderboardEntry] { &self.entries[..n.min(self.entries.len())] }
}

// ── Player Account ────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlayerAccount {
    pub id:          String,
    pub username:    String,
    pub display_name:String,
    pub email:       Option<String>,
    pub avatar_url:  Option<String>,
    pub level:       u32,
    pub xp:          u64,
    pub currency:    u64,
    pub premium_currency: u64,
    pub created:     DateTime<Utc>,
    pub last_login:  DateTime<Utc>,
    pub playtime_total_secs: u64,
    pub region:      String,
    pub platform:    String,
    pub friends:     Vec<String>,
    pub blocked:     Vec<String>,
    pub preferences: HashMap<String,serde_json::Value>,
    pub entitlements:Vec<String>,
    pub subscription: Option<String>,
}

// ── Crash Reporter ────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CrashReport {
    pub id:          String,
    pub timestamp:   DateTime<Utc>,
    pub engine_ver:  String,
    pub os:          String,
    pub arch:        String,
    pub gpu:         String,
    pub error:       String,
    pub backtrace:   String,
    pub game_state:  serde_json::Value,
    pub reproduced:  bool,
    pub submitted:   bool,
    pub user_comment:Option<String>,
}

// ── Runtime Manager ───────────────────────────────────────────────
pub struct RuntimeManager {
    pub saves:        SaveSystem,
    pub achievements: AchievementSystem,
    pub leaderboards: HashMap<String,Leaderboard>,
    pub account:      Option<PlayerAccount>,
    pub crash_log:    Vec<CrashReport>,
    pub session_id:   String,
    pub session_start:DateTime<Utc>,
}

impl RuntimeManager {
    pub fn new(save_dir:&str) -> Self {
        Self { saves:SaveSystem::new(save_dir), achievements:AchievementSystem::new(),
               leaderboards:HashMap::new(), account:None, crash_log:Vec::new(),
               session_id:format!("session_{}", Utc::now().timestamp_millis()),
               session_start:Utc::now() }
    }
    pub fn tick(&mut self, delta:f32) {
        if self.saves.tick(delta) {
            tracing::debug!("Autosave triggered");
        }
    }
    pub fn session_secs(&self) -> f64 { (Utc::now()-self.session_start).num_seconds() as f64 }
}
extern crate tracing;
