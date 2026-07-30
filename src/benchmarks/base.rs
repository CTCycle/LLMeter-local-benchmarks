use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::benchmarks::registry::BenchmarkSuite;
use crate::errors::LLMeterError;
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
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.runs == 0 {
            return Err(LLMeterError::InvalidOption(
                "Benchmark runs must be greater than zero.".to_string(),
            )
            .into());
        }
        if self.max_tokens == 0 {
            return Err(LLMeterError::InvalidOption(
                "Maximum output tokens must be greater than zero.".to_string(),
            )
            .into());
        }
        if !self.temperature.is_finite() || self.temperature < 0.0 {
            return Err(LLMeterError::InvalidOption(
                "Temperature must be a finite non-negative number.".to_string(),
            )
            .into());
        }
        if !self.timeout.is_finite() || self.timeout <= 0.0 {
            return Err(LLMeterError::InvalidOption(
                "Benchmark timeout must be finite and greater than zero.".to_string(),
            )
            .into());
        }
        Ok(())
    }

    pub fn request_options(&self, _temperature: Option<f64>, _max_tokens: Option<u32>) -> Value {
        serde_json::json!(self.options)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkStepStatus {
    Started,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkStepUpdate {
    pub status: BenchmarkStepStatus,
    pub step_index: u32,
    pub total_steps: u32,
    pub run_index: Option<u32>,
    pub prompt_name: Option<String>,
    pub message: String,
}

pub trait BenchmarkProgressSink {
    fn on_step(&mut self, update: BenchmarkStepUpdate);
}

#[derive(Debug, Default)]
pub struct NullBenchmarkProgressSink;

impl BenchmarkProgressSink for NullBenchmarkProgressSink {
    fn on_step(&mut self, _update: BenchmarkStepUpdate) {}
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
    fn suite(&self) -> BenchmarkSuite;
    fn planned_steps(&self, context: &BenchmarkContext) -> u32;
    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
        progress: &mut dyn BenchmarkProgressSink,
    ) -> Vec<BenchmarkResultRecord>;
}

#[cfg(test)]
mod tests {
    use super::BenchmarkContext;
    use std::collections::HashMap;

    fn context() -> BenchmarkContext {
        BenchmarkContext {
            runs: 1,
            max_tokens: 1,
            temperature: 0.0,
            timeout: 1.0,
            options: HashMap::new(),
        }
    }

    #[test]
    fn benchmark_context_rejects_invalid_programmatic_values() {
        let mut invalid = context();
        invalid.runs = 0;
        assert!(invalid.validate().is_err());

        invalid = context();
        invalid.max_tokens = 0;
        assert!(invalid.validate().is_err());

        invalid = context();
        invalid.temperature = f64::NAN;
        assert!(invalid.validate().is_err());

        invalid = context();
        invalid.temperature = -1.0;
        assert!(invalid.validate().is_err());
    }
}
