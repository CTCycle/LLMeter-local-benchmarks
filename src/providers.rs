use std::fmt;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use anyhow::Context;
use clap::ValueEnum;
use reqwest::blocking::{Client as HttpClient, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::LLMeterError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    Ollama,
    Lmstudio,
    LlamaCpp,
    OpenaiCompatible,
}

impl ProviderKind {
    pub fn default_base_url(self) -> &'static str {
        match self {
            ProviderKind::Ollama => "http://localhost:11434/v1",
            ProviderKind::Lmstudio => "http://localhost:1234/v1",
            ProviderKind::LlamaCpp => "http://localhost:8080/v1",
            ProviderKind::OpenaiCompatible => "http://localhost:8000/v1",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProviderKind::Ollama => "ollama",
            ProviderKind::Lmstudio => "lmstudio",
            ProviderKind::LlamaCpp => "llama-cpp",
            ProviderKind::OpenaiCompatible => "openai-compatible",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ProviderKind::Ollama => "Ollama",
            ProviderKind::Lmstudio => "LM Studio",
            ProviderKind::LlamaCpp => "llama.cpp",
            ProviderKind::OpenaiCompatible => "OpenAI-compatible",
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
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
            other => Err(LLMeterError::InvalidOption(format!(
                "Unknown provider '{other}'. Use ollama, lmstudio, llama-cpp, or openai-compatible."
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
    pub fn new(provider: ProviderKind, base_url: &str, timeout: f64) -> Self {
        let client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs_f64(timeout))
            .build()
            .expect("Failed to build HTTP client");

        ProviderClient {
            provider,
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
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
                error: Some(error.to_string()),
            },
        }
    }

    pub fn is_running(&self) -> bool {
        self.list_models().is_ok()
    }

    pub fn list_models(&self) -> anyhow::Result<Vec<Value>> {
        let payload = self.get_json("models")?;
        Ok(payload
            .get("data")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
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
        merge_object(&mut body, extra);
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
        merge_object(&mut body, extra);
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
        })
    }

    fn post_streaming(
        &self,
        path: &str,
        body: &Value,
        kind: StreamKind,
    ) -> anyhow::Result<ApiResult> {
        let response = self
            .client
            .post(self.url(path))
            .json(body)
            .send()
            .with_context(|| format!("POST {} (stream) failed", path))?;

        if !response.status().is_success() {
            return Err(LLMeterError::Provider(Self::decode_error(response)).into());
        }

        let started = Instant::now();
        let mut first_token_at: Option<Instant> = None;
        let mut chunks = Vec::new();
        let mut final_payload = serde_json::json!({});
        let reader = BufReader::new(response);

        for line_result in reader.lines() {
            let mut line = line_result.context("Failed to read streaming line")?;
            line = line.trim().to_string();
            if line.is_empty() {
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
                    chunks.push(token);
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
            final_payload = serde_json::json!({"choices": [], "text": chunks.join("")});
        }

        Ok(ApiResult {
            endpoint: format!("/v1/{path}"),
            response_text: chunks.join(""),
            raw: final_payload,
            wall_time_ns: ended.duration_since(started).as_nanos(),
            time_to_first_token_ns: first_token_at.map(|t| t.duration_since(started).as_nanos()),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum StreamKind {
    Chat,
}

fn merge_object(body: &mut Value, extra: Option<&Value>) {
    let Some(extra) = extra.and_then(|v| v.as_object()) else {
        return;
    };
    let Some(body_object) = body.as_object_mut() else {
        return;
    };
    for (key, value) in extra {
        body_object.insert(key.clone(), value.clone());
    }
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
