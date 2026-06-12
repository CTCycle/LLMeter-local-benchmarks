use std::collections::HashMap;

use serde_json::Value;

use crate::benchmarks::base::{Benchmark, BenchmarkContext, BenchmarkResultRecord};
use crate::benchmarks::metrics::{generation_metrics, preview};
use crate::ollama::client::OllamaClient;
use crate::prompts::prompts_by_size;

pub struct PromptSizePerformanceBenchmark;

impl Benchmark for PromptSizePerformanceBenchmark {
    fn id(&self) -> &str {
        "prompt-sizes"
    }

    fn name(&self) -> &str {
        "Performance across prompt sizes"
    }

    fn description(&self) -> &str {
        "Runs short, medium, and long prompts to compare prompt processing and generation timing."
    }

    fn run(
        &self,
        client: &OllamaClient,
        model: &str,
        context: &BenchmarkContext,
    ) -> Vec<BenchmarkResultRecord> {
        let mut records = Vec::new();
        let options = context.generation_options(Some(0.0), None);
        let prompts = prompts_by_size();

        for (prompt_name, prompt) in &prompts {
            for run_index in 1..=context.runs {
                let record = match client.generate_stream(
                    model,
                    prompt,
                    Some(&options),
                    Some(&context.keep_alive),
                ) {
                    Ok(result) => {
                        let mut metrics = generation_metrics(&result);
                        metrics.insert(
                            "prompt_chars".to_string(),
                            Value::from(prompt.len() as u64),
                        );
                        BenchmarkResultRecord {
                            benchmark_id: self.id().to_string(),
                            benchmark_name: self.name().to_string(),
                            model: model.to_string(),
                            run_index: Some(run_index),
                            prompt_name: Some(prompt_name.to_string()),
                            metrics,
                            response_preview: Some(preview(&result.response, 180)),
                            error: None,
                            metadata: Some({
                                let mut m = HashMap::new();
                                m.insert("options".to_string(), options.clone());
                                m
                            }),
                        }
                    }
                    Err(e) => BenchmarkResultRecord {
                        benchmark_id: self.id().to_string(),
                        benchmark_name: self.name().to_string(),
                        model: model.to_string(),
                        run_index: Some(run_index),
                        prompt_name: Some(prompt_name.to_string()),
                        metrics: HashMap::new(),
                        response_preview: None,
                        error: Some(e.to_string()),
                        metadata: Some({
                            let mut m = HashMap::new();
                            m.insert("options".to_string(), options.clone());
                            m.insert("prompt_chars".to_string(), Value::from(prompt.len() as u64));
                            m
                        }),
                    },
                };
                records.push(record);
            }
        }
        records
    }
}
