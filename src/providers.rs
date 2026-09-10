use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Context;
use clap::ValueEnum;
use reqwest::blocking::{Client as HttpClient, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{redirect::Policy, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::errors::LLMeterError;
use crate::utils::error_chain;

const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;
const MAX_JSON_RESPONSE_BYTES: usize = 10 * 1024 * 1024;
const MAX_STREAM_EVENT_BYTES: usize = 1024 * 1024;
const CONNECT_TIMEOUT_CAP_SECS: f64 = 10.0;

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
struct ProviderDefinition {
    label: &'static str,
    display_name: &'static str,
    default_base_url: &'static str,
    tier: ProviderCompatibilityTier,
    notes: &'static str,
    base_url_env: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderCatalogEntry {
    pub provider: ProviderKind,
    pub tier: ProviderCompatibilityTier,
    pub default_base_url: &'static str,
    pub notes: &'static str,
}

impl ProviderKind {
    fn definition(self) -> ProviderDefinition {
        match self {
            Self::Ollama => ProviderDefinition {
                label: "ollama",
                display_name: "Ollama",
                default_base_url: "http://localhost:11434/v1",
                tier: ProviderCompatibilityTier::FirstClass,
                notes: "Existing first-class target",
                base_url_env: Some("OLLAMA_HOST"),
            },
            Self::Lmstudio => ProviderDefinition {
                label: "lmstudio",
                display_name: "LM Studio",
                default_base_url: "http://localhost:1234/v1",
                tier: ProviderCompatibilityTier::FirstClass,
                notes: "Existing first-class target",
                base_url_env: Some("LMSTUDIO_BASE_URL"),
            },
            Self::LlamaCpp => ProviderDefinition {
                label: "llama-cpp",
                display_name: "llama.cpp",
                default_base_url: "http://localhost:8080/v1",
                tier: ProviderCompatibilityTier::FirstClass,
                notes: "Existing first-class target",
                base_url_env: Some("LLAMA_CPP_BASE_URL"),
            },
            Self::OpenaiCompatible => ProviderDefinition {
                label: "openai-compatible",
                display_name: "OpenAI-compatible",
                default_base_url: "http://localhost:8000/v1",
                tier: ProviderCompatibilityTier::Custom,
                notes: "User-supplied local /v1 server",
                base_url_env: None,
            },
            Self::Vllm => ProviderDefinition {
                label: "vllm",
                display_name: "vLLM",
                default_base_url: "http://localhost:8000/v1",
                tier: ProviderCompatibilityTier::OpenAiCompatibleKnown,
                notes: "OpenAI-compatible server preset",
                base_url_env: Some("VLLM_BASE_URL"),
            },
            Self::Sglang => ProviderDefinition {
                label: "sglang",
                display_name: "SGLang",
                default_base_url: "http://localhost:30000/v1",
                tier: ProviderCompatibilityTier::OpenAiCompatibleKnown,
                notes: "OpenAI-compatible server preset",
                base_url_env: Some("SGLANG_BASE_URL"),
            },
            Self::Localai => ProviderDefinition {
                label: "localai",
                display_name: "LocalAI",
                default_base_url: "http://localhost:8080/v1",
                tier: ProviderCompatibilityTier::OpenAiCompatibleKnown,
                notes: "OpenAI-compatible server preset",
                base_url_env: Some("LOCALAI_BASE_URL"),
            },
            Self::Litellm => ProviderDefinition {
                label: "litellm",
                display_name: "LiteLLM",
                default_base_url: "http://localhost:4000/v1",
                tier: ProviderCompatibilityTier::OpenAiCompatibleKnown,
                notes: "OpenAI-compatible proxy preset",
                base_url_env: Some("LITELLM_BASE_URL"),
            },
            Self::Tgi => ProviderDefinition {
                label: "tgi",
                display_name: "TGI",
                default_base_url: "http://localhost:8080/v1",
                tier: ProviderCompatibilityTier::OpenAiCompatibleBestEffort,
                notes: "Requires OpenAI-compatible router mode",
                base_url_env: Some("TGI_BASE_URL"),
            },
            Self::TextGenerationWebui => ProviderDefinition {
                label: "text-generation-webui",
                display_name: "text-generation-webui",
                default_base_url: "http://localhost:5000/v1",
                tier: ProviderCompatibilityTier::OpenAiCompatibleBestEffort,
                notes: "API shape can vary by extension",
                base_url_env: Some("TEXT_GENERATION_WEBUI_BASE_URL"),
            },
            Self::Jan => ProviderDefinition {
                label: "jan",
                display_name: "Jan",
                default_base_url: "http://localhost:1337/v1",
                tier: ProviderCompatibilityTier::OpenAiCompatibleBestEffort,
                notes: "Local API behavior can vary by version",
                base_url_env: Some("JAN_BASE_URL"),
            },
            Self::MlxLm => ProviderDefinition {
                label: "mlx-lm",
                display_name: "MLX-LM",
                default_base_url: "http://localhost:8080/v1",
                tier: ProviderCompatibilityTier::OpenAiCompatibleBestEffort,
                notes: "OpenAI-compatible server shape can vary",
                base_url_env: Some("MLX_LM_BASE_URL"),
            },
        }
    }

    pub fn default_base_url(self) -> &'static str {
        self.definition().default_base_url
    }

    pub fn label(self) -> &'static str {
        self.definition().label
    }

    pub fn display_name(self) -> &'static str {
        self.definition().display_name
    }

    pub fn compatibility_tier(self) -> ProviderCompatibilityTier {
        self.definition().tier
    }

    pub fn base_url_env(self) -> Option<&'static str> {
        self.definition().base_url_env
    }

    pub fn catalog() -> Vec<ProviderCatalogEntry> {
        Self::value_variants()
            .iter()
            .copied()
            .map(|provider| {
                let definition = provider.definition();
                ProviderCatalogEntry {
                    provider,
                    tier: definition.tier,
                    default_base_url: definition.default_base_url,
                    notes: definition.notes,
                }
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
        let canonical = value.trim().to_ascii_lowercase();
        Self::value_variants()
        .iter()
        .copied()
        .find(|provider| provider.label() == canonical.as_str())
        .ok_or_else(|| {
            LLMeterError::InvalidOption(format!(
                "Unknown provider '{canonical}'. Use `llmeter providers list` for supported presets."
            ))
        })
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
    pub chunk_timings_ns: Vec<u128>,
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
    model_catalog: Arc<Mutex<Option<Vec<Value>>>>,
}

impl ProviderClient {
    pub fn new(provider: ProviderKind, base_url: &str, timeout: f64) -> anyhow::Result<Self> {
        if !timeout.is_finite() || timeout <= 0.0 || timeout > 24.0 * 60.0 * 60.0 {
            return Err(LLMeterError::InvalidOption(format!(
                "Invalid request timeout '{timeout}'. Expected a finite value greater than 0 and at most 86400 seconds."
            ))
            .into());
        }
        let base_url = validate_client_base_url(base_url)?;
        let mut default_headers = HeaderMap::new();
        if let Some(api_key) = read_ephemeral_api_key()? {
            let mut authorization = HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(
                |_| {
                    LLMeterError::Configuration(
                        "Invalid LLMETER_API_KEY. The value cannot be represented as an HTTP Authorization header."
                            .to_string(),
                    )
                },
            )?;
            authorization.set_sensitive(true);
            default_headers.insert(AUTHORIZATION, authorization);
        }
        let client = HttpClient::builder()
            .timeout(Duration::from_secs_f64(timeout))
            .connect_timeout(Duration::from_secs_f64(
                timeout.min(CONNECT_TIMEOUT_CAP_SECS),
            ))
            .redirect(Policy::none())
            .no_proxy()
            .user_agent(format!("llmeter/{}", env!("CARGO_PKG_VERSION")))
            .default_headers(default_headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(ProviderClient {
            provider,
            base_url,
            client,
            model_catalog: Arc::new(Mutex::new(None)),
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
        match read_response_body(response, MAX_ERROR_BODY_BYTES) {
            Ok((body, truncated)) => {
                let suffix = if truncated { " [body truncated]" } else { "" };
                format!("HTTP {}: {}{suffix}", status.as_u16(), body)
            }
            Err(error) => format!(
                "HTTP {}: unable to read error body: {error}",
                status.as_u16()
            ),
        }
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

    fn get_json(&self, path: &str) -> anyhow::Result<(Value, StatusCode)> {
        let response = self
            .client
            .get(self.url(path))
            .send()
            .with_context(|| format!("GET {} failed", path))?;

        if !response.status().is_success() {
            return Err(LLMeterError::Provider(Self::decode_error(response)).into());
        }

        let status = response.status();
        let (body, truncated) = read_response_body(response, MAX_JSON_RESPONSE_BYTES)?;
        if truncated {
            return Err(LLMeterError::Provider(format!(
                "GET {path} response exceeded the {} byte limit.",
                MAX_JSON_RESPONSE_BYTES
            ))
            .into());
        }
        let payload =
            serde_json::from_str(&body).with_context(|| format!("Invalid JSON from GET {path}"))?;
        Ok((payload, status))
    }

    fn post_json(&self, path: &str, payload: &Value) -> anyhow::Result<(Value, StatusCode)> {
        let response = self
            .client
            .post(self.url(path))
            .json(payload)
            .send()
            .with_context(|| format!("POST {} failed", path))?;

        if !response.status().is_success() {
            return Err(LLMeterError::Provider(Self::decode_error(response)).into());
        }

        let status = response.status();
        let (body, truncated) = read_response_body(response, MAX_JSON_RESPONSE_BYTES)?;
        if truncated {
            return Err(LLMeterError::Provider(format!(
                "POST {path} response exceeded the {} byte limit.",
                MAX_JSON_RESPONSE_BYTES
            ))
            .into());
        }
        let payload = serde_json::from_str(&body)
            .with_context(|| format!("Invalid JSON from POST {path}"))?;
        Ok((payload, status))
    }

    pub fn status(&self) -> ProviderStatus {
        match self.list_models_fresh() {
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

    pub fn list_models_cached(&self) -> anyhow::Result<Vec<Value>> {
        if let Some(models) = self
            .model_catalog
            .lock()
            .map_err(|_| {
                LLMeterError::Provider("Model catalog cache lock was poisoned.".to_string())
            })?
            .clone()
        {
            return Ok(models);
        }

        self.list_models_fresh()
    }

    pub fn list_models_fresh(&self) -> anyhow::Result<Vec<Value>> {
        let (payload, _) = self
            .get_json("models")
            .map_err(|error| self.provider_failure("list models", error))?;
        let models = payload
            .get("data")
            .and_then(|value| value.as_array())
            .cloned()
            .ok_or_else(|| {
                self.provider_failure(
                    "list models",
                    "provider response must contain a top-level data array",
                )
            })?;
        *self.model_catalog.lock().map_err(|_| {
            LLMeterError::Provider("Model catalog cache lock was poisoned.".to_string())
        })? = Some(models.clone());
        Ok(models)
    }

    pub fn invalidate_model_catalog(&self) -> anyhow::Result<()> {
        *self.model_catalog.lock().map_err(|_| {
            LLMeterError::Provider("Model catalog cache lock was poisoned.".to_string())
        })? = None;
        Ok(())
    }

    pub fn refresh_model_catalog(&self) -> anyhow::Result<Vec<Value>> {
        self.invalidate_model_catalog()?;
        self.list_models_fresh()
    }

    pub fn model_names(&self) -> anyhow::Result<Vec<String>> {
        Ok(self
            .list_models_cached()?
            .iter()
            .filter_map(|model| {
                model
                    .get("id")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
            })
            .collect())
    }

    pub fn show_model(&self, model: &str) -> anyhow::Result<Value> {
        let models = self.list_models_cached()?;
        models
            .into_iter()
            .find(|item| item.get("id").and_then(|value| value.as_str()) == Some(model))
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
        let (payload, status) = self.post_json(path, body)?;
        let ended = Instant::now();
        Ok(ApiResult {
            endpoint: format!("/v1/{path}"),
            response_text: extract_text(&payload),
            raw: payload,
            wall_time_ns: ended.duration_since(started).as_nanos(),
            time_to_first_token_ns: None,
            chunk_timings_ns: Vec::new(),
            http_status: Some(status.as_u16()),
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

        let status = response.status();
        if !status.is_success() {
            return Err(LLMeterError::Provider(Self::decode_error(response)).into());
        }

        let mut first_token_at: Option<Instant> = None;
        let mut response_text = String::new();
        let mut chunk_timings_ns = Vec::new();
        let mut final_payload = serde_json::json!({});
        let mut reader = BufReader::new(response);

        let mut event_data = Vec::new();
        while let Some(line) = read_stream_line_limited(reader.by_ref())? {
            let line = std::str::from_utf8(&line).context("Streaming event was not valid UTF-8")?;
            if line.is_empty() {
                if process_stream_event(
                    &mut event_data,
                    kind,
                    started,
                    &mut first_token_at,
                    &mut response_text,
                    &mut chunk_timings_ns,
                    &mut final_payload,
                )? {
                    break;
                }
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                event_data.push(data.trim_start().to_string());
            }
        }
        if !event_data.is_empty() {
            let _ = process_stream_event(
                &mut event_data,
                kind,
                started,
                &mut first_token_at,
                &mut response_text,
                &mut chunk_timings_ns,
                &mut final_payload,
            )?;
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
            chunk_timings_ns,
            http_status: Some(status.as_u16()),
        })
    }
}

fn read_ephemeral_api_key() -> anyhow::Result<Option<String>> {
    match std::env::var("LLMETER_API_KEY") {
        Ok(value) if value.is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(LLMeterError::Configuration(format!(
            "Invalid LLMETER_API_KEY: {error}. Expected valid Unicode text."
        ))
        .into()),
    }
}

fn validate_client_base_url(value: &str) -> anyhow::Result<String> {
    let url = Url::parse(value).map_err(|error| {
        LLMeterError::InvalidOption(format!(
            "Invalid provider base URL '{value}': {error}. Expected an absolute HTTP or HTTPS /v1 URL."
        ))
    })?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path().trim_end_matches('/') != "/v1"
    {
        return Err(LLMeterError::InvalidOption(format!(
            "Invalid provider base URL '{value}'. Expected an absolute HTTP or HTTPS /v1 URL without credentials, query strings, or fragments."
        ))
        .into());
    }
    Ok(url.to_string().trim_end_matches('/').to_string())
}

fn read_response_body(mut response: Response, limit: usize) -> anyhow::Result<(String, bool)> {
    let mut bytes = Vec::with_capacity(limit.min(8192));
    let mut limited = response.by_ref().take((limit + 1) as u64);
    limited
        .read_to_end(&mut bytes)
        .context("Failed to read HTTP response body")?;
    let truncated = bytes.len() > limit;
    if truncated {
        bytes.truncate(limit);
    }
    Ok((String::from_utf8_lossy(&bytes).into_owned(), truncated))
}

fn read_stream_line_limited<R: BufRead>(reader: &mut R) -> anyhow::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    loop {
        let buffer = reader.fill_buf().context("Failed to read streaming line")?;
        if buffer.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Ok(Some(line))
            };
        }
        let newline_at = buffer.iter().position(|byte| *byte == b'\n');
        let take = newline_at.map_or(buffer.len(), |index| index + 1);
        if line.len() + take > MAX_STREAM_EVENT_BYTES {
            return Err(LLMeterError::Provider(format!(
                "Streaming event line exceeded the {} byte limit.",
                MAX_STREAM_EVENT_BYTES
            ))
            .into());
        }
        line.extend_from_slice(&buffer[..take]);
        reader.consume(take);
        if newline_at.is_some() {
            while matches!(line.last(), Some(b'\n' | b'\r')) {
                line.pop();
            }
            return Ok(Some(line));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn process_stream_event(
    event_data: &mut Vec<String>,
    kind: StreamKind,
    started: Instant,
    first_token_at: &mut Option<Instant>,
    response_text: &mut String,
    chunk_timings_ns: &mut Vec<u128>,
    final_payload: &mut Value,
) -> anyhow::Result<bool> {
    if event_data.is_empty() {
        return Ok(false);
    }
    let payload = std::mem::take(event_data).join("\n");
    if payload == "[DONE]" {
        return Ok(true);
    }
    let chunk: Value = serde_json::from_str(&payload).context("Invalid streaming JSON event")?;
    if let Some(token) = extract_stream_delta(&chunk, kind).filter(|token| !token.is_empty()) {
        let now = Instant::now();
        if first_token_at.is_none() {
            *first_token_at = Some(now);
        }
        chunk_timings_ns.push(now.duration_since(started).as_nanos());
        response_text.push_str(&token);
    }
    if chunk.get("usage").is_some() {
        *final_payload = chunk;
    }
    Ok(false)
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
    use std::io::Cursor;
    use std::time::Instant;

    use clap::ValueEnum;
    use serde_json::json;

    use super::{
        parse_openai_stream_line, process_stream_event, read_stream_line_limited, ProviderClient,
        ProviderKind, StreamEvent, StreamKind,
    };

    #[test]
    fn provider_parsing_accepts_only_canonical_labels() {
        for provider in ProviderKind::catalog() {
            assert_eq!(provider.provider.label().parse(), Ok(provider.provider));
        }
        for alias in ["lm-studio", "llama.cpp", "custom", "sgl", "swebench"] {
            assert!(alias.parse::<ProviderKind>().is_err(), "{alias}");
        }
    }

    #[test]
    fn provider_value_enum_names_match_canonical_labels() {
        for provider in ProviderKind::value_variants().iter().copied() {
            let possible = provider.to_possible_value().expect("provider value");
            assert_eq!(possible.get_name(), provider.label());
        }
    }

    #[test]
    fn list_models_returns_friendly_provider_error() {
        let client =
            ProviderClient::new(ProviderKind::Ollama, "http://127.0.0.1:1/v1", 0.1).unwrap();
        let error = client.list_models_fresh().unwrap_err().to_string();
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

    #[test]
    fn client_rejects_untrusted_base_url_shapes() {
        for value in [
            "localhost:1234/v1",
            "file:///tmp/v1",
            "http://user:secret@localhost:1234/v1",
            "http://localhost:1234/v1?token=secret",
            "http://localhost:1234/api",
        ] {
            let error = ProviderClient::new(ProviderKind::Ollama, value, 1.0)
                .unwrap_err()
                .to_string();
            assert!(error.contains("Invalid provider base URL"), "{error}");
        }
    }

    #[test]
    fn streaming_reader_preserves_multiline_data_events_and_ignores_metadata() {
        let mut reader = Cursor::new(
            b": heartbeat\nretry: 1000\ndata: {\"choices\":[\ndata: {\"delta\":{\"content\":\"hello\"}}]}\n\ndata: [DONE]\n\n"
                .as_slice(),
        );
        let started = Instant::now();
        let mut event_data = Vec::new();
        let mut first_token_at = None;
        let mut response_text = String::new();
        let mut chunk_timings_ns = Vec::new();
        let mut final_payload = json!({});

        while let Some(line) = read_stream_line_limited(&mut reader).unwrap() {
            let line = std::str::from_utf8(&line).unwrap();
            if line.is_empty() {
                if process_stream_event(
                    &mut event_data,
                    StreamKind::Chat,
                    started,
                    &mut first_token_at,
                    &mut response_text,
                    &mut chunk_timings_ns,
                    &mut final_payload,
                )
                .unwrap()
                {
                    break;
                }
            } else if let Some(data) = line.strip_prefix("data:") {
                event_data.push(data.trim_start().to_string());
            }
        }

        assert_eq!(response_text, "hello");
        assert_eq!(chunk_timings_ns.len(), 1);
        assert!(first_token_at.is_some());
    }
}
