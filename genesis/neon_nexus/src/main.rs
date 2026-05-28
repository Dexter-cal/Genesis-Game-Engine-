use genesis_core::GenesisApp;
use genesis_ecs::World;
use genesis_math::Vec3;
use genesis_agents::{Agent, AgentKind, AgentCouncil};

mod combat;
mod quest;
use combat::{Combatant, CombatState};

fn main() {
    println!("Starting Neon Nexus - Built with Genesis Engine");

    let mut app = GenesisApp::new();
    let mut world = World::new();

    // Create Player
    let player = world.spawn()
        .with(Vec3::new(0.0, 0.0, 0.0))
        .with("Player".to_string())
        .id();

    // Create AI Guardian
    let mut guardian_agent = Agent::new("Guardian_01", AgentKind::CombatAgent, "qwen2.5:7b", "ollama");
    let guardian_combat = Combatant { health: 100.0, aggression: 0.8, state: CombatState::Idle };

    let guardian = world.spawn()
        .with(Vec3::new(10.0, 0.0, 10.0))
        .with(guardian_agent)
        .id();

    println!("World initialized with entities.");

    // Trigger Council Tick (Simulation)
    let mut council = AgentCouncil::new();
    council.tick(0.016);

    // Dynamic Quest
    let mut quest_agent = Agent::new("Oracle", AgentKind::QuestAgent, "qwen2.5:7b", "ollama");
    let quest = quest::generate_dynamic_quest(&mut quest_agent);
    println!("New Dynamic Quest: {} - {}", quest.title, quest.description);

    println!("Neon Nexus Demo: Systems Operational.");
}
