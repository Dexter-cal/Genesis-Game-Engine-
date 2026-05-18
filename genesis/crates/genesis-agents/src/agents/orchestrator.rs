//! Orchestrator Agent — master router, breaks goals into agent tasks

use anyhow::Result;
use async_trait::async_trait;
use tracing::info;
use genesis_core::events::{AgentEvent, EventEnvelope};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};

pub struct OrchestratorAgent { base: AgentBase }
impl OrchestratorAgent {
    pub fn new() -> Self {
        Self { base: AgentBase::new(AgentSoul {
            id: "orchestrator".to_string(),
            name: "Orchestrator Agent".to_string(),
            description: "Master router. Receives high-level goals, breaks into tasks, dispatches to specialist agents.".to_string(),
            system_prompt: "You are the Orchestrator. Your job is to understand what needs to be done and dispatch the right agents in the right order. Think step by step. Be efficient.".to_string(),
            tools: vec!["dispatch_agent".to_string(), "create_plan".to_string(), "query_agent_status".to_string()],
            listens_to: vec!["story_beat_triggered".to_string(), "world_chunk_requested".to_string(), "tool_creation_requested".to_string()],
            can_emit: vec!["asset_3d_requested".to_string(), "music_requested".to_string(), "world_chunk_requested".to_string()],
            max_tokens_per_call: 1000,
            parallelizable: false,
            priority: 10,
        })}
    }
}
#[async_trait]
impl Agent for OrchestratorAgent {
    fn id(&self) -> &str { "orchestrator" }
    fn name(&self) -> &str { "Orchestrator Agent" }
    fn description(&self) -> &str { "Master router and planner" }
    fn status(&self) -> &AgentStatus { &self.base.status }
    fn subscriptions(&self) -> Vec<&'static str> {
        vec!["story_beat_triggered", "world_chunk_requested", "tool_creation_requested"]
    }
    async fn handle_event(&mut self, event: &EventEnvelope<AgentEvent>, ctx: &AgentContext) -> Result<()> {
        info!("Orchestrator received event from {:?}", event.source);
        Ok(())
    }
}
