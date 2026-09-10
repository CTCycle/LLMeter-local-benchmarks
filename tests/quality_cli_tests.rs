use llmeter::quality::adapter::build_quality_plan;
use llmeter::quality::catalog::QualityFramework;

#[test]
fn quality_plan_builds_dry_run_command_preview_with_resolved_base_url() {
    let plan = build_quality_plan(
        QualityFramework::LightEval,
        "leaderboard|mmlu|5",
        "llama3.1",
        "http://localhost:9123/v1",
    );
    assert!(plan.dry_run);
    assert!(plan.command_preview[0].contains("uvx lighteval"));
    assert!(plan.command_preview[0].contains("leaderboard|mmlu|5"));
    assert!(plan.command_preview[0].contains("base_url=http://localhost:9123/v1"));
    assert!(!plan.command_preview[0].contains("localhost:8000"));
}
