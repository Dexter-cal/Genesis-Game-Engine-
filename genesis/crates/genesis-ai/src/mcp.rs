//! MCP Integration for Genesis — Model Context Protocol
//! Connecting Genesis agents to external tools and knowledge bases (ClawHub, etc.)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub url: String,
    pub status: McpStatus,
    pub tools: Vec<McpTool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum McpStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

pub struct McpRegistry {
    pub servers: HashMap<String, McpServer>,
}

impl McpRegistry {
    pub fn new() -> Self {
        let mut servers = HashMap::new();

        // Mocking ClawHub integration (3,200+ skills)
        servers.insert("ClawHub".to_string(), McpServer {
            name: "ClawHub".to_string(),
            url: "https://mcp.clawhub.io/v1".to_string(),
            status: McpStatus::Connected,
            tools: vec![
                McpTool {
                    name: "search_assets".to_string(),
                    description: "Search for 3D models and textures in the global marketplace".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": { "type": "string" },
                            "format": { "type": "string", "enum": ["glb", "fbx", "obj"] }
                        }
                    }),
                },
                McpTool {
                    name: "auto_rig".to_string(),
                    description: "Automatically rig a humanoid mesh using AI skeletons".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "mesh_id": { "type": "string" }
                        }
                    }),
                }
            ],
        });

        Self { servers }
    }

    pub fn get_tool(&self, server_name: &str, tool_name: &str) -> Option<&McpTool> {
        self.servers.get(server_name)?.tools.iter().find(|t| t.name == tool_name)
    }

    pub fn list_all_tools(&self) -> Vec<(&String, &McpTool)> {
        self.servers.iter()
            .flat_map(|(name, server)| server.tools.iter().map(move |t| (name, t)))
            .collect()
    }
}
