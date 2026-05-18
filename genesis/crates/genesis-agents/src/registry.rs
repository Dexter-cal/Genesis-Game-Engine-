//! Agent Registry — manages all registered agents
//!
//! Handles:
//! - Agent registration and discovery
//! - Event routing to subscribed agents
//! - Parallel agent execution
//! - Agent status tracking for UI

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use anyhow::Result;
use tracing::{info, warn, error, debug};

use genesis_core::events::{AgentEvent, EventBus, EventEnvelope};
use crate::base::{Agent, AgentContext, AgentStatus};

pub struct AgentRegistry {
    agents: HashMap<String, Box<dyn Agent>>,
    /// event_type → list of agent IDs that handle it
    subscriptions: HashMap<String, Vec<String>>,
    /// Total events dispatched
    events_dispatched: u64,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            subscriptions: HashMap::new(),
            events_dispatched: 0,
        }
    }

    /// Register an agent with the registry
    pub fn register<A: Agent + 'static>(&mut self, agent: A) {
        let id = agent.id().to_string();
        let subs = agent.subscriptions();

        // Build subscription index
        for event_type in &subs {
            self.subscriptions
                .entry(event_type.to_string())
                .or_default()
                .push(id.clone());
        }

        info!("Registered agent: {} ({})", agent.name(), id);
        self.agents.insert(id, Box::new(agent));
    }

    /// Dispatch an event to all subscribed agents
    pub async fn dispatch(
        &mut self,
        event: &EventEnvelope<AgentEvent>,
        ctx: &AgentContext,
    ) -> Result<()> {
        self.events_dispatched += 1;

        // Determine which event type this is
        let event_type = Self::event_type_name(&event.event);

        let agent_ids: Vec<String> = self.subscriptions
            .get(event_type)
            .cloned()
            .unwrap_or_default();

        if agent_ids.is_empty() {
            debug!("No agents subscribed to event type: {}", event_type);
            return Ok(());
        }

        // Dispatch to each subscribed agent sequentially
        // (parallelization would need Arc<Mutex<>> on agents — future optimization)
        for agent_id in &agent_ids {
            if let Some(agent) = self.agents.get_mut(agent_id) {
                match agent.handle_event(event, ctx).await {
                    Ok(()) => debug!("Agent '{}' handled event '{}'", agent_id, event_type),
                    Err(e) => {
                        error!("Agent '{}' failed on '{}': {}", agent_id, event_type, e);
                        // Don't propagate — one agent failure shouldn't crash others
                    }
                }
            }
        }

        Ok(())
    }

    /// Run scheduled ticks for all agents that have a tick interval
    pub async fn tick(&mut self, ctx: &AgentContext, delta: f32) -> Result<()> {
        for agent in self.agents.values_mut() {
            if agent.tick_interval().is_some() {
                if let Err(e) = agent.tick(ctx, delta).await {
                    warn!("Agent '{}' tick error: {}", agent.id(), e);
                }
            }
        }
        Ok(())
    }

    /// Get status of all agents (for the editor's Council Log)
    pub fn all_statuses(&self) -> Vec<(&str, &str, &AgentStatus)> {
        self.agents.values()
            .map(|a| (a.id(), a.name(), a.status()))
            .collect()
    }

    /// Get a specific agent's status
    pub fn agent_status(&self, id: &str) -> Option<&AgentStatus> {
        self.agents.get(id).map(|a| a.status())
    }

    pub fn agent_count(&self) -> usize { self.agents.len() }
    pub fn events_dispatched(&self) -> u64 { self.events_dispatched }

    /// Initialize all agents
    pub async fn initialize_all(&mut self, ctx: &AgentContext) -> Result<()> {
        for agent in self.agents.values_mut() {
            agent.initialize(ctx).await?;
        }
        info!("All {} agents initialized", self.agents.len());
        Ok(())
    }

    /// Shutdown all agents
    pub async fn shutdown_all(&mut self) -> Result<()> {
        for agent in self.agents.values_mut() {
            if let Err(e) = agent.shutdown().await {
                warn!("Agent '{}' shutdown error: {}", agent.id(), e);
            }
        }
        Ok(())
    }

    /// Map an AgentEvent to its type name string for routing
    fn event_type_name(event: &AgentEvent) -> &'static str {
        match event {
            AgentEvent::Asset3DRequested { .. }    => "asset_3d_requested",
            AgentEvent::Asset3DGenerated { .. }    => "asset_3d_generated",
            AgentEvent::AssetRigRequested { .. }   => "asset_rig_requested",
            AgentEvent::AssetRigComplete { .. }    => "asset_rig_complete",
            AgentEvent::TextureRequested { .. }    => "texture_requested",
            AgentEvent::TextureGenerated { .. }    => "texture_generated",
            AgentEvent::MusicRequested { .. }      => "music_requested",
            AgentEvent::MusicGenerated { .. }      => "music_generated",
            AgentEvent::SfxRequested { .. }        => "sfx_requested",
            AgentEvent::SfxGenerated { .. }        => "sfx_generated",
            AgentEvent::NpcDecisionRequested { .. }=> "npc_decision_requested",
            AgentEvent::NpcDecisionMade { .. }     => "npc_decision_made",
            AgentEvent::NpcDialogueRequested { .. }=> "npc_dialogue_requested",
            AgentEvent::NpcDialogueGenerated { .. }=> "npc_dialogue_generated",
            AgentEvent::NpcMemoryUpdated { .. }    => "npc_memory_updated",
            AgentEvent::NpcEmotionChanged { .. }   => "npc_emotion_changed",
            AgentEvent::VoiceSynthRequested { .. } => "voice_synth_requested",
            AgentEvent::VoiceSynthComplete { .. }  => "voice_synth_complete",
            AgentEvent::WorldChunkRequested { .. } => "world_chunk_requested",
            AgentEvent::WorldChunkGenerated { .. } => "world_chunk_generated",
            AgentEvent::QuestGenerated { .. }      => "quest_generated",
            AgentEvent::ItemGenerated { .. }       => "item_generated",
            AgentEvent::VisualCheckRequested { .. }=> "visual_check_requested",
            AgentEvent::VisualIssueFound { .. }    => "visual_issue_found",
            AgentEvent::VisualCheckPassed { .. }   => "visual_check_passed",
            AgentEvent::StoryBeatTriggered { .. }  => "story_beat_triggered",
            AgentEvent::StoryBeatGenerated { .. }  => "story_beat_generated",
            AgentEvent::CutsceneRequested { .. }   => "cutscene_requested",
            AgentEvent::CutsceneGenerated { .. }   => "cutscene_generated",
            AgentEvent::ImprovementOpportunity { .. } => "improvement_opportunity",
            AgentEvent::ImprovementApplied { .. }  => "improvement_applied",
            AgentEvent::ImprovementFailed { .. }   => "improvement_failed",
            AgentEvent::ToolCreationRequested { .. }=> "tool_creation_requested",
            AgentEvent::ToolCreated { .. }         => "tool_created",
            AgentEvent::ResearchRequested { .. }   => "research_requested",
            AgentEvent::ResearchComplete { .. }    => "research_complete",
            AgentEvent::GpuLabJobSubmitted { .. }  => "gpu_lab_job_submitted",
            AgentEvent::GpuLabJobComplete { .. }   => "gpu_lab_job_complete",
            AgentEvent::GpuLabJobFailed { .. }     => "gpu_lab_job_failed",
            AgentEvent::AnalyticsEvent { .. }      => "analytics_event",
            AgentEvent::PerformanceAlert { .. }    => "performance_alert",
        }
    }
}
