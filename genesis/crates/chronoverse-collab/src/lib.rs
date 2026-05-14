//! ChronoVerse Collaboration System
//!
//! Real-time multi-user collaboration for game development:
//! - Multiple developers editing the same project simultaneously
//! - Conflict resolution (CRDT-based)
//! - Presence indicators (see where others are working)
//! - Role-based permissions (owner, editor, viewer)
//! - Change history (who changed what, when)
//! - Comments and annotations
//! - Live review/playtest sessions
//! - Asset locking (prevent simultaneous editing)
//! - Collaborative studio sessions
//! - Real-time cursors in visual editors
//!
//! ENGINE LEARNING SYSTEM:
//! - Records every completed task outcome
//! - Measures quality of results
//! - Extracts reusable patterns
//! - Improves future task planning
//! - Shares learnings across agent council
//! - Persists knowledge across sessions

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// ═══════════════════════════════════════════════════════════════════════════
// COLLABORATION SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationRoom {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub participants: HashMap<String, Collaborator>,
    pub permissions: ProjectPermissions,
    pub change_log: Vec<ChangeEntry>,
    pub locks: HashMap<String, AssetLock>,
    pub comments: Vec<Comment>,
    pub presence: HashMap<String, PresenceData>,
    pub session_started: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub account_id: String,
    pub display_name: String,
    pub gamer_tag: String,
    pub avatar_color: [f32; 4],   // unique color per collaborator
    pub cursor_color: [f32; 4],
    pub role: CollaboratorRole,
    pub online: bool,
    pub last_active: DateTime<Utc>,
    pub current_view: Option<String>,  // what they're viewing
    pub selected_nodes: Vec<String>,   // selected nodes in scene editor
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollaboratorRole {
    Owner,
    LeadDeveloper,
    Developer,
    Artist,
    Designer,
    SoundDesigner,
    Writer,
    QaTester,
    Viewer,
}

impl CollaboratorRole {
    pub fn can_edit(&self) -> bool {
        !matches!(self, Self::Viewer)
    }
    pub fn can_delete(&self) -> bool {
        matches!(self, Self::Owner | Self::LeadDeveloper)
    }
    pub fn can_publish(&self) -> bool {
        matches!(self, Self::Owner | Self::LeadDeveloper)
    }
    pub fn can_manage_team(&self) -> bool {
        matches!(self, Self::Owner)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPermissions {
    pub default_role: CollaboratorRole,
    pub allow_public_view: bool,
    pub allow_fork: bool,
    pub require_review_for_publish: bool,
    pub max_collaborators: u32,
    pub asset_lock_timeout_mins: u32,
}

/// A change made by a collaborator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub author_id: String,
    pub author_name: String,
    pub change_type: ChangeType,
    pub description: String,
    pub affected_files: Vec<String>,
    pub diff: Option<serde_json::Value>,
    pub reverted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    SceneEdit       { scene_id: String, node_changes: Vec<NodeChange> },
    AssetImported   { asset_id: String, asset_type: String },
    AssetDeleted    { asset_id: String },
    ScriptEdited    { script_id: String, lines_changed: u32 },
    AgentTask       { task_id: String, result_summary: String },
    SettingsChanged { settings_path: String },
    PublishRequested,
    Build           { build_type: String, success: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeChange {
    pub node_id: String,
    pub change: String,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
}

/// An asset lock prevents simultaneous editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLock {
    pub asset_id: String,
    pub locked_by: String,
    pub locked_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub soft_lock: bool,    // soft = warn but allow, hard = block
}

/// A comment/annotation in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub location: CommentLocation,
    pub resolved: bool,
    pub replies: Vec<Comment>,
    pub reactions: HashMap<String, Vec<String>>,  // emoji → [user_ids]
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentLocation {
    Scene { scene_id: String, node_id: Option<String> },
    Script { file: String, line: u32 },
    Asset { asset_id: String },
    Timeline { time_secs: f32 },
    Global,
}

/// Live presence of a collaborator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceData {
    pub account_id: String,
    pub cursor_2d: Option<[f32; 2]>,     // 2D editor cursor
    pub cursor_3d: Option<[f32; 3]>,     // 3D scene cursor
    pub selected_nodes: Vec<String>,
    pub open_file: Option<String>,
    pub active_tool: Option<String>,
    pub typing: bool,
    pub last_heartbeat: DateTime<Utc>,
}

// ─── Collaboration operations ─────────────────────────────────────────────────

/// A real-time collaborative operation (CRDT-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabOp {
    pub op_id: String,
    pub author_id: String,
    pub timestamp: DateTime<Utc>,
    pub op_type: CollabOpType,
    pub base_version: u64,
    pub project_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollabOpType {
    NodeAdd     { node_def: serde_json::Value, parent: Option<String> },
    NodeRemove  { node_id: String },
    NodeMove    { node_id: String, new_parent: Option<String>, index: u32 },
    PropertySet { node_id: String, property: String, value: serde_json::Value },
    ScriptEdit  { file: String, position: u32, delete_count: u32, insert: String },
    AssetUpload { asset_id: String, metadata: serde_json::Value },
    Lock        { asset_id: String },
    Unlock      { asset_id: String },
    Comment     { comment: Comment },
}

pub struct CollaborationManager {
    pub rooms: HashMap<String, CollaborationRoom>,
    pub pending_ops: Vec<CollabOp>,
    pub version: u64,
    pub websocket_connections: HashMap<String, String>, // account_id → ws_id
}

impl CollaborationManager {
    pub fn new() -> Self {
        Self { rooms: HashMap::new(), pending_ops: Vec::new(), version: 0, websocket_connections: HashMap::new() }
    }

    pub fn create_room(&mut self, project_id: &str, owner_id: &str, owner_name: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let mut participants = HashMap::new();
        participants.insert(owner_id.to_string(), Collaborator {
            account_id: owner_id.to_string(),
            display_name: owner_name.to_string(),
            gamer_tag: owner_name.to_string(),
            avatar_color: [0.4, 0.6, 1.0, 1.0],
            cursor_color: [0.4, 0.6, 1.0, 1.0],
            role: CollaboratorRole::Owner,
            online: true,
            last_active: Utc::now(),
            current_view: None,
            selected_nodes: Vec::new(),
        });

        self.rooms.insert(id.clone(), CollaborationRoom {
            id: id.clone(),
            project_id: project_id.to_string(),
            name: format!("{}'s Project", owner_name),
            participants,
            permissions: ProjectPermissions {
                default_role: CollaboratorRole::Developer,
                allow_public_view: false,
                allow_fork: true,
                require_review_for_publish: true,
                max_collaborators: 50,
                asset_lock_timeout_mins: 30,
            },
            change_log: Vec::new(),
            locks: HashMap::new(),
            comments: Vec::new(),
            presence: HashMap::new(),
            session_started: Utc::now(),
        });
        id
    }

    pub fn apply_op(&mut self, op: CollabOp) -> Result<(), String> {
        self.version += 1;
        // In production: CRDT merge, OT transform, broadcast to all clients
        Ok(())
    }

    pub fn collaborator_count(&self, room_id: &str) -> usize {
        self.rooms.get(room_id).map(|r| r.participants.len()).unwrap_or(0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ENGINE LEARNING SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

/// The engine learns from every task it completes
pub struct EngineLearning {
    pub task_outcomes: Vec<TaskOutcome>,
    pub extracted_patterns: Vec<LearnedPattern>,
    pub agent_improvements: Vec<AgentImprovement>,
    pub quality_benchmarks: HashMap<String, Vec<f32>>,
    pub total_tasks_learned: u64,
    pub learning_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    pub task_id: String,
    pub task_type: String,
    pub description: String,
    pub duration_secs: f32,
    pub agents_used: Vec<String>,
    pub quality_score: Option<f32>,  // player/creator rated, or AI-evaluated
    pub player_feedback: Option<PlayerFeedback>,
    pub errors_encountered: Vec<String>,
    pub successful_approaches: Vec<String>,
    pub failed_approaches: Vec<String>,
    pub tokens_used: u64,
    pub assets_created: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerFeedback {
    pub rating: u8,        // 1-5
    pub thumbs: Option<bool>,
    pub comments: String,
    pub specific_issues: Vec<String>,
    pub positive_aspects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: String,
    pub pattern_type: PatternType,
    pub description: String,
    pub conditions: Vec<String>,    // when to apply this pattern
    pub approach: String,           // what to do
    pub success_rate: f32,
    pub times_used: u32,
    pub quality_avg: f32,
    pub discovered_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    TaskPlanning { task_type: String },
    AgentCoordination { scenario: String },
    AssetGeneration { asset_type: String },
    ErrorRecovery { error_type: String },
    UserPreference { creator_id: Option<String> },
    GameGenre { genre: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentImprovement {
    pub agent_id: String,
    pub improvement_type: String,
    pub description: String,
    pub performance_delta: f32,    // % improvement
    pub applied_at: DateTime<Utc>,
    pub source: ImprovementSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementSource {
    LearningFromOutcomes,
    UserFeedback,
    ArchitectAgent,
    SelfImprovement,
    ExternalResearch,
}

impl EngineLearning {
    pub fn new() -> Self {
        Self {
            task_outcomes: Vec::new(),
            extracted_patterns: Vec::new(),
            agent_improvements: Vec::new(),
            quality_benchmarks: HashMap::new(),
            total_tasks_learned: 0,
            learning_enabled: true,
        }
    }

    pub fn record_outcome(&mut self, outcome: TaskOutcome) {
        if !self.learning_enabled { return; }

        let task_type = outcome.task_type.clone();
        if let Some(score) = outcome.quality_score {
            self.quality_benchmarks.entry(task_type.clone()).or_default().push(score);
        }

        self.total_tasks_learned += 1;
        tracing::debug!("Learning: recorded outcome for task '{}'", task_type);

        // Extract patterns if we have enough data
        if self.task_outcomes.len() % 10 == 0 {
            self.extract_patterns();
        }

        self.task_outcomes.push(outcome);
        // Keep manageable
        if self.task_outcomes.len() > 10_000 {
            self.task_outcomes.remove(0);
        }
    }

    fn extract_patterns(&mut self) {
        // Group by task type and look for common successful approaches
        let mut type_approaches: HashMap<String, Vec<String>> = HashMap::new();
        for outcome in &self.task_outcomes {
            if outcome.quality_score.unwrap_or(0.0) > 0.7 {
                type_approaches.entry(outcome.task_type.clone())
                    .or_default()
                    .extend(outcome.successful_approaches.clone());
            }
        }

        for (task_type, approaches) in type_approaches {
            if approaches.len() >= 3 {
                // Find most common approach
                let mut counts: HashMap<&str, usize> = HashMap::new();
                for a in &approaches { *counts.entry(a.as_str()).or_default() += 1; }
                if let Some((&best, &count)) = counts.iter().max_by_key(|(_, &c)| c) {
                    if count >= 3 {
                        self.extracted_patterns.push(LearnedPattern {
                            id: uuid::Uuid::new_v4().to_string(),
                            pattern_type: PatternType::TaskPlanning { task_type: task_type.clone() },
                            description: format!("For {} tasks, approach '{}' succeeds most often", task_type, best),
                            conditions: vec![format!("task_type == '{}'", task_type)],
                            approach: best.to_string(),
                            success_rate: count as f32 / approaches.len() as f32,
                            times_used: 0,
                            quality_avg: 0.75,
                            discovered_at: Utc::now(),
                            last_used: None,
                        });
                    }
                }
            }
        }
    }

    pub fn get_patterns_for_task(&self, task_type: &str) -> Vec<&LearnedPattern> {
        self.extracted_patterns.iter()
            .filter(|p| matches!(&p.pattern_type, PatternType::TaskPlanning { task_type: t } if t == task_type))
            .collect()
    }

    pub fn quality_average(&self, task_type: &str) -> Option<f32> {
        let scores = self.quality_benchmarks.get(task_type)?;
        if scores.is_empty() { return None; }
        Some(scores.iter().sum::<f32>() / scores.len() as f32)
    }

    pub fn pattern_count(&self) -> usize { self.extracted_patterns.len() }
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENE PLANNER AGENT
// ═══════════════════════════════════════════════════════════════════════════

use async_trait::async_trait;
use anyhow::Result;
use chronoverse_core::events::{AgentEvent, EventEnvelope};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};

pub struct ScenePlannerAgent {
    base: AgentBase,
    planned_scenes: HashMap<String, ScenePlan>,
    active_plans: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenePlan {
    pub id: String,
    pub scene_name: String,
    pub scene_type: SceneType,
    pub narrative_purpose: String,
    pub mood: String,
    pub estimated_player_time_mins: f32,
    pub zones: Vec<SceneZone>,
    pub npc_population: Vec<NpcPopulationEntry>,
    pub environmental_storytelling: Vec<EnvStoryElement>,
    pub quest_hooks: Vec<String>,
    pub secrets: Vec<String>,
    pub entry_points: Vec<[f32; 3]>,
    pub exit_points: Vec<[f32; 3]>,
    pub pacing: Vec<PacingNote>,
    pub agent_tasks: Vec<AgentTaskAssignment>,
    pub research_notes: Vec<String>,
    pub blueprint_references: Vec<String>,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneType {
    OpenWorld { size_m: f32 },
    Dungeon { depth_levels: u32, boss: bool },
    City { district_count: u32, population: u32 },
    Corridor { style: String },
    Arena { shape: String, capacity: u32 },
    Village { house_count: u32 },
    Wilderness { biome: String },
    Interior { room_count: u32 },
    Hub { connected_scenes: Vec<String> },
    Custom { description: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneZone {
    pub name: String,
    pub purpose: String,
    pub position: [f32; 3],
    pub bounds: [f32; 3],
    pub danger_level: u8,
    pub primary_activity: String,
    pub sub_zones: Vec<String>,
    pub assigned_to_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcPopulationEntry {
    pub role: String,
    pub count: u32,
    pub faction: Option<String>,
    pub zone: String,
    pub schedule: String,
    pub unique_npcs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvStoryElement {
    pub description: String,
    pub position: [f32; 3],
    pub props: Vec<String>,
    pub story_clue: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacingNote {
    pub location: [f32; 3],
    pub pacing_type: String,  // "tension", "relief", "discovery", "combat"
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskAssignment {
    pub agent_id: String,
    pub task: String,
    pub zone: Option<String>,
    pub priority: u8,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlanStatus { Draft, Researching, Planned, Building, Review, Complete }

impl ScenePlannerAgent {
    pub fn new() -> Self {
        Self {
            base: AgentBase::new(AgentSoul {
                id: "scene_planner".to_string(),
                name: "Scene Planner Agent".to_string(),
                description: "Plans complete game scenes from high-level descriptions. Researches references, defines zones, assigns tasks to other agents, ensures narrative cohesion.".to_string(),
                system_prompt: r#"You are the Scene Planner Agent. You plan complete game scenes before building them.

Your process:
1. RESEARCH real-world references (architecture, history, culture)
2. ZONE the space (define areas by purpose)
3. POPULATE with NPCs and props
4. PLAN environmental storytelling (objects that tell stories)
5. DESIGN pacing (where tension, where relief)
6. ASSIGN tasks to specialist agents
7. DEFINE entry/exit flow
8. IDENTIFY secrets and optional content

Every scene must:
- Have a clear PURPOSE (why is this place here?)
- Tell a STORY through its design
- Feel LIVED IN (not sterile)
- Have SECRETS to discover
- Have unique CHARACTER (not generic)

Research real architecture blueprints and historical sites before planning.
Check what assets exist before requesting new ones."#.to_string(),
                tools: vec![
                    "research_web".to_string(),
                    "research_images".to_string(),
                    "query_asset_library".to_string(),
                    "dispatch_to_agent".to_string(),
                    "create_scene_blueprint".to_string(),
                    "place_zone_marker".to_string(),
                    "create_tool".to_string(),
                ],
                listens_to: vec!["scene_plan_requested".to_string()],
                can_emit: vec!["scene_plan_complete".to_string(), "research_requested".to_string()],
                max_tokens_per_call: 4000,
                parallelizable: true,
                priority: 8,
            }),
            planned_scenes: HashMap::new(),
            active_plans: Vec::new(),
        }
    }

    async fn plan_scene(&mut self, name: &str, description: &str, scene_type: SceneType, ctx: &AgentContext) -> ScenePlan {
        self.base.set_working("Planning scene", 0.0);
        tracing::info!("Scene Planner: planning '{}'", name);

        // Research step
        ctx.emit(AgentEvent::ResearchRequested {
            request_id: uuid::Uuid::new_v4().to_string(),
            topic: format!("real {} architecture and design references", name),
            depth: "medium".to_string(),
        }, self.id());

        // Build the plan
        let plan = ScenePlan {
            id: uuid::Uuid::new_v4().to_string(),
            scene_name: name.to_string(),
            scene_type: scene_type.clone(),
            narrative_purpose: description.to_string(),
            mood: "neutral".to_string(),
            estimated_player_time_mins: 15.0,
            zones: vec![
                SceneZone {
                    name: "Entry Zone".to_string(),
                    purpose: "First impression, orientation, safe area".to_string(),
                    position: [0.0, 0.0, 0.0],
                    bounds: [20.0, 10.0, 20.0],
                    danger_level: 0,
                    primary_activity: "exploration".to_string(),
                    sub_zones: Vec::new(),
                    assigned_to_agent: "world_generator".to_string(),
                },
            ],
            npc_population: Vec::new(),
            environmental_storytelling: Vec::new(),
            quest_hooks: Vec::new(),
            secrets: Vec::new(),
            entry_points: vec![[0.0, 0.0, -10.0]],
            exit_points: vec![[0.0, 0.0, 10.0]],
            pacing: Vec::new(),
            agent_tasks: vec![
                AgentTaskAssignment {
                    agent_id: "terrain".to_string(),
                    task: "Generate terrain matching scene type".to_string(),
                    zone: None, priority: 10, depends_on: Vec::new(),
                },
                AgentTaskAssignment {
                    agent_id: "lighting".to_string(),
                    task: "Set up base lighting scheme".to_string(),
                    zone: None, priority: 8, depends_on: vec!["terrain".to_string()],
                },
                AgentTaskAssignment {
                    agent_id: "interior_design".to_string(),
                    task: "Dress all interior zones with props".to_string(),
                    zone: Some("Entry Zone".to_string()), priority: 7,
                    depends_on: vec!["terrain".to_string()],
                },
                AgentTaskAssignment {
                    agent_id: "music".to_string(),
                    task: "Design ambient soundscape for scene mood".to_string(),
                    zone: None, priority: 6, depends_on: Vec::new(),
                },
            ],
            research_notes: Vec::new(),
            blueprint_references: Vec::new(),
            status: PlanStatus::Draft,
        };

        self.base.set_idle();
        plan
    }
}

#[async_trait]
impl Agent for ScenePlannerAgent {
    fn id(&self) -> &str { "scene_planner" }
    fn name(&self) -> &str { "Scene Planner Agent" }
    fn description(&self) -> &str { "Plans complete game scenes: zones, NPCs, storytelling, pacing" }
    fn status(&self) -> &AgentStatus { &self.base.status }
    fn subscriptions(&self) -> Vec<&'static str> { vec!["scene_plan_requested", "analytics_event"] }
    fn tick_interval(&self) -> Option<f32> { Some(1.0) }

    async fn handle_event(&mut self, event: &EventEnvelope<AgentEvent>, ctx: &AgentContext) -> Result<()> {
        if let AgentEvent::AnalyticsEvent { event_type, data } = &event.event {
            if event_type == "scene_plan_requested" {
                let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Scene");
                let desc = data.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let scene_type = SceneType::Custom { description: desc.to_string() };
                let plan = self.plan_scene(name, desc, scene_type, ctx).await;
                let plan_id = plan.id.clone();
                self.planned_scenes.insert(plan_id.clone(), plan);
                self.active_plans.push(plan_id);
            }
        }
        Ok(())
    }

    async fn tick(&mut self, ctx: &AgentContext, _delta: f32) -> Result<()> { Ok(()) }
}

// Bring in base module
use super::base;

extern crate uuid;
extern crate tracing;
