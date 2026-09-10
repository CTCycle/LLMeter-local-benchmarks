use crate::quality::catalog::{find_catalog_entry, QualityFramework};
use crate::quality::manifest::QualityPlan;

pub fn build_quality_plan(
    framework: QualityFramework,
    task: &str,
    model: &str,
    base_url: &str,
) -> QualityPlan {
    let command_preview = match framework {
        QualityFramework::LightEval => vec![format!(
            "uvx lighteval endpoint openai {task} --model-args model_name={model},base_url={base_url}"
        )],
        QualityFramework::Inspect => vec![format!(
            "uvx inspect-ai eval {task} --model openai/{model} --model-base-url {base_url}"
        )],
        QualityFramework::LmEvalHarness => vec![format!(
            "uvx lm_eval --model local-completions --model_args model={model},base_url={base_url} --tasks {task}"
        )],
        QualityFramework::SweBench => vec![format!(
            "python -m swebench.harness.run_evaluation --predictions_path predictions.json --max_workers 1 --split {task} --model_name_or_path {model}"
        )],
    };

    QualityPlan {
        framework,
        task: task.to_string(),
        model: model.to_string(),
        dry_run: true,
        command_preview,
        catalog_entry: find_catalog_entry(task),
    }
}
