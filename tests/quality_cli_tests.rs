// Copyright © 2026 CTCycle
// Licensed under the MIT License.

use std::process::Command;

use llmeter::quality::adapter::build_quality_plan;
use llmeter::quality::catalog::{default_catalog, QualityFramework};
use serde_json::Value;
use tempfile::TempDir;

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

#[test]
fn every_quality_framework_has_a_cli_dry_run_plan_and_catalog_mapping() {
    let catalog = default_catalog();
    let base_url = "http://localhost:9123/v1";
    let model = "fixture-model";

    let frameworks = [
        QualityFramework::LightEval,
        QualityFramework::Inspect,
        QualityFramework::LmEvalHarness,
        QualityFramework::SweBench,
    ];

    for framework in frameworks {
        let entry = catalog
            .iter()
            .find(|entry| entry.framework_hint == framework)
            .unwrap_or_else(|| panic!("missing catalog entry for {}", framework.label()));
        let home = TempDir::new().expect("create quality CLI home");
        let output = Command::new(env!("CARGO_BIN_EXE_llmeter"))
            .env("LLMETER_HOME", home.path())
            .args([
                "--base-url",
                base_url,
                "quality",
                "plan",
                "--framework",
                framework.label(),
                "--task",
                entry.id.as_str(),
                "--model",
                model,
            ])
            .output()
            .expect("run quality plan CLI");

        assert!(
            output.status.success(),
            "{}: {}",
            framework.label(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let plan: Value = serde_json::from_slice(&output.stdout).expect("serialized quality plan");
        assert_eq!(
            plan["framework"],
            serde_json::to_value(framework).expect("serialize framework identity")
        );
        assert_eq!(plan["task"], entry.id);
        assert_eq!(plan["model"], model);
        assert_eq!(plan["dry_run"], true);
        assert_eq!(plan["catalog_entry"]["id"], entry.id);
        assert_eq!(plan["catalog_entry"]["requires_external_tool"], true);
        assert_eq!(plan["catalog_entry"]["requires_dataset"], true);
        assert_eq!(
            plan["catalog_entry"]["requires_code_execution"],
            entry.requires_code_execution
        );

        let preview = plan["command_preview"][0]
            .as_str()
            .expect("quality command preview");
        assert!(preview.contains(entry.id.as_str()), "{preview}");
        assert!(preview.contains(model), "{preview}");
        if framework != QualityFramework::SweBench {
            assert!(preview.contains(base_url), "{preview}");
        }
    }

    for entry in catalog {
        let plan = build_quality_plan(entry.framework_hint, &entry.id, model, base_url);
        let serialized = serde_json::to_value(&plan).expect("serialize catalog quality plan");
        assert_eq!(serialized["catalog_entry"]["id"], entry.id);
        assert_eq!(
            serialized["catalog_entry"]["requires_external_tool"],
            entry.requires_external_tool
        );
        assert_eq!(
            serialized["catalog_entry"]["requires_dataset"],
            entry.requires_dataset
        );
        assert_eq!(
            serialized["catalog_entry"]["requires_code_execution"],
            entry.requires_code_execution
        );
    }
}

#[test]
fn quality_framework_parser_rejects_unknown_frameworks_and_unknown_tasks_stay_planned() {
    assert!("not-a-framework".parse::<QualityFramework>().is_err());

    let plan = build_quality_plan(
        QualityFramework::Inspect,
        "framework-specific-unknown-task",
        "fixture-model",
        "http://localhost:9123/v1",
    );
    assert!(plan.dry_run);
    assert!(plan.catalog_entry.is_none());
    let serialized = serde_json::to_value(plan).expect("serialize unknown quality task");
    assert!(serialized["catalog_entry"].is_null());
}
