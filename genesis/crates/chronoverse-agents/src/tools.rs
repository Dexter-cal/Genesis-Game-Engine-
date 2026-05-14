//! Tool Registry — agents use tools to interact with the world
//!
//! Tools are functions agents can call.
//! Agents can also CREATE new tools if they need capabilities
//! that don't exist yet (meta-tool: create_tool).

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use async_trait::async_trait;

/// A tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParam {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

/// Metadata about a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParam>,
    pub returns: String,
    pub category: ToolCategory,
    pub created_by: String, // agent that created it, or "system"
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    WorldManipulation,
    AssetGeneration,
    NpcControl,
    AudioControl,
    SceneQuery,
    FileSystem,
    Network,
    Computation,
    AgentCommunication,
    CodeGeneration,
    Research,
    Custom(String),
}

/// A callable tool function
#[async_trait]
pub trait Tool: Send + Sync {
    fn metadata(&self) -> &ToolMetadata;
    async fn call(&self, params: serde_json::Value) -> Result<serde_json::Value>;
    fn name(&self) -> &str { &self.metadata().name }
}

/// Registry of all available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self { tools: HashMap::new() };
        registry.register_built_in_tools();
        registry
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_all(&self) -> Vec<&ToolMetadata> {
        self.tools.values().map(|t| t.metadata()).collect()
    }

    pub fn call(&self, name: &str, params: serde_json::Value)
        -> Option<impl std::future::Future<Output = Result<serde_json::Value>> + '_>
    {
        let tool = self.tools.get(name)?.clone();
        Some(async move { tool.call(params).await })
    }

    fn register_built_in_tools(&mut self) {
        // Built-in tools registered at startup
        // (implementations inline as closures)

        // Entity query tools, world manipulation, etc.
        // Full implementations in the respective subsystems
    }
}

// ─── Built-in Tool Implementations ───────────────────────────────────────────

/// Spawns an entity in the world
pub struct SpawnEntityTool;

#[async_trait]
impl Tool for SpawnEntityTool {
    fn metadata(&self) -> &ToolMetadata {
        use once_cell::sync::Lazy;
        static META: Lazy<ToolMetadata> = Lazy::new(|| ToolMetadata {
            name: "spawn_entity".to_string(),
            description: "Spawn a new entity in the scene at a given position".to_string(),
            parameters: vec![
                ToolParam { name: "entity_type".to_string(), param_type: "string".to_string(),
                    description: "Type: npc, prop, light, trigger".to_string(), required: true, default_value: None },
                ToolParam { name: "name".to_string(), param_type: "string".to_string(),
                    description: "Human-readable name".to_string(), required: true, default_value: None },
                ToolParam { name: "position".to_string(), param_type: "[f32;3]".to_string(),
                    description: "World position [x, y, z]".to_string(), required: true, default_value: None },
                ToolParam { name: "properties".to_string(), param_type: "object".to_string(),
                    description: "Extra properties".to_string(), required: false, default_value: Some(serde_json::json!({})) },
            ],
            returns: "entity_id: string".to_string(),
            category: ToolCategory::WorldManipulation,
            created_by: "system".to_string(),
            version: "1.0.0".to_string(),
        });
        &META
    }

    async fn call(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        // In the real engine this would create the entity in the ECS World
        // Here we return a mock response
        let entity_id = format!("{}-{}",
            params["entity_type"].as_str().unwrap_or("ENT"),
            &uuid::Uuid::new_v4().to_string()[..8]
        );
        Ok(serde_json::json!({ "entity_id": entity_id, "success": true }))
    }
}

/// Tool that allows an agent to create a new tool
pub struct CreateToolTool;

#[async_trait]
impl Tool for CreateToolTool {
    fn metadata(&self) -> &ToolMetadata {
        use once_cell::sync::Lazy;
        static META: Lazy<ToolMetadata> = Lazy::new(|| ToolMetadata {
            name: "create_tool".to_string(),
            description: "Create a new tool that agents can use. Provide the tool spec and implementation in Rust. The Architect Agent will validate and integrate it.".to_string(),
            parameters: vec![
                ToolParam { name: "tool_name".to_string(), param_type: "string".to_string(),
                    description: "Unique snake_case name for the tool".to_string(), required: true, default_value: None },
                ToolParam { name: "description".to_string(), param_type: "string".to_string(),
                    description: "What the tool does".to_string(), required: true, default_value: None },
                ToolParam { name: "implementation".to_string(), param_type: "string".to_string(),
                    description: "Rust code implementing the tool".to_string(), required: true, default_value: None },
                ToolParam { name: "parameters".to_string(), param_type: "array".to_string(),
                    description: "Parameter definitions".to_string(), required: true, default_value: None },
            ],
            returns: "tool_id: string, status: string".to_string(),
            category: ToolCategory::CodeGeneration,
            created_by: "system".to_string(),
            version: "1.0.0".to_string(),
        });
        &META
    }

    async fn call(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        // In production: sends to Architect Agent for validation
        let tool_name = params["tool_name"].as_str().unwrap_or("unnamed");
        tracing::info!("New tool creation requested: {}", tool_name);
        Ok(serde_json::json!({
            "status": "submitted_to_architect",
            "tool_id": format!("tool-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
            "message": "Architect Agent will validate and integrate this tool."
        }))
    }
}

/// Agent planning system
pub struct AgentPlan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub current_step: usize,
    pub completed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub tool: Option<String>,
    pub tool_params: Option<serde_json::Value>,
    pub depends_on: Vec<String>,
    pub completed: bool,
    pub result: Option<serde_json::Value>,
}

impl AgentPlan {
    pub fn new(goal: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            goal: goal.to_string(),
            steps: Vec::new(),
            current_step: 0,
            completed: false,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn add_step(&mut self, description: &str, tool: Option<&str>) -> &mut PlanStep {
        self.steps.push(PlanStep {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            tool: tool.map(|s| s.to_string()),
            tool_params: None,
            depends_on: Vec::new(),
            completed: false,
            result: None,
        });
        self.steps.last_mut().unwrap()
    }

    pub fn next_step(&self) -> Option<&PlanStep> {
        self.steps.iter().find(|s| !s.completed)
    }

    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() { return 1.0; }
        let done = self.steps.iter().filter(|s| s.completed).count();
        done as f32 / self.steps.len() as f32
    }
}
