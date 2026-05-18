//! Node-based AI logic system for Genesis Engine
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Condition,
    Action,
    Selector,
    Sequence,
    Parallel,
    AiPrompt(String), // Specialized node for LLM-driven decision making
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub name: String,
    pub inputs: Vec<Uuid>,
    pub outputs: Vec<Uuid>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBehaviorGraph {
    pub id: Uuid,
    pub name: String,
    pub nodes: HashMap<Uuid, AiNode>,
    pub root_node: Option<Uuid>,
}

impl AiBehaviorGraph {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            nodes: HashMap::new(),
            root_node: None,
        }
    }

    pub fn add_node(&mut self, node: AiNode) {
        self.nodes.insert(node.id, node);
    }
}

pub struct AiNodeSystem;

impl AiNodeSystem {
    pub fn tick(graph: &AiBehaviorGraph) {
        // Logic to traverse the graph and execute AI behaviors
        // This integrates with genesis-agents and genesis-ai
    }
}
