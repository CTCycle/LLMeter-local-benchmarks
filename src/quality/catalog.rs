use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum QualityFamily {
    Knowledge,
    Reasoning,
    Truthfulness,
    Code,
    SoftwareEngineering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QualityFramework {
    LightEval,
    Inspect,
    LmEvalHarness,
    SweBench,
}

impl QualityFramework {
    pub fn label(self) -> &'static str {
        match self {
            QualityFramework::LightEval => "lighteval",
            QualityFramework::Inspect => "inspect-ai",
            QualityFramework::LmEvalHarness => "lm-eval-harness",
            QualityFramework::SweBench => "swe-bench",
        }
    }
}

impl std::str::FromStr for QualityFramework {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "lighteval" => Ok(Self::LightEval),
            "inspect" | "inspect-ai" => Ok(Self::Inspect),
            "lm-eval-harness" | "lm_eval" | "lm-eval" => Ok(Self::LmEvalHarness),
            "swe-bench" | "swebench" => Ok(Self::SweBench),
            other => Err(format!("Unknown framework '{other}'.")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityBenchmarkCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub family: QualityFamily,
    pub framework_hint: QualityFramework,
    pub default_metric: String,
    pub requires_external_tool: bool,
    pub requires_dataset: bool,
    pub requires_code_execution: bool,
    pub notes: String,
}

pub fn default_catalog() -> Vec<QualityBenchmarkCatalogEntry> {
    vec![
        entry(
            "mmlu",
            "MMLU",
            QualityFamily::Knowledge,
            QualityFramework::LightEval,
            "accuracy",
            false,
        ),
        entry(
            "gsm8k",
            "GSM8K",
            QualityFamily::Reasoning,
            QualityFramework::LightEval,
            "accuracy",
            false,
        ),
        entry(
            "arc-challenge",
            "ARC Challenge",
            QualityFamily::Reasoning,
            QualityFramework::LmEvalHarness,
            "accuracy",
            false,
        ),
        entry(
            "hellaswag",
            "HellaSwag",
            QualityFamily::Reasoning,
            QualityFramework::LmEvalHarness,
            "accuracy",
            false,
        ),
        entry(
            "truthfulqa",
            "TruthfulQA",
            QualityFamily::Truthfulness,
            QualityFramework::Inspect,
            "truthfulness",
            false,
        ),
        entry(
            "winogrande",
            "Winogrande",
            QualityFamily::Reasoning,
            QualityFramework::LmEvalHarness,
            "accuracy",
            false,
        ),
        entry(
            "humaneval",
            "HumanEval",
            QualityFamily::Code,
            QualityFramework::Inspect,
            "pass@1",
            true,
        ),
        entry(
            "swe-bench-lite",
            "SWE-bench Lite",
            QualityFamily::SoftwareEngineering,
            QualityFramework::SweBench,
            "% resolved",
            true,
        ),
        entry(
            "swe-bench-verified",
            "SWE-bench Verified",
            QualityFamily::SoftwareEngineering,
            QualityFramework::SweBench,
            "% resolved",
            true,
        ),
        entry(
            "swe-bench-full",
            "SWE-bench Full",
            QualityFamily::SoftwareEngineering,
            QualityFramework::SweBench,
            "% resolved",
            true,
        ),
        entry(
            "swe-bench-multilingual",
            "SWE-bench Multilingual",
            QualityFamily::SoftwareEngineering,
            QualityFramework::SweBench,
            "% resolved",
            true,
        ),
    ]
}

pub fn find_catalog_entry(id: &str) -> Option<QualityBenchmarkCatalogEntry> {
    default_catalog().into_iter().find(|entry| entry.id == id)
}

fn entry(
    id: &str,
    display_name: &str,
    family: QualityFamily,
    framework_hint: QualityFramework,
    default_metric: &str,
    requires_code_execution: bool,
) -> QualityBenchmarkCatalogEntry {
    QualityBenchmarkCatalogEntry {
        id: id.to_string(),
        display_name: display_name.to_string(),
        family,
        framework_hint,
        default_metric: default_metric.to_string(),
        requires_external_tool: true,
        requires_dataset: true,
        requires_code_execution,
        notes: format!(
            "{display_name} should be executed through {} rather than reimplemented inside the Rust CLI.",
            framework_hint.label()
        ),
    }
}
