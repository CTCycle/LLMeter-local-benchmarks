use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::providers::ProviderClient;

#[derive(Debug, Clone)]
pub struct BenchmarkContext {
    pub runs: u32,
    pub max_tokens: u32,
    pub temperature: f64,
    pub timeout: f64,
    pub options: HashMap<String, Value>,
}

impl BenchmarkContext {
    pub fn request_options(&self, temperature: Option<f64>, max_tokens: Option<u32>) -> Value {
        let mut merged = self.options.clone();
        merged
            .entry("temperature".to_string())
            .or_insert_with(|| Value::from(temperature.unwrap_or(self.temperature)));
        merged
            .entry("max_tokens".to_string())
            .or_insert_with(|| Value::from(max_tokens.unwrap_or(self.max_tokens) as i64));
        serde_json::json!(merged)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResultRecord {
    pub benchmark_id: String,
    pub benchmark_name: String,
    pub model: String,
    pub run_index: Option<u32>,
    pub prompt_name: Option<String>,
    pub metrics: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

pub trait Benchmark: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
    ) -> Vec<BenchmarkResultRecord>;
}
