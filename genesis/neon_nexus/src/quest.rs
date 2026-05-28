//! Dynamic Quest System for Neon Nexus
use genesis_agents::Agent;

pub struct Quest {
    pub title: String,
    pub description: String,
    pub reward_xp: u32,
}

pub fn generate_dynamic_quest(agent: &mut Agent) -> Quest {
    let quest = Quest {
        title: "Secure the Data Terminal".to_string(),
        description: "An AI core in the lower levels is leaking encrypted data. Secure it before the Guardians do.".to_string(),
        reward_xp: 500,
    };

    agent.memory.push(genesis_agents::MemoryKind::Decision, &format!("Generated quest: {}", quest.title), "QuestSystem");
    quest
}
