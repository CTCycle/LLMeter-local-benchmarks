use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::benchmarks::base::BenchmarkResultRecord;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub run_id: String,
    pub created_at: String,
    pub models: Vec<String>,
    pub benchmark_ids: Vec<String>,
    pub config: HashMap<String, Value>,
    pub results: Vec<BenchmarkResultRecord>,
}

pub struct ResultStore {
    output_dir: PathBuf,
}

impl ResultStore {
    pub fn new(output_dir: &PathBuf) -> Self {
        ResultStore {
            output_dir: output_dir.clone(),
        }
    }

    pub fn new_run_id(&self, models: &[String]) -> String {
        let stamp = utils::utc_now_iso()
            .replace(':', "")
            .replace("+0000", "Z")
            .replace("+00:00", "Z");
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
        format!("{stamp}-{model_part}")
    }

    pub fn save_json(&self, run: &BenchmarkRun) -> anyhow::Result<PathBuf> {
        let path = self.output_dir.join(format!("{}.json", run.run_id));
        utils::ensure_dir(&self.output_dir)
            .with_context(|| format!("Failed to create output directory: {}", self.output_dir.display()))?;
        let content = serde_json::to_string_pretty(run)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write JSON result: {}", path.display()))?;
        Ok(path)
    }

    pub fn save_csv(&self, run: &BenchmarkRun) -> anyhow::Result<PathBuf> {
        use std::io::Write;

        let path = self.output_dir.join(format!("{}.csv", run.run_id));
        utils::ensure_dir(&self.output_dir)
            .with_context(|| format!("Failed to create output directory: {}", self.output_dir.display()))?;

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
            let mut row: Vec<String> = vec![
                run.run_id.clone(),
                run.created_at.clone(),
                record.benchmark_id.clone(),
                record.benchmark_name.clone(),
                record.model.clone(),
                record.run_index.map(|i| i.to_string()).unwrap_or_default(),
                record.prompt_name.clone().unwrap_or_default(),
                record.error.clone().unwrap_or_default(),
                record.response_preview.clone().unwrap_or_default(),
            ];
            for key in &metric_keys {
                let value = record
                    .metrics
                    .get(key)
                    .map(|v| {
                        match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => v.to_string(),
                        }
                    })
                    .unwrap_or_default();
                row.push(value);
            }
            wtr.write_record(&row)?;
        }

        let csv_content = wtr.into_inner()?;
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;
        file.write_all(&csv_content)
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
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
        let run: BenchmarkRun =
            serde_json::from_str(&content).with_context(|| format!("Invalid JSON in {}", path.display()))?;
        Ok(run)
    }
}
