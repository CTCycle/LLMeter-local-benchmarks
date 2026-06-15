use clap::Parser;
use llmeter::cli::{Cli, Commands, QualityCommands};
use llmeter::quality::adapter::build_quality_plan;
use llmeter::quality::catalog::{default_catalog, QualityFramework};

#[test]
fn quality_list_catalog_contains_expected_entries() {
    let catalog = default_catalog();
    assert!(catalog.iter().any(|entry| entry.id == "mmlu"));
    assert!(catalog.iter().any(|entry| entry.id == "swe-bench-lite"));
}

#[test]
fn quality_plan_cli_parses_framework_and_task() {
    let cli = Cli::parse_from([
        "llmeter",
        "quality",
        "plan",
        "--framework",
        "swe-bench",
        "--task",
        "swe-bench-lite",
        "--model",
        "llama3.1",
    ]);

    match cli.command {
        Some(Commands::Quality {
            quality_command:
                QualityCommands::Plan {
                    framework,
                    task,
                    model,
                },
        }) => {
            assert_eq!(framework, QualityFramework::SweBench);
            assert_eq!(task, "swe-bench-lite");
            assert_eq!(model, "llama3.1");
        }
        _ => panic!("expected quality plan"),
    }
}

#[test]
fn quality_plan_builds_dry_run_command_preview() {
    let plan = build_quality_plan(
        QualityFramework::LightEval,
        "leaderboard|mmlu|5",
        "llama3.1",
    );
    assert!(plan.dry_run);
    assert!(plan.command_preview[0].contains("uvx lighteval"));
    assert!(plan.command_preview[0].contains("leaderboard|mmlu|5"));
}
