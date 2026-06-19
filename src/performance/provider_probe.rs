use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::performance::config::PerformancePlan;
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

pub fn probe_provider_capabilities(
    client: &ProviderClient,
    plan: &PerformancePlan,
) -> ProviderCapabilityReport {
    let mut endpoints = Vec::new();
    let mut warnings = Vec::new();
    let started = Instant::now();
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

    let probe_model = plan
        .models
        .first()
        .cloned()
        .or_else(|| first_model_id(&model_values));

    if let Some(model) = probe_model {
        endpoints.push(probe_chat(client, &model, false));
        if plan.stream || plan.probe_all_endpoints {
            endpoints.push(probe_chat(client, &model, true));
        }
        if plan.probe_all_endpoints {
            endpoints.push(probe_embeddings(client, &model));
            endpoints.push(probe_responses(client, &model));
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
