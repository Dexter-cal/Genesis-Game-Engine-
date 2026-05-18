//! ChronoVerse CLI
//!
//! Usage:
//!   chronoverse-cli new my-game         Create a new game project
//!   chronoverse-cli build               Build the current project
//!   chronoverse-cli run                 Run the current project
//!   chronoverse-cli publish             Publish to ChronoVerse Store
//!   chronoverse-cli status              Show build status
//!   chronoverse-cli agent list          List all agents
//!   chronoverse-cli agent disable NAME  Disable an agent
//!   chronoverse-cli export --platform pc  Export for PC
//!   chronoverse-cli export --platform web Export for WebGL
//!   chronoverse-cli model download NAME  Download an AI model

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(2).map(|s| s.as_str()).unwrap_or("my-game");
            create_project(name)?;
        }
        Some("build") => {
            println!("🔨 Starting build...");
            println!("Connect the engine and run build from there.");
        }
        Some("run") => {
            println!("▶️  Running project...");
        }
        Some("version") => {
            println!("ChronoVerse CLI v{}", chronoverse_core::ENGINE_VERSION);
        }
        Some("help") | None => {
            print_help();
        }
        Some(cmd) => {
            eprintln!("Unknown command: {}. Run `chronoverse-cli help` for usage.", cmd);
        }
    }
    Ok(())
}

fn create_project(name: &str) -> Result<()> {
    println!("Creating project: {}", name);
    std::fs::create_dir_all(format!("{}/assets/models", name))?;
    std::fs::create_dir_all(format!("{}/assets/audio", name))?;
    std::fs::create_dir_all(format!("{}/assets/textures", name))?;
    std::fs::create_dir_all(format!("{}/scenes", name))?;
    std::fs::create_dir_all(format!("{}/scripts", name))?;
    std::fs::create_dir_all(format!("{}/models", name))?;
    std::fs::create_dir_all(format!("{}/builds", name))?;

    // Create default project config
    let config = serde_json::json!({
        "project_name": name,
        "project_version": "0.1.0",
        "target_fps": 60.0,
        "fixed_physics_hz": 120.0,
        "window": { "title": name, "width": 1920, "height": 1080 }
    });
    std::fs::write(
        format!("{}/project.chrono", name),
        serde_json::to_string_pretty(&config)?
    )?;

    // Create hello world script
    std::fs::write(
        format!("{}/scripts/hello.cv", name),
        r#"# Welcome to ChronoVerse!
# This is your first ChronoScript

@on("GAME_START")
def on_start():
    debug.log("Hello, ChronoVerse!")
    world.spawn_entity("prop", "Magic Orb", position=[0, 1, 0])

@tick(interval=2.0)
def heartbeat():
    debug.log("Game is running! Time: " + str(world.time))
"#
    )?;

    println!("✅ Project '{}' created!", name);
    println!("");
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  chronoverse          # Open in editor");
    println!("  chronoverse-cli run  # Run directly");

    Ok(())
}

fn print_help() {
    println!("ChronoVerse CLI v{}", chronoverse_core::ENGINE_VERSION);
    println!("");
    println!("USAGE:");
    println!("  chronoverse-cli <COMMAND> [OPTIONS]");
    println!("");
    println!("COMMANDS:");
    println!("  new <name>           Create a new game project");
    println!("  run                  Run the project in the current directory");
    println!("  build                Build without running");
    println!("  publish              Publish to ChronoVerse Store");
    println!("  export               Export to platforms (--platform pc|web|android|ios)");
    println!("  agent list           List all AI agents and their status");
    println!("  agent disable <id>   Disable a specific agent");
    println!("  model list           List installed AI models");
    println!("  model download <id>  Download an AI model from HuggingFace");
    println!("  model convert <path> Convert model to GGUF format");
    println!("  lab generate         Generate a Kaggle/Colab notebook");
    println!("  version              Show version information");
    println!("  help                 Show this help");
    println!("");
    println!("EXAMPLES:");
    println!("  chronoverse-cli new my-rpg");
    println!("  chronoverse-cli export --platform web --output ./dist");
    println!("  chronoverse-cli model download kokoro-82m");
}
