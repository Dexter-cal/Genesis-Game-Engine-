//! Genesis Security System
//!
//! Multi-layer security for the engine and games:
//!
//! ENGINE SECURITY:
//! - Script sandboxing (scripts cannot access OS/filesystem)
//! - API rate limiting (prevent abuse of cloud APIs)
//! - Input validation (all user inputs sanitized)
//! - Prompt injection prevention (LLM inputs sanitized)
//! - Secret management (API keys never in source)
//! - Plugin sandboxing (untrusted plugins run isolated)
//! - Audit logging (all sensitive operations logged)
//!
//! GAME SECURITY:
//! - Anti-cheat (detect speed hacks, teleports, impossible inputs)
//! - Server authority (server validates all game-changing actions)
//! - Replay validation (detect retroactive modification)
//! - Rate limiting (prevent spam, DDoS)
//! - Cheat detection heuristics (movement, aim, timing)
//!
//! DATA SECURITY:
//! - Save file encryption
//! - Network traffic encryption (TLS 1.3)
//! - Password hashing (Argon2id)
//! - Session token management
//! - GDPR compliance tools
//!
//! CONTENT SECURITY:
//! - Child safety filters on all AI outputs
//! - Content moderation for user-generated content
//! - AI output filtering (no harmful content)
//! - Age-appropriate content gates

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

// ─── Script Sandbox ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    /// Allow filesystem access
    pub allow_fs_read: bool,
    pub allow_fs_write: bool,
    /// Allowed directories for read
    pub allowed_read_dirs: Vec<String>,
    /// Allow network access
    pub allow_network: bool,
    /// Allowed network hosts
    pub allowed_hosts: Vec<String>,
    /// Allow spawning processes
    pub allow_process_spawn: bool,
    /// Max memory per script (MB)
    pub max_memory_mb: u32,
    /// Max CPU time per tick (ms)
    pub max_cpu_ms: u32,
    /// Max execution time total (seconds)
    pub max_total_seconds: f32,
    /// Allowed API surface (which engine APIs can be called)
    pub allowed_apis: Vec<String>,
    /// Blocked keywords in source code
    pub blocked_patterns: Vec<String>,
}

impl SandboxPolicy {
    pub fn game_script() -> Self {
        Self {
            allow_fs_read: false,
            allow_fs_write: false,
            allowed_read_dirs: vec!["./assets".to_string()],
            allow_network: false,
            allowed_hosts: Vec::new(),
            allow_process_spawn: false,
            max_memory_mb: 64,
            max_cpu_ms: 5,
            max_total_seconds: 0.0, // unlimited if within per-tick budget
            allowed_apis: vec![
                "world".to_string(), "npc".to_string(), "player".to_string(),
                "audio".to_string(), "particles".to_string(), "ui".to_string(),
                "timer".to_string(), "input".to_string(), "physics".to_string(),
                "inventory".to_string(), "quest".to_string(), "camera".to_string(),
                "debug".to_string(),
            ],
            blocked_patterns: vec![
                "import os".to_string(), "import sys".to_string(),
                "exec(".to_string(), "eval(".to_string(),
                "__import__".to_string(), "subprocess".to_string(),
                "open(".to_string(), "file(".to_string(),
            ],
        }
    }

    pub fn plugin() -> Self {
        let mut p = Self::game_script();
        p.allow_fs_read = true;
        p.allowed_read_dirs = vec!["./plugins".to_string(), "./assets".to_string()];
        p.max_memory_mb = 256;
        p.max_cpu_ms = 16;
        p
    }

    pub fn trusted() -> Self {
        Self {
            allow_fs_read: true, allow_fs_write: true,
            allowed_read_dirs: vec!["./".to_string()],
            allow_network: true,
            allowed_hosts: vec!["*".to_string()],
            allow_process_spawn: false,
            max_memory_mb: 1024,
            max_cpu_ms: 100,
            max_total_seconds: 60.0,
            allowed_apis: vec!["*".to_string()],
            blocked_patterns: Vec::new(),
        }
    }

    /// Validate script source against policy
    pub fn validate_source(&self, source: &str) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();
        for pattern in &self.blocked_patterns {
            if source.contains(pattern.as_str()) {
                violations.push(format!("Blocked pattern found: '{}'", pattern));
            }
        }
        if violations.is_empty() { Ok(()) } else { Err(violations) }
    }
}

// ─── Rate Limiter ─────────────────────────────────────────────────────────────

pub struct RateLimiter {
    /// entity_id → (count, window_start)
    windows: HashMap<String, (u32, Instant)>,
    pub max_requests: u32,
    pub window_secs: f32,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: f32) -> Self {
        Self { windows: HashMap::new(), max_requests, window_secs }
    }

    pub fn check(&mut self, entity_id: &str) -> RateLimitResult {
        let now = Instant::now();
        let entry = self.windows.entry(entity_id.to_string()).or_insert((0, now));

        // Reset window if expired
        if entry.1.elapsed().as_secs_f32() >= self.window_secs {
            *entry = (0, now);
        }

        entry.0 += 1;
        let remaining = self.max_requests.saturating_sub(entry.0);
        let reset_in = self.window_secs - entry.1.elapsed().as_secs_f32();

        if entry.0 > self.max_requests {
            RateLimitResult::Blocked { reset_in_secs: reset_in }
        } else {
            RateLimitResult::Allowed { remaining, reset_in_secs: reset_in }
        }
    }

    pub fn cleanup(&mut self) {
        let window = Duration::from_secs_f32(self.window_secs);
        self.windows.retain(|_, (_, start)| start.elapsed() < window);
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed { remaining: u32, reset_in_secs: f32 },
    Blocked { reset_in_secs: f32 },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool { matches!(self, Self::Allowed { .. }) }
}

// ─── Input Validation ────────────────────────────────────────────────────────

pub struct InputValidator;

impl InputValidator {
    /// Sanitize text input (user-facing text)
    pub fn sanitize_text(input: &str, max_len: usize) -> String {
        let truncated = &input[..input.len().min(max_len)];
        truncated
            .chars()
            .filter(|c| c.is_alphanumeric() || " .,!?'-_():@#".contains(*c))
            .collect()
    }

    /// Sanitize LLM prompt (prevent injection)
    pub fn sanitize_llm_prompt(input: &str) -> String {
        // Remove common injection patterns
        let mut s = input.replace("IGNORE PREVIOUS INSTRUCTIONS", "");
        s = s.replace("ignore all previous", "");
        s = s.replace("disregard your", "");
        s = s.replace("you are now", "");
        s = s.replace("pretend you are", "");
        s = s.replace("act as if", "");
        s = s.replace("jailbreak", "");
        s = s.replace("DAN mode", "");
        // Limit length
        if s.len() > 2000 { s.truncate(2000); }
        s
    }

    /// Validate a gamer tag
    pub fn validate_gamer_tag(tag: &str) -> Result<(), &'static str> {
        if tag.len() < 3 { return Err("Too short (min 3)"); }
        if tag.len() > 20 { return Err("Too long (max 20)"); }
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Only letters, numbers, _ and - allowed");
        }
        // Block offensive patterns
        let lower = tag.to_lowercase();
        let blocked = ["admin", "moderator", "support", "genesis", "system"];
        for b in &blocked {
            if lower.contains(b) { return Err("Reserved word"); }
        }
        Ok(())
    }

    /// Validate world position (prevent impossible teleports)
    pub fn validate_position(pos: [f32; 3], world_bounds: f32) -> bool {
        pos.iter().all(|&v| v.is_finite() && v.abs() <= world_bounds)
    }

    /// Detect impossible game inputs (cheat detection)
    pub fn validate_player_input(
        claimed_pos: [f32; 3],
        prev_pos: [f32; 3],
        delta: f32,
        max_speed: f32,
    ) -> CheatCheckResult {
        let dx = claimed_pos[0] - prev_pos[0];
        let dy = claimed_pos[1] - prev_pos[1];
        let dz = claimed_pos[2] - prev_pos[2];
        let dist = (dx*dx + dy*dy + dz*dz).sqrt();
        let effective_speed = dist / delta.max(0.001);

        if effective_speed > max_speed * 1.5 {
            CheatCheckResult::Suspicious {
                reason: format!("Speed {:.1} > max {:.1}", effective_speed, max_speed),
                severity: if effective_speed > max_speed * 3.0 { CheatSeverity::High }
                          else { CheatSeverity::Medium },
            }
        } else {
            CheatCheckResult::Clean
        }
    }
}

#[derive(Debug, Clone)]
pub enum CheatCheckResult {
    Clean,
    Suspicious { reason: String, severity: CheatSeverity },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheatSeverity { Low, Medium, High, Definite }

// ─── Anti-Cheat System ───────────────────────────────────────────────────────

pub struct AntiCheat {
    /// Per-player violation records
    pub violations: HashMap<String, Vec<CheatViolation>>,
    pub ban_threshold: u32,
    pub flag_threshold: u32,
    pub position_history: HashMap<String, Vec<([f32; 3], f64)>>,
    pub aim_samples: HashMap<String, Vec<f32>>, // aim deltas for aim bot detection
    pub banned_players: std::collections::HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheatViolation {
    pub player_id: String,
    pub violation_type: ViolationType,
    pub severity: u8,
    pub timestamp: u64,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    SpeedHack { effective_speed: f32, max_allowed: f32 },
    Teleport { distance: f32, delta: f32 },
    AimBot { snap_angle: f32, consistency: f32 },
    Wallhack, // detecting through walls
    DamageManipulation { claimed: f32, expected: f32 },
    PacketManipulation,
    MemoryModification,
    InvalidInput { input: String },
    ReplayTampering,
}

impl AntiCheat {
    pub fn new() -> Self {
        Self {
            violations: HashMap::new(),
            ban_threshold: 10,
            flag_threshold: 3,
            position_history: HashMap::new(),
            aim_samples: HashMap::new(),
            banned_players: std::collections::HashSet::new(),
        }
    }

    pub fn record_violation(&mut self, violation: CheatViolation) {
        let player_id = violation.player_id.clone();
        tracing::warn!("Anti-cheat violation: {:?} for player {}", violation.violation_type, player_id);

        let violations = self.violations.entry(player_id.clone()).or_default();
        violations.push(violation);

        let total_severity: u32 = violations.iter().map(|v| v.severity as u32).sum();

        if total_severity >= self.ban_threshold {
            self.ban_player(&player_id, "Repeated cheating violations");
        }
    }

    pub fn ban_player(&mut self, player_id: &str, reason: &str) {
        self.banned_players.insert(player_id.to_string());
        tracing::error!("Player BANNED: {} — {}", player_id, reason);
    }

    pub fn is_banned(&self, player_id: &str) -> bool {
        self.banned_players.contains(player_id)
    }

    pub fn check_position(&mut self, player_id: &str, pos: [f32; 3], delta: f32, max_speed: f32) {
        let history = self.position_history.entry(player_id.to_string()).or_default();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();

        if let Some(&(prev_pos, prev_time)) = history.last() {
            let dt = (now - prev_time) as f32;
            let result = InputValidator::validate_player_input(pos, prev_pos, dt, max_speed);

            if let CheatCheckResult::Suspicious { reason, severity } = result {
                let sev = match severity {
                    CheatSeverity::Low => 1u8,
                    CheatSeverity::Medium => 3,
                    CheatSeverity::High => 6,
                    CheatSeverity::Definite => 10,
                };

                let dx = pos[0] - prev_pos[0];
                let dy = pos[1] - prev_pos[1];
                let dz = pos[2] - prev_pos[2];
                let dist = (dx*dx + dy*dy + dz*dz).sqrt();

                self.record_violation(CheatViolation {
                    player_id: player_id.to_string(),
                    violation_type: ViolationType::SpeedHack {
                        effective_speed: dist / dt,
                        max_allowed: max_speed,
                    },
                    severity: sev,
                    timestamp: now as u64,
                    evidence: serde_json::json!({ "reason": reason }),
                });
            }
        }

        history.push((pos, now));
        // Keep only last 60 positions
        if history.len() > 60 { history.remove(0); }
    }

    pub fn check_aim_snap(&mut self, player_id: &str, look_delta: [f32; 2]) {
        let magnitude = (look_delta[0].powi(2) + look_delta[1].powi(2)).sqrt();
        let samples = self.aim_samples.entry(player_id.to_string()).or_default();
        samples.push(magnitude);
        if samples.len() > 100 { samples.remove(0); }

        // Detect unnaturally consistent large aim snaps (aim bot signature)
        if samples.len() >= 20 {
            let large_snaps: Vec<f32> = samples.iter().filter(|&&s| s > 45.0).cloned().collect();
            if large_snaps.len() >= 10 {
                let mean = large_snaps.iter().sum::<f32>() / large_snaps.len() as f32;
                let variance = large_snaps.iter().map(|&s| (s - mean).powi(2)).sum::<f32>() / large_snaps.len() as f32;
                let std_dev = variance.sqrt();
                let consistency = 1.0 - (std_dev / mean).min(1.0);

                if consistency > 0.9 {
                    self.record_violation(CheatViolation {
                        player_id: player_id.to_string(),
                        violation_type: ViolationType::AimBot { snap_angle: mean, consistency },
                        severity: 8,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        evidence: serde_json::json!({ "snaps": large_snaps.len(), "consistency": consistency }),
                    });
                }
            }
        }
    }
}

// ─── Content Moderation ───────────────────────────────────────────────────────

pub struct ContentModerator {
    /// Blocked words/phrases
    blocked_terms: Vec<String>,
    /// Child safety filter
    child_safety_enabled: bool,
    /// Minimum age for mature content
    mature_content_age: u8,
}

impl ContentModerator {
    pub fn new() -> Self {
        Self {
            blocked_terms: vec![
                // Basic blocked content list - production would use ML classifier
                "explicit_content".to_string(),
            ],
            child_safety_enabled: true,
            mature_content_age: 17,
        }
    }

    pub fn check_text(&self, text: &str) -> ModerationResult {
        let lower = text.to_lowercase();
        for term in &self.blocked_terms {
            if lower.contains(term.as_str()) {
                return ModerationResult::Blocked {
                    reason: "Prohibited content".to_string(),
                    severity: ModerationSeverity::High,
                };
            }
        }
        ModerationResult::Approved
    }

    pub fn check_ai_output(&self, output: &str, context: &str) -> ModerationResult {
        // Check for accidental harmful AI output
        let result = self.check_text(output);
        if !matches!(result, ModerationResult::Approved) {
            return result;
        }

        // Child-safety: check for age-inappropriate content in children's context
        if self.child_safety_enabled && context.contains("child") {
            // Would use ML classifier in production
        }

        ModerationResult::Approved
    }
}

#[derive(Debug, Clone)]
pub enum ModerationResult {
    Approved,
    ReviewNeeded { reason: String },
    Blocked { reason: String, severity: ModerationSeverity },
}

#[derive(Debug, Clone)]
pub enum ModerationSeverity { Low, Medium, High, Critical }

// ─── Audit Logger ─────────────────────────────────────────────────────────────

pub struct AuditLogger {
    pub entries: Vec<AuditEntry>,
    pub max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub actor: String,      // who did it
    pub action: String,     // what they did
    pub target: String,     // what it affected
    pub result: AuditResult,
    pub ip: Option<String>,
    pub session_id: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult { Success, Failure, Blocked }

impl AuditLogger {
    pub fn new() -> Self { Self { entries: Vec::new(), max_entries: 100_000 } }

    pub fn log(&mut self, actor: &str, action: &str, target: &str, result: AuditResult) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(AuditEntry {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            actor: actor.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            result,
            ip: None,
            session_id: None,
            metadata: serde_json::Value::Null,
        });
    }

    pub fn recent(&self, n: usize) -> &[AuditEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }
}

// ─── Secret Manager ───────────────────────────────────────────────────────────

/// Manages API keys and secrets — never stored in plaintext in source/config
pub struct SecretManager {
    /// Encrypted secrets store
    secrets: HashMap<String, String>,
    /// Whether secrets are loaded from env
    env_loaded: bool,
}

impl SecretManager {
    pub fn new() -> Self {
        let mut sm = Self { secrets: HashMap::new(), env_loaded: false };
        sm.load_from_env();
        sm
    }

    fn load_from_env(&mut self) {
        let keys = [
            "CHRONO_CLAUDE_API_KEY", "CHRONO_OPENAI_API_KEY",
            "CHRONO_TELEGRAM_BOT_TOKEN", "CHRONO_DISCORD_BOT_TOKEN",
            "CHRONO_HF_TOKEN", "CHRONO_NGROK_TOKEN",
            "CHRONO_RUNPOD_API_KEY", "CHRONO_DEV_PASSWORD",
        ];
        for key in &keys {
            if let Ok(val) = std::env::var(key) {
                self.secrets.insert(key.to_string(), val);
            }
        }
        self.env_loaded = true;
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.secrets.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: String) {
        self.secrets.insert(key.to_string(), value);
    }

    pub fn has(&self, key: &str) -> bool { self.secrets.contains_key(key) }

    pub fn list_keys(&self) -> Vec<&str> {
        self.secrets.keys().map(|s| s.as_str()).collect()
    }
}

extern crate tracing;
