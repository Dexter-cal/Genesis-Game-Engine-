//! Genesis Anti-Cheat — behavioral analysis, server validation, enforcement
use serde::{Serialize,Deserialize};
use std::collections::{HashMap,VecDeque};
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlayerSuspicion {
    pub player_id: String, pub total_score: f32,
    pub status: SuspicionStatus, pub violations: Vec<Violation>,
    pub movement_samples: VecDeque<MovSample>, pub aim_samples: VecDeque<AimSample>,
    pub reports_received: Vec<PlayerReport>, pub session_violations: u32,
    pub last_reviewed: Option<DateTime<Utc>>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SuspicionStatus {
    Clean, Monitoring{since:DateTime<Utc>}, Flagged{reason:String},
    SoftBanned{started:DateTime<Utc>}, HardBanned{started:DateTime<Utc>,permanent:bool},
    Cleared,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Violation {
    pub id: String, pub kind: ViolationKind,
    pub severity: Severity, pub suspicion: f32,
    pub timestamp: DateTime<Utc>, pub evidence: String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ViolationKind {
    SpeedHack{detected:f32,max:f32}, TeleportHack{dist_m:f32},
    AimBot{variance:f32}, WallClip{pos:[f32;3]},
    RateLimitExceeded{action:String,rate:f32},
    MemoryTamper{region:String}, CheatSoftware{name:String},
    EconomyAnomaly{gained:u64,expected:u64},
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Severity { Low, Medium, High, Critical }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MovSample { pub pos:[f32;3], pub speed:f32, pub ts:DateTime<Utc> }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AimSample { pub pitch:f32, pub yaw:f32, pub dp:f32, pub dy:f32, pub ts:DateTime<Utc> }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlayerReport { pub reporter:String, pub reason:String, pub ts:DateTime<Utc> }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ValidationResult {
    Allowed, Flagged{suspicion:f32}, RateLimited{retry_ms:f32},
    Rejected{reason:String}, Banned{permanent:bool},
}
impl ValidationResult { pub fn ok(&self)->bool{matches!(self,Self::Allowed|Self::Flagged{..})} }

pub struct AntiCheatManager {
    pub enabled: bool,
    pub profiles: HashMap<String,PlayerSuspicion>,
    pub sessions: HashMap<String,SessionMonitor>,
    pub soft_ban_threshold: f32, pub hard_ban_threshold: f32,
    pub max_speed_mult: f32, pub teleport_dist: f32,
    pub whitelist: Vec<String>, pub bans: u64, pub violations_total: u64,
}

pub struct SessionMonitor {
    pub player_id: String, pub session_id: String,
    pub last_pos: Option<[f32;3]>, pub violations: u32,
    pub action_counts: HashMap<String,u32>, pub action_times: HashMap<String,DateTime<Utc>>,
}

impl AntiCheatManager {
    pub fn new() -> Self {
        Self { enabled:true, profiles:HashMap::new(), sessions:HashMap::new(),
               soft_ban_threshold:40.0, hard_ban_threshold:80.0,
               max_speed_mult:1.25, teleport_dist:15.0,
               whitelist:Vec::new(), bans:0, violations_total:0 }
    }

    pub fn start_session(&mut self, player_id:&str, session_id:&str) {
        if !self.enabled { return; }
        self.sessions.insert(session_id.to_string(), SessionMonitor {
            player_id:player_id.to_string(), session_id:session_id.to_string(),
            last_pos:None, violations:0, action_counts:HashMap::new(), action_times:HashMap::new(),
        });
    }

    pub fn validate_movement(&mut self, session_id:&str, pos:[f32;3], claimed_speed:f32, max_speed:f32) -> ValidationResult {
        if !self.enabled { return ValidationResult::Allowed; }
        let Some(sess) = self.sessions.get_mut(session_id) else { return ValidationResult::Allowed; };
        if self.whitelist.contains(&sess.player_id) { return ValidationResult::Allowed; }
        let pid = sess.player_id.clone();

        // Teleport check
        if let Some(last) = sess.last_pos {
            let dx=pos[0]-last[0]; let dz=pos[2]-last[2];
            let dist=(dx*dx+dz*dz).sqrt();
            if dist > self.teleport_dist {
                sess.violations += 1;
                self.add_violation(&pid, ViolationKind::TeleportHack{dist_m:dist}, Severity::High, 15.0);
                return ValidationResult::Rejected{reason:"Teleport detected".to_string()};
            }
        }
        sess.last_pos = Some(pos);

        // Speed check
        if claimed_speed > max_speed * self.max_speed_mult {
            self.add_violation(&pid, ViolationKind::SpeedHack{detected:claimed_speed,max:max_speed}, Severity::Medium, 5.0);
            if sess.violations > 5 { return ValidationResult::Flagged{suspicion:5.0}; }
        }
        ValidationResult::Allowed
    }

    pub fn validate_action_rate(&mut self, session_id:&str, action:&str, max_per_sec:f32) -> ValidationResult {
        if !self.enabled { return ValidationResult::Allowed; }
        let Some(sess) = self.sessions.get_mut(session_id) else { return ValidationResult::Allowed; };
        let now = Utc::now();
        let count = sess.action_counts.entry(action.to_string()).or_insert(0);
        let last = sess.action_times.get(action).cloned().unwrap_or(now);
        let elapsed = (now - last).num_milliseconds() as f32 / 1000.0;
        if elapsed < 1.0 {
            *count += 1;
            if *count as f32 > max_per_sec {
                let pid = sess.player_id.clone();
                let rate = *count as f32 / elapsed.max(0.001);
                self.add_violation(&pid, ViolationKind::RateLimitExceeded{action:action.to_string(),rate}, Severity::Low, 2.0);
                return ValidationResult::RateLimited{retry_ms:1000.0/max_per_sec};
            }
        } else {
            *count = 1;
            sess.action_times.insert(action.to_string(), now);
        }
        ValidationResult::Allowed
    }

    fn add_violation(&mut self, player_id:&str, kind:ViolationKind, severity:Severity, score:f32) {
        self.violations_total += 1;
        let p = self.profiles.entry(player_id.to_string()).or_insert_with(|| PlayerSuspicion {
            player_id:player_id.to_string(), total_score:0.0, status:SuspicionStatus::Clean,
            violations:Vec::new(), movement_samples:VecDeque::new(), aim_samples:VecDeque::new(),
            reports_received:Vec::new(), session_violations:0, last_reviewed:None,
        });
        p.total_score = (p.total_score + score).min(100.0);
        p.violations.push(Violation {
            id: format!("v_{}", Utc::now().timestamp_millis()),
            kind, severity, suspicion:score, timestamp:Utc::now(), evidence:String::new(),
        });
        if p.total_score >= self.hard_ban_threshold {
            p.status = SuspicionStatus::HardBanned{started:Utc::now(),permanent:false};
            self.bans += 1;
        } else if p.total_score >= self.soft_ban_threshold {
            p.status = SuspicionStatus::SoftBanned{started:Utc::now()};
        } else if p.total_score > 20.0 {
            p.status = SuspicionStatus::Monitoring{since:Utc::now()};
        }
    }

    pub fn is_banned(&self, player_id:&str) -> bool {
        self.profiles.get(player_id).map(|p| matches!(p.status,
            SuspicionStatus::HardBanned{..}|SuspicionStatus::SoftBanned{..})).unwrap_or(false)
    }
    pub fn end_session(&mut self, session_id:&str) { self.sessions.remove(session_id); }
    pub fn monitored(&self) -> usize { self.profiles.len() }
    pub fn active_sessions(&self) -> usize { self.sessions.len() }
}
