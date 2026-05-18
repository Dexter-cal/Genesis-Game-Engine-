//! Agent Memory System
//!
//! 4-tier memory architecture:
//! 1. Working Memory   — current session, fast, RAM only
//! 2. Episode Memory   — this session log, persisted to SQLite
//! 3. Long-term Memory — across all sessions, semantic search
//! 4. World State      — complete game snapshot

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Factual observation ("player helped Serafina at 14:32")
    Observation,
    /// Decision made by the agent
    Decision,
    /// Relationship change ("trust with player: +0.1")
    RelationshipChange,
    /// World event witnessed
    WorldEvent,
    /// Agent-created tool or code
    ToolCreation,
    /// Improvement applied to engine
    EngineImprovement,
    /// Research finding
    Research,
}

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub entity_id: Option<String>,   // related entity
    pub scene_id: Option<String>,    // where this happened
    pub significance: f32,           // 0.0 - 1.0
    pub timestamp: DateTime<Utc>,
    pub tags: Vec<String>,
    /// Embedding vector for semantic search (if computed)
    #[serde(skip)]
    pub embedding: Option<Vec<f32>>,
}

impl MemoryEntry {
    pub fn new(content: &str, memory_type: MemoryType, significance: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            memory_type,
            content: content.to_string(),
            entity_id: None,
            scene_id: None,
            significance,
            timestamp: Utc::now(),
            tags: Vec::new(),
            embedding: None,
        }
    }

    pub fn with_entity(mut self, entity_id: &str) -> Self {
        self.entity_id = Some(entity_id.to_string());
        self
    }

    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// The agent's memory store
pub struct AgentMemory {
    /// Working memory (most recent N entries in RAM)
    working: VecDeque<MemoryEntry>,
    working_max: usize,
    /// All memories (would be backed by SQLite in production)
    long_term: Vec<MemoryEntry>,
    /// Total memories ever stored
    total_stored: u64,
}

impl AgentMemory {
    pub fn new(working_max: usize) -> Self {
        Self {
            working: VecDeque::new(),
            working_max,
            long_term: Vec::new(),
            total_stored: 0,
        }
    }

    /// Store a new memory
    pub fn remember(&mut self, entry: MemoryEntry) {
        self.total_stored += 1;

        // Always add to working memory
        self.working.push_back(entry.clone());
        while self.working.len() > self.working_max {
            self.working.pop_front();
        }

        // Significant memories go to long-term
        if entry.significance >= 0.3 {
            self.long_term.push(entry);
            // Keep long-term manageable
            if self.long_term.len() > 10_000 {
                // Remove least significant
                self.long_term.sort_by(|a, b| a.significance.partial_cmp(&b.significance).unwrap());
                self.long_term.truncate(8_000);
            }
        }
    }

    /// Quick observation (low significance)
    pub fn observe(&mut self, content: &str) {
        self.remember(MemoryEntry::new(content, MemoryType::Observation, 0.2));
    }

    /// Important event (high significance)
    pub fn mark_important(&mut self, content: &str, entity: Option<&str>) {
        let mut entry = MemoryEntry::new(content, MemoryType::WorldEvent, 0.8);
        if let Some(e) = entity { entry.entity_id = Some(e.to_string()); }
        self.remember(entry);
    }

    /// Recent working memory as context string for LLM
    pub fn recent_context(&self, n: usize) -> String {
        self.working.iter()
            .rev()
            .take(n)
            .rev()
            .map(|m| format!("[{}] {}", m.timestamp.format("%H:%M"), m.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Search memories by keyword (simple substring match)
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&MemoryEntry> = self.long_term.iter()
            .filter(|m| m.content.to_lowercase().contains(&query_lower))
            .collect();
        results.sort_by(|a, b| b.significance.partial_cmp(&a.significance).unwrap());
        results.truncate(20);
        results
    }

    /// Search memories by entity ID
    pub fn search_by_entity(&self, entity_id: &str) -> Vec<&MemoryEntry> {
        self.long_term.iter()
            .filter(|m| m.entity_id.as_deref() == Some(entity_id))
            .collect()
    }

    /// Get memories from working memory in order
    pub fn working_memories(&self) -> impl Iterator<Item = &MemoryEntry> {
        self.working.iter()
    }

    pub fn total_stored(&self) -> u64 { self.total_stored }
    pub fn working_count(&self) -> usize { self.working.len() }
    pub fn long_term_count(&self) -> usize { self.long_term.len() }

    /// Clear working memory (new session)
    pub fn clear_working(&mut self) { self.working.clear(); }
}
