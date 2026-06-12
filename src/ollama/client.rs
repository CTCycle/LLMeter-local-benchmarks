use std::io::{BufRead, BufReader};
use std::time::Instant;

use anyhow::Context;
use reqwest::blocking::{Client as HttpClient, Response};
use serde_json::Value;

use crate::errors::LLMeterError;

#[derive(Debug, Clone)]
pub struct GenerateResult {
    pub model: String,
    pub prompt: String,
    pub response: String,
    pub raw: Value,
    pub wall_time_ns: u128,
    pub time_to_first_token_ns: Option<u128>,
}

impl GenerateResult {
    pub fn tokens_per_second(&self) -> Option<f64> {
        let count = self.raw.get("eval_count").and_then(|v| v.as_u64())?;
        let duration = self.raw.get("eval_duration").and_then(|v| v.as_u64())?;
        if duration == 0 {
            return None;
        }
        Some((count as f64) / (duration as f64 / 1_000_000_000.0))
    }
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    api_base_url: String,
    timeout: f64,
    client: HttpClient,
}

impl OllamaClient {
    pub fn new(api_base_url: &str, timeout: f64) -> Self {
        let client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs_f64(timeout))
            .build()
            .expect("Failed to build HTTP client");

        let url = api_base_url.trim_end_matches('/').to_string();
        OllamaClient {
            api_base_url: url,
            timeout,
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.api_base_url, path)
    }

    fn decode_error(response: Response) -> String {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        format!("HTTP {}: {}", status.as_u16(), body)
    }

    pub fn get_json(&self, path: &str) -> anyhow::Result<Value> {
        let response = self
            .client
            .get(&self.url(path))
            .send()
            .with_context(|| format!("GET {} failed", path))?;

        if !response.status().is_success() {
            let msg = Self::decode_error(response);
            return Err(LLMeterError::OllamaServer(msg).into());
        }

        let body: Value = response.json().with_context(|| format!("Invalid JSON from GET {path}"))?;
        Ok(body)
    }

    pub fn post_json(&self, path: &str, payload: &Value) -> anyhow::Result<Value> {
        let response = self
            .client
            .post(&self.url(path))
            .json(payload)
            .send()
            .with_context(|| format!("POST {} failed", path))?;

        if !response.status().is_success() {
            let msg = Self::decode_error(response);
            return Err(LLMeterError::OllamaServer(msg).into());
        }

        let body: Value = response.json().with_context(|| format!("Invalid JSON from POST {path}"))?;
        Ok(body)
    }

    pub fn version(&self) -> anyhow::Result<Value> {
        let client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let response = client
            .get(&self.url("version"))
            .send()
            .context("GET version failed")?;

        if !response.status().is_success() {
            let msg = Self::decode_error(response);
            return Err(LLMeterError::OllamaServer(msg).into());
        }

        let body: Value = response.json().context("Invalid JSON from version")?;
        Ok(body)
    }

    pub fn is_running(&self) -> bool {
        self.version().is_ok()
    }

    pub fn list_models(&self) -> anyhow::Result<Vec<Value>> {
        let payload = self.get_json("tags")?;
        Ok(payload
            .get("models")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }

    pub fn show_model(&self, model: &str) -> anyhow::Result<Value> {
        self.post_json("show", &serde_json::json!({"model": model}))
    }

    pub fn running_models(&self) -> anyhow::Result<Vec<Value>> {
        let payload = self.get_json("ps")?;
        Ok(payload
            .get("models")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }

    pub fn generate(
        &self,
        model: &str,
        prompt: &str,
        options: Option<&Value>,
        keep_alive: Option<&str>,
    ) -> anyhow::Result<GenerateResult> {
        let mut body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });
        if let Some(opts) = options {
            body["options"] = opts.clone();
        }
        if let Some(ka) = keep_alive {
            body["keep_alive"] = serde_json::json!(ka);
        }

        let started = Instant::now();
        let payload = self.post_json("generate", &body)?;
        let ended = Instant::now();

        Ok(GenerateResult {
            model: model.to_string(),
            prompt: prompt.to_string(),
            response: payload.get("response").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            raw: payload,
            wall_time_ns: ended.duration_since(started).as_nanos(),
            time_to_first_token_ns: None,
        })
    }

    pub fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        options: Option<&Value>,
        keep_alive: Option<&str>,
    ) -> anyhow::Result<GenerateResult> {
        let mut body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": true,
        });
        if let Some(opts) = options {
            body["options"] = opts.clone();
        }
        if let Some(ka) = keep_alive {
            body["keep_alive"] = serde_json::json!(ka);
        }

        let response = self
            .client
            .post(&self.url("generate"))
            .json(&body)
            .send()
            .context("POST generate (stream) failed")?;

        if !response.status().is_success() {
            let msg = Self::decode_error(response);
            return Err(LLMeterError::OllamaServer(msg).into());
        }

        let mut chunks: Vec<String> = Vec::new();
        let mut final_payload: Option<Value> = None;
        let mut first_token_at: Option<Instant> = None;
        let started = Instant::now();

        let reader = BufReader::new(response);
        for line_result in reader.lines() {
            let line = line_result.context("Failed to read streaming line")?;
            if line.trim().is_empty() {
                continue;
            }
            let chunk: Value =
                serde_json::from_str(&line).context("Invalid streaming JSON chunk")?;

            let token = chunk.get("response").and_then(|v| v.as_str()).unwrap_or("");
            if !token.is_empty() && first_token_at.is_none() {
                first_token_at = Some(Instant::now());
                chunks.push(token.to_string());
            } else if !token.is_empty() {
                chunks.push(token.to_string());
            }

            if chunk.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                final_payload = Some(chunk);
                break;
            }
        }

        let ended = Instant::now();
        let response_text = chunks.join("");

        let final_payload = final_payload.unwrap_or_else(|| {
            serde_json::json!({"response": response_text, "done": false})
        });

        Ok(GenerateResult {
            model: model.to_string(),
            prompt: prompt.to_string(),
            response: response_text,
            raw: final_payload,
            wall_time_ns: ended.duration_since(started).as_nanos(),
            time_to_first_token_ns: first_token_at.map(|t| t.duration_since(started).as_nanos()),
        })
    }

    pub fn unload_model(&self, model: &str) -> anyhow::Result<Value> {
        self.post_json(
            "generate",
            &serde_json::json!({"model": model, "keep_alive": 0}),
        )
    }
}
