use serde_json::json;
use crate::{AgentTool, ToolCategory};

pub fn default_tools() -> Vec<AgentTool> {
    vec![
        AgentTool {
            name: "auto_rig".to_string(),
            description: "Automatically rig a humanoid 3D mesh using AI skeleton prediction.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "mesh_id": { "type": "string" },
                    "skeleton_type": { "type": "string", "enum": ["humanoid", "quadruped", "biped_winged"] }
                }
            }),
            category: ToolCategory::AssetOp,
            requires: vec!["mesh_edit".to_string()],
            cost_tokens: 500,
            timeout_ms: 5000,
        },
        AgentTool {
            name: "generate_vfx".to_string(),
            description: "Generate a node-based VFX graph from a natural language description.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string" },
                    "target_node": { "type": "string" }
                }
            }),
            category: ToolCategory::SceneEdit,
            requires: vec!["vfx_write".to_string()],
            cost_tokens: 800,
            timeout_ms: 10000,
        }
    ]
}
