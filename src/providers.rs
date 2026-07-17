use std::fmt;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use anyhow::Context;
use clap::ValueEnum;
use reqwest::blocking::{Client as HttpClient, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::LLMeterError;
use crate::utils::error_chain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    Ollama,
    Lmstudio,
    LlamaCpp,
    OpenaiCompatible,
    Vllm,
    Sglang,
    Localai,
    Litellm,
    Tgi,
    TextGenerationWebui,
    Jan,
    MlxLm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderCompatibilityTier {
    FirstClass,
    OpenAiCompatibleKnown,
    OpenAiCompatibleBestEffort,
    Custom,
}

impl ProviderCompatibilityTier {
    pub fn label(self) -> &'static str {
        match self {
            Self::FirstClass => "first-class",
            Self::OpenAiCompatibleKnown => "known OpenAI-compatible",
            Self::OpenAiCompatibleBestEffort => "best effort",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderCatalogEntry {
    pub provider: ProviderKind,
    pub tier: ProviderCompatibilityTier,
    pub default_base_url: &'static str,
    pub notes: &'static str,
}

impl ProviderKind {
    pub fn default_base_url(self) -> &'static str {
        match self {
            ProviderKind::Ollama => "http://localhost:11434/v1",
            ProviderKind::Lmstudio => "http://localhost:1234/v1",
            ProviderKind::LlamaCpp => "http://localhost:8080/v1",
            ProviderKind::OpenaiCompatible => "http://localhost:8000/v1",
            ProviderKind::Vllm => "http://localhost:8000/v1",
            ProviderKind::Sglang => "http://localhost:30000/v1",
            ProviderKind::Localai => "http://localhost:8080/v1",
            ProviderKind::Litellm => "http://localhost:4000/v1",
            ProviderKind::Tgi => "http://localhost:8080/v1",
            ProviderKind::TextGenerationWebui => "http://localhost:5000/v1",
            ProviderKind::Jan => "http://localhost:1337/v1",
            ProviderKind::MlxLm => "http://localhost:8080/v1",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProviderKind::Ollama => "ollama",
            ProviderKind::Lmstudio => "lmstudio",
            ProviderKind::LlamaCpp => "llama-cpp",
            ProviderKind::OpenaiCompatible => "openai-compatible",
            ProviderKind::Vllm => "vllm",
            ProviderKind::Sglang => "sglang",
            ProviderKind::Localai => "localai",
            ProviderKind::Litellm => "litellm",
            ProviderKind::Tgi => "tgi",
            ProviderKind::TextGenerationWebui => "text-generation-webui",
            ProviderKind::Jan => "jan",
            ProviderKind::MlxLm => "mlx-lm",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ProviderKind::Ollama => "Ollama",
            ProviderKind::Lmstudio => "LM Studio",
            ProviderKind::LlamaCpp => "llama.cpp",
            ProviderKind::OpenaiCompatible => "OpenAI-compatible",
            ProviderKind::Vllm => "vLLM",
            ProviderKind::Sglang => "SGLang",
            ProviderKind::Localai => "LocalAI",
            ProviderKind::Litellm => "LiteLLM",
            ProviderKind::Tgi => "TGI",
            ProviderKind::TextGenerationWebui => "text-generation-webui",
            ProviderKind::Jan => "Jan",
            ProviderKind::MlxLm => "MLX-LM",
        }
    }

    pub fn compatibility_tier(self) -> ProviderCompatibilityTier {
        match self {
            Self::Ollama | Self::Lmstudio | Self::LlamaCpp => ProviderCompatibilityTier::FirstClass,
            Self::Vllm | Self::Sglang | Self::Localai | Self::Litellm => {
                ProviderCompatibilityTier::OpenAiCompatibleKnown
            }
            Self::Tgi | Self::TextGenerationWebui | Self::Jan | Self::MlxLm => {
                ProviderCompatibilityTier::OpenAiCompatibleBestEffort
            }
            Self::OpenaiCompatible => ProviderCompatibilityTier::Custom,
        }
    }

    pub fn catalog() -> Vec<ProviderCatalogEntry> {
        [
            (Self::Ollama, "Existing first-class target"),
            (Self::Lmstudio, "Existing first-class target"),
            (Self::LlamaCpp, "Existing first-class target"),
            (Self::OpenaiCompatible, "User-supplied local /v1 server"),
            (Self::Vllm, "OpenAI-compatible server preset"),
            (Self::Sglang, "OpenAI-compatible server preset"),
            (Self::Localai, "OpenAI-compatible server preset"),
            (Self::Litellm, "OpenAI-compatible proxy preset"),
            (Self::Tgi, "Requires OpenAI-compatible router mode"),
            (Self::TextGenerationWebui, "API shape can vary by extension"),
            (Self::Jan, "Local API behavior can vary by version"),
            (Self::MlxLm, "OpenAI-compatible server shape can vary"),
        ]
        .into_iter()
        .map(|(provider, notes)| ProviderCatalogEntry {
            provider,
            tier: provider.compatibility_tier(),
            default_base_url: provider.default_base_url(),
            notes,
        })
        .collect()
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl ProviderKind {
    pub fn default_cache_dir(self) -> Option<std::path::PathBuf> {
        match self {
            ProviderKind::Ollama => dirs::home_dir().map(|h| h.join(".ollama").join("models")),
            ProviderKind::Lmstudio => {
                #[cfg(target_os = "windows")]
                {
                    dirs::data_dir().map(|h| h.join("lm-studio").join("models"))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    dirs::home_dir().map(|h| h.join(".cache").join("lm-studio").join("models"))
                }
            }
            _ => None,
        }
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = LLMeterError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ollama" => Ok(ProviderKind::Ollama),
            "lmstudio" | "lm-studio" | "lm_studio" => Ok(ProviderKind::Lmstudio),
            "llama-cpp" | "llama.cpp" | "llamacpp" | "llama_cpp" => Ok(ProviderKind::LlamaCpp),
            "openai-compatible" | "openai" | "custom" => Ok(ProviderKind::OpenaiCompatible),
            "vllm" | "vllm-openai" => Ok(ProviderKind::Vllm),
            "sglang" | "sgl" => Ok(ProviderKind::Sglang),
            "localai" | "local-ai" => Ok(ProviderKind::Localai),
            "litellm" | "lite-llm" => Ok(ProviderKind::Litellm),
            "tgi" | "text-generation-inference" => Ok(ProviderKind::Tgi),
            "text-generation-webui" | "oobabooga" => Ok(ProviderKind::TextGenerationWebui),
            "jan" => Ok(ProviderKind::Jan),
            "mlx-lm" | "mlxlm" | "mlx_lm" => Ok(ProviderKind::MlxLm),
            other => Err(LLMeterError::InvalidOption(format!(
                "Unknown provider '{other}'. Use `llmeter providers list` for supported presets."
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub provider: ProviderKind,
    pub base_url: String,
    pub running: bool,
    pub models: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiResult {
    pub endpoint: String,
    pub response_text: String,
    pub raw: Value,
    pub wall_time_ns: u128,
    pub time_to_first_token_ns: Option<u128>,
    pub token_timings_ns: Vec<u128>,
    pub http_status: Option<u16>,
}

impl ApiResult {
    pub fn output_tokens(&self) -> Option<u64> {
        self.raw
            .pointer("/usage/completion_tokens")
            .or_else(|| self.raw.pointer("/usage/output_tokens"))
            .and_then(|v| v.as_u64())
    }

    pub fn tokens_per_second(&self) -> Option<f64> {
        let tokens = self.output_tokens()?;
        if self.wall_time_ns == 0 {
            return None;
        }
        Some(tokens as f64 / (self.wall_time_ns as f64 / 1_000_000_000.0))
    }
}

#[derive(Debug, Clone)]
pub struct ProviderClient {
    provider: ProviderKind,
    base_url: String,
    client: HttpClient,
}

impl ProviderClient {
    pub fn new(provider: ProviderKind, base_url: &str, timeout: f64) -> anyhow::Result<Self> {
        if !timeout.is_finite() || timeout <= 0.0 || timeout > 24.0 * 60.0 * 60.0 {
            return Err(LLMeterError::InvalidOption(format!(
                "Invalid request timeout '{timeout}'. Expected a finite value greater than 0 and at most 86400 seconds."
            ))
            .into());
        }
        let client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs_f64(timeout))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(ProviderClient {
            provider,
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        })
    }

    pub fn provider(&self) -> ProviderKind {
        self.provider
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.base_url, path)
    }

    fn decode_error(response: Response) -> String {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        format!("HTTP {}: {}", status.as_u16(), body)
    }

    fn provider_failure(&self, action: &str, detail: impl fmt::Display) -> anyhow::Error {
        LLMeterError::Provider(format!(
            "Failed to {action} from {} at {}: {}",
            self.provider.display_name(),
            self.base_url,
            detail
        ))
        .into()
    }

    pub fn get_json(&self, path: &str) -> anyhow::Result<Value> {
        let response = self
            .client
            .get(self.url(path))
            .send()
            .with_context(|| format!("GET {} failed", path))?;

        if !response.status().is_success() {
            return Err(LLMeterError::Provider(Self::decode_error(response)).into());
        }

        response
            .json()
            .with_context(|| format!("Invalid JSON from GET {path}"))
    }

    pub fn post_json(&self, path: &str, payload: &Value) -> anyhow::Result<Value> {
        let response = self
            .client
            .post(self.url(path))
            .json(payload)
            .send()
            .with_context(|| format!("POST {} failed", path))?;

        if !response.status().is_success() {
            return Err(LLMeterError::Provider(Self::decode_error(response)).into());
        }

        response
            .json()
            .with_context(|| format!("Invalid JSON from POST {path}"))
    }

    pub fn status(&self) -> ProviderStatus {
        match self.list_models() {
            Ok(models) => ProviderStatus {
                provider: self.provider,
                base_url: self.base_url.clone(),
                running: true,
                models: models.len(),
                error: None,
            },
            Err(error) => ProviderStatus {
                provider: self.provider,
                base_url: self.base_url.clone(),
                running: false,
                models: 0,
                error: Some(error_chain(&*error)),
            },
        }
    }

    pub fn is_running(&self) -> bool {
        self.status().running
    }

    pub fn list_models(&self) -> anyhow::Result<Vec<Value>> {
        let payload = self
            .get_json("models")
            .map_err(|error| self.provider_failure("list models", error))?;
        payload
            .get("data")
            .and_then(|value| value.as_array())
            .cloned()
            .ok_or_else(|| {
                self.provider_failure(
                    "list models",
                    "provider response must contain a top-level data array",
                )
            })
    }

    pub fn model_names(&self) -> anyhow::Result<Vec<String>> {
        Ok(self
            .list_models()?
            .iter()
            .filter_map(|m| {
                m.get("id")
                    .or_else(|| m.get("name"))
                    .or_else(|| m.get("model"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect())
    }

    pub fn show_model(&self, model: &str) -> anyhow::Result<Value> {
        let models = self.list_models()?;
        models
            .into_iter()
            .find(|m| {
                m.get("id")
                    .or_else(|| m.get("name"))
                    .or_else(|| m.get("model"))
                    .and_then(|v| v.as_str())
                    == Some(model)
            })
            .ok_or_else(|| LLMeterError::ModelNotFound(format!("Model not found: {model}")).into())
    }

    pub fn chat_completion(
        &self,
        model: &str,
        messages: Value,
        max_tokens: u32,
        temperature: f64,
        stream: bool,
        extra: Option<&Value>,
    ) -> anyhow::Result<ApiResult> {
        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "stream": stream,
        });
        merge_object(
            &mut body,
            extra,
            &[
                "model",
                "messages",
                "stream",
                "stream_options",
                "max_tokens",
                "temperature",
            ],
        )?;
        if stream {
            body["stream_options"] = serde_json::json!({"include_usage": true});
            self.post_streaming("chat/completions", &body, StreamKind::Chat)
        } else {
            self.post_timed("chat/completions", &body, extract_chat_text)
        }
    }

    pub fn responses(
        &self,
        model: &str,
        input: Value,
        max_tokens: u32,
        temperature: f64,
        extra: Option<&Value>,
    ) -> anyhow::Result<ApiResult> {
        let mut body = serde_json::json!({
            "model": model,
            "input": input,
            "temperature": temperature,
            "max_output_tokens": max_tokens,
        });
        merge_object(
            &mut body,
            extra,
            &["model", "input", "temperature", "max_output_tokens"],
        )?;
        self.post_timed("responses", &body, extract_response_text)
    }

    pub fn embeddings(&self, model: &str, input: Value) -> anyhow::Result<ApiResult> {
        let body = serde_json::json!({
            "model": model,
            "input": input,
        });
        self.post_timed("embeddings", &body, |_| String::new())
    }

    fn post_timed(
        &self,
        path: &str,
        body: &Value,
        extract_text: fn(&Value) -> String,
    ) -> anyhow::Result<ApiResult> {
        let started = Instant::now();
        let payload = self.post_json(path, body)?;
        let ended = Instant::now();
        Ok(ApiResult {
            endpoint: format!("/v1/{path}"),
            response_text: extract_text(&payload),
            raw: payload,
            wall_time_ns: ended.duration_since(started).as_nanos(),
            time_to_first_token_ns: None,
            token_timings_ns: Vec::new(),
            http_status: Some(200),
        })
    }

    fn post_streaming(
        &self,
        path: &str,
        body: &Value,
        kind: StreamKind,
    ) -> anyhow::Result<ApiResult> {
        let started = Instant::now();
        let response = self
            .client
            .post(self.url(path))
            .json(body)
            .send()
            .with_context(|| format!("POST {} (stream) failed", path))?;

        if !response.status().is_success() {
            return Err(LLMeterError::Provider(Self::decode_error(response)).into());
        }

        let mut first_token_at: Option<Instant> = None;
        let mut response_text = String::new();
        let mut token_timings_ns = Vec::new();
        let mut final_payload = serde_json::json!({});
        let reader = BufReader::new(response);

        for line_result in reader.lines() {
            let mut line = line_result.context("Failed to read streaming line")?;
            line = line.trim().to_string();
            if line.is_empty()
                || line.starts_with(':')
                || line.starts_with("event:")
                || line.starts_with("id:")
                || line.starts_with("retry:")
            {
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                line = data.trim().to_string();
            }
            if line == "[DONE]" {
                break;
            }

            let chunk: Value =
                serde_json::from_str(&line).context("Invalid streaming JSON chunk")?;
            if let Some(token) = extract_stream_delta(&chunk, kind) {
                if !token.is_empty() {
                    if first_token_at.is_none() {
                        first_token_at = Some(Instant::now());
                    }
                    token_timings_ns.push(Instant::now().duration_since(started).as_nanos());
                    response_text.push_str(&token);
                }
            }
            if chunk.get("usage").is_some() {
                final_payload = chunk;
            }
        }

        let ended = Instant::now();
        if final_payload
            .as_object()
            .map(|o| o.is_empty())
            .unwrap_or(false)
        {
            final_payload = serde_json::json!({"choices": [], "text": response_text.clone()});
        }

        Ok(ApiResult {
            endpoint: format!("/v1/{path}"),
            response_text,
            raw: final_payload,
            wall_time_ns: ended.duration_since(started).as_nanos(),
            time_to_first_token_ns: first_token_at.map(|t| t.duration_since(started).as_nanos()),
            token_timings_ns,
            http_status: Some(200),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum StreamKind {
    Chat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    Done,
    Json(Value),
}

fn merge_object(
    body: &mut Value,
    extra: Option<&Value>,
    reserved_keys: &[&str],
) -> anyhow::Result<()> {
    let Some(extra) = extra.and_then(|v| v.as_object()) else {
        return Ok(());
    };
    let Some(body_object) = body.as_object_mut() else {
        return Ok(());
    };
    for (key, value) in extra {
        if reserved_keys.contains(&key.as_str()) {
            return Err(LLMeterError::InvalidOption(format!(
                "Provider parameter '{key}' is reserved and cannot override a core request field."
            ))
            .into());
        }
        body_object.insert(key.clone(), value.clone());
    }
    Ok(())
}

fn extract_chat_text(payload: &Value) -> String {
    payload
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn extract_response_text(payload: &Value) -> String {
    if let Some(text) = payload.get("output_text").and_then(|v| v.as_str()) {
        return text.to_string();
    }
    payload
        .get("output")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .flat_map(|item| {
            item.get("content")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|content| {
            content
                .get("text")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect::<Vec<_>>()
        .join("")
}

fn extract_stream_delta(chunk: &Value, _kind: StreamKind) -> Option<String> {
    chunk
        .pointer("/choices/0/delta/content")
        .and_then(|v| v.as_str())
        .or_else(|| chunk.pointer("/choices/0/text").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
}

pub fn parse_openai_stream_line(line: &str) -> Option<StreamEvent> {
    let mut line = line.trim();
    if line.is_empty() {
        return None;
    }
    if let Some(data) = line.strip_prefix("data:") {
        line = data.trim();
    }
    if line == "[DONE]" {
        return Some(StreamEvent::Done);
    }
    serde_json::from_str::<Value>(line)
        .ok()
        .map(StreamEvent::Json)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{parse_openai_stream_line, ProviderClient, ProviderKind, StreamEvent};

    #[test]
    fn list_models_returns_friendly_provider_error() {
        let client =
            ProviderClient::new(ProviderKind::Ollama, "http://127.0.0.1:1/v1", 0.1).unwrap();
        let error = client.list_models().unwrap_err().to_string();
        assert!(error.contains("Failed to list models from Ollama"));
        assert!(error.contains("http://127.0.0.1:1/v1"));
    }

    #[test]
    fn parse_openai_stream_line_handles_done_and_json() {
        assert_eq!(
            parse_openai_stream_line("data: [DONE]"),
            Some(StreamEvent::Done)
        );
        assert_eq!(
            parse_openai_stream_line(r#"data: {"choices":[{"delta":{"content":"hi"}}]}"#),
            Some(StreamEvent::Json(
                json!({"choices":[{"delta":{"content":"hi"}}]})
            ))
        );
        assert_eq!(parse_openai_stream_line(""), None);
    }

    #[test]
    fn reserved_chat_fields_cannot_be_overridden_by_extra_parameters() {
        let client =
            ProviderClient::new(ProviderKind::Ollama, "http://127.0.0.1:1/v1", 1.0).unwrap();
        let error = client
            .chat_completion(
                "model",
                json!([{"role": "user", "content": "hi"}]),
                2,
                0.0,
                false,
                Some(&json!({"model": "attacker"})),
            )
            .unwrap_err()
            .to_string();
        assert!(error.contains("reserved"), "{error}");
    }
}
