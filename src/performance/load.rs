use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::performance::config::{LoadMeasurementMode, PerformancePlan};
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

pub fn measure_model_load(
    client: &ProviderClient,
    models: &[String],
    plan: &PerformancePlan,
) -> Vec<ModelLoadMeasurement> {
    if plan.load_measurement == LoadMeasurementMode::Off {
        return Vec::new();
    }

    models
        .iter()
        .map(|model| measure_one_model(client, model, plan))
        .collect()
}

fn measure_one_model(
    client: &ProviderClient,
    model: &str,
    plan: &PerformancePlan,
) -> ModelLoadMeasurement {
    let mut notes = vec![
        "Load overhead is estimated from client-side probe timing, not true model-load telemetry."
            .to_string(),
    ];

    let status_started = Instant::now();
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

    let first = tiny_probe(client, model, plan.stream);
    let mut warm_wall = Vec::new();
    let mut warm_ttft = Vec::new();
    for _ in 0..plan.load_probe_runs {
        match tiny_probe(client, model, plan.stream) {
            Ok(sample) => {
                warm_wall.push(sample.wall_ms);
                if let Some(ttft) = sample.ttft_ms {
                    warm_ttft.push(ttft);
                }
            }
            Err(error) => notes.push(format!("Warm load probe failed: {error}")),
        }
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
