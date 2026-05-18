//! Token Budget Manager
//!
//! Tracks token usage across all agents.
//! Routes requests to cheapest/fastest model that meets quality needs.
//! Enforces daily budgets. Caches repeated prompts.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;

/// Model quality tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModelTier {
    /// Behavior tree / pure code (0 tokens)
    Code,
    /// Tiny local model (0.8B, ~400MB) — phone
    LocalTiny,
    /// Small local model (4B) — good quality, fast
    LocalSmall,
    /// Large local model (70B) — high quality, slow
    LocalLarge,
    /// Cheap cloud API (GPT-4o-mini, Gemini Flash)
    CloudCheap,
    /// Premium cloud API (Claude, GPT-4o) — best quality
    CloudPremium,
}

/// A model routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub task_type: String,
    pub min_tier: ModelTier,
    pub preferred_tier: ModelTier,
    pub max_tokens: u32,
    pub cache_ttl_secs: u64,
}

/// Token budget for cloud APIs (daily limits)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimit {
    pub daily_cloud_tokens: u64,
    pub daily_vision_calls: u32,
    pub daily_gpu_lab_jobs: u32,
}

impl Default for BudgetLimit {
    fn default() -> Self {
        Self {
            daily_cloud_tokens: 500_000,
            daily_vision_calls: 100,
            daily_gpu_lab_jobs: 50,
        }
    }
}

/// Usage tracking for current period
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UsageTracker {
    pub cloud_tokens_used: u64,
    pub vision_calls: u32,
    pub gpu_lab_jobs: u32,
    pub local_tokens_used: u64,
    pub cache_hits: u64,
    pub reset_at_unix: u64, // epoch seconds when this resets
}

impl UsageTracker {
    pub fn new() -> Self {
        let tomorrow = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs() + 86400;
        Self { reset_at_unix: tomorrow, ..Default::default() }
    }

    pub fn should_reset(&self) -> bool {
        SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs() > self.reset_at_unix
    }

    pub fn reset(&mut self) {
        let tomorrow = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs() + 86400;
        *self = Self { reset_at_unix: tomorrow, ..Default::default() }
    }
}

/// Response cache entry
struct CacheEntry {
    response: String,
    expires_at: u64,
    hits: u32,
}

/// The token budget manager
pub struct TokenBudget {
    pub limits: BudgetLimit,
    pub usage: UsageTracker,
    /// Prompt hash → cached response
    cache: HashMap<u64, CacheEntry>,
    routing_rules: Vec<RoutingRule>,
    /// Local model availability
    pub local_tiny_available: bool,
    pub local_small_available: bool,
    pub local_large_available: bool,
    /// Cloud key availability
    pub cloud_keys: HashMap<String, String>,
}

impl TokenBudget {
    pub fn new(limits: BudgetLimit) -> Self {
        Self {
            limits,
            usage: UsageTracker::new(),
            cache: HashMap::new(),
            routing_rules: Self::default_routing_rules(),
            local_tiny_available: false,
            local_small_available: false,
            local_large_available: false,
            cloud_keys: HashMap::new(),
        }
    }

    fn default_routing_rules() -> Vec<RoutingRule> {
        vec![
            RoutingRule {
                task_type: "npc_simple_dialogue".to_string(),
                min_tier: ModelTier::LocalTiny,
                preferred_tier: ModelTier::LocalSmall,
                max_tokens: 200,
                cache_ttl_secs: 300,
            },
            RoutingRule {
                task_type: "npc_complex_thought".to_string(),
                min_tier: ModelTier::LocalSmall,
                preferred_tier: ModelTier::LocalLarge,
                max_tokens: 500,
                cache_ttl_secs: 60,
            },
            RoutingRule {
                task_type: "story_planning".to_string(),
                min_tier: ModelTier::LocalLarge,
                preferred_tier: ModelTier::CloudPremium,
                max_tokens: 4000,
                cache_ttl_secs: 3600,
            },
            RoutingRule {
                task_type: "code_generation".to_string(),
                min_tier: ModelTier::LocalSmall,
                preferred_tier: ModelTier::CloudPremium,
                max_tokens: 2000,
                cache_ttl_secs: 3600,
            },
            RoutingRule {
                task_type: "npc_basic_action".to_string(),
                min_tier: ModelTier::Code,
                preferred_tier: ModelTier::Code,
                max_tokens: 0,
                cache_ttl_secs: 0,
            },
            RoutingRule {
                task_type: "world_generation".to_string(),
                min_tier: ModelTier::LocalLarge,
                preferred_tier: ModelTier::CloudPremium,
                max_tokens: 8000,
                cache_ttl_secs: 86400,
            },
        ]
    }

    /// Determine the best model tier for a given task
    pub fn route(&self, task_type: &str, complexity: f32) -> ModelTier {
        // Check if task uses pure code (0 tokens)
        if complexity < 0.2 {
            return ModelTier::Code;
        }

        // Find rule for this task type
        let rule = self.routing_rules.iter()
            .find(|r| r.task_type == task_type)
            .cloned()
            .unwrap_or(RoutingRule {
                task_type: task_type.to_string(),
                min_tier: ModelTier::LocalSmall,
                preferred_tier: ModelTier::LocalSmall,
                max_tokens: 500,
                cache_ttl_secs: 60,
            });

        // Try preferred tier first, fall back to available
        let preferred = rule.preferred_tier;

        match preferred {
            ModelTier::LocalTiny if self.local_tiny_available => ModelTier::LocalTiny,
            ModelTier::LocalSmall if self.local_small_available => ModelTier::LocalSmall,
            ModelTier::LocalLarge if self.local_large_available => ModelTier::LocalLarge,
            ModelTier::CloudPremium if self.has_cloud_budget() => ModelTier::CloudPremium,
            ModelTier::CloudCheap if self.has_cloud_budget() => ModelTier::CloudCheap,
            // Fallbacks
            _ if self.local_small_available => ModelTier::LocalSmall,
            _ if self.local_tiny_available => ModelTier::LocalTiny,
            _ if self.has_cloud_budget() => ModelTier::CloudCheap,
            _ => ModelTier::Code, // Last resort: behavior tree
        }
    }

    /// Check if we have cloud API budget remaining
    pub fn has_cloud_budget(&self) -> bool {
        !self.cloud_keys.is_empty() &&
        self.usage.cloud_tokens_used < self.limits.daily_cloud_tokens
    }

    /// Check and get cached response
    pub fn get_cache(&mut self, prompt_hash: u64) -> Option<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if let Some(entry) = self.cache.get_mut(&prompt_hash) {
            if entry.expires_at > now {
                entry.hits += 1;
                self.usage.cache_hits += 1;
                return Some(entry.response.clone());
            } else {
                self.cache.remove(&prompt_hash);
            }
        }
        None
    }

    /// Store response in cache
    pub fn cache_response(&mut self, prompt_hash: u64, response: String, ttl_secs: u64) {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs() + ttl_secs;
        self.cache.insert(prompt_hash, CacheEntry { response, expires_at, hits: 0 });
    }

    /// Record cloud token usage
    pub fn record_cloud_usage(&mut self, tokens: u64) {
        self.usage.cloud_tokens_used += tokens;
        if self.usage.should_reset() { self.usage.reset(); }
    }

    /// Record local token usage (no cost, just tracking)
    pub fn record_local_usage(&mut self, tokens: u64) {
        self.usage.local_tokens_used += tokens;
    }

    /// Add a cloud API key
    pub fn add_cloud_key(&mut self, provider: &str, key: &str) {
        self.cloud_keys.insert(provider.to_string(), key.to_string());
    }

    /// Get percentage of daily cloud budget used
    pub fn cloud_budget_percent(&self) -> f32 {
        self.usage.cloud_tokens_used as f32 / self.limits.daily_cloud_tokens as f32
    }

    /// Total cache hit count
    pub fn cache_hits(&self) -> u64 { self.usage.cache_hits }

    /// Evict expired cache entries
    pub fn evict_expired_cache(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.cache.retain(|_, entry| entry.expires_at > now);
    }
}

/// Simple string hash for cache keys
pub fn hash_prompt(s: &str) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}
