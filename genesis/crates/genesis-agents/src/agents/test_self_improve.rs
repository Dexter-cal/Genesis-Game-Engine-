#[cfg(test)]
mod tests {
    use crate::*;
    use crate::agents::self_improve::SelfImproveAgent;

    #[test]
    fn test_assessment() {
        let mut agent = Agent::new("tester", AgentKind::CombatAgent, "test-model", "test-provider");
        agent.total_errors = 10;

        let improver = SelfImproveAgent;
        let limits = improver.assess_limitations(&agent);
        assert!(limits.iter().any(|l| l.contains("High error rate")));

        let tool = improver.propose_tool(&limits[0]);
        assert_eq!(tool.name, "dynamic_optimizer");
    }
}
