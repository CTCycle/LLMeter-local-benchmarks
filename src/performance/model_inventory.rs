use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::performance::config::PerformancePlan;
use crate::progress::{ProgressEventKind, ProgressPhase, ProgressSink, ProgressUpdate};
use crate::providers::{ProviderClient, ProviderKind};
use crate::utils::error_chain;

const MAX_CACHE_SCAN_ENTRIES: usize = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInventoryMeasurement {
    pub model: String,
    pub provider: ProviderKind,
    pub metadata_latency_ms: Option<f64>,
    pub metadata_payload_bytes: Option<u64>,
    pub cache_dir: Option<String>,
    pub cache_bytes: Option<u64>,
    pub cache_scope: Option<String>,
    pub notes: Vec<String>,
}

pub fn planned_inventory_steps(plan: &PerformancePlan, models_len: usize) -> u32 {
    models_len as u32 + u32::from(plan.scan_model_cache)
}

pub fn measure_model_inventory(
    client: &ProviderClient,
    models: &[String],
    plan: &PerformancePlan,
) -> Vec<ModelInventoryMeasurement> {
    let mut null_sink = crate::progress::NullProgressSink;
    measure_model_inventory_with_progress(
        client,
        models,
        plan,
        &mut null_sink,
        0,
        planned_inventory_steps(plan, models.len()),
    )
}

pub fn measure_model_inventory_with_progress(
    client: &ProviderClient,
    models: &[String],
    plan: &PerformancePlan,
    sink: &mut dyn ProgressSink,
    completed_units_before: u32,
    total_units: u32,
) -> Vec<ModelInventoryMeasurement> {
    let total_steps = planned_inventory_steps(plan, models.len());
    let mut progress = InventoryProgress::new(
        sink,
        completed_units_before,
        total_units,
        total_steps,
        models.len(),
    );
    let (cache_bytes, cache_error) = if plan.scan_model_cache {
        progress.start("Scanning provider model cache", "all models", 0);
        let result = plan
            .model_cache_dir
            .as_deref()
            .map(Path::new)
            .ok_or_else(|| "No model cache directory was configured".to_string())
            .and_then(directory_size);
        progress.finish("Scanned provider model cache", "all models", 0);
        match result {
            Ok(size) => (Some(size), None),
            Err(error) => (None, Some(error)),
        }
    } else {
        (None, None)
    };

    models
        .iter()
        .enumerate()
        .map(|(index, model)| {
            measure_one_model(
                client,
                model,
                plan,
                index,
                &mut progress,
                if index == 0 { cache_bytes } else { None },
                if index == 0 {
                    cache_error.as_deref()
                } else {
                    None
                },
            )
        })
        .collect()
}

fn measure_one_model(
    client: &ProviderClient,
    model: &str,
    plan: &PerformancePlan,
    model_index: usize,
    progress: &mut InventoryProgress<'_>,
    cache_bytes: Option<u64>,
    cache_error: Option<&str>,
) -> ModelInventoryMeasurement {
    let mut notes = Vec::new();
    let started = Instant::now();
    progress.start("Fetching model metadata", model, model_index);
    let (metadata_latency_ms, metadata_payload_bytes) = match client.show_model(model) {
        Ok(value) => (
            Some(started.elapsed().as_secs_f64() * 1000.0),
            serde_json::to_vec(&value)
                .ok()
                .map(|bytes| bytes.len() as u64),
        ),
        Err(error) => {
            notes.push(format!(
                "Model metadata probe failed: {}",
                error_chain(&*error)
            ));
            (None, None)
        }
    };
    progress.finish("Fetched model metadata", model, model_index);

    let cache_dir = plan.model_cache_dir.clone();
    if let Some(error) = cache_error {
        notes.push(format!("Provider cache scan failed: {error}"));
    } else if plan.scan_model_cache && cache_bytes.is_some() && model_index == 0 {
        notes.push(
            "Cache size is the provider-wide directory total, not model-attributed storage."
                .to_string(),
        );
    } else if !plan.scan_model_cache {
        notes
            .push("Skipped — enter a model cache directory path to measure disk usage".to_string());
    }

    ModelInventoryMeasurement {
        model: model.to_string(),
        provider: client.provider(),
        metadata_latency_ms,
        metadata_payload_bytes,
        cache_dir,
        cache_bytes,
        cache_scope: cache_bytes.map(|_| "provider-cache-directory".to_string()),
        notes,
    }
}

fn directory_size(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Err(format!("{} does not exist", path.display()));
    }
    let mut total = 0u64;
    let mut stack = vec![PathBuf::from(path)];
    let mut visited = std::collections::HashSet::new();
    let mut scanned_entries = 0usize;
    while let Some(current) = stack.pop() {
        let current = std::fs::canonicalize(&current).map_err(|error| error.to_string())?;
        if !visited.insert(current.clone()) {
            continue;
        }
        let entries = std::fs::read_dir(&current).map_err(|error| error.to_string())?;
        for entry in entries {
            scanned_entries = scanned_entries.saturating_add(1);
            if scanned_entries > MAX_CACHE_SCAN_ENTRIES {
                return Err(format!(
                    "cache scan exceeded the {} entry limit",
                    MAX_CACHE_SCAN_ENTRIES
                ));
            }
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() {
                continue;
            }
            let metadata = entry.metadata().map_err(|error| error.to_string())?;
            if metadata.is_dir() {
                stack.push(entry.path());
            } else {
                total = total.saturating_add(metadata.len());
            }
        }
    }
    Ok(total)
}

struct InventoryProgress<'a> {
    sink: &'a mut dyn ProgressSink,
    completed_units_before: u32,
    total_units: u32,
    completed_steps: u32,
    total_steps: u32,
    total_models: usize,
}

impl<'a> InventoryProgress<'a> {
    fn new(
        sink: &'a mut dyn ProgressSink,
        completed_units_before: u32,
        total_units: u32,
        total_steps: u32,
        total_models: usize,
    ) -> Self {
        Self {
            sink,
            completed_units_before,
            total_units,
            completed_steps: 0,
            total_steps,
            total_models,
        }
    }

    fn start(&mut self, message: &str, model: &str, model_index: usize) {
        self.sink.on_update(ProgressUpdate {
            kind: ProgressEventKind::StepStarted,
            phase: ProgressPhase::Running,
            message: message.to_string(),
            completed_units: self.completed_units_before + self.completed_steps,
            total_units: self.total_units,
            model_name: Some(model.to_string()),
            model_index: Some(model_index + 1),
            total_models: Some(self.total_models.max(1)),
            benchmark_id: Some("model-inventory".to_string()),
            benchmark_name: Some("Model inventory".to_string()),
            benchmark_index: Some((self.completed_steps + 1) as usize),
            total_benchmarks: Some(self.total_steps as usize),
            step_index: Some(self.completed_units_before + self.completed_steps + 1),
            total_steps: Some(self.total_units),
            run_index: None,
            prompt_name: None,
        });
    }

    fn finish(&mut self, message: &str, model: &str, model_index: usize) {
        self.completed_steps += 1;
        self.sink.on_update(ProgressUpdate {
            kind: ProgressEventKind::StepCompleted,
            phase: ProgressPhase::Running,
            message: message.to_string(),
            completed_units: self.completed_units_before + self.completed_steps,
            total_units: self.total_units,
            model_name: Some(model.to_string()),
            model_index: Some(model_index + 1),
            total_models: Some(self.total_models.max(1)),
            benchmark_id: Some("model-inventory".to_string()),
            benchmark_name: Some("Model inventory".to_string()),
            benchmark_index: Some(self.completed_steps as usize),
            total_benchmarks: Some(self.total_steps as usize),
            step_index: Some(self.completed_units_before + self.completed_steps),
            total_steps: Some(self.total_units),
            run_index: None,
            prompt_name: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{directory_size, planned_inventory_steps};
    use crate::performance::config::{
        LoadMeasurementMode, PerformancePlan, PerformanceProfile, ReportDetailLevel, TelemetryLevel,
    };
    use crate::providers::ProviderKind;
    use std::collections::HashMap;

    #[test]
    fn planned_inventory_steps_track_cache_scan_toggle() {
        let base = PerformancePlan::from_cli(
            ProviderKind::Ollama,
            vec!["model-a".to_string()],
            PerformanceProfile::Smoke,
            None,
            None,
            None,
            Some(0),
            Some(1),
            true,
            None,
            HashMap::new(),
            LoadMeasurementMode::Off,
            1,
            TelemetryLevel::Standard,
            1000,
            None,
            false,
            false,
            None,
            false,
            ReportDetailLevel::Summary,
            None,
        )
        .unwrap();
        let with_scan = PerformancePlan::from_cli(
            ProviderKind::Ollama,
            vec!["model-a".to_string()],
            PerformanceProfile::Smoke,
            None,
            None,
            None,
            Some(0),
            Some(1),
            true,
            None,
            HashMap::new(),
            LoadMeasurementMode::Off,
            1,
            TelemetryLevel::Standard,
            1000,
            None,
            false,
            false,
            Some("C:\\cache".to_string()),
            true,
            ReportDetailLevel::Summary,
            None,
        )
        .unwrap();

        assert_eq!(planned_inventory_steps(&base, 1), 1);
        assert_eq!(planned_inventory_steps(&with_scan, 1), 2);
    }

    #[test]
    fn directory_size_scans_nested_files_once() {
        let root = tempfile::tempdir().expect("temporary cache directory");
        std::fs::create_dir(root.path().join("nested")).expect("nested directory");
        std::fs::write(root.path().join("one.bin"), [0u8; 3]).expect("first cache file");
        std::fs::write(root.path().join("nested").join("two.bin"), [0u8; 5])
            .expect("second cache file");

        assert_eq!(directory_size(root.path()), Ok(8));
    }

    #[cfg(unix)]
    #[test]
    fn directory_size_skips_symlinked_directories() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().expect("temporary cache directory");
        let linked_target = tempfile::tempdir().expect("symlink target directory");
        std::fs::write(linked_target.path().join("linked.bin"), [0u8; 13])
            .expect("linked cache file");
        std::fs::write(root.path().join("direct.bin"), [0u8; 2]).expect("direct cache file");
        symlink(linked_target.path(), root.path().join("linked-directory"))
            .expect("create directory symlink");

        assert_eq!(directory_size(root.path()), Ok(2));
    }
}
