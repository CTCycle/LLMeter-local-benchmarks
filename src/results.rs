use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::benchmarks::base::BenchmarkResultRecord;
use crate::performance::config::PerformancePlan;
use crate::performance::load::ModelLoadMeasurement;
use crate::performance::model_inventory::ModelInventoryMeasurement;
use crate::performance::provider_probe::ProviderCapabilityReport;
use crate::performance::resource::EnvironmentSnapshot;
use crate::performance::telemetry::TelemetrySummary;
use crate::quality::manifest::QualityPlan;
use crate::utils;

pub const RESULT_SCHEMA_VERSION: &str = "2.3";

fn default_schema_version() -> String {
    RESULT_SCHEMA_VERSION.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BenchmarkRunKind {
    Benchmark,
    Performance,
    QualityPlan,
}

impl BenchmarkRunKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Benchmark => "benchmark",
            Self::Performance => "performance",
            Self::QualityPlan => "quality-plan",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub run_id: String,
    pub created_at: String,
    pub models: Vec<String>,
    pub benchmark_ids: Vec<String>,
    pub config: HashMap<String, Value>,
    pub results: Vec<BenchmarkResultRecord>,
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub run_kind: Option<BenchmarkRunKind>,
    #[serde(default)]
    pub environment: Option<EnvironmentSnapshot>,
    #[serde(default)]
    pub performance_plan: Option<PerformancePlan>,
    #[serde(default)]
    pub quality_plan: Option<QualityPlan>,
    #[serde(default)]
    pub provider_capabilities: Option<ProviderCapabilityReport>,
    #[serde(default)]
    pub model_load_measurements: Option<Vec<ModelLoadMeasurement>>,
    #[serde(default)]
    pub model_inventory_measurements: Option<Vec<ModelInventoryMeasurement>>,
    #[serde(default)]
    pub telemetry_summary: Option<TelemetrySummary>,
}

pub struct ResultStore {
    output_dir: PathBuf,
}

impl ResultStore {
    pub fn new(output_dir: &Path) -> Self {
        ResultStore {
            output_dir: output_dir.to_path_buf(),
        }
    }

    pub fn new_run_id(&self, models: &[String]) -> String {
        build_run_id(&utils::utc_now_run_id_stamp(), std::process::id(), models)
    }

    pub fn save_json(&self, run: &BenchmarkRun) -> anyhow::Result<PathBuf> {
        let path = self.output_dir.join(format!("{}.json", run.run_id));
        utils::ensure_dir(&self.output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                self.output_dir.display()
            )
        })?;
        let content = serde_json::to_string_pretty(run)?;
        utils::atomic_write(&path, content.as_bytes())
            .with_context(|| format!("Failed to write JSON result: {}", path.display()))?;
        Ok(path)
    }

    pub fn save_csv(&self, run: &BenchmarkRun) -> anyhow::Result<PathBuf> {
        let path = self.output_dir.join(format!("{}.csv", run.run_id));
        utils::ensure_dir(&self.output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                self.output_dir.display()
            )
        })?;

        // Collect all metric keys across all records
        let mut metric_keys_set: HashSet<String> = HashSet::new();
        for record in &run.results {
            for key in record.metrics.keys() {
                metric_keys_set.insert(key.clone());
            }
        }
        let mut metric_keys: Vec<String> = metric_keys_set.into_iter().collect();
        metric_keys.sort();

        let mut wtr = csv::Writer::from_writer(Vec::new());

        // Write header row
        let mut header_row: Vec<String> = vec![
            "run_id".to_string(),
            "created_at".to_string(),
            "schema_version".to_string(),
            "run_kind".to_string(),
            "provider".to_string(),
            "base_url".to_string(),
            "profile".to_string(),
            "telemetry_level".to_string(),
            "load_measurement_mode".to_string(),
            "estimated_load_overhead_ms".to_string(),
            "load_overhead_confidence".to_string(),
            "swap_used_ratio".to_string(),
            "memory_used_ratio".to_string(),
            "gpu_names".to_string(),
            "benchmark_id".to_string(),
            "benchmark_name".to_string(),
            "model".to_string(),
            "run_index".to_string(),
            "prompt_name".to_string(),
            "error".to_string(),
            "response_preview".to_string(),
        ];
        header_row.extend(metric_keys.iter().cloned());
        wtr.write_record(&header_row)?;

        // Write data rows
        for record in &run.results {
            let load = run
                .model_load_measurements
                .as_ref()
                .and_then(|items| items.iter().find(|item| item.model == record.model));
            let mut row: Vec<String> = vec![
                run.run_id.clone(),
                run.created_at.clone(),
                run.schema_version.clone(),
                run.run_kind
                    .as_ref()
                    .map(|kind| kind.label().to_string())
                    .unwrap_or_default(),
                run.config
                    .get("provider")
                    .map(value_to_cell)
                    .unwrap_or_default(),
                run.config
                    .get("base_url")
                    .map(value_to_cell)
                    .unwrap_or_default(),
                run.config
                    .get("profile")
                    .map(value_to_cell)
                    .unwrap_or_default(),
                run.performance_plan
                    .as_ref()
                    .map(|plan| format!("{:?}", plan.telemetry))
                    .unwrap_or_default(),
                run.performance_plan
                    .as_ref()
                    .map(|plan| format!("{:?}", plan.load_measurement))
                    .unwrap_or_default(),
                load.and_then(|item| item.estimated_load_overhead_ms)
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                load.map(|item| format!("{:?}", item.confidence))
                    .unwrap_or_default(),
                run.environment
                    .as_ref()
                    .map(|env| env.swap_used_ratio.to_string())
                    .unwrap_or_default(),
                run.environment
                    .as_ref()
                    .map(|env| env.memory_used_ratio.to_string())
                    .unwrap_or_default(),
                run.environment
                    .as_ref()
                    .and_then(|env| env.gpu_probe_output.clone())
                    .map(|value| sanitize_csv_text(&value))
                    .unwrap_or_default(),
                sanitize_csv_text(&record.benchmark_id),
                sanitize_csv_text(&record.benchmark_name),
                sanitize_csv_text(&record.model),
                record.run_index.map(|i| i.to_string()).unwrap_or_default(),
                record
                    .prompt_name
                    .as_deref()
                    .map(sanitize_csv_text)
                    .unwrap_or_default(),
                record
                    .error
                    .as_deref()
                    .map(sanitize_csv_text)
                    .unwrap_or_default(),
                record
                    .response_preview
                    .as_deref()
                    .map(sanitize_csv_text)
                    .unwrap_or_default(),
            ];
            for key in &metric_keys {
                let value = record
                    .metrics
                    .get(key)
                    .map(value_to_cell)
                    .unwrap_or_default();
                row.push(value);
            }
            wtr.write_record(&row)?;
        }

        let csv_content = wtr.into_inner()?;
        utils::atomic_write(&path, &csv_content)
            .with_context(|| format!("Failed to write CSV file: {}", path.display()))?;

        Ok(path)
    }

    pub fn latest_json_files(&self, limit: usize) -> Vec<PathBuf> {
        if !self.output_dir.exists() {
            return Vec::new();
        }
        let mut files: Vec<PathBuf> = self
            .output_dir
            .read_dir()
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension()? == "json" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        files.sort_by(|a, b| {
            b.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
        });
        files.truncate(limit);
        files
    }

    pub fn latest_report_files(&self, limit: usize) -> Vec<PathBuf> {
        if !self.output_dir.exists() {
            return Vec::new();
        }
        let mut files: Vec<PathBuf> = self
            .output_dir
            .read_dir()
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_string_lossy().to_string();
                if name.ends_with(".report.md") || name.ends_with(".report.html") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        files.sort_by(|a, b| {
            b.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
        });
        files.truncate(limit);
        files
    }

    pub fn load_json(&self, path: &PathBuf) -> anyhow::Result<BenchmarkRun> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let run: BenchmarkRun = serde_json::from_str(&content)
            .with_context(|| format!("Invalid JSON in {}", path.display()))?;
        Ok(run)
    }
}

fn value_to_cell(value: &Value) -> String {
    match value {
        Value::String(s) => sanitize_csv_text(s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => sanitize_csv_text(&value.to_string()),
    }
}

fn sanitize_csv_text(value: &str) -> String {
    if matches!(value.as_bytes().first(), Some(b'=' | b'+' | b'-' | b'@')) {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

fn build_run_id(stamp: &str, pid: u32, models: &[String]) -> String {
    let model_part = if models.is_empty() {
        "models".to_string()
    } else {
        let joined = models.iter().take(2).cloned().collect::<Vec<_>>().join("-");
        let slug = utils::slugify(&joined);
        if slug.len() > 60 {
            slug[..60].to_string()
        } else {
            slug
        }
    };
    format!("{stamp}-p{pid}-{model_part}")
}

#[cfg(test)]
mod tests {
    use super::{build_run_id, sanitize_csv_text};

    #[test]
    fn build_run_id_varies_by_process_for_same_timestamp_and_models() {
        let models = vec!["qwen3.5:2b".to_string()];
        let first = build_run_id("2026-06-16T135142123456Z", 100, &models);
        let second = build_run_id("2026-06-16T135142123456Z", 101, &models);

        assert_ne!(first, second);
        assert!(first.contains("-p100-"));
        assert!(second.contains("-p101-"));
    }

    #[test]
    fn sanitize_csv_text_neutralizes_formula_prefixes_only() {
        for prefix in ['=', '+', '-', '@'] {
            let value = format!("{prefix}SUM(A1:A2)");
            assert_eq!(sanitize_csv_text(&value), format!("'{value}"));
        }
        assert_eq!(sanitize_csv_text("42"), "42");
        assert_eq!(sanitize_csv_text("normal text"), "normal text");
    }
}
