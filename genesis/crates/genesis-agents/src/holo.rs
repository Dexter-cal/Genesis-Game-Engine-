use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Holo Layer: A versioned behavior layer for AI agents.
/// Layers can be stacked to modify or refine an agent's logic without altering the base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoloLayer {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    /// System prompt fragments added by this layer
    pub prompt_extension: Option<String>,
    /// Behavioral parameters (e.g. "aggression", "creativity")
    pub parameter_overrides: HashMap<String, f32>,
    /// Tool availability overrides
    pub enabled_tools: Vec<String>,
    pub disabled_tools: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub active: bool,
    /// Priority in the stack (higher = applied later/stronger)
    pub priority: i32,
}

impl HoloLayer {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            id: format!("holo_{}_{}", name.to_lowercase().replace(" ", "_"), Utc::now().timestamp()),
            name: name.to_string(),
            version: version.to_string(),
            author: "System".to_string(),
            prompt_extension: None,
            parameter_overrides: HashMap::new(),
            enabled_tools: Vec::new(),
            disabled_tools: Vec::new(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            active: true,
            priority: 0,
        }
    }
}

/// Holo Stack: Manages the collection of layers for an agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HoloStack {
    pub layers: Vec<HoloLayer>,
}

impl HoloStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn add_layer(&mut self, layer: HoloLayer) {
        self.layers.push(layer);
        self.layers.sort_by_key(|l| l.priority);
    }

    pub fn remove_layer(&mut self, id: &str) {
        self.layers.retain(|l| l.id != id);
    }

    pub fn get_effective_prompt(&self, base_prompt: &str) -> String {
        let mut prompt = base_prompt.to_string();
        for layer in &self.layers {
            if layer.active {
                if let Some(ext) = &layer.prompt_extension {
                    prompt = format!("{}\n\n[Holo Layer: {} v{}]\n{}", prompt, layer.name, layer.version, ext);
                }
            }
        }
        prompt
    }

    pub fn apply_parameters(&self, mut params: HashMap<String, f32>) -> HashMap<String, f32> {
        for layer in &self.layers {
            if layer.active {
                for (k, v) in &layer.parameter_overrides {
                    params.insert(k.clone(), *v);
                }
            }
        }
        params
    }
}
