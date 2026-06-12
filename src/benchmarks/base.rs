use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ollama::client::OllamaClient;

#[derive(Debug, Clone)]
pub struct BenchmarkContext {
    pub runs: u32,
    pub num_predict: u32,
    pub temperature: f64,
    pub timeout: f64,
    pub keep_alive: String,
    pub options: HashMap<String, Value>,
}

impl BenchmarkContext {
    pub fn generation_options(&self, temperature: Option<f64>, num_predict: Option<u32>) -> Value {
        let mut merged = self.options.clone();
        merged
            .entry("temperature".to_string())
            .or_insert_with(|| {
                Value::from(temperature.unwrap_or(self.temperature))
            });
        merged
            .entry("num_predict".to_string())
            .or_insert_with(|| {
                Value::from(num_predict.unwrap_or(self.num_predict) as i64)
            });
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
        client: &OllamaClient,
        model: &str,
        context: &BenchmarkContext,
    ) -> Vec<BenchmarkResultRecord>;
}
