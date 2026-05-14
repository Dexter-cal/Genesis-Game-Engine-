//! World Generator Agent

use anyhow::Result;
use async_trait::async_trait;
use tracing::info;
use chronoverse_core::events::{AgentEvent, EventEnvelope};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};

macro_rules! simple_agent {
    ($name:ident, $id:expr, $display:expr, $desc:expr, $subs:expr, $emits:expr, $priority:expr, $tick:expr) => {
        pub struct $name { pub base: AgentBase }
        impl $name {
            pub fn new() -> Self {
                Self { base: AgentBase::new(AgentSoul {
                    id: $id.to_string(), name: $display.to_string(),
                    description: $desc.to_string(),
                    system_prompt: concat!("You are the ", $display, ". ", $desc).to_string(),
                    tools: vec!["create_tool".to_string()],
                    listens_to: $subs.iter().map(|s: &&str| s.to_string()).collect(),
                    can_emit: $emits.iter().map(|s: &&str| s.to_string()).collect(),
                    max_tokens_per_call: 1000,
                    parallelizable: true,
                    priority: $priority,
                })}
            }
        }
        #[async_trait]
        impl Agent for $name {
            fn id(&self) -> &str { $id }
            fn name(&self) -> &str { $display }
            fn description(&self) -> &str { $desc }
            fn status(&self) -> &AgentStatus { &self.base.status }
            fn subscriptions(&self) -> Vec<&'static str> { vec![$($subs),*] }
            fn tick_interval(&self) -> Option<f32> { $tick }
            async fn handle_event(&mut self, event: &EventEnvelope<AgentEvent>, ctx: &AgentContext) -> Result<()> {
                Ok(())
            }
        }
    };
}

simple_agent!(WorldGeneratorAgent, "world_generator", "World Generator Agent",
    "Generates 3D world chunks, biomes, terrain features, and environmental props.",
    &["world_chunk_requested"], &["world_chunk_generated", "asset_3d_requested"], 8, Some(5.0));

simple_agent!(VoiceAgent, "voice", "Voice Agent",
    "Manages NPC voice synthesis, voice cloning, lip sync, and emotional voice modulation.",
    &["voice_synth_requested", "npc_dialogue_generated"], &["voice_synth_complete"], 7, None);

simple_agent!(MusicAgent, "music", "Music Agent",
    "Generates adaptive music, manages music stems, transitions music based on scene mood.",
    &["music_requested"], &["music_generated"], 6, Some(2.0));

simple_agent!(VfxAgent, "vfx", "VFX Agent",
    "Creates and manages particle systems, visual effects, post-processing for every scene event.",
    &["sfx_requested"], &["sfx_generated"], 6, None);

simple_agent!(LightingAgent, "lighting", "Lighting Agent",
    "Controls dynamic lighting, shadow maps, GI, mood-driven color grading for scenes.",
    &["world_chunk_generated"], &[], 5, Some(1.0));

simple_agent!(CameraAgent, "camera", "Camera Agent",
    "Controls all camera modes (1st/3rd/iso/top-down/VR), cinematic framing, transitions.",
    &["story_beat_generated", "cutscene_requested"], &[], 7, Some(0.2));

simple_agent!(RiggingAgent, "rigging", "Rigging Agent",
    "Auto-rigs 3D models using UniRig/HumanRig. Falls back to Mixamo automation.",
    &["asset_rig_requested", "asset_3d_generated"], &["asset_rig_complete"], 5, None);

simple_agent!(CinematicDirectorAgent, "cinematic", "Cinematic Director Agent",
    "Creates cutscene timelines, camera choreography, dramatic lighting for story moments.",
    &["cutscene_requested"], &["cutscene_generated"], 6, None);

simple_agent!(TerrainAgent, "terrain", "Terrain Agent",
    "Sculpts terrain, applies erosion simulation, manages biome transitions and foliage.",
    &["world_chunk_requested"], &["world_chunk_generated"], 5, Some(10.0));

simple_agent!(WeatherAgent, "weather", "Weather Agent",
    "Controls weather system, seasonal changes, weather effects on gameplay.",
    &[], &[], 4, Some(5.0));

simple_agent!(ResearchAgent, "research", "Research Agent",
    "Searches the web for real-world accuracy data, lore research, asset discovery.",
    &["research_requested"], &["research_complete"], 4, None);

simple_agent!(QaScoutAgent, "qa_scout", "QA Scout Agent",
    "Plays ahead of the player, finds bugs, validates scenes, files bug reports.",
    &["world_chunk_generated"], &["visual_check_requested"], 4, Some(10.0));

simple_agent!(VisionAgent, "vision", "Vision Agent",
    "Captures game screenshots and uses vision models to detect visual issues and validate placement.",
    &["visual_check_requested"], &["visual_issue_found", "visual_check_passed"], 5, None);

simple_agent!(AnalyticsAgent, "analytics", "Analytics Agent",
    "Tracks player behavior, generates heatmaps, provides game balance recommendations.",
    &["analytics_event"], &[], 3, Some(30.0));

simple_agent!(EconomyAgent, "economy", "Economy Agent",
    "Manages dynamic pricing, item balance, trade routes, and market simulation.",
    &[], &[], 3, Some(60.0));

simple_agent!(LocalizationAgent, "localization", "Localization Agent",
    "Translates all game text using Qwen-MT, handles RTL languages, manages string tables.",
    &[], &[], 4, None);

simple_agent!(AccessibilityAgent, "accessibility", "Accessibility Agent",
    "Ensures all content meets accessibility standards: captions, color-blindness, audio cues.",
    &[], &[], 4, None);

simple_agent!(UiAgent, "ui", "UI Agent",
    "Auto-generates HUD layouts from Mechanics Bible, manages dynamic UI state changes.",
    &[], &[], 6, Some(0.1));

simple_agent!(MapAgent, "map", "Map Agent",
    "Generates minimaps and world maps from scene data, manages fog of war, quest markers.",
    &["world_chunk_generated"], &[], 4, Some(1.0));

simple_agent!(CombatAgent, "combat", "Combat Agent",
    "Manages combat flow, damage calculation, boss AI patterns, status effects.",
    &[], &[], 8, Some(0.016));

simple_agent!(MarketingAgent, "marketing", "Marketing Agent",
    "Generates game posters, trailers, press kits, Steam page descriptions on publish.",
    &[], &[], 2, None);

simple_agent!(ChaosAgent, "chaos", "Chaos Agent",
    "Introduces unscripted emergent events to keep the world feeling alive and unpredictable.",
    &[], &[], 3, Some(300.0));

simple_agent!(ImportAgent, "import", "Import Agent",
    "Handles asset pipeline: format conversion, LOD generation, compression, atlas packing.",
    &["asset_3d_generated", "texture_generated"], &[], 5, None);

simple_agent!(GpuLabAgent, "gpu_lab", "GPU Lab Agent",
    "Manages Kaggle/Colab GPU lab connections, job submission, model generation queue.",
    &["asset_3d_requested", "music_requested", "sfx_requested"],
    &["gpu_lab_job_submitted", "gpu_lab_job_complete", "gpu_lab_job_failed"], 5, Some(2.0));
