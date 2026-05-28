//! Master Orchestrator Agent — The Brain of the Council
//!
//! The Master Orchestrator is the TOP-LEVEL coordinator.
//! Every major task flows through here.
//!
//! Responsibilities:
//! - Receive high-level goals from creator (via hub/chat/editor)
//! - Break goals into sub-tasks using planning
//! - Spawn sub-agents dynamically for specific tasks
//! - Route agent-to-agent requests (e.g. Interior → Lighting)
//! - Track all running tasks and their dependencies
//! - Handle task failures and retry/fallback
//! - Ensure ALL agents collaborate (not isolated silos)
//! - Research anything it doesn't know via ResearchAgent
//! - Every game created is UNIQUE — never reuse templates blindly
//!
//! Sub-agent spawning:
//! - Interior needs lighting → spawn LightingSubAgent
//! - Interior needs furniture → spawn AssetGeneratorSubAgent
//! - Sound for footsteps → spawn ProceduralSoundSubAgent
//! - Each sub-agent gets a specific scoped task
//!
//! Pipeline example for "design a medieval tavern interior":
//! 1. ResearchAgent scrapes web for real tavern layouts, 1400s architecture
//! 2. InteriorDesignAgent creates floor plan based on research
//! 3. AssetGeneratorAgent generates missing furniture (GPU Lab)
//! 4. LightingAgent places torches, fireplace, candles per plan
//! 5. SoundAgent assigns creaking floors, fire crackling, ambient crowd
//! 6. PropsAgent dresses with books, barrels, mugs, wanted posters
//! 7. NpcAgent populates with bartender, patrons with schedules
//! 8. WeatherAgent sets evening fog outside windows

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use chrono::{DateTime, Utc};

use genesis_core::events::{AgentEvent, EventEnvelope, EventPriority};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};
use crate::planning::{AgentPlan, PlanStep};

// ─── Task System ─────────────────────────────────────────────────────────────

/// A top-level creative task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub priority: u8,
    pub status: TaskStatus,
    pub plan: Option<TaskPlan>,
    pub sub_tasks: Vec<SubTask>,
    pub spawned_agents: Vec<SpawnedAgent>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub requested_by: String, // "creator", "player", "agent:interior_design", etc.
    pub result: Option<serde_json::Value>,
    pub errors: Vec<String>,
    /// Research results gathered for this task
    pub research: Vec<ResearchFinding>,
    /// Creativity constraints (ensure uniqueness)
    pub uniqueness_seed: u64,
    pub style_directives: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    // World building
    DesignInterior    { room_id: String, style: String },
    GenerateExterior  { area_id: String, biome: String },
    BuildDungeon      { dungeon_id: String, theme: String },
    CreateCity        { city_id: String, era: String, culture: String },
    DesignPOI         { poi_type: String, location: [f32; 3] },
    // Characters
    CreateNpc         { role: String, faction: String, uniqueness_prompt: String },
    DesignBoss        { boss_concept: String, arena_description: String },
    CreatePlayerCharacter { class: String, backstory: String },
    // Game systems
    ImplementMechanic { description: String, genre: String },
    DesignQuestChain  { theme: String, length: u32 },
    CreateGameMode    { description: String },
    // Assets
    GenerateAsset     { asset_type: String, description: String, style: String },
    // Audio
    DesignSoundscape  { scene_id: String, mood: String },
    // Free-form
    ExecuteDescription { text: String },
    Research          { topic: String, depth: String },
    ImproveEngine     { feature: String, specification: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Researching,
    Planning,
    InProgress { step: String, progress: f32 },
    AwaitingAgents,
    Verifying,
    Complete,
    Failed { reason: String },
    Cancelled,
}

/// A step-by-step plan for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub steps: Vec<TaskStep>,
    pub parallel_groups: Vec<Vec<String>>, // step IDs that can run in parallel
    pub estimated_duration_secs: f32,
    pub required_agents: Vec<String>,
    pub required_assets: Vec<String>,
    pub research_queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assigned_agent: String,
    pub depends_on: Vec<String>,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
    pub sub_agent_spawn: Option<SubAgentSpec>,
}

/// A spawned sub-agent for a specific task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnedAgent {
    pub id: String,
    pub agent_type: String,
    pub task_id: String,
    pub step_id: String,
    pub status: SubAgentStatus,
    pub context: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub spawned_at: DateTime<Utc>,
    pub ttl_secs: f32, // auto-despawn after this time
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubAgentStatus { Spawning, Running, Complete, Failed(String), Expired }

/// Specification for spawning a sub-agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentSpec {
    pub agent_type: String,
    pub task_description: String,
    pub tools: Vec<String>,
    pub context: serde_json::Value,
    pub max_tokens: u32,
    pub ttl_secs: f32,
}

/// A sub-task spawned from a parent task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub parent_task_id: String,
    pub description: String,
    pub assigned_to: String,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
}

/// Research finding from web search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchFinding {
    pub query: String,
    pub summary: String,
    pub sources: Vec<String>,
    pub key_insights: Vec<String>,
    pub images_found: Vec<String>,
}

// ─── Creativity Engine ────────────────────────────────────────────────────────

/// Ensures uniqueness across all generated content
pub struct CreativityEngine {
    /// Hashes of things already created (prevent duplicates)
    created_hashes: std::collections::HashSet<u64>,
    /// Style combinations used
    used_styles: Vec<(String, String)>,
    /// NPC names used
    used_names: std::collections::HashSet<String>,
    /// Building layouts used
    used_layouts: Vec<String>,
    rng_seed: u64,
}

impl CreativityEngine {
    pub fn new() -> Self {
        Self {
            created_hashes: std::collections::HashSet::new(),
            used_styles: Vec::new(),
            used_names: std::collections::HashSet::new(),
            used_layouts: Vec::new(),
            rng_seed: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
        }
    }

    /// Generate uniqueness directives for a task
    pub fn uniqueness_directives(&mut self, task_type: &TaskType) -> Vec<String> {
        let mut directives = Vec::new();

        match task_type {
            TaskType::CreateNpc { role, .. } => {
                directives.push(format!(
                    "This {} must be unlike any previously created. Give them an unexpected quirk, \
                     unusual appearance feature, or surprising backstory element that makes them \
                     memorable. Avoid clichés for their role.",
                    role
                ));
                directives.push("Consider: What does this person want MORE than anything? \
                    What are they afraid of? What secret are they hiding?".to_string());
                directives.push("Their voice, mannerisms, and vocabulary should reflect \
                    their specific cultural background, not generic fantasy speech.".to_string());
            }
            TaskType::DesignInterior { style, .. } => {
                directives.push(format!(
                    "This {} interior must tell a specific person's story through objects. \
                     No two rooms should look alike. Include at least one element that \
                     surprises the player.",
                    style
                ));
                directives.push("Research real examples before designing. \
                    Use actual architectural principles.".to_string());
            }
            TaskType::DesignBoss { boss_concept, .. } => {
                directives.push(format!(
                    "This boss ({}) should have a fight mechanic the player has \
                     NEVER seen before. Do not use generic attack patterns. \
                     The boss's attacks should visually communicate their personality.",
                    boss_concept
                ));
            }
            _ => {
                directives.push(
                    "Create something genuinely unique. Research real-world references. \
                     Avoid generic/default choices. Be specific and creative.".to_string()
                );
            }
        }

        directives
    }

    /// Generate a unique seed for this task
    pub fn next_seed(&mut self) -> u64 {
        self.rng_seed = self.rng_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.rng_seed
    }

    /// Has this content type been generated before?
    pub fn is_duplicate(&self, hash: u64) -> bool {
        self.created_hashes.contains(&hash)
    }

    pub fn register_created(&mut self, hash: u64) {
        self.created_hashes.insert(hash);
    }
}

// ─── Agent Router ─────────────────────────────────────────────────────────────

/// Maps task types and steps to the best agent
pub struct AgentRouter {
    /// agent_id → capability list
    capabilities: HashMap<String, Vec<String>>,
    /// Current load per agent (0-1)
    load: HashMap<String, f32>,
}

impl AgentRouter {
    pub fn new() -> Self {
        let mut capabilities = HashMap::new();

        capabilities.insert("interior_design".to_string(), vec![
            "room_layout", "furniture_placement", "style_matching", "space_planning",
        ].into_iter().map(String::from).collect());

        capabilities.insert("lighting".to_string(), vec![
            "light_placement", "atmosphere", "color_grading", "shadow_config",
        ].into_iter().map(String::from).collect());

        capabilities.insert("world_generator".to_string(), vec![
            "terrain_gen", "chunk_generation", "prop_scatter", "exterior_design",
        ].into_iter().map(String::from).collect());

        capabilities.insert("npc_brain".to_string(), vec![
            "npc_creation", "personality", "dialogue", "schedule",
        ].into_iter().map(String::from).collect());

        capabilities.insert("music".to_string(), vec![
            "music_generation", "ambience", "sound_design", "adaptive_music",
        ].into_iter().map(String::from).collect());

        capabilities.insert("research".to_string(), vec![
            "web_search", "image_search", "fact_check", "reference_gathering",
        ].into_iter().map(String::from).collect());

        capabilities.insert("gpu_lab".to_string(), vec![
            "asset_3d_generation", "texture_generation", "model_training",
            "voice_cloning", "image_generation",
        ].into_iter().map(String::from).collect());

        capabilities.insert("rigging".to_string(), vec![
            "auto_rig", "animation_retarget", "mixamo_upload", "motion_capture",
        ].into_iter().map(String::from).collect());

        capabilities.insert("voice".to_string(), vec![
            "tts", "voice_clone", "lip_sync", "emotion_voice",
        ].into_iter().map(String::from).collect());

        capabilities.insert("vfx".to_string(), vec![
            "particle_effect", "shader_effect", "decal", "destruction_vfx",
        ].into_iter().map(String::from).collect());

        capabilities.insert("architect".to_string(), vec![
            "engine_improvement", "code_generation", "tool_creation", "feature_implementation",
        ].into_iter().map(String::from).collect());

        Self { capabilities, load: HashMap::new() }
    }

    /// Find best agent for a capability
    pub fn find_agent(&self, capability: &str) -> Option<&str> {
        self.capabilities.iter()
            .filter(|(_, caps)| caps.iter().any(|c| c == capability))
            .min_by(|(a_id, _), (b_id, _)| {
                let a_load = self.load.get(*a_id).copied().unwrap_or(0.0);
                let b_load = self.load.get(*b_id).copied().unwrap_or(0.0);
                a_load.partial_cmp(&b_load).unwrap()
            })
            .map(|(id, _)| id.as_str())
    }

    pub fn set_load(&mut self, agent_id: &str, load: f32) {
        self.load.insert(agent_id.to_string(), load);
    }
}

// ─── Master Orchestrator ──────────────────────────────────────────────────────

pub struct MasterOrchestratorAgent {
    base: AgentBase,
    tasks: HashMap<String, OrchestratorTask>,
    spawned_agents: HashMap<String, SpawnedAgent>,
    creativity: CreativityEngine,
    router: AgentRouter,
    task_queue: Vec<String>, // task IDs in priority order
    /// Max concurrent tasks
    max_concurrent: usize,
    active_count: usize,
    /// Research cache (query → results)
    research_cache: HashMap<String, Vec<ResearchFinding>>,
}

impl MasterOrchestratorAgent {
    pub fn new() -> Self {
        Self {
            base: AgentBase::new(AgentSoul {
                id: "master_orchestrator".to_string(),
                name: "Master Orchestrator".to_string(),
                description: "Top-level coordinator. Receives goals, plans with research, spawns sub-agents, routes inter-agent requests. Ensures all created content is unique and creative.".to_string(),
                system_prompt: r#"You are the Master Orchestrator of ChronoVerse.

Your core philosophy:
1. EVERY game, NPC, room, and mechanic must be UNIQUE
2. RESEARCH FIRST — always gather real-world references before creating
3. PIPELINE THINKING — break tasks into specialized agent chains
4. PARALLEL when possible — many agents work simultaneously
5. QUALITY CHECK — verify outputs before marking complete

When given a task:
1. Research relevant real-world references (web search)
2. Build a detailed step-by-step plan
3. Assign each step to the best-fit agent
4. Identify which steps can run in parallel
5. Spawn sub-agents for complex sub-tasks
6. Monitor progress and handle failures
7. Verify the result meets quality standards

For creative tasks:
- Inject specific, unexpected details
- Avoid all clichés and defaults
- Make every character, room, mechanic feel crafted for THIS specific game
- Think like a master game designer who has played 10,000 games

You can directly communicate with the Architect Agent to request
engine improvements or new features."#.to_string(),
                tools: vec![
                    "dispatch_to_agent".to_string(),
                    "spawn_sub_agent".to_string(),
                    "research_web".to_string(),
                    "research_images".to_string(),
                    "create_task_plan".to_string(),
                    "get_agent_status".to_string(),
                    "cancel_task".to_string(),
                    "prompt_architect".to_string(),
                    "create_tool".to_string(),
                    "query_knowledge_base".to_string(),
                ],
                listens_to: vec![
                    "creator_command".to_string(),
                    "agent_request_routing".to_string(),
                    "task_complete".to_string(),
                    "task_failed".to_string(),
                    "sub_agent_result".to_string(),
                    "interior_design_requested".to_string(),
                    "story_beat_triggered".to_string(),
                ],
                can_emit: vec![
                    "dispatch_agent_task".to_string(),
                    "spawn_sub_agent".to_string(),
                    "task_complete".to_string(),
                    "task_failed".to_string(),
                    "research_requested".to_string(),
                    "improvement_opportunity".to_string(),
                ],
                max_tokens_per_call: 4000,
                parallelizable: false,
                priority: 10,
            }),
            tasks: HashMap::new(),
            spawned_agents: HashMap::new(),
            creativity: CreativityEngine::new(),
            router: AgentRouter::new(),
            task_queue: Vec::new(),
            max_concurrent: 8,
            active_count: 0,
            research_cache: HashMap::new(),
        }
    }

    /// Submit a new task to the orchestrator
    pub fn submit_task(&mut self, task_type: TaskType, description: &str, requested_by: &str) -> String {
        let seed = self.creativity.next_seed();
        let style_directives = self.creativity.uniqueness_directives(&task_type);

        let task = OrchestratorTask {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("{:?}", task_type).split('{').next().unwrap_or("Task").trim().to_string(),
            description: description.to_string(),
            task_type,
            priority: 5,
            status: TaskStatus::Queued,
            plan: None,
            sub_tasks: Vec::new(),
            spawned_agents: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            requested_by: requested_by.to_string(),
            result: None,
            errors: Vec::new(),
            research: Vec::new(),
            uniqueness_seed: seed,
            style_directives,
        };

        let id = task.id.clone();
        info!("Task submitted: '{}' ({})", task.title, id);
        self.task_queue.push(id.clone());
        self.tasks.insert(id.clone(), task);
        id
    }

    /// Plan a task into steps
    async fn plan_task(&mut self, task_id: &str, ctx: &AgentContext) -> Result<TaskPlan> {
        let task = match self.tasks.get(task_id) {
            Some(t) => t.clone(),
            None => return Err(anyhow::anyhow!("Task not found: {}", task_id)),
        };

        info!("Planning task: {}", task.title);
        self.base.set_working(&format!("Planning: {}", task.title), 0.1);

        let plan = match &task.task_type {
            TaskType::DesignInterior { room_id, style } => {
                self.plan_interior_design(room_id, style, &task)
            }
            TaskType::CreateNpc { role, faction, uniqueness_prompt } => {
                self.plan_npc_creation(role, faction, uniqueness_prompt, &task)
            }
            TaskType::DesignBoss { boss_concept, arena_description } => {
                self.plan_boss_design(boss_concept, arena_description, &task)
            }
            TaskType::ExecuteDescription { text } => {
                self.plan_from_description(text, &task)
            }
            TaskType::ImproveEngine { feature, specification } => {
                self.plan_engine_improvement(feature, specification, &task)
            }
            _ => TaskPlan {
                steps: vec![TaskStep {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "Execute".to_string(),
                    description: task.description.clone(),
                    assigned_agent: "world_generator".to_string(),
                    depends_on: Vec::new(),
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                }],
                parallel_groups: Vec::new(),
                estimated_duration_secs: 30.0,
                required_agents: vec!["world_generator".to_string()],
                required_assets: Vec::new(),
                research_queries: Vec::new(),
            },
        };

        Ok(plan)
    }

    fn plan_interior_design(&self, room_id: &str, style: &str, task: &OrchestratorTask) -> TaskPlan {
        let step_research = TaskStep {
            id: "step_research".to_string(),
            name: "Research Real References".to_string(),
            description: format!("Search web for real {} interior design, architectural blueprints, authentic historical references", style),
            assigned_agent: "research".to_string(),
            depends_on: Vec::new(),
            status: TaskStatus::Queued,
            result: None,
            sub_agent_spawn: None,
        };

        let step_plan = TaskStep {
            id: "step_plan".to_string(),
            name: "Create Floor Plan".to_string(),
            description: "Based on research, design the room layout with furniture zones, traffic flow, focal points".to_string(),
            assigned_agent: "interior_design".to_string(),
            depends_on: vec!["step_research".to_string()],
            status: TaskStatus::Queued,
            result: None,
            sub_agent_spawn: None,
        };

        let step_assets = TaskStep {
            id: "step_assets".to_string(),
            name: "Generate Missing Assets".to_string(),
            description: "Identify furniture/props not in asset library and generate them via GPU Lab".to_string(),
            assigned_agent: "gpu_lab".to_string(),
            depends_on: vec!["step_plan".to_string()],
            status: TaskStatus::Queued,
            result: None,
            sub_agent_spawn: Some(SubAgentSpec {
                agent_type: "asset_generator".to_string(),
                task_description: format!("Generate {} style furniture and props", style),
                tools: vec!["generate_3d_asset".to_string(), "generate_texture".to_string()],
                context: serde_json::json!({ "style": style, "room_id": room_id }),
                max_tokens: 1000,
                ttl_secs: 300.0,
            }),
        };

        // These can run in parallel after step_assets
        let step_place = TaskStep {
            id: "step_place".to_string(),
            name: "Place Furniture".to_string(),
            description: "Use spatial calculations to optimally place furniture respecting traffic flow, sightlines, clearances".to_string(),
            assigned_agent: "interior_design".to_string(),
            depends_on: vec!["step_assets".to_string()],
            status: TaskStatus::Queued,
            result: None,
            sub_agent_spawn: None,
        };

        let step_lighting = TaskStep {
            id: "step_lighting".to_string(),
            name: "Design Lighting".to_string(),
            description: "Place ambient, accent, and task lighting with correct color temperatures and intensities".to_string(),
            assigned_agent: "lighting".to_string(),
            depends_on: vec!["step_assets".to_string()],
            status: TaskStatus::Queued,
            result: None,
            sub_agent_spawn: None,
        };

        let step_sound = TaskStep {
            id: "step_sound".to_string(),
            name: "Design Soundscape".to_string(),
            description: "Assign surface sounds (floor type, materials), ambient audio (fire, wind, crowd), door creaks, etc.".to_string(),
            assigned_agent: "music".to_string(),
            depends_on: vec!["step_place".to_string()],
            status: TaskStatus::Queued,
            result: None,
            sub_agent_spawn: Some(SubAgentSpec {
                agent_type: "procedural_sound".to_string(),
                task_description: "Generate surface-specific footstep sounds, ambient loops, interaction sounds".to_string(),
                tools: vec!["generate_audio".to_string(), "download_sfx".to_string()],
                context: serde_json::json!({ "style": style }),
                max_tokens: 500,
                ttl_secs: 120.0,
            }),
        };

        let step_props = TaskStep {
            id: "step_props".to_string(),
            name: "Prop Dressing".to_string(),
            description: "Add small detail props (books, candles, dirt, personal items) that tell the inhabitant's story".to_string(),
            assigned_agent: "interior_design".to_string(),
            depends_on: vec!["step_place".to_string()],
            status: TaskStatus::Queued,
            result: None,
            sub_agent_spawn: None,
        };

        let step_verify = TaskStep {
            id: "step_verify".to_string(),
            name: "Visual QA".to_string(),
            description: "Take screenshot of room, use vision model to verify style consistency, check for clipping, rate quality".to_string(),
            assigned_agent: "qa_scout".to_string(),
            depends_on: vec!["step_lighting".to_string(), "step_props".to_string(), "step_sound".to_string()],
            status: TaskStatus::Queued,
            result: None,
            sub_agent_spawn: None,
        };

        TaskPlan {
            steps: vec![step_research, step_plan, step_assets, step_place, step_lighting, step_sound, step_props, step_verify],
            parallel_groups: vec![
                vec!["step_place".to_string(), "step_lighting".to_string()],
                vec!["step_sound".to_string(), "step_props".to_string()],
            ],
            estimated_duration_secs: 120.0,
            required_agents: vec!["research", "interior_design", "lighting", "music", "gpu_lab", "qa_scout"]
                .into_iter().map(String::from).collect(),
            required_assets: Vec::new(),
            research_queries: vec![
                format!("real {} interior design blueprints", style),
                format!("{} furniture authentic historical reference images", style),
                format!("{} interior lighting design principles", style),
            ],
        }
    }

    fn plan_npc_creation(&self, role: &str, faction: &str, uniqueness: &str, task: &OrchestratorTask) -> TaskPlan {
        TaskPlan {
            steps: vec![
                TaskStep {
                    id: "step_research".to_string(),
                    name: "Research Character Archetype".to_string(),
                    description: format!("Research real historical/cultural references for {} in {} setting", role, faction),
                    assigned_agent: "research".to_string(),
                    depends_on: Vec::new(),
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "step_design".to_string(),
                    name: "Design Character Soul".to_string(),
                    description: format!("Create unique personality, backstory, goals, fears for {} - uniqueness: {}", role, uniqueness),
                    assigned_agent: "npc_brain".to_string(),
                    depends_on: vec!["step_research".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "step_visuals".to_string(),
                    name: "Generate Appearance".to_string(),
                    description: "Generate 3D model, textures, clothing that reflects their personality and background".to_string(),
                    assigned_agent: "gpu_lab".to_string(),
                    depends_on: vec!["step_design".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "step_rig".to_string(),
                    name: "Rig and Animate".to_string(),
                    description: "Auto-rig model, generate idle/walk/talk animations matching personality".to_string(),
                    assigned_agent: "rigging".to_string(),
                    depends_on: vec!["step_visuals".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "step_voice".to_string(),
                    name: "Create Voice Profile".to_string(),
                    description: "Clone or synthesize unique voice, generate sample lines, create lip sync data".to_string(),
                    assigned_agent: "voice".to_string(),
                    depends_on: vec!["step_design".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "step_schedule".to_string(),
                    name: "Create Daily Schedule".to_string(),
                    description: "Design realistic daily routine, patrol routes, social interactions".to_string(),
                    assigned_agent: "npc_brain".to_string(),
                    depends_on: vec!["step_design".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
            ],
            parallel_groups: vec![
                vec!["step_voice".to_string(), "step_schedule".to_string()],
            ],
            estimated_duration_secs: 180.0,
            required_agents: vec!["research", "npc_brain", "gpu_lab", "rigging", "voice"]
                .into_iter().map(String::from).collect(),
            required_assets: Vec::new(),
            research_queries: vec![
                format!("{} historical reference photographs", role),
                format!("{} culture clothing architecture", faction),
            ],
        }
    }

    fn plan_boss_design(&self, concept: &str, arena: &str, task: &OrchestratorTask) -> TaskPlan {
        TaskPlan {
            steps: vec![
                TaskStep {
                    id: "research".to_string(),
                    name: "Research Boss Design Patterns".to_string(),
                    description: "Research memorable boss fights from games, mythology, unique mechanics".to_string(),
                    assigned_agent: "research".to_string(),
                    depends_on: Vec::new(),
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "mechanics".to_string(),
                    name: "Design Unique Fight Mechanics".to_string(),
                    description: format!("Create fight phases and unique mechanics for {}", concept),
                    assigned_agent: "combat_agent".to_string(),
                    depends_on: vec!["research".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: Some(SubAgentSpec {
                        agent_type: "boss_mechanics_designer".to_string(),
                        task_description: "Design multi-phase boss fight with unique, never-before-seen mechanics".to_string(),
                        tools: vec!["design_attack_pattern".to_string(), "create_mechanic".to_string()],
                        context: serde_json::json!({ "concept": concept, "arena": arena }),
                        max_tokens: 2000,
                        ttl_secs: 180.0,
                    }),
                },
                TaskStep {
                    id: "model".to_string(),
                    name: "Generate Boss Model".to_string(),
                    description: format!("Create imposing 3D model for {}", concept),
                    assigned_agent: "gpu_lab".to_string(),
                    depends_on: vec!["mechanics".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "arena".to_string(),
                    name: "Build Arena".to_string(),
                    description: format!("Design arena that integrates with fight mechanics: {}", arena),
                    assigned_agent: "world_generator".to_string(),
                    depends_on: vec!["mechanics".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "music".to_string(),
                    name: "Compose Boss Theme".to_string(),
                    description: "Generate dynamic multi-phase boss music with intensity matching phases".to_string(),
                    assigned_agent: "music".to_string(),
                    depends_on: vec!["mechanics".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
            ],
            parallel_groups: vec![
                vec!["model".to_string(), "arena".to_string(), "music".to_string()],
            ],
            estimated_duration_secs: 300.0,
            required_agents: vec!["research", "combat_agent", "gpu_lab", "world_generator", "music"]
                .into_iter().map(String::from).collect(),
            required_assets: Vec::new(),
            research_queries: vec![
                "most creative boss fight mechanics in video games".to_string(),
                format!("{} mythology creature design", concept),
            ],
        }
    }

    fn plan_from_description(&self, text: &str, task: &OrchestratorTask) -> TaskPlan {
        // Free-form: parse description and build plan
        TaskPlan {
            steps: vec![
                TaskStep {
                    id: "parse".to_string(),
                    name: "Parse and Plan".to_string(),
                    description: format!("Analyze: '{}' and create execution plan", text),
                    assigned_agent: "master_orchestrator".to_string(),
                    depends_on: Vec::new(),
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
            ],
            parallel_groups: Vec::new(),
            estimated_duration_secs: 60.0,
            required_agents: Vec::new(),
            required_assets: Vec::new(),
            research_queries: Vec::new(),
        }
    }

    fn plan_engine_improvement(&self, feature: &str, spec: &str, task: &OrchestratorTask) -> TaskPlan {
        TaskPlan {
            steps: vec![
                TaskStep {
                    id: "analyze".to_string(),
                    name: "Analyze Current Codebase".to_string(),
                    description: format!("Read relevant source files for: {}", feature),
                    assigned_agent: "architect".to_string(),
                    depends_on: Vec::new(),
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "implement".to_string(),
                    name: "Implement Feature".to_string(),
                    description: format!("Write Rust code for: {} — spec: {}", feature, spec),
                    assigned_agent: "architect".to_string(),
                    depends_on: vec!["analyze".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
                TaskStep {
                    id: "test".to_string(),
                    name: "Test and Validate".to_string(),
                    description: "Run compile check, unit tests, integration tests".to_string(),
                    assigned_agent: "architect".to_string(),
                    depends_on: vec!["implement".to_string()],
                    status: TaskStatus::Queued,
                    result: None,
                    sub_agent_spawn: None,
                },
            ],
            parallel_groups: Vec::new(),
            estimated_duration_secs: 600.0,
            required_agents: vec!["architect".to_string()],
            required_assets: Vec::new(),
            research_queries: vec![
                format!("Rust best practices for {}", feature),
                format!("{} game engine implementation reference", feature),
            ],
        }
    }

    /// Spawn a sub-agent for a specific task
    async fn spawn_sub_agent(&mut self, spec: &SubAgentSpec, task_id: &str, step_id: &str) -> String {
        let agent_id = format!("sub_{}_{}", spec.agent_type, &uuid::Uuid::new_v4().to_string()[..8]);

        info!("Spawning sub-agent: {} for task {}/{}", agent_id, task_id, step_id);

        let spawned = SpawnedAgent {
            id: agent_id.clone(),
            agent_type: spec.agent_type.clone(),
            task_id: task_id.to_string(),
            step_id: step_id.to_string(),
            status: SubAgentStatus::Spawning,
            context: spec.context.clone(),
            result: None,
            spawned_at: Utc::now(),
            ttl_secs: spec.ttl_secs,
        };

        self.spawned_agents.insert(agent_id.clone(), spawned);
        agent_id
    }

    /// Route an inter-agent request to the right agent
    pub fn route_request(&self, from_agent: &str, capability: &str) -> Option<&str> {
        info!("Routing request: {} needs capability '{}'", from_agent, capability);
        self.router.find_agent(capability)
    }

    pub fn task_count(&self) -> usize { self.tasks.len() }
    pub fn active_task_count(&self) -> usize {
        self.tasks.values().filter(|t| {
            matches!(t.status, TaskStatus::InProgress { .. } | TaskStatus::Researching | TaskStatus::Planning)
        }).count()
    }
}

#[async_trait]
impl Agent for MasterOrchestratorAgent {
    fn id(&self) -> &str { "master_orchestrator" }
    fn name(&self) -> &str { "Master Orchestrator" }
    fn description(&self) -> &str { "Top-level coordinator: plans, routes, spawns sub-agents, ensures creativity and uniqueness" }
    fn status(&self) -> &AgentStatus { &self.base.status }

    fn subscriptions(&self) -> Vec<&'static str> {
        vec![
            "creator_command",
            "agent_request_routing",
            "story_beat_triggered",
            "analytics_event",
        ]
    }

    fn tick_interval(&self) -> Option<f32> { Some(0.5) }

    async fn initialize(&mut self, ctx: &AgentContext) -> Result<()> {
        info!("Master Orchestrator initialized. Ready to coordinate {} capabilities.", self.router.capabilities.len());
        Ok(())
    }

    async fn handle_event(&mut self, event: &EventEnvelope<AgentEvent>, ctx: &AgentContext) -> Result<()> {
        match &event.event {
            AgentEvent::AnalyticsEvent { event_type, data } => {
                match event_type.as_str() {
                    "creator_command" => {
                        let text = data.get("text").and_then(|v| v.as_str()).unwrap_or("");
                        info!("Creator command received: {}", text);
                        let task_id = self.submit_task(
                            TaskType::ExecuteDescription { text: text.to_string() },
                            text,
                            "creator",
                        );
                        info!("Task created: {}", task_id);
                    }
                    "interior_design_requested" => {
                        let room_id = data.get("room_id").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let style = data.get("style").and_then(|v| v.as_str()).unwrap_or("generic");
                        let task_id = self.submit_task(
                            TaskType::DesignInterior { room_id: room_id.to_string(), style: style.to_string() },
                            &format!("Design {} interior for room {}", style, room_id),
                            "system",
                        );
                        info!("Interior design task: {}", task_id);
                    }
                    "route_to_agent" => {
                        // Inter-agent routing
                        let from = data.get("from").and_then(|v| v.as_str()).unwrap_or("");
                        let capability = data.get("capability").and_then(|v| v.as_str()).unwrap_or("");
                        if let Some(target) = self.route_request(from, capability) {
                            info!("Routing {} → {} for capability '{}'", from, target, capability);
                            ctx.emit(AgentEvent::AnalyticsEvent {
                                event_type: "agent_routed".to_string(),
                                data: serde_json::json!({
                                    "from": from, "to": target, "capability": capability,
                                    "original_data": data
                                }),
                            }, self.id());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self, ctx: &AgentContext, delta: f32) -> Result<()> {
        // Process task queue
        if self.active_count < self.max_concurrent {
            let queued: Vec<String> = self.tasks.iter()
                .filter(|(_, t)| t.status == TaskStatus::Queued)
                .map(|(id, _)| id.clone())
                .collect();

            for task_id in queued.into_iter().take(self.max_concurrent - self.active_count) {
                if let Ok(plan) = self.plan_task(&task_id, ctx).await {
                    if let Some(task) = self.tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Planning;
                        task.started_at = Some(Utc::now());
                        task.plan = Some(plan);
                    }
                    self.active_count += 1;
                }
            }
        }

        // Age and expire sub-agents
        let now = Utc::now();
        for agent in self.spawned_agents.values_mut() {
            if agent.status == SubAgentStatus::Running {
                let age = (now - agent.spawned_at).num_seconds() as f32;
                if age > agent.ttl_secs {
                    agent.status = SubAgentStatus::Expired;
                    warn!("Sub-agent expired: {}", agent.id);
                }
            }
        }

        Ok(())
    }
}

extern crate uuid;
