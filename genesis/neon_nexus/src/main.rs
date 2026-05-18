use genesis_core::Engine;
use genesis_ecs::World;
use genesis_agents::AgentCouncil;

fn main() {
    let mut engine = Engine::new("Neon Nexus");
    let mut world = World::new();
    let mut council = AgentCouncil::new();

    tracing::info!("Starting Neon Nexus...");

    // Initializing the world with AI agents
    engine.run(&mut world, &mut council);
}
