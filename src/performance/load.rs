use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::performance::config::{LoadMeasurementMode, PerformancePlan};
use crate::progress::{ProgressEventKind, ProgressPhase, ProgressSink, ProgressUpdate};
use crate::providers::{ProviderClient, ProviderKind};
use crate::utils::{error_chain, ns_to_ms};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoadOverheadConfidence {
    Unavailable,
    Estimated,
    ProviderNative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLoadMeasurement {
    pub model: String,
    pub mode: LoadMeasurementMode,
    pub provider: ProviderKind,
    pub provider_status_latency_ms: Option<f64>,
    pub first_probe_wall_ms: Option<f64>,
    pub first_probe_ttft_ms: Option<f64>,
    pub warm_probe_wall_ms_mean: Option<f64>,
    pub warm_probe_ttft_ms_mean: Option<f64>,
    pub estimated_load_overhead_ms: Option<f64>,
    pub confidence: LoadOverheadConfidence,
    pub notes: Vec<String>,
}

pub fn planned_load_steps(plan: &PerformancePlan, models_len: usize) -> u32 {
    if plan.load_measurement == LoadMeasurementMode::Off {
        0
    } else {
        models_len as u32 * (plan.load_probe_runs + 2)
    }
}

pub fn measure_model_load(
    client: &ProviderClient,
    models: &[String],
    plan: &PerformancePlan,
) -> Vec<ModelLoadMeasurement> {
    let mut null_sink = crate::progress::NullProgressSink;
    measure_model_load_with_progress(
        client,
        models,
        plan,
        &mut null_sink,
        0,
        planned_load_steps(plan, models.len()),
    )
}

pub fn measure_model_load_with_progress(
    client: &ProviderClient,
    models: &[String],
    plan: &PerformancePlan,
    sink: &mut dyn ProgressSink,
    completed_units_before: u32,
    total_units: u32,
) -> Vec<ModelLoadMeasurement> {
    if plan.load_measurement == LoadMeasurementMode::Off {
        return Vec::new();
    }

    let total_steps = planned_load_steps(plan, models.len());
    let mut progress = LoadProgress::new(
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
    progress: &mut LoadProgress<'_>,
) -> ModelLoadMeasurement {
    let mut notes = vec![
        "Load overhead is estimated from client-side probe timing, not true model-load telemetry."
            .to_string(),
    ];

    let status_started = Instant::now();
    progress.start(
        "Checking provider status for load estimate",
        model,
        model_index,
    );
    let status_latency = match client.list_models() {
        Ok(_) => Some(status_started.elapsed().as_secs_f64() * 1000.0),
        Err(error) => {
            notes.push(format!(
                "Provider status probe failed: {}",
                error_chain(&*error)
            ));
            None
        }
    };
    progress.finish(
        "Checked provider status for load estimate",
        model,
        model_index,
    );

    progress.start("Running first load probe", model, model_index);
    let first = tiny_probe(client, model, plan.stream);
    progress.finish("Completed first load probe", model, model_index);
    let mut warm_wall = Vec::new();
    let mut warm_ttft = Vec::new();
    for probe_index in 0..plan.load_probe_runs {
        progress.start(
            &format!(
                "Running warm load probe {}/{}",
                probe_index + 1,
                plan.load_probe_runs
            ),
            model,
            model_index,
        );
        match tiny_probe(client, model, plan.stream) {
            Ok(sample) => {
                warm_wall.push(sample.wall_ms);
                if let Some(ttft) = sample.ttft_ms {
                    warm_ttft.push(ttft);
                }
            }
            Err(error) => notes.push(format!("Warm load probe failed: {error}")),
        }
        progress.finish(
            &format!(
                "Completed warm load probe {}/{}",
                probe_index + 1,
                plan.load_probe_runs
            ),
            model,
            model_index,
        );
    }

    let (first_wall, first_ttft) = match first {
        Ok(sample) => (Some(sample.wall_ms), sample.ttft_ms),
        Err(error) => {
            notes.push(format!("First load probe failed: {error}"));
            (None, None)
        }
    };
    let warm_wall_mean = mean(&warm_wall);
    let warm_ttft_mean = mean(&warm_ttft);
    let estimated = first_wall
        .zip(warm_wall_mean)
        .map(|(first, warm)| (first - warm).max(0.0));

    ModelLoadMeasurement {
        model: model.to_string(),
        mode: plan.load_measurement,
        provider: client.provider(),
        provider_status_latency_ms: status_latency,
        first_probe_wall_ms: first_wall,
        first_probe_ttft_ms: first_ttft,
        warm_probe_wall_ms_mean: warm_wall_mean,
        warm_probe_ttft_ms_mean: warm_ttft_mean,
        estimated_load_overhead_ms: estimated,
        confidence: if estimated.is_some() {
            LoadOverheadConfidence::Estimated
        } else {
            LoadOverheadConfidence::Unavailable
        },
        notes,
    }
}

struct LoadProgress<'a> {
    sink: &'a mut dyn ProgressSink,
    completed_units_before: u32,
    total_units: u32,
    completed_steps: u32,
    total_steps: u32,
    total_models: usize,
}

impl<'a> LoadProgress<'a> {
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
            benchmark_id: Some("performance-load".to_string()),
            benchmark_name: Some("Load estimate".to_string()),
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
            benchmark_id: Some("performance-load".to_string()),
            benchmark_name: Some("Load estimate".to_string()),
            benchmark_index: Some(self.completed_steps as usize),
            total_benchmarks: Some(self.total_steps as usize),
            step_index: Some(self.completed_units_before + self.completed_steps),
            total_steps: Some(self.total_units),
            run_index: None,
            prompt_name: None,
        });
    }
}

struct ProbeSample {
    wall_ms: f64,
    ttft_ms: Option<f64>,
}

fn tiny_probe(client: &ProviderClient, model: &str, stream: bool) -> Result<ProbeSample, String> {
    client
        .chat_completion(
            model,
            serde_json::json!([{ "role": "user", "content": "Reply with the word ok." }]),
            2,
            0.0,
            stream,
            None,
        )
        .map(|result| ProbeSample {
            wall_ms: ns_to_ms(Some(result.wall_time_ns)).unwrap_or_default(),
            ttft_ms: ns_to_ms(result.time_to_first_token_ns),
        })
        .map_err(|error| error_chain(&*error))
}

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::planned_load_steps;
    use crate::performance::config::{
        LoadMeasurementMode, PerformancePlan, PerformanceProfile, ReportDetailLevel, TelemetryLevel,
    };
    use crate::providers::ProviderKind;
    use std::collections::HashMap;

    #[test]
    fn planned_load_steps_accounts_for_probe_count() {
        let plan = PerformancePlan::from_cli(
            ProviderKind::Ollama,
            vec!["model-a".to_string(), "model-b".to_string()],
            PerformanceProfile::Smoke,
            None,
            None,
            None,
            Some(0),
            Some(1),
            true,
            None,
            HashMap::new(),
            LoadMeasurementMode::WarmBaseline,
            3,
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

        assert_eq!(planned_load_steps(&plan, 2), 10);
    }
}
