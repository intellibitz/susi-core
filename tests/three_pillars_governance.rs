//! Governance drift checks for the three-pillar contract.
#[test]
fn current_guidance_names_exactly_three_architectural_pillars() {
    for (name, text) in [
        ("AGENTS.md", include_str!("../AGENTS.md")),
        ("README.md", include_str!("../README.md")),
    ] {
        assert!(
            text.contains("three core pillars"),
            "{name}: current directive missing"
        );
        for stale in [
            "five pillars",
            "five Apache-2.0 core pillars",
            "5 open-core pillars",
            "5 primary open-core pillars",
        ] {
            assert!(!text.contains(stale), "{name}: stale taxonomy {stale}");
        }
    }
}
