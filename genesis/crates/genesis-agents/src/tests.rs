#[cfg(test)]
mod tests {
    use crate::{Agent, AgentKind};
    use crate::research::ResearchAgent;

    #[test]
    fn test_research_agent() {
        let mut agent = ResearchAgent::new();
        let report = agent.crawl("RPG");
        assert!(report.contains("RPG"));
    }

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("Test", AgentKind::CombatAgent, "model", "provider");
        assert_eq!(agent.id, "Test");
    }
}
