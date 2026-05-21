//! Autonomous Self-Improvement System for Genesis Agents
//! Agents can assess their own performance and propose code changes or tool additions.

use serde::{Serialize, Deserialize};
use crate::Agent;
use crate::code_fix::{CodeFixAgent, CodeFixRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementProposal {
    pub agent_name: String,
    pub assessment: String,
    pub proposed_tool: Option<String>,
    pub proposed_fix: Option<CodeFixRequest>,
    pub confidence: f32,
}

pub struct SelfImproveAgent {
    pub history: Vec<ImprovementProposal>,
}

impl SelfImproveAgent {
    pub fn new() -> Self {
        Self { history: Vec::new() }
    }

    pub fn assess(&mut self, agent: &Agent) -> Option<ImprovementProposal> {
        // Mock logic: If an agent's name suggests it's struggling with a task, propose a tool
        if agent.name.contains("World") && !self.has_improvement(&agent.name) {
            let proposal = ImprovementProposal {
                agent_name: agent.name.clone(),
                assessment: "Current world generation lacks biological diversity in forest biomes.".to_string(),
                proposed_tool: Some("flora_scatter_pro".to_string()),
                confidence: 0.88,
            };
            self.history.push(proposal.clone());
            return Some(proposal);
        }
        None
    }

    fn has_improvement(&self, name: &str) -> bool {
        self.history.iter().any(|p| p.agent_name == name)
    }
}

pub trait Improve {
    fn self_diagnose(&self) -> String;
}
