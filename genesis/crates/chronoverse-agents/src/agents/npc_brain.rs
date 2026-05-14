//! NPC Brain Agent
//!
//! The most complex agent in ChronoVerse.
//! Gives every NPC a mind: personality, memory, goals, emotions, decisions.
//!
//! Architecture:
//! - Fast path: Behavior Tree handles routine decisions (0 tokens)
//! - Medium path: Small local LLM for contextual decisions
//! - Slow path: Large LLM for complex emotional/social situations
//!
//! Each NPC's brain runs independently. The agent manages ALL NPCs.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, debug};

use chronoverse_core::events::{AgentEvent, EventEnvelope, EventPriority};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};
use crate::memory::{MemoryEntry, MemoryType};

/// NPC personality profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcPersonality {
    /// Openness to experience (0-1)
    pub openness: f32,
    /// Conscientiousness (0-1)
    pub conscientiousness: f32,
    /// Extraversion (0-1)
    pub extraversion: f32,
    /// Agreeableness (0-1)
    pub agreeableness: f32,
    /// Neuroticism / emotional stability (0-1)
    pub neuroticism: f32,
    /// Courage / bravery (0-1)
    pub courage: f32,
    /// Greed (0-1)
    pub greed: f32,
    /// Loyalty (0-1)
    pub loyalty: f32,
    /// Honesty (0-1)
    pub honesty: f32,
    /// Curiosity (0-1)
    pub curiosity: f32,
}

impl NpcPersonality {
    pub fn random(seed: u64) -> Self {
        // Simple LCG for deterministic personality from ID hash
        let r = |n: u64| ((n.wrapping_mul(6364136223846793005) + 1442695040888963407) >> 48) as f32 / 65535.0;
        Self {
            openness:          r(seed),
            conscientiousness: r(seed ^ 1),
            extraversion:      r(seed ^ 2),
            agreeableness:     r(seed ^ 3),
            neuroticism:       r(seed ^ 4),
            courage:           r(seed ^ 5),
            greed:             r(seed ^ 6),
            loyalty:           r(seed ^ 7),
            honesty:           r(seed ^ 8),
            curiosity:         r(seed ^ 9),
        }
    }

    pub fn to_summary(&self) -> String {
        let mut traits = Vec::new();
        if self.courage > 0.7 { traits.push("brave"); }
        else if self.courage < 0.3 { traits.push("cowardly"); }
        if self.loyalty > 0.7 { traits.push("loyal"); }
        if self.greed > 0.7 { traits.push("greedy"); }
        if self.honesty > 0.7 { traits.push("honest"); }
        else if self.honesty < 0.3 { traits.push("deceptive"); }
        if self.extraversion > 0.7 { traits.push("outgoing"); }
        else if self.extraversion < 0.3 { traits.push("introverted"); }
        if self.curiosity > 0.7 { traits.push("curious"); }
        traits.join(", ")
    }
}

/// Emotional state of an NPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcEmotion {
    pub happiness: f32,   // -1 to 1
    pub fear: f32,        // 0 to 1
    pub anger: f32,       // 0 to 1
    pub sadness: f32,     // 0 to 1
    pub surprise: f32,    // 0 to 1
    pub disgust: f32,     // 0 to 1
    pub trust: f32,       // 0 to 1
    pub anticipation: f32,// 0 to 1
}

impl NpcEmotion {
    pub fn neutral() -> Self {
        Self { happiness: 0.5, fear: 0.0, anger: 0.0, sadness: 0.0,
               surprise: 0.0, disgust: 0.0, trust: 0.5, anticipation: 0.3 }
    }

    pub fn dominant_emotion(&self) -> &'static str {
        let vals = [
            (self.happiness, "happy"),
            (self.fear, "fearful"),
            (self.anger, "angry"),
            (self.sadness, "sad"),
            (self.surprise, "surprised"),
            (self.disgust, "disgusted"),
            (self.trust, "trusting"),
            (self.anticipation, "anticipating"),
        ];
        vals.iter().max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, name)| *name)
            .unwrap_or("neutral")
    }

    pub fn decay(&mut self, delta: f32) {
        let rate = delta * 0.05; // emotions decay slowly
        self.fear = (self.fear - rate).max(0.0);
        self.anger = (self.anger - rate).max(0.0);
        self.sadness = (self.sadness - rate * 0.5).max(0.0);
        self.surprise = (self.surprise - rate * 2.0).max(0.0);
    }
}

/// An NPC's goal (what they're trying to accomplish)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcGoal {
    pub id: String,
    pub description: String,
    pub priority: f32,
    pub deadline: Option<f64>, // game time
    pub progress: f32,
    pub blocked_by: Vec<String>,
}

/// An NPC's relationship with another entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRelationship {
    pub target_id: String,
    pub trust: f32,         // -1 to 1
    pub respect: f32,       // -1 to 1
    pub affection: f32,     // -1 to 1
    pub fear_of: f32,       // 0 to 1
    pub rivalry: f32,       // 0 to 1
    pub history: Vec<String>, // significant interactions
}

impl NpcRelationship {
    pub fn neutral(target_id: &str) -> Self {
        Self { target_id: target_id.to_string(), trust: 0.0, respect: 0.0,
               affection: 0.0, fear_of: 0.0, rivalry: 0.0, history: Vec::new() }
    }
}

/// Complete NPC brain state
#[derive(Debug, Clone)]
pub struct NpcBrainState {
    pub npc_id: String,
    pub name: String,
    pub role: String,
    pub backstory: String,
    pub personality: NpcPersonality,
    pub emotion: NpcEmotion,
    pub goals: Vec<NpcGoal>,
    pub relationships: HashMap<String, NpcRelationship>,
    pub knowledge: Vec<String>, // facts this NPC knows
    pub current_action: Option<String>,
    pub current_dialogue_topic: Option<String>,
    pub schedule: Vec<ScheduleEntry>,
    pub live_ai_threshold: f32, // when to use LLM vs behavior tree
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub time_start: f32,   // 0-24 hours
    pub time_end: f32,
    pub activity: String,
    pub location: Option<String>,
    pub priority: u8,
}

impl NpcBrainState {
    pub fn new(npc_id: &str, name: &str, role: &str) -> Self {
        let seed = npc_id.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        Self {
            npc_id: npc_id.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            backstory: String::new(),
            personality: NpcPersonality::random(seed),
            emotion: NpcEmotion::neutral(),
            goals: Vec::new(),
            relationships: HashMap::new(),
            knowledge: Vec::new(),
            current_action: None,
            current_dialogue_topic: None,
            schedule: Vec::new(),
            live_ai_threshold: 0.5,
        }
    }

    pub fn context_for_llm(&self, player_input: &str) -> String {
        format!(
            "You are {}, a {} in this game world.\n\
             Personality: {}\n\
             Current emotion: {}\n\
             Current goal: {}\n\
             Known facts: {}\n\
             Player says: \"{}\"\n\
             Respond in-character. Be concise (1-3 sentences). \
             Show emotion through word choice, not narration.",
            self.name,
            self.role,
            self.personality.to_summary(),
            self.emotion.dominant_emotion(),
            self.goals.first().map(|g| g.description.as_str()).unwrap_or("none"),
            self.knowledge.join("; "),
            player_input
        )
    }
}

/// The NPC Brain Agent
pub struct NpcBrainAgent {
    base: AgentBase,
    /// Brain state per NPC
    brains: HashMap<String, NpcBrainState>,
    /// Pending decisions queue
    pending_decisions: Vec<(String, serde_json::Value)>,
    /// Tick accumulator per NPC (for thinking intervals)
    think_timers: HashMap<String, f32>,
    /// How often each NPC "thinks" (in game seconds)
    think_interval: f32,
}

impl NpcBrainAgent {
    pub fn new() -> Self {
        Self {
            base: AgentBase::new(AgentSoul {
                id: "npc_brain".to_string(),
                name: "NPC Brain Agent".to_string(),
                description: "Gives every NPC a mind: memory, personality, goals, emotions, decisions".to_string(),
                system_prompt: "You are the NPC Brain Agent. You give non-player characters realistic thoughts, emotions, and responses. Stay in character. Be concise. React authentically based on personality and relationships.".to_string(),
                tools: vec![
                    "spawn_entity".to_string(),
                    "set_npc_action".to_string(),
                    "update_npc_relationship".to_string(),
                    "fire_game_event".to_string(),
                    "query_world_state".to_string(),
                    "create_tool".to_string(),
                ],
                listens_to: vec![
                    "npc_decision_requested".to_string(),
                    "npc_dialogue_requested".to_string(),
                    "npc_memory_updated".to_string(),
                    "npc_emotion_changed".to_string(),
                ],
                can_emit: vec![
                    "npc_decision_made".to_string(),
                    "npc_dialogue_generated".to_string(),
                    "npc_emotion_changed".to_string(),
                    "npc_memory_updated".to_string(),
                ],
                max_tokens_per_call: 300,
                parallelizable: true,
                priority: 8,
            }),
            brains: HashMap::new(),
            pending_decisions: Vec::new(),
            think_timers: HashMap::new(),
            think_interval: 0.5,
        }
    }

    /// Get or create brain state for an NPC
    pub fn get_or_create_brain(&mut self, npc_id: &str) -> &mut NpcBrainState {
        self.brains.entry(npc_id.to_string())
            .or_insert_with(|| NpcBrainState::new(npc_id, "Unknown NPC", "villager"))
    }

    /// Register a new NPC with its full data
    pub fn register_npc(&mut self, npc_id: &str, name: &str, role: &str, backstory: &str) {
        let mut brain = NpcBrainState::new(npc_id, name, role);
        brain.backstory = backstory.to_string();
        self.brains.insert(npc_id.to_string(), brain);
        self.think_timers.insert(npc_id.to_string(), 0.0);
        info!("NPC Brain registered: {} ({})", name, npc_id);
    }

    /// Fast path: behavior tree decision for simple situations
    fn behavior_tree_decide(&self, brain: &NpcBrainState, context: &serde_json::Value) -> Option<String> {
        // Simple rule-based decisions that don't need an LLM
        let player_nearby = context.get("player_nearby").and_then(|v| v.as_bool()).unwrap_or(false);
        let in_danger = context.get("threat_level").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.5;
        let health_pct = context.get("health_pct").and_then(|v| v.as_f64()).unwrap_or(1.0);

        // Low health + threat → flee (always, regardless of personality)
        if health_pct < 0.2 && in_danger {
            return Some("flee".to_string());
        }

        // Cowardly NPC → flee from any threat
        if in_danger && brain.personality.courage < 0.3 {
            return Some("flee".to_string());
        }

        // Brave NPC + threat → fight
        if in_danger && brain.personality.courage > 0.7 {
            return Some("fight".to_string());
        }

        // Friendly NPC + player nearby → greet
        if player_nearby && brain.personality.extraversion > 0.5 {
            if brain.current_action.is_none() {
                return Some("greet_player".to_string());
            }
        }

        // Continue current activity
        if let Some(action) = &brain.current_action {
            return Some(action.clone());
        }

        // Default: idle
        Some("idle".to_string())
    }

    /// Generate dialogue response using LLM
    async fn generate_dialogue(
        &mut self,
        npc_id: &str,
        player_input: &str,
        ctx: &AgentContext,
    ) -> Result<String> {
        let brain = match self.brains.get(npc_id) {
            Some(b) => b,
            None => {
                return Ok("...".to_string()); // NPC not registered
            }
        };

        let prompt = brain.context_for_llm(player_input);
        self.base.set_thinking(&format!("{}...", &prompt[..50.min(prompt.len())]));

        // Check budget and route to appropriate model
        let response = {
            let budget = ctx.budget.read();
            let tier = budget.route("npc_complex_thought", 0.6);
            drop(budget);

            // In production: actual LLM call based on tier
            // For now: placeholder that shows the system works
            format!("I understand what you're saying about '{}'. \
                     As a {}, I must tell you that...",
                     &player_input[..20.min(player_input.len())],
                     brain.role)
        };

        self.base.record_call(50); // 50 tokens used

        // Update NPC memory
        if let Some(brain) = self.brains.get_mut(npc_id) {
            brain.knowledge.push(format!("Player said: '{}'", player_input));
            if brain.knowledge.len() > 50 {
                brain.knowledge.remove(0);
            }
        }

        Ok(response)
    }
}

#[async_trait]
impl Agent for NpcBrainAgent {
    fn id(&self) -> &str { "npc_brain" }
    fn name(&self) -> &str { "NPC Brain Agent" }
    fn description(&self) -> &str { "Gives every NPC a mind: personality, memory, goals, emotions" }

    fn subscriptions(&self) -> Vec<&'static str> {
        vec![
            "npc_decision_requested",
            "npc_dialogue_requested",
            "npc_memory_updated",
            "npc_emotion_changed",
        ]
    }

    fn status(&self) -> &AgentStatus { &self.base.status }
    fn tick_interval(&self) -> Option<f32> { Some(0.5) }

    async fn initialize(&mut self, ctx: &AgentContext) -> Result<()> {
        info!("NPC Brain Agent initialized — managing {} NPCs", self.brains.len());
        Ok(())
    }

    async fn handle_event(
        &mut self,
        event: &EventEnvelope<AgentEvent>,
        ctx: &AgentContext,
    ) -> Result<()> {
        match &event.event {
            AgentEvent::NpcDecisionRequested { npc_id, context } => {
                let npc_id = npc_id.clone();
                let context = context.clone();

                // Try fast behavior tree first
                if let Some(brain) = self.brains.get(&npc_id) {
                    if let Some(action) = self.behavior_tree_decide(brain, &context) {
                        ctx.emit(
                            AgentEvent::NpcDecisionMade {
                                npc_id: npc_id.clone(),
                                action: action.clone(),
                                parameters: serde_json::json!({}),
                            },
                            self.id(),
                        );
                        debug!("BT decision for {}: {}", npc_id, action);
                        return Ok(());
                    }
                }

                // Fall through to LLM decision
                self.pending_decisions.push((npc_id, context));
            }

            AgentEvent::NpcDialogueRequested { npc_id, player_input, context } => {
                let npc_id = npc_id.clone();
                let player_input = player_input.clone();

                let response = self.generate_dialogue(&npc_id, &player_input, ctx).await?;

                // Get emotion for the response
                let emotion = self.brains.get(&npc_id)
                    .map(|b| b.emotion.dominant_emotion().to_string())
                    .unwrap_or("neutral".to_string());

                ctx.emit(
                    AgentEvent::NpcDialogueGenerated {
                        npc_id,
                        text: response,
                        emotion,
                    },
                    self.id(),
                );
            }

            AgentEvent::NpcMemoryUpdated { npc_id, memory_type, content } => {
                if let Some(brain) = self.brains.get_mut(npc_id) {
                    brain.knowledge.push(content.clone());
                    debug!("NPC {} memory updated: {}", npc_id, content);
                }
            }

            AgentEvent::NpcEmotionChanged { npc_id, emotion, intensity } => {
                if let Some(brain) = self.brains.get_mut(npc_id) {
                    match emotion.as_str() {
                        "happy"    => brain.emotion.happiness = *intensity,
                        "fearful"  => brain.emotion.fear = *intensity,
                        "angry"    => brain.emotion.anger = *intensity,
                        "sad"      => brain.emotion.sadness = *intensity,
                        "surprised"=> brain.emotion.surprise = *intensity,
                        _ => {}
                    }
                }
            }

            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self, ctx: &AgentContext, delta: f32) -> Result<()> {
        // Decay emotions for all NPCs
        for brain in self.brains.values_mut() {
            brain.emotion.decay(delta);
        }

        // Process pending LLM decisions (throttled)
        // In production: process up to N per tick to avoid overwhelming
        if let Some((npc_id, context)) = self.pending_decisions.pop() {
            // Simple fallback for now
            ctx.emit(
                AgentEvent::NpcDecisionMade {
                    npc_id: npc_id.clone(),
                    action: "idle".to_string(),
                    parameters: serde_json::json!({ "reason": "llm_pending" }),
                },
                self.id(),
            );
        }

        Ok(())
    }
}
