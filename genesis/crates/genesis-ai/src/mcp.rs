use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
}

pub struct McpRegistry {
    pub servers: Vec<McpServer>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self { servers: Vec::new() }
    }

    pub fn connect_claw_hub(&mut self) {
        tracing::info!("Connecting to ClawHub MCP registry (3,200+ skills available)");
    }
}
