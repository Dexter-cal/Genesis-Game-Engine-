pub mod orchestrator;
pub mod story;
pub mod world;
pub mod npc_brain;
pub mod interior_design;
pub mod architect;

pub use world::*;
pub use orchestrator::OrchestratorAgent;
pub use story::StoryAgent;
pub use npc_brain::NpcBrainAgent;
pub use interior_design::InteriorDesignAgent;
pub use architect::ArchitectAgent;

pub mod voice { pub use super::world::VoiceAgent; }
pub mod music { pub use super::world::MusicAgent; }
pub mod vfx { pub use super::world::VfxAgent; }
pub mod physics_agent { pub use super::world::PhysicsAgent; }
pub mod lighting { pub use super::world::LightingAgent; }
pub mod camera { pub use super::world::CameraAgent; }
pub mod rigging { pub use super::world::RiggingAgent; }
pub mod cinematic { pub use super::world::CinematicDirectorAgent; }
pub mod terrain { pub use super::world::TerrainAgent; }
pub mod weather { pub use super::world::WeatherAgent; }
pub mod research { pub use super::world::ResearchAgent; }
pub mod qa_scout { pub use super::world::QaScoutAgent; }
pub mod vision { pub use super::world::VisionAgent; }
pub mod analytics { pub use super::world::AnalyticsAgent; }
pub mod economy { pub use super::world::EconomyAgent; }
pub mod localization { pub use super::world::LocalizationAgent; }
pub mod accessibility { pub use super::world::AccessibilityAgent; }
pub mod ui_agent { pub use super::world::UiAgent; }
pub mod map_agent { pub use super::world::MapAgent; }
pub mod combat_agent { pub use super::world::CombatAgent; }
pub mod caption { pub use super::world::MarketingAgent; }
pub mod caption_agent { pub use super::world::MarketingAgent; }
pub mod marketing { pub use super::world::MarketingAgent; }
pub mod chaos { pub use super::world::ChaosAgent; }
pub mod import_agent { pub use super::world::ImportAgent; }
pub mod gpu_lab { pub use super::world::GpuLabAgent; }

pub fn create_all_agents() -> Vec<Box<dyn crate::base::Agent>> {
    vec![
        Box::new(OrchestratorAgent::new()),
        Box::new(StoryAgent::new()),
        Box::new(WorldGeneratorAgent::new()),
        Box::new(NpcBrainAgent::new()),
        Box::new(InteriorDesignAgent::new()),
        Box::new(VoiceAgent::new()),
        Box::new(MusicAgent::new()),
        Box::new(VfxAgent::new()),
        Box::new(PhysicsAgent::new()),
        Box::new(LightingAgent::new()),
        Box::new(CameraAgent::new()),
        Box::new(RiggingAgent::new()),
        Box::new(CinematicDirectorAgent::new()),
        Box::new(TerrainAgent::new()),
        Box::new(WeatherAgent::new()),
        Box::new(ResearchAgent::new()),
        Box::new(QaScoutAgent::new()),
        Box::new(VisionAgent::new()),
        Box::new(ArchitectAgent::new()),
        Box::new(AnalyticsAgent::new()),
        Box::new(EconomyAgent::new()),
        Box::new(LocalizationAgent::new()),
        Box::new(AccessibilityAgent::new()),
        Box::new(UiAgent::new()),
        Box::new(MapAgent::new()),
        Box::new(CombatAgent::new()),
        Box::new(MarketingAgent::new()),
        Box::new(ChaosAgent::new()),
        Box::new(ImportAgent::new()),
        Box::new(GpuLabAgent::new()),
    ]
}
