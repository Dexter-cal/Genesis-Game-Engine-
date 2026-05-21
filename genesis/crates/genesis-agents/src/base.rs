//! Base Agent trait and context
//!
//! Every agent in Genesis implements the Agent trait.
//! Agents are async, event-driven, and token-budget-aware.

use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;
use chrono::{DateTime, Utc};

use genesis_core::events::{AgentEvent, GameEvent, EventBus, EventEnvelope, EventPriority};
use crate::budget::TokenBudget;
use crate::memory::AgentMemory;
use crate::tools::ToolRegistry;

/// The "soul" of an agent — its identity and purpose
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSoul {
    /// Unique agent identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// What this agent does (shown in Council Log)
    pub description: String,
    /// System prompt / personality instructions
    pub system_prompt: String,
    /// List of tools this agent can use
    pub tools: Vec<String>,
    /// Events this agent subscribes to
    pub listens_to: Vec<String>,
    /// Events this agent can emit
    pub can_emit: Vec<String>,
    /// Maximum tokens per request
    pub max_tokens_per_call: u32,
    /// Can this agent run in parallel with others?
    pub parallelizable: bool,
    /// Priority (higher = runs first in conflicts)
    pub priority: u8,
}

/// Current execution status of an agent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Thinking { prompt_preview: String },
    Working { task: String, progress: f32 },
    Waiting { waiting_for: String },
    Error { message: String },
    Disabled,
}

/// Shared context passed to every agent on each call
pub struct AgentContext {
    /// The agent event bus for publishing/subscribing
    pub agent_bus: Arc<EventBus<AgentEvent>>,
    /// The game event bus (read-only for agents)
    pub game_bus: Arc<EventBus<GameEvent>>,
    /// Token budget manager
    pub budget: Arc<RwLock<TokenBudget>>,
    /// Tool registry
    pub tools: Arc<ToolRegistry>,
    /// Current game world context (JSON snapshot)
    pub world_context: Arc<RwLock<serde_json::Value>>,
    /// Timestamp of this context
    pub timestamp: DateTime<Utc>,
}

impl AgentContext {
    pub fn new(
        agent_bus: Arc<EventBus<AgentEvent>>,
        game_bus:  Arc<EventBus<GameEvent>>,
        budget:    Arc<RwLock<TokenBudget>>,
        tools:     Arc<ToolRegistry>,
    ) -> Self {
        Self {
            agent_bus,
            game_bus,
            budget,
            tools,
            world_context: Arc::new(RwLock::new(serde_json::json!({}))),
            timestamp: Utc::now(),
        }
    }

    /// Emit an agent event
    pub fn emit(&self, event: AgentEvent, source: &str) {
        self.agent_bus.publish(event, EventPriority::Normal, Some(source.to_string()));
    }

    /// Emit with priority
    pub fn emit_priority(&self, event: AgentEvent, priority: EventPriority, source: &str) {
        self.agent_bus.publish(event, priority, Some(source.to_string()));
    }
}

/// The core Agent trait — every agent implements this
#[async_trait]
pub trait Agent: Send + Sync + 'static {
    /// Unique identifier for this agent type
    fn id(&self) -> &str;
    /// Human-readable name
    fn name(&self) -> &str;
    /// What this agent does
    fn description(&self) -> &str;

    /// Called once when the agent is registered
    async fn initialize(&mut self, ctx: &AgentContext) -> Result<()> { Ok(()) }

    /// Process an incoming agent event
    async fn handle_event(
        &mut self,
        event: &EventEnvelope<AgentEvent>,
        ctx: &AgentContext,
    ) -> Result<()>;

    /// Called on a schedule (if agent has a tick interval)
    async fn tick(&mut self, ctx: &AgentContext, delta: f32) -> Result<()> { Ok(()) }

    /// Tick interval in seconds (None = event-only, no scheduled ticks)
    fn tick_interval(&self) -> Option<f32> { None }

    /// Which event types this agent handles
    fn subscriptions(&self) -> Vec<&'static str>;

    /// Current status of the agent
    fn status(&self) -> &AgentStatus;

    /// Can this agent create new tools?
    fn can_create_tools(&self) -> bool { true }

    /// Called when the agent should stop
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}

/// Common fields all agents share — embed in your agent struct
pub struct AgentBase {
    pub soul: AgentSoul,
    pub status: AgentStatus,
    pub memory: AgentMemory,
    pub total_calls: u64,
    pub total_tokens_used: u64,
    pub last_active: Option<DateTime<Utc>>,
    pub enabled: bool,
}

impl AgentBase {
    pub fn new(soul: AgentSoul) -> Self {
        Self {
            soul,
            status: AgentStatus::Idle,
            memory: AgentMemory::new(200),
            total_calls: 0,
            total_tokens_used: 0,
            last_active: None,
            enabled: true,
        }
    }

    pub fn set_thinking(&mut self, preview: &str) {
        self.status = AgentStatus::Thinking { prompt_preview: preview.to_string() };
        self.last_active = Some(Utc::now());
    }

    pub fn set_working(&mut self, task: &str, progress: f32) {
        self.status = AgentStatus::Working { task: task.to_string(), progress };
    }

    pub fn set_idle(&mut self) {
        self.status = AgentStatus::Idle;
    }

    pub fn set_error(&mut self, msg: &str) {
        self.status = AgentStatus::Error { message: msg.to_string() };
        tracing::error!("Agent '{}' error: {}", self.soul.name, msg);
    }

    pub fn record_call(&mut self, tokens_used: u64) {
        self.total_calls += 1;
        self.total_tokens_used += tokens_used;
        self.last_active = Some(Utc::now());
        self.set_idle();
    }
}
