//! Consistency System — Game Bible, narrative tracking, continuity, AI content validation
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ── Game Bible ────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct GameBible {
    pub title:          String,
    pub tone:           String,
    pub themes:         Vec<String>,
    pub visual_style:   String,
    pub audio_style:    String,
    pub lore:           HashMap<String,String>,
    pub world_rules:    Vec<WorldRule>,
    pub character_rules:HashMap<String,CharacterRule>,
    pub forbidden_content: Vec<String>,
    pub required_elements: Vec<String>,
    pub glossary:       HashMap<String,String>,
    pub timeline:       Vec<TimelineEntry>,
    pub factions:       HashMap<String,FactionProfile>,
    pub locations:      HashMap<String,LocationProfile>,
    pub magic_system:   Option<MagicSystem>,
    pub tech_level:     String,
    pub version:        u32,
    pub last_modified:  Option<DateTime<Utc>>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WorldRule {
    pub id:          String,
    pub rule:        String,
    pub rationale:   String,
    pub hard:        bool,   // hard=always enforced, soft=AI can break if justified
    pub category:    String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CharacterRule {
    pub character_id:    String,
    pub name:            String,
    pub appearance:      String,
    pub appearance_locked: bool,
    pub personality_core:  Vec<String>,
    pub speech_patterns:   Vec<String>,
    pub forbidden_actions: Vec<String>,
    pub relationships:     HashMap<String,String>,
    pub secrets:           Vec<String>,
    pub arc:               String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TimelineEntry {
    pub year:    String,
    pub event:   String,
    pub impact:  String,
    pub public:  bool,   // player can know vs hidden lore
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FactionProfile {
    pub id:        String,
    pub name:      String,
    pub ideology:  String,
    pub goals:     Vec<String>,
    pub methods:   Vec<String>,
    pub enemies:   Vec<String>,
    pub allies:    Vec<String>,
    pub resources: Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LocationProfile {
    pub id:          String,
    pub name:        String,
    pub description: String,
    pub atmosphere:  String,
    pub inhabitants: Vec<String>,
    pub secrets:     Vec<String>,
    pub connected_to:Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MagicSystem {
    pub name:        String,
    pub source:      String,
    pub rules:       Vec<String>,
    pub costs:       Vec<String>,
    pub limits:      Vec<String>,
    pub forbidden:   Vec<String>,
    pub schools:     Vec<String>,
}

// ── Narrative Tracker ─────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct NarrativeTracker {
    pub flags:           HashMap<String,bool>,
    pub variables:       HashMap<String,serde_json::Value>,
    pub quests:          HashMap<String,QuestRecord>,
    pub choices:         Vec<PlayerChoice>,
    pub factions:        HashMap<String,i32>,
    pub npc_attitudes:   HashMap<String,i32>,
    pub completed_arcs:  Vec<String>,
    pub active_arcs:     Vec<String>,
    pub world_state:     HashMap<String,String>,
    pub playtime_secs:   f64,
    pub choice_count:    u64,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct QuestRecord {
    pub id:          String,
    pub status:      QuestStatus,
    pub objective:   Option<String>,
    pub started:     Option<DateTime<Utc>>,
    pub completed:   Option<DateTime<Utc>>,
    pub flags:       HashMap<String,bool>,
    pub choices:     Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum QuestStatus { Unknown, Active, Complete, Failed, Abandoned }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlayerChoice {
    pub id:        String,
    pub context:   String,
    pub options:   Vec<String>,
    pub chosen:    String,
    pub ts:        DateTime<Utc>,
    pub consequence: Option<String>,
}

impl NarrativeTracker {
    pub fn set_flag(&mut self, k:&str, v:bool) { self.flags.insert(k.to_string(), v); }
    pub fn flag(&self, k:&str) -> bool { *self.flags.get(k).unwrap_or(&false) }
    pub fn set_var<V:serde::Serialize>(&mut self, k:&str, v:V) {
        self.variables.insert(k.to_string(), serde_json::to_value(v).unwrap_or_default());
    }
    pub fn get_var<V:serde::de::DeserializeOwned>(&self, k:&str) -> Option<V> {
        self.variables.get(k).and_then(|v|serde_json::from_value(v.clone()).ok())
    }
    pub fn start_quest(&mut self, id:&str) {
        self.quests.insert(id.to_string(), QuestRecord {
            id:id.to_string(), status:QuestStatus::Active, objective:None,
            started:Some(Utc::now()), completed:None, flags:HashMap::new(), choices:Vec::new(),
        });
    }
    pub fn complete_quest(&mut self, id:&str) {
        if let Some(q) = self.quests.get_mut(id) {
            q.status = QuestStatus::Complete;
            q.completed = Some(Utc::now());
        }
    }
    pub fn record_choice(&mut self, ctx:&str, options:Vec<String>, chosen:&str) {
        self.choices.push(PlayerChoice {
            id: format!("choice_{}", self.choice_count),
            context: ctx.to_string(), options, chosen: chosen.to_string(),
            ts: Utc::now(), consequence: None,
        });
        self.choice_count += 1;
    }
    pub fn shift_faction(&mut self, faction:&str, delta:i32) {
        let cur = *self.factions.get(faction).unwrap_or(&0);
        self.factions.insert(faction.to_string(), (cur+delta).clamp(-100,100));
    }
    pub fn quest_status(&self, id:&str) -> Option<&QuestStatus> {
        self.quests.get(id).map(|q|&q.status)
    }
    pub fn active_quests(&self) -> Vec<&QuestRecord> {
        self.quests.values().filter(|q|matches!(q.status,QuestStatus::Active)).collect()
    }
}

// ── Violation ─────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Violation {
    pub id:        String,
    pub kind:      ViolationKind,
    pub severity:  Severity,
    pub rule_id:   Option<String>,
    pub desc:      String,
    pub context:   String,
    pub ts:        DateTime<Utc>,
    pub auto_fixed:bool,
    pub suggestion:Option<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ViolationKind {
    ForbiddenContent{content:String},
    CharacterOOC{character:String,action:String},
    WorldRuleBroken{rule:String},
    ContinuityBreak{what:String},
    ToneViolation{expected:String,got:String},
    Anachronism{item:String},
    Custom(String),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Severity { Info, Warning, Error, Critical }

// ── Consistency Manager ───────────────────────────────────────────
pub struct ConsistencyManager {
    pub bible:       GameBible,
    pub narrative:   NarrativeTracker,
    pub violations:  Vec<Violation>,
    pub auto_fix:    bool,
    pub strict:      bool,
    pub checks_run:  u64,
    pub fixes_applied:u64,
}

impl ConsistencyManager {
    pub fn new(bible:GameBible) -> Self {
        Self { bible, narrative:NarrativeTracker::default(),
               violations:Vec::new(), auto_fix:true, strict:false,
               checks_run:0, fixes_applied:0 }
    }

    pub fn check_content(&mut self, content:&str, ctx:&str) -> Vec<Violation> {
        self.checks_run += 1;
        let mut vs = Vec::new();
        let cl = content.to_lowercase();

        // Forbidden content check
        for f in &self.bible.forbidden_content {
            if cl.contains(&f.to_lowercase()) {
                vs.push(Violation {
                    id: format!("v_{}", self.violations.len()),
                    kind: ViolationKind::ForbiddenContent{content:f.clone()},
                    severity: Severity::Error,
                    rule_id: None,
                    desc: format!("Forbidden content '{}' found in {}", f, ctx),
                    context: ctx.to_string(),
                    ts: Utc::now(), auto_fixed: false,
                    suggestion: Some(format!("Remove or rephrase references to '{}'", f)),
                });
            }
        }

        // Tone check (basic keyword matching)
        if !self.bible.tone.is_empty() {
            let tone = self.bible.tone.to_lowercase();
            if tone.contains("dark") && (cl.contains("whimsical") || cl.contains("silly")) {
                vs.push(Violation {
                    id: format!("v_{}", self.violations.len()+vs.len()),
                    kind: ViolationKind::ToneViolation{expected:self.bible.tone.clone(),got:"lighthearted".to_string()},
                    severity: Severity::Warning,
                    rule_id: None,
                    desc: format!("Tone mismatch in {}", ctx),
                    context: ctx.to_string(),
                    ts: Utc::now(), auto_fixed: false, suggestion: None,
                });
            }
        }

        self.violations.extend(vs.clone());
        vs
    }

    pub fn check_character(&mut self, char_id:&str, action:&str, ctx:&str) -> Option<Violation> {
        if let Some(rule) = self.bible.character_rules.get(char_id) {
            let al = action.to_lowercase();
            for forbidden in &rule.forbidden_actions {
                if al.contains(&forbidden.to_lowercase()) {
                    let v = Violation {
                        id: format!("v_{}", self.violations.len()),
                        kind: ViolationKind::CharacterOOC{character:rule.name.clone(),action:action.to_string()},
                        severity: Severity::Error,
                        rule_id: None,
                        desc: format!("{} cannot '{}'", rule.name, action),
                        context: ctx.to_string(),
                        ts: Utc::now(), auto_fixed: false,
                        suggestion: Some(format!("{} would instead...", rule.name)),
                    };
                    self.violations.push(v.clone());
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn set_flag(&mut self, k:&str, v:bool) { self.narrative.set_flag(k,v); }
    pub fn flag(&self, k:&str) -> bool { self.narrative.flag(k) }
    pub fn start_quest(&mut self, id:&str) { self.narrative.start_quest(id); }
    pub fn complete_quest(&mut self, id:&str) { self.narrative.complete_quest(id); }

    pub fn violation_count(&self) -> usize { self.violations.len() }
    pub fn error_count(&self) -> usize { self.violations.iter().filter(|v|matches!(v.severity,Severity::Error|Severity::Critical)).count() }
    pub fn clear_violations(&mut self) { self.violations.clear(); }
}

impl Default for ConsistencyManager {
    fn default() -> Self { Self::new(GameBible::default()) }
}
