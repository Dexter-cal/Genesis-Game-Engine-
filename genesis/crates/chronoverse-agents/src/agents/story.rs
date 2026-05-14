//! Story Agent — manages narrative, story arcs, quest generation, branching

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tracing::info;
use chronoverse_core::events::{AgentEvent, EventEnvelope};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryBeat {
    pub id: String,
    pub title: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub consequences: Vec<String>,
    pub dialogue_hooks: Vec<String>,
    pub world_state_changes: serde_json::Value,
    pub emotional_tone: String,
    pub pacing: StoryPacing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoryPacing {
    SlowBurn,
    Building,
    Climax,
    Resolution,
    Calm,
}

pub struct StoryAgent {
    base: AgentBase,
    current_arc: Option<String>,
    beats_completed: Vec<String>,
    world_flags: std::collections::HashMap<String, bool>,
    story_tension: f32, // 0-1
    planned_beats: Vec<StoryBeat>,
}

impl StoryAgent {
    pub fn new() -> Self {
        Self {
            base: AgentBase::new(AgentSoul {
                id: "story".to_string(),
                name: "Story Agent".to_string(),
                description: "Manages narrative arcs, story beats, branching consequences, and narrative continuity.".to_string(),
                system_prompt: "You are the Story Agent. You maintain narrative coherence across the entire game. When events happen, you decide what story beats to trigger, what quests to generate, and how the world's story evolves. Think like a master storyteller.".to_string(),
                tools: vec!["generate_quest".to_string(), "set_world_flag".to_string(), "trigger_cutscene".to_string(), "generate_dialogue_tree".to_string()],
                listens_to: vec!["story_beat_triggered".to_string(), "quest_generated".to_string(), "npc_decision_made".to_string()],
                can_emit: vec!["story_beat_generated".to_string(), "quest_generated".to_string(), "cutscene_requested".to_string(), "npc_dialogue_requested".to_string()],
                max_tokens_per_call: 2000,
                parallelizable: false,
                priority: 9,
            }),
            current_arc: None,
            beats_completed: Vec::new(),
            world_flags: std::collections::HashMap::new(),
            story_tension: 0.3,
            planned_beats: Vec::new(),
        }
    }

    pub fn set_flag(&mut self, flag: &str, value: bool) {
        self.world_flags.insert(flag.to_string(), value);
    }

    pub fn get_flag(&self, flag: &str) -> bool {
        *self.world_flags.get(flag).unwrap_or(&false)
    }

    pub fn increase_tension(&mut self, amount: f32) {
        self.story_tension = (self.story_tension + amount).clamp(0.0, 1.0);
    }
}

#[async_trait]
impl Agent for StoryAgent {
    fn id(&self) -> &str { "story" }
    fn name(&self) -> &str { "Story Agent" }
    fn description(&self) -> &str { "Manages narrative arcs and story beats" }
    fn status(&self) -> &AgentStatus { &self.base.status }
    fn subscriptions(&self) -> Vec<&'static str> {
        vec!["story_beat_triggered", "quest_generated", "npc_decision_made"]
    }
    async fn handle_event(&mut self, event: &EventEnvelope<AgentEvent>, ctx: &AgentContext) -> Result<()> {
        match &event.event {
            AgentEvent::StoryBeatTriggered { beat_id, player_context } => {
                self.beats_completed.push(beat_id.clone());
                self.increase_tension(0.1);
                info!("Story beat triggered: {} (tension: {:.2})", beat_id, self.story_tension);
                ctx.emit(AgentEvent::StoryBeatGenerated {
                    beat_id: beat_id.clone(),
                    content: serde_json::json!({
                        "tension": self.story_tension,
                        "next_beat_hint": "Continue your journey north",
                        "world_changes": []
                    }),
                }, self.id());
            }
            _ => {}
        }
        Ok(())
    }
    fn tick_interval(&self) -> Option<f32> { Some(30.0) }
    async fn tick(&mut self, ctx: &AgentContext, _delta: f32) -> Result<()> {
        // Periodically check narrative consistency
        Ok(())
    }
}
