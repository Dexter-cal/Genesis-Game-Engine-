//! Agent-to-Agent Collaboration Protocol
//! Allows agents to "negotiate" and share tasks autonomously.

use serde::{Serialize, Deserialize};
use crate::AgentKind;
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationMessage {
    pub from: String,
    pub to: AgentKind,
    pub task_details: String,
    pub priority: u8,
}

pub struct CollabHub {
    pub messages: VecDeque<CollaborationMessage>,
}

impl CollabHub {
    pub fn new() -> Self {
        Self { messages: VecDeque::new() }
    }

    pub fn send(&mut self, msg: CollaborationMessage) {
        tracing::info!("Agent Collaboration: {} -> {:?} ({})", msg.from, msg.to, msg.task_details);
        self.messages.push_back(msg);
    }

    pub fn poll_for_agent(&mut self, kind: &AgentKind) -> Option<CollaborationMessage> {
        let idx = self.messages.iter().position(|m| m.to == *kind)?;
        self.messages.remove(idx)
    }
}
