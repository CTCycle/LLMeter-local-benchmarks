use std::collections::HashMap;

use crate::benchmarks::base::{Benchmark, BenchmarkContext, BenchmarkResultRecord};
use crate::benchmarks::metrics::{generation_metrics, preview};
use crate::ollama::client::OllamaClient;
use crate::prompts::SHORT_PROMPT;

pub struct BasicGenerationLatencyBenchmark;

impl Benchmark for BasicGenerationLatencyBenchmark {
    fn id(&self) -> &str {
        "generation-latency"
    }

    fn name(&self) -> &str {
        "Basic generation latency"
    }

    fn description(&self) -> &str {
        "Measures wall time, time to first token, Ollama duration fields, and output token throughput."
    }

    fn run(
        &self,
        client: &OllamaClient,
        model: &str,
        context: &BenchmarkContext,
    ) -> Vec<BenchmarkResultRecord> {
        let mut records = Vec::new();
        let options = context.generation_options(Some(0.0), None);

        for run_index in 1..=context.runs {
            let record = match client.generate_stream(
                model,
                SHORT_PROMPT,
                Some(&options),
                Some(&context.keep_alive),
            ) {
                Ok(result) => BenchmarkResultRecord {
                    benchmark_id: self.id().to_string(),
                    benchmark_name: self.name().to_string(),
                    model: model.to_string(),
                    run_index: Some(run_index),
                    prompt_name: Some("short".to_string()),
                    metrics: generation_metrics(&result),
                    response_preview: Some(preview(&result.response, 180)),
                    error: None,
                    metadata: Some({
                        let mut m = HashMap::new();
                        m.insert("options".to_string(), options.clone());
                        m
                    }),
                },
                Err(e) => BenchmarkResultRecord {
                    benchmark_id: self.id().to_string(),
                    benchmark_name: self.name().to_string(),
                    model: model.to_string(),
                    run_index: Some(run_index),
                    prompt_name: Some("short".to_string()),
                    metrics: HashMap::new(),
                    response_preview: None,
                    error: Some(e.to_string()),
                    metadata: Some({
                        let mut m = HashMap::new();
                        m.insert("options".to_string(), options.clone());
                        m
                    }),
                },
            };
            records.push(record);
        }
        records
    }
}
