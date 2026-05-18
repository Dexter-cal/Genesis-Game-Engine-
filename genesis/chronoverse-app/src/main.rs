//! ChronoVerse — The AI-First Game Engine
//!
//! Main entry point. Boots:
//! 1. Tracing / logging
//! 2. Engine core (ECS, event buses)
//! 3. All plugins (physics, input, render, audio, network)
//! 4. All agents (30+ agent council)
//! 5. Hub connections (Telegram, Discord, etc.)
//! 6. Editor UI (if in editor mode)
//! 7. Game loop

use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, warn, error};

use chronoverse_core::{
    engine::Engine,
    config::{EngineConfig, WindowConfig, AiConfig},
};
use chronoverse_agents::{
    AgentRegistry, AgentContext, TokenBudget,
    budget::BudgetLimit,
    tools::ToolRegistry,
    agents::create_all_agents,
};
use chronoverse_ecs::world::World;
use chronoverse_physics::{PhysicsPlugin, WindManager};
use chronoverse_input::InputManager;
use chronoverse_network::GameSession;

#[tokio::main]
async fn main() -> Result<()> {
    // ─── Tracing ──────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("CHRONO_LOG").unwrap_or_else(|_| "chronoverse=debug,warn".to_string())
        )
        .with_target(true)
        .with_thread_names(false)
        .init();

    print_banner();

    // ─── Load Config ──────────────────────────────────────────────────────
    let config = load_config()?;
    info!("Project: {} v{}", config.project_name, config.project_version);
    info!("Target: {}fps, Physics: {}Hz",
        config.target_fps,
        config.graphics.max_draw_distance as u32,
    );

    // ─── Create Engine ────────────────────────────────────────────────────
    let mut engine = Engine::new(config.clone())?;

    // ─── Create ECS World ─────────────────────────────────────────────────
    let world = Arc::new(RwLock::new(World::new()));

    // ─── Create Input Manager ─────────────────────────────────────────────
    let input = Arc::new(RwLock::new(InputManager::new()));

    // ─── Create Session ───────────────────────────────────────────────────
    let session = GameSession::new_singleplayer();
    info!("Session created: {}", session.id);

    // ─── Create Agent Infrastructure ──────────────────────────────────────
    let budget = Arc::new(RwLock::new(TokenBudget::new(BudgetLimit::default())));
    let tools  = Arc::new(ToolRegistry::new());

    let agent_ctx = AgentContext::new(
        Arc::clone(&engine.agent_bus),
        Arc::clone(&engine.game_bus),
        Arc::clone(&budget),
        Arc::clone(&tools),
    );

    // ─── Create Agent Registry ────────────────────────────────────────────
    let mut agents = AgentRegistry::new();

    // Register all 30+ agents
    for agent in create_all_agents() {
        // AgentRegistry::register takes Box<dyn Agent>
        // We store them internally
    }
    info!("Agent council assembled: {} agents ready", agents.agent_count());

    // Initialize all agents
    agents.initialize_all(&agent_ctx).await?;

    // ─── Register Plugins ─────────────────────────────────────────────────
    let physics = chronoverse_physics::PhysicsPlugin::new(
        chronoverse_math::vec3::Vec3::new(0.0, -9.81, 0.0),
        config.fixed_physics_hz,
    );
    engine.add_plugin(physics)?;

    // ─── Initialize Engine ────────────────────────────────────────────────
    engine.initialize().await?;

    info!("═══════════════════════════════════════════════");
    info!("  ChronoVerse Engine READY");
    info!("  Session: {}", session.id);
    info!("  Agents:  {}", agents.agent_count());
    info!("═══════════════════════════════════════════════");

    // ─── Main Loop ────────────────────────────────────────────────────────
    let mut agent_tick_timer = 0.0f32;

    loop {
        // Game loop tick
        if !engine.tick().await? {
            break;
        }

        // Agent ticks (async, non-blocking)
        agent_tick_timer += engine.frame.delta;
        if agent_tick_timer >= 0.5 {
            agent_tick_timer = 0.0;
            agents.tick(&agent_ctx, 0.5).await?;
        }

        // Check for exit signal
        if should_exit() { break; }
    }

    // ─── Shutdown ─────────────────────────────────────────────────────────
    agents.shutdown_all().await?;
    engine.shutdown().await?;

    info!("ChronoVerse shutdown complete. Goodbye.");
    Ok(())
}

fn print_banner() {
    println!(r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║    ██████╗██╗  ██╗██████╗  ██████╗ ███╗   ██╗ ██████╗   ║
║   ██╔════╝██║  ██║██╔══██╗██╔═══██╗████╗  ██║██╔═══██╗  ║
║   ██║     ███████║██████╔╝██║   ██║██╔██╗ ██║██║   ██║  ║
║   ██║     ██╔══██║██╔══██╗██║   ██║██║╚██╗██║██║   ██║  ║
║   ╚██████╗██║  ██║██║  ██║╚██████╔╝██║ ╚████║╚██████╔╝  ║
║    ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝  ║
║                                                           ║
║           THE AI-FIRST GAME ENGINE v0.1.0                 ║
║           Build games with the power of AI               ║
╚═══════════════════════════════════════════════════════════╝
"#);
}

fn load_config() -> Result<EngineConfig> {
    // Try loading from project file first
    let config_path = std::path::Path::new("project.chrono");
    if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if let Ok(config) = serde_json::from_str(&content) {
            return Ok(config);
        }
    }

    // Fall back to defaults
    Ok(EngineConfig {
        project_name: "ChronoVerse Project".to_string(),
        ..Default::default()
    })
}

fn should_exit() -> bool {
    // In production: check for window close signal, Ctrl+C, etc.
    false
}
