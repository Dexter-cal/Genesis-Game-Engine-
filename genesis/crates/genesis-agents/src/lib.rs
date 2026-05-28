//! Agent Council — 32 AI agents, task system, memory, token budgets, tool calling
use serde::{Serialize,Deserialize};
use std::collections::{HashMap,VecDeque};
use chrono::{DateTime,Utc};

pub mod holo;
pub mod self_improve;
pub mod code_fix;
pub mod research;
pub mod collaboration;
pub mod explainer;

use crate::holo::HoloStack;
use crate::self_improve::SelfImproveAgent;

// ═══ AGENT TYPES ══════════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum AgentKind {
    // Orchestration
    MasterOrchestrator,
    // World & Environment
    WorldGenerator, InteriorDesigner, ArchitectAgent,
    LightingAgent, WeatherAgent,
    // Characters & Story
    NpcBrain, StoryAgent, DialogueAgent, QuestAgent,
    FactionAgent, ConsistencyAgent,
    // Combat & Gameplay
    CombatAgent, EnemyDesigner, BossDesigner, BalanceAgent,
    // Creative
    MusicAgent, AudioAgent, VfxAgent, AnimationAgent,
    AiModeler, ShadingAgent,
    // Production
    CinematicsAgent, CameraAgent, DirectorAgent,
    // Pipeline
    RetopologyAgent, UvAgent, MaterialAgent, RenderAgent,
    ExportAgent, PipelineAgent,
    // Meta
    ResearchAgent, SelfImproveAgent, OrganisationAgent,
    // User-defined
    Custom(String),
}

impl AgentKind {
    pub fn name(&self) -> &str {
        match self {
            Self::MasterOrchestrator => "Master Orchestrator",
            Self::WorldGenerator     => "World Generator",
            Self::InteriorDesigner   => "Interior Designer",
            Self::ArchitectAgent     => "Architect",
            Self::LightingAgent      => "Lighting Artist",
            Self::WeatherAgent       => "Weather Agent",
            Self::NpcBrain           => "NPC Brain",
            Self::StoryAgent         => "Story Agent",
            Self::DialogueAgent      => "Dialogue Writer",
            Self::QuestAgent         => "Quest Designer",
            Self::FactionAgent       => "Faction Agent",
            Self::ConsistencyAgent   => "Consistency Guard",
            Self::CombatAgent        => "Combat Agent",
            Self::EnemyDesigner      => "Enemy Designer",
            Self::BossDesigner       => "Boss Designer",
            Self::BalanceAgent       => "Balance Agent",
            Self::MusicAgent         => "Music Composer",
            Self::AudioAgent         => "Audio Designer",
            Self::VfxAgent           => "VFX Artist",
            Self::AnimationAgent     => "Animator",
            Self::AiModeler          => "3D Modeler",
            Self::ShadingAgent       => "Shader Artist",
            Self::CinematicsAgent    => "Cinematics Director",
            Self::CameraAgent        => "Camera Agent",
            Self::DirectorAgent      => "AI Director",
            Self::RetopologyAgent    => "Retopology Agent",
            Self::UvAgent            => "UV Agent",
            Self::MaterialAgent      => "Material Agent",
            Self::RenderAgent        => "Render Agent",
            Self::ExportAgent        => "Export Agent",
            Self::PipelineAgent      => "Pipeline Manager",
            Self::ResearchAgent      => "Research Agent",
            Self::SelfImproveAgent   => "Self-Improve Agent",
            Self::OrganisationAgent  => "Organisation Agent",
            Self::Custom(s)          => s,
        }
    }

    pub fn group(&self) -> &'static str {
        match self {
            Self::MasterOrchestrator                       => "Orchestration",
            Self::WorldGenerator|Self::InteriorDesigner|
            Self::ArchitectAgent|Self::LightingAgent|
            Self::WeatherAgent                             => "World & Environment",
            Self::NpcBrain|Self::StoryAgent|
            Self::DialogueAgent|Self::QuestAgent|
            Self::FactionAgent|Self::ConsistencyAgent      => "Characters & Story",
            Self::CombatAgent|Self::EnemyDesigner|
            Self::BossDesigner|Self::BalanceAgent          => "Combat & Gameplay",
            Self::MusicAgent|Self::AudioAgent|Self::VfxAgent|
            Self::AnimationAgent|Self::AiModeler|
            Self::ShadingAgent                             => "Creative",
            Self::CinematicsAgent|Self::CameraAgent|
            Self::DirectorAgent                            => "Production",
            Self::RetopologyAgent|Self::UvAgent|
            Self::MaterialAgent|Self::RenderAgent|
            Self::ExportAgent|Self::PipelineAgent          => "Pipeline",
            Self::ResearchAgent|Self::SelfImproveAgent|
            Self::OrganisationAgent                        => "Meta",
            Self::Custom(_)                                => "Custom",
        }
    }
}

// ═══ TASK SYSTEM ══════════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum TaskStatus { Pending, Running, Paused, Complete, Failed, Cancelled }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AgentTask {
    pub id:           String,
    pub title:        String,
    pub description:  String,
    pub agent_kind:   AgentKind,
    pub assigned_to:  Option<String>,
    pub status:       TaskStatus,
    pub priority:     u32,
    pub progress:     f32,
    pub subtasks:     Vec<AgentTask>,
    pub dependencies: Vec<String>,
    pub result:       Option<serde_json::Value>,
    pub error:        Option<String>,
    pub tools_used:   Vec<String>,
    pub tokens_used:  u32,
    pub cost_usd:     f64,
    pub created_at:   DateTime<Utc>,
    pub started_at:   Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub timeout_secs: Option<f32>,
    pub retries:      u32,
    pub max_retries:  u32,
}

impl AgentTask {
    pub fn new(title:&str,desc:&str,kind:AgentKind) -> Self {
        Self {
            id: format!("task_{}", Utc::now().timestamp_millis()),
            title: title.to_string(), description: desc.to_string(),
            agent_kind: kind, assigned_to: None, status: TaskStatus::Pending,
            priority: 5, progress: 0.0, subtasks: Vec::new(),
            dependencies: Vec::new(), result: None, error: None,
            tools_used: Vec::new(), tokens_used: 0, cost_usd: 0.0,
            created_at: Utc::now(), started_at: None, completed_at: None,
            timeout_secs: Some(120.0), retries: 0, max_retries: 3,
        }
    }
    pub fn is_done(&self)->bool{ matches!(self.status, TaskStatus::Complete|TaskStatus::Failed|TaskStatus::Cancelled) }
    pub fn duration_secs(&self)->f64{
        match (self.started_at, self.completed_at) {
            (Some(s),Some(e)) => (e-s).num_milliseconds() as f64/1000.0,
            (Some(s),None)    => (Utc::now()-s).num_milliseconds() as f64/1000.0,
            _ => 0.0,
        }
    }
}

// ═══ AGENT MEMORY ════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AgentMemory {
    pub short_term:   VecDeque<MemoryEntry>,   // last N interactions
    pub long_term:    Vec<MemoryEntry>,         // persisted important facts
    pub working:      HashMap<String,serde_json::Value>, // current task context
    pub short_term_cap: usize,
    pub total_entries:  u64,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MemoryEntry {
    pub id:        String,
    pub kind:      MemoryKind,
    pub content:   String,
    pub embedding: Vec<f32>,    // for semantic search (empty until embedded)
    pub relevance: f32,
    pub ts:        DateTime<Utc>,
    pub source:    String,      // which task/agent created this
    pub pinned:    bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MemoryKind {
    Fact, Decision, Observation, Tool, Feedback, Error, WorldState, Custom(String)
}

impl AgentMemory {
    pub fn new(cap:usize) -> Self {
        Self { short_term:VecDeque::new(), long_term:Vec::new(),
               working:HashMap::new(), short_term_cap:cap, total_entries:0 }
    }
    pub fn push(&mut self, kind:MemoryKind, content:&str, source:&str) {
        let entry = MemoryEntry {
            id: format!("mem_{}", self.total_entries),
            kind, content: content.to_string(), embedding: Vec::new(),
            relevance: 1.0, ts: Utc::now(), source: source.to_string(), pinned: false,
        };
        self.total_entries += 1;
        self.short_term.push_back(entry);
        while self.short_term.len() > self.short_term_cap { self.short_term.pop_front(); }
    }
    pub fn pin(&mut self, id:&str) {
        if let Some(e) = self.short_term.iter_mut().find(|e|e.id==id) {
            let mut cloned = e.clone(); cloned.pinned = true;
            self.long_term.push(cloned);
        }
    }
    pub fn recent(&self, n:usize) -> impl Iterator<Item=&MemoryEntry> {
        self.short_term.iter().rev().take(n)
    }
    pub fn set_working(&mut self, k:&str, v:serde_json::Value) { self.working.insert(k.to_string(),v); }
    pub fn get_working(&self, k:&str) -> Option<&serde_json::Value> { self.working.get(k) }
    pub fn clear_working(&mut self) { self.working.clear(); }
    pub fn total(&self)->u64 { self.total_entries }
}

// ═══ TOKEN BUDGET ════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TokenBudget {
    pub daily_limit:     u64,
    pub hourly_limit:    u64,
    pub used_today:      u64,
    pub used_this_hour:  u64,
    pub cost_today_usd:  f64,
    pub cost_limit_usd:  f64,
    pub alert_pct:       f32,
    pub last_reset_day:  u32,
    pub last_reset_hour: u32,
    pub total_all_time:  u64,
}

impl TokenBudget {
    pub fn new(daily:u64, cost_limit:f64) -> Self {
        Self { daily_limit:daily, hourly_limit:daily/24, used_today:0, used_this_hour:0,
               cost_today_usd:0.0, cost_limit_usd:cost_limit, alert_pct:0.8,
               last_reset_day:0, last_reset_hour:0, total_all_time:0 }
    }
    pub fn consume(&mut self, tokens:u32, cost:f64) -> bool {
        if self.used_today + tokens as u64 > self.daily_limit { return false; }
        if self.cost_today_usd + cost > self.cost_limit_usd { return false; }
        self.used_today += tokens as u64;
        self.used_this_hour += tokens as u64;
        self.cost_today_usd += cost;
        self.total_all_time += tokens as u64;
        true
    }
    pub fn remaining_today(&self) -> u64 { self.daily_limit.saturating_sub(self.used_today) }
    pub fn usage_pct(&self) -> f32 { self.used_today as f32 / self.daily_limit as f32 }
    pub fn is_low(&self) -> bool { self.usage_pct() >= self.alert_pct }
    pub fn reset_daily(&mut self) { self.used_today=0; self.cost_today_usd=0.0; }
    pub fn reset_hourly(&mut self) { self.used_this_hour=0; }
}

// ═══ TOOL REGISTRY ═══════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AgentTool {
    pub name:        String,
    pub description: String,
    pub parameters:  serde_json::Value,  // JSON schema
    pub category:    ToolCategory,
    pub requires:    Vec<String>,        // permissions/capabilities needed
    pub cost_tokens: u32,                // estimated token cost to use
    pub timeout_ms:  u32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ToolCategory {
    SceneEdit, AssetOp, FileSystem, WebSearch, CodeExec,
    DatabaseQuery, ApiCall, UiInteraction, Memory, Custom(String),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ToolCall {
    pub tool:    String,
    pub args:    serde_json::Value,
    pub result:  Option<serde_json::Value>,
    pub error:   Option<String>,
    pub tokens:  u32,
    pub ms:      u32,
    pub ts:      DateTime<Utc>,
}

// ═══ AGENT INSTANCE ══════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum AgentStatus { Idle, Thinking, Working{task:String}, Waiting{for_:String}, Paused, Error(String) }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Agent {
    pub id:           String,
    pub kind:         AgentKind,
    pub status:       AgentStatus,
    pub enabled:      bool,
    pub memory:       AgentMemory,
    pub budget:       TokenBudget,
    pub current_task: Option<String>,
    pub task_queue:   VecDeque<String>,
    pub tool_history: VecDeque<ToolCall>,
    pub holo_stack:   HoloStack,
    pub model:        String,           // which LLM model to use
    pub provider:     String,           // which AI provider
    pub system_prompt:String,
    pub temperature:  f32,
    pub max_tokens:   u32,
    pub total_calls:  u64,
    pub total_errors: u32,
    pub avg_latency_ms: f32,
    pub created_at:   DateTime<Utc>,
    pub last_active:  Option<DateTime<Utc>>,
}

impl Agent {
    pub fn new(id:&str, kind:AgentKind, model:&str, provider:&str) -> Self {
        let system = format!(
            "You are the {} for the GENESIS game engine. \
             You have access to the full game scene, asset library, and other agents. \
             Always output valid JSON when structured output is needed. \
             Be concise but thorough. Prioritize game quality and playability.",
            kind.name()
        );
        Self {
            id: id.to_string(), kind, status: AgentStatus::Idle, enabled: true,
            memory: AgentMemory::new(50), budget: TokenBudget::new(100_000, 2.0),
            current_task: None, task_queue: VecDeque::new(), tool_history: VecDeque::new(),
            holo_stack: HoloStack::new(),
            model: model.to_string(), provider: provider.to_string(),
            system_prompt: system, temperature: 0.7, max_tokens: 2048,
            total_calls: 0, total_errors: 0, avg_latency_ms: 0.0,
            created_at: Utc::now(), last_active: None,
        }
    }

    pub fn assign_task(&mut self, task_id:&str) {
        self.task_queue.push_back(task_id.to_string());
        tracing::debug!("Task assigned to {}: {}", self.kind.name(), task_id);
    }

    pub fn start_next_task(&mut self) -> Option<String> {
        if let Some(id) = self.task_queue.pop_front() {
            self.current_task = Some(id.clone());
            self.status = AgentStatus::Working { task: id.clone() };
            self.last_active = Some(Utc::now());
            self.total_calls += 1;
            Some(id)
        } else { None }
    }

    pub fn complete_task(&mut self, tokens:u32, cost:f64, latency_ms:f32) {
        self.budget.consume(tokens, cost);
        self.current_task = None;
        self.status = AgentStatus::Idle;
        let n = self.total_calls as f32;
        self.avg_latency_ms = self.avg_latency_ms*(n-1.0)/n + latency_ms/n;
        self.memory.push(MemoryKind::Fact,
            &format!("Completed task using {}ms, {} tokens", latency_ms as u32, tokens),
            &self.id.clone());
    }

    pub fn fail_task(&mut self, error:&str) {
        self.total_errors += 1;
        self.current_task = None;
        self.status = AgentStatus::Error(error.to_string());
        self.memory.push(MemoryKind::Error, error, &self.id.clone());
        tracing::warn!("Agent {} failed: {}", self.kind.name(), error);
    }

    pub fn is_available(&self) -> bool {
        self.enabled && matches!(self.status, AgentStatus::Idle) && !self.budget.is_low()
    }

    pub fn queue_depth(&self) -> usize { self.task_queue.len() }
}

// ═══ AGENT COUNCIL ═══════════════════════════════════════════════
pub struct AgentCouncil {
    pub agents:        HashMap<String,Agent>,
    pub tasks:         HashMap<String,AgentTask>,
    pub tools:         HashMap<String,AgentTool>,
    pub orchestrator:  Option<String>,
    pub enabled:       bool,
    pub auto_assign:   bool,
    pub max_parallel:  u32,
    pub active_count:  u32,
    pub total_tasks:   u64,
    pub global_budget: TokenBudget,
    pub improver:      SelfImproveAgent,
}

impl AgentCouncil {
    pub fn new() -> Self {
        let mut council = Self {
            agents:HashMap::new(), tasks:HashMap::new(), tools:HashMap::new(),
            orchestrator:None, enabled:true, auto_assign:true, max_parallel:4,
            active_count:0, total_tasks:0,
            global_budget: TokenBudget::new(1_000_000, 10.0),
            improver: SelfImproveAgent::new(),
        };
        council.spawn_default_agents();
        council
    }

    fn spawn_default_agents(&mut self) {
        let agents = [
            ("orchestrator",  AgentKind::MasterOrchestrator,  "qwen2.5:7b"),
            ("world_gen",     AgentKind::WorldGenerator,       "qwen2.5:7b"),
            ("interior",      AgentKind::InteriorDesigner,     "qwen2.5:7b"),
            ("architect",     AgentKind::ArchitectAgent,       "qwen2.5:7b"),
            ("lighting",      AgentKind::LightingAgent,        "qwen2.5:7b"),
            ("weather",       AgentKind::WeatherAgent,         "qwen2.5:7b"),
            ("npc",           AgentKind::NpcBrain,             "qwen2.5:7b"),
            ("story",         AgentKind::StoryAgent,           "qwen2.5:7b"),
            ("dialogue",      AgentKind::DialogueAgent,        "qwen2.5:7b"),
            ("quest",         AgentKind::QuestAgent,           "qwen2.5:7b"),
            ("faction",       AgentKind::FactionAgent,         "qwen2.5:7b"),
            ("consistency",   AgentKind::ConsistencyAgent,     "qwen2.5:7b"),
            ("combat",        AgentKind::CombatAgent,          "qwen2.5:7b"),
            ("enemy",         AgentKind::EnemyDesigner,        "qwen2.5:7b"),
            ("boss",          AgentKind::BossDesigner,         "qwen2.5:7b"),
            ("balance",       AgentKind::BalanceAgent,         "qwen2.5:7b"),
            ("music",         AgentKind::MusicAgent,           "qwen2.5:7b"),
            ("audio",         AgentKind::AudioAgent,           "qwen2.5:7b"),
            ("vfx",           AgentKind::VfxAgent,             "qwen2.5:7b"),
            ("animation",     AgentKind::AnimationAgent,       "qwen2.5:7b"),
            ("modeler",       AgentKind::AiModeler,            "qwen2.5:7b"),
            ("shading",       AgentKind::ShadingAgent,         "qwen2.5:7b"),
            ("cinematics",    AgentKind::CinematicsAgent,      "qwen2.5:7b"),
            ("camera",        AgentKind::CameraAgent,          "qwen2.5:7b"),
            ("director",      AgentKind::DirectorAgent,        "qwen2.5:7b"),
            ("retopo",        AgentKind::RetopologyAgent,      "qwen2.5:7b"),
            ("uv",            AgentKind::UvAgent,              "qwen2.5:7b"),
            ("material",      AgentKind::MaterialAgent,        "qwen2.5:7b"),
            ("render",        AgentKind::RenderAgent,          "qwen2.5:7b"),
            ("export",        AgentKind::ExportAgent,          "qwen2.5:7b"),
            ("pipeline",      AgentKind::PipelineAgent,        "qwen2.5:7b"),
            ("research",      AgentKind::ResearchAgent,        "qwen2.5:7b"),
        ];

        for (id, kind, model) in agents {
            let agent = Agent::new(id, kind, model, "ollama");
            self.agents.insert(id.to_string(), agent);
        }
        self.orchestrator = Some("orchestrator".to_string());
        tracing::info!("Agent Council: {} agents spawned", self.agents.len());
    }

    pub fn submit_task(&mut self, mut task:AgentTask) -> String {
        let id = task.id.clone();
        self.total_tasks += 1;

        // Find available agent of the right kind
        if let Some(agent) = self.agents.values_mut()
            .find(|a| a.kind == task.agent_kind && a.is_available()) {
            task.assigned_to = Some(agent.id.clone());
            agent.assign_task(&id);
            tracing::info!("Task '{}' → agent '{}'", task.title, agent.kind.name());
        } else {
            tracing::warn!("No available agent for {:?} — task queued", task.agent_kind);
        }

        self.tasks.insert(id.clone(), task);
        id
    }

    pub fn get_agent(&self, id:&str) -> Option<&Agent> { self.agents.get(id) }
    pub fn get_agent_mut(&mut self, id:&str) -> Option<&mut Agent> { self.agents.get_mut(id) }
    pub fn get_task(&self, id:&str) -> Option<&AgentTask> { self.tasks.get(id) }

    pub fn idle_agents(&self) -> Vec<&Agent> {
        self.agents.values().filter(|a| a.is_available()).collect()
    }

    pub fn agent_count(&self) -> usize { self.agents.len() }
    pub fn enabled_count(&self) -> usize { self.agents.values().filter(|a|a.enabled).count() }
    pub fn active_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.status == TaskStatus::Running).count()
    }

    pub fn total_tokens(&self) -> u64 { self.agents.values().map(|a|a.budget.total_all_time).sum() }
    pub fn total_cost(&self) -> f64 { self.agents.values().map(|a|a.budget.cost_today_usd).sum() }

    pub fn tick(&mut self, _delta:f32) {
        if !self.enabled { return; }

        // Check for self-improvement opportunities
        let mut improvements = Vec::new();
        for agent in self.agents.values() {
            if let Some(imp) = self.improver.assess(agent) {
                improvements.push(imp);
            }
        }
        for imp in improvements {
            tracing::info!("Agent Improvement Proposed: {} -> {}", imp.agent_name, imp.assessment);
        }

        // Auto-start queued tasks for idle agents
        for agent in self.agents.values_mut() {
            if matches!(agent.status, AgentStatus::Idle) && !agent.task_queue.is_empty() {
                agent.start_next_task();
            }
        }
    }
}

impl Default for AgentCouncil { fn default() -> Self { Self::new() } }
extern crate tracing;
mod tests;
