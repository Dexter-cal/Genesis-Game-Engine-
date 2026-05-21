//! Genesis Dedicated Server
//! Runs a headless game server for multiplayer sessions.

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    info!("Genesis Dedicated Server starting...");
    info!("Listening for player connections...");
    // Full server implementation in the network crate
    tokio::signal::ctrl_c().await?;
    info!("Server shutting down.");
    Ok(())
}
