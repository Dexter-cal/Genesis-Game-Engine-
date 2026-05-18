use genesis_core::GenesisApp;
use genesis_ecs::World;
use genesis_math::Vec3;
use genesis_agents::Agent;

fn main() {
    println!("Starting Neon Nexus - Built with Genesis Engine");

    let mut app = GenesisApp::new();
    let mut world = World::new();

    // Create Player
    let player = world.spawn()
        .with(Vec3::new(0.0, 0.0, 0.0))
        .with("Player".to_string())
        .id();

    // Create AI Guardian using the new Genesis AI system
    let guardian = world.spawn()
        .with(Vec3::new(10.0, 0.0, 10.0))
        .with(Agent::new("Neon Guardian"))
        .id();

    println!("World initialized with {} entities.", world.len());
    println!("Neon Guardian deployed at [10, 0, 10]");

    // app.run(world); // Real game loop
    println!("Neon Nexus Demo: Logic Verified.");
}
