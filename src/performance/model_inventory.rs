use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::performance::config::PerformancePlan;
use crate::progress::{ProgressEventKind, ProgressPhase, ProgressSink, ProgressUpdate};
use crate::providers::{ProviderClient, ProviderKind};
use crate::utils::error_chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInventoryMeasurement {
    pub model: String,
    pub provider: ProviderKind,
    pub metadata_latency_ms: Option<f64>,
    pub metadata_payload_bytes: Option<u64>,
    pub cache_dir: Option<String>,
    pub cache_bytes: Option<u64>,
    pub notes: Vec<String>,
}

pub fn planned_inventory_steps(plan: &PerformancePlan, models_len: usize) -> u32 {
    let per_model = if plan.scan_model_cache { 2 } else { 1 };
    models_len as u32 * per_model
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
    models
        .iter()
        .enumerate()
        .map(|(index, model)| measure_one_model(client, model, plan, index, &mut progress))
        .collect()
}

fn measure_one_model(
    client: &ProviderClient,
    model: &str,
    plan: &PerformancePlan,
    model_index: usize,
    progress: &mut InventoryProgress<'_>,
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
    let cache_bytes = if plan.scan_model_cache {
        progress.start("Scanning local model cache", model, model_index);
        cache_dir
            .as_deref()
            .map(Path::new)
            .and_then(|path| match directory_size(path) {
                Ok(size) => Some(size),
                Err(error) => {
                    notes.push(format!("Cache scan failed: {error}"));
                    None
                }
            })
            .tap(|_| progress.finish("Scanned local model cache", model, model_index))
    } else {
        notes.push("Local cache scan disabled; pass --scan-model-cache to opt in.".to_string());
        None
    };

    ModelInventoryMeasurement {
        model: model.to_string(),
        provider: client.provider(),
        metadata_latency_ms,
        metadata_payload_bytes,
        cache_dir,
        cache_bytes,
        notes,
    }
}

fn directory_size(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Err(format!("{} does not exist", path.display()));
    }
    let mut total = 0u64;
    let mut stack = vec![PathBuf::from(path)];
    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(&current).map_err(|error| error.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
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

trait Tap: Sized {
    fn tap<F: FnOnce(&Self)>(self, f: F) -> Self {
        f(&self);
        self
    }
}

impl<T> Tap for T {}

#[cfg(test)]
mod tests {
    use super::planned_inventory_steps;
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
        )
        .unwrap();

        assert_eq!(planned_inventory_steps(&base, 1), 1);
        assert_eq!(planned_inventory_steps(&with_scan, 1), 2);
    }
}
