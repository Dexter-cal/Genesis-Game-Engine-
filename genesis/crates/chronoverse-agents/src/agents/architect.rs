//! Architect Agent — Self-Improving Engine Agent
use anyhow::Result;
use async_trait::async_trait;
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use chrono::{DateTime, Utc};
use chronoverse_core::events::{AgentEvent, EventEnvelope};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineImprovement {
    pub id: String,
    pub domain: String,
    pub title: String,
    pub description: String,
    pub risk_level: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

pub struct ArchitectAgent {
    pub base: AgentBase,
    pub applied: Vec<EngineImprovement>,
    pub enabled: bool,
    pub require_human_approval: bool,
    pub total_improvements: u32,
    cycle_timer: f32,
    cycle_interval: f32,
    observations: VecDeque<(String, f64, f64)>,
}

impl ArchitectAgent {
    pub fn new() -> Self {
        Self {
            base: AgentBase::new(AgentSoul {
                id: "architect".to_string(),
                name: "Architect Agent".to_string(),
                description: "Self-improving engine agent. Analyzes performance, writes Rust improvements, validates in sandbox.".to_string(),
                system_prompt: "You are the Architect Agent. Continuously improve ChronoVerse engine code. Write safe, idiomatic Rust. Always benchmark before and after. Explain your reasoning.".to_string(),
                tools: vec!["read_source_file".to_string(),"write_source_file".to_string(),"run_benchmarks".to_string(),"run_tests".to_string(),"compile_check".to_string(),"git_commit".to_string(),"create_tool".to_string()],
                listens_to: vec!["improvement_opportunity".to_string(),"performance_alert".to_string(),"tool_creation_requested".to_string()],
                can_emit: vec!["improvement_applied".to_string(),"improvement_failed".to_string(),"tool_created".to_string()],
                max_tokens_per_call: 4000,
                parallelizable: false,
                priority: 1,
            }),
            applied: Vec::new(),
            enabled: false,
            require_human_approval: true,
            total_improvements: 0,
            cycle_timer: 0.0,
            cycle_interval: 3600.0,
            observations: VecDeque::new(),
        }
    }

    pub fn enable(&mut self) { self.enabled = true; info!("Architect: self-improvement ENABLED"); }

    pub fn observe_perf(&mut self, system: &str, value: f64, threshold: f64) {
        self.observations.push_back((system.to_string(), value, threshold));
        while self.observations.len() > 500 { self.observations.pop_front(); }
    }

    async fn build_tool_for_agent(&self, agent: &str, desc: &str, ctx: &AgentContext) {
        let name = desc.split_whitespace().take(3).collect::<Vec<_>>().join("_").to_lowercase();
        info!("Building tool '{}' for agent '{}'", name, agent);
        ctx.emit(AgentEvent::ToolCreated { tool_name: name.clone(), tool_path: format!("src/tools/{}.rs", name) }, self.id());
    }
}

#[async_trait]
impl Agent for ArchitectAgent {
    fn id(&self) -> &str { "architect" }
    fn name(&self) -> &str { "Architect Agent" }
    fn description(&self) -> &str { "Self-improving engine agent" }
    fn status(&self) -> &AgentStatus { &self.base.status }
    fn subscriptions(&self) -> Vec<&'static str> {
        vec!["improvement_opportunity", "performance_alert", "tool_creation_requested"]
    }
    fn tick_interval(&self) -> Option<f32> { Some(60.0) }

    async fn handle_event(&mut self, event: &EventEnvelope<AgentEvent>, ctx: &AgentContext) -> Result<()> {
        match &event.event {
            AgentEvent::ToolCreationRequested { agent, tool_description } => {
                let (a, d) = (agent.clone(), tool_description.clone());
                self.build_tool_for_agent(&a, &d, ctx).await;
            }
            AgentEvent::PerformanceAlert { system, value, threshold, .. } => {
                self.observe_perf(system, *value, *threshold);
            }
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self, ctx: &AgentContext, delta: f32) -> Result<()> {
        self.cycle_timer += delta;
        if self.enabled && self.cycle_timer >= self.cycle_interval {
            self.cycle_timer = 0.0;
            info!("Architect: running improvement cycle...");
        }
        Ok(())
    }
}
