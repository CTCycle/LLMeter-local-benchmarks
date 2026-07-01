use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::performance::config::PerformancePlan;
use crate::progress::{ProgressEventKind, ProgressPhase, ProgressSink, ProgressUpdate};
use crate::providers::{ProviderClient, ProviderCompatibilityTier, ProviderKind};
use crate::utils::{error_chain, utc_now_iso};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilityReport {
    pub provider: ProviderKind,
    pub base_url: String,
    pub compatibility_tier: ProviderCompatibilityTier,
    pub checked_at: String,
    pub endpoints: Vec<EndpointProbe>,
    pub models: Vec<ModelCapability>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointProbe {
    pub name: String,
    pub method: String,
    pub path: String,
    pub supported: bool,
    pub http_status: Option<u16>,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapability {
    pub id: String,
    pub owned_by: Option<String>,
    pub context_length: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub architecture: Option<String>,
    pub quantization: Option<String>,
    pub raw: Value,
}

pub fn planned_probe_steps(plan: &PerformancePlan) -> u32 {
    let mut steps = 1;
    let has_probe_model = !plan.models.is_empty();
    if has_probe_model {
        steps += 1;
        if plan.stream || plan.probe_all_endpoints {
            steps += 1;
        }
        if plan.probe_all_endpoints {
            steps += 2;
        }
    }
    steps
}

pub fn probe_provider_capabilities(
    client: &ProviderClient,
    plan: &PerformancePlan,
) -> ProviderCapabilityReport {
    let mut null_sink = crate::progress::NullProgressSink;
    probe_provider_capabilities_with_progress(
        client,
        plan,
        &mut null_sink,
        0,
        planned_probe_steps(plan),
    )
}

pub fn probe_provider_capabilities_with_progress(
    client: &ProviderClient,
    plan: &PerformancePlan,
    sink: &mut dyn ProgressSink,
    completed_units_before: u32,
    total_units: u32,
) -> ProviderCapabilityReport {
    let mut endpoints = Vec::new();
    let mut warnings = Vec::new();
    let probe_steps = planned_probe_steps(plan);
    let mut probe_progress = ProbeProgress::new(
        sink,
        completed_units_before,
        total_units,
        probe_steps,
        plan.models.len(),
    );
    let started = Instant::now();
    probe_progress.start("Checking model catalog", None, "Models");
    let models_result = client.list_models();
    let model_error = models_result
        .as_ref()
        .err()
        .map(|error| error_chain(&**error));
    let model_values = models_result.unwrap_or_default();
    endpoints.push(endpoint_probe_from_result(
        "Models",
        "GET",
        "/v1/models",
        started.elapsed().as_secs_f64() * 1000.0,
        if model_error.is_none() {
            Some(200)
        } else {
            None
        },
        model_error,
    ));
    probe_progress.finish("Checked model catalog", None, "Models");

    let probe_model = plan
        .models
        .first()
        .cloned()
        .or_else(|| first_model_id(&model_values));

    if let Some(model) = probe_model {
        probe_progress.start(
            "Checking chat completions",
            Some(&model),
            "Chat completions",
        );
        endpoints.push(probe_chat(client, &model, false));
        probe_progress.finish("Checked chat completions", Some(&model), "Chat completions");
        if plan.stream || plan.probe_all_endpoints {
            probe_progress.start(
                "Checking streaming chat completions",
                Some(&model),
                "Chat completions streaming",
            );
            endpoints.push(probe_chat(client, &model, true));
            probe_progress.finish(
                "Checked streaming chat completions",
                Some(&model),
                "Chat completions streaming",
            );
        }
        if plan.probe_all_endpoints {
            probe_progress.start("Checking embeddings", Some(&model), "Embeddings");
            endpoints.push(probe_embeddings(client, &model));
            probe_progress.finish("Checked embeddings", Some(&model), "Embeddings");
            probe_progress.start("Checking responses", Some(&model), "Responses");
            endpoints.push(probe_responses(client, &model));
            probe_progress.finish("Checked responses", Some(&model), "Responses");
        }
    } else {
        warnings
            .push("No model was available for chat, embeddings, or responses probes.".to_string());
    }

    if !plan.stream {
        warnings.push("Streaming is disabled; TTFT will be unavailable unless a provider reports native timing.".to_string());
    }
    if client.provider().compatibility_tier() != ProviderCompatibilityTier::FirstClass {
        warnings.push(
            "Provider support is OpenAI-compatible best effort unless endpoint probes pass."
                .to_string(),
        );
    }

    ProviderCapabilityReport {
        provider: client.provider(),
        base_url: client.base_url().to_string(),
        compatibility_tier: client.provider().compatibility_tier(),
        checked_at: utc_now_iso(),
        endpoints,
        models: parse_model_capabilities(&model_values),
        warnings,
    }
}

struct ProbeProgress<'a> {
    sink: &'a mut dyn ProgressSink,
    completed_units_before: u32,
    total_units: u32,
    completed_probe_steps: u32,
    total_probe_steps: u32,
    total_models: usize,
}

impl<'a> ProbeProgress<'a> {
    fn new(
        sink: &'a mut dyn ProgressSink,
        completed_units_before: u32,
        total_units: u32,
        total_probe_steps: u32,
        total_models: usize,
    ) -> Self {
        Self {
            sink,
            completed_units_before,
            total_units,
            completed_probe_steps: 0,
            total_probe_steps,
            total_models,
        }
    }

    fn start(&mut self, message: &str, model_name: Option<&str>, benchmark_name: &str) {
        self.sink.on_update(ProgressUpdate {
            kind: ProgressEventKind::StepStarted,
            phase: ProgressPhase::Validating,
            message: message.to_string(),
            completed_units: self.completed_units_before + self.completed_probe_steps,
            total_units: self.total_units,
            model_name: model_name.map(str::to_string),
            model_index: model_name.map(|_| 1),
            total_models: model_name.map(|_| self.total_models.max(1)),
            benchmark_id: Some("provider-probe".to_string()),
            benchmark_name: Some(benchmark_name.to_string()),
            benchmark_index: Some((self.completed_probe_steps + 1) as usize),
            total_benchmarks: Some(self.total_probe_steps as usize),
            step_index: Some(self.completed_units_before + self.completed_probe_steps + 1),
            total_steps: Some(self.total_units),
            run_index: None,
            prompt_name: None,
        });
    }

    fn finish(&mut self, message: &str, model_name: Option<&str>, benchmark_name: &str) {
        self.completed_probe_steps += 1;
        self.sink.on_update(ProgressUpdate {
            kind: ProgressEventKind::StepCompleted,
            phase: ProgressPhase::Validating,
            message: message.to_string(),
            completed_units: self.completed_units_before + self.completed_probe_steps,
            total_units: self.total_units,
            model_name: model_name.map(str::to_string),
            model_index: model_name.map(|_| 1),
            total_models: model_name.map(|_| self.total_models.max(1)),
            benchmark_id: Some("provider-probe".to_string()),
            benchmark_name: Some(benchmark_name.to_string()),
            benchmark_index: Some(self.completed_probe_steps as usize),
            total_benchmarks: Some(self.total_probe_steps as usize),
            step_index: Some(self.completed_units_before + self.completed_probe_steps),
            total_steps: Some(self.total_units),
            run_index: None,
            prompt_name: None,
        });
    }
}

pub fn parse_model_capabilities(models: &[Value]) -> Vec<ModelCapability> {
    models
        .iter()
        .filter_map(|model| {
            let id = model
                .get("id")
                .or_else(|| model.get("name"))
                .or_else(|| model.get("model"))
                .and_then(|value| value.as_str())?
                .to_string();
            Some(ModelCapability {
                id,
                owned_by: model
                    .get("owned_by")
                    .and_then(|value| value.as_str())
                    .map(str::to_string),
                context_length: first_u64(
                    model,
                    &["context_length", "max_context_length", "n_ctx"],
                ),
                max_output_tokens: first_u64(model, &["max_output_tokens", "max_tokens"]),
                architecture: first_string(model, &["architecture", "family"]),
                quantization: first_string(model, &["quantization", "quantization_level"]),
                raw: model.clone(),
            })
        })
        .collect()
}

pub fn endpoint_probe_from_result(
    name: &str,
    method: &str,
    path: &str,
    latency_ms: f64,
    http_status: Option<u16>,
    error: Option<String>,
) -> EndpointProbe {
    EndpointProbe {
        name: name.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        supported: error.is_none(),
        http_status,
        latency_ms: Some(latency_ms),
        error,
    }
}

fn probe_chat(client: &ProviderClient, model: &str, stream: bool) -> EndpointProbe {
    let started = Instant::now();
    let result = client.chat_completion(
        model,
        serde_json::json!([{ "role": "user", "content": "Reply with the word ok." }]),
        2,
        0.0,
        stream,
        None,
    );
    endpoint_probe_from_result(
        if stream {
            "Chat completions streaming"
        } else {
            "Chat completions"
        },
        "POST",
        "/v1/chat/completions",
        started.elapsed().as_secs_f64() * 1000.0,
        result
            .as_ref()
            .ok()
            .and_then(|response| response.http_status),
        result.err().map(|error| error_chain(&*error)),
    )
}

fn probe_embeddings(client: &ProviderClient, model: &str) -> EndpointProbe {
    let started = Instant::now();
    let result = client.embeddings(model, serde_json::json!("ok"));
    endpoint_probe_from_result(
        "Embeddings",
        "POST",
        "/v1/embeddings",
        started.elapsed().as_secs_f64() * 1000.0,
        result
            .as_ref()
            .ok()
            .and_then(|response| response.http_status),
        result.err().map(|error| error_chain(&*error)),
    )
}

fn probe_responses(client: &ProviderClient, model: &str) -> EndpointProbe {
    let started = Instant::now();
    let result = client.responses(model, serde_json::json!("ok"), 2, 0.0, None);
    endpoint_probe_from_result(
        "Responses",
        "POST",
        "/v1/responses",
        started.elapsed().as_secs_f64() * 1000.0,
        result
            .as_ref()
            .ok()
            .and_then(|response| response.http_status),
        result.err().map(|error| error_chain(&*error)),
    )
}

fn first_model_id(models: &[Value]) -> Option<String> {
    parse_model_capabilities(models)
        .into_iter()
        .next()
        .map(|model| model.id)
}

fn first_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|item| item.as_u64()))
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(|item| item.as_str())
            .map(str::to_string)
    })
}

#[cfg(test)]
mod tests {
    use super::{
        endpoint_probe_from_result, parse_model_capabilities, planned_probe_steps,
        probe_provider_capabilities_with_progress,
    };
    use crate::performance::config::{
        LoadMeasurementMode, PerformancePlan, PerformanceProfile, ReportDetailLevel, TelemetryLevel,
    };
    use crate::progress::{ProgressEventKind, ProgressSink, ProgressUpdate};
    use crate::providers::{ProviderClient, ProviderKind};
    use serde_json::json;
    use std::collections::HashMap;

    #[derive(Default)]
    struct RecordingProgressSink {
        updates: Vec<ProgressUpdate>,
    }

    impl ProgressSink for RecordingProgressSink {
        fn on_update(&mut self, update: ProgressUpdate) {
            self.updates.push(update);
        }
    }

    fn test_plan(stream: bool, probe_all_endpoints: bool) -> PerformancePlan {
        PerformancePlan::from_cli(
            ProviderKind::Ollama,
            vec!["qwen3.5:2b".to_string()],
            PerformanceProfile::Smoke,
            None,
            None,
            None,
            Some(0),
            Some(1),
            stream,
            None,
            HashMap::new(),
            LoadMeasurementMode::Off,
            1,
            TelemetryLevel::Standard,
            1000,
            None,
            true,
            probe_all_endpoints,
            None,
            false,
            ReportDetailLevel::Summary,
            None,
        )
        .unwrap()
    }

    #[test]
    fn planned_probe_steps_match_basic_and_full_modes() {
        assert_eq!(planned_probe_steps(&test_plan(false, false)), 2);
        assert_eq!(planned_probe_steps(&test_plan(true, false)), 3);
        assert_eq!(planned_probe_steps(&test_plan(false, true)), 5);
    }

    #[test]
    fn parse_model_capabilities_extracts_common_metadata() {
        let models = vec![json!({
            "id": "llama3.1",
            "owned_by": "local",
            "context_length": 8192,
            "max_output_tokens": 2048,
            "architecture": "llama",
            "quantization": "q4"
        })];

        let parsed = parse_model_capabilities(&models);

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "llama3.1");
        assert_eq!(parsed[0].owned_by.as_deref(), Some("local"));
        assert_eq!(parsed[0].context_length, Some(8192));
        assert_eq!(parsed[0].max_output_tokens, Some(2048));
        assert_eq!(parsed[0].architecture.as_deref(), Some("llama"));
        assert_eq!(parsed[0].quantization.as_deref(), Some("q4"));
    }

    #[test]
    fn endpoint_probe_marks_errors_as_unsupported() {
        let probe = endpoint_probe_from_result(
            "Models",
            "GET",
            "/v1/models",
            12.5,
            None,
            Some("connection refused".to_string()),
        );

        assert!(!probe.supported);
        assert_eq!(probe.latency_ms, Some(12.5));
        assert_eq!(probe.error.as_deref(), Some("connection refused"));
    }

    #[test]
    fn probe_reports_progress_for_each_step() {
        let client =
            ProviderClient::new(ProviderKind::Ollama, "http://127.0.0.1:1/v1", 0.1).unwrap();
        let plan = test_plan(false, false);
        let mut sink = RecordingProgressSink::default();

        let report = probe_provider_capabilities_with_progress(
            &client,
            &plan,
            &mut sink,
            0,
            planned_probe_steps(&plan),
        );

        assert_eq!(report.endpoints.len(), 2);
        let started = sink
            .updates
            .iter()
            .filter(|update| matches!(update.kind, ProgressEventKind::StepStarted))
            .count();
        let completed = sink
            .updates
            .iter()
            .filter(|update| matches!(update.kind, ProgressEventKind::StepCompleted))
            .count();
        assert_eq!(started, 2);
        assert_eq!(completed, 2);
        assert_eq!(
            sink.updates.last().map(|update| update.completed_units),
            Some(2)
        );
        assert_eq!(
            sink.updates.last().map(|update| update.total_units),
            Some(2)
        );
    }
}
