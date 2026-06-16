use std::collections::HashMap;

use crate::benchmarks::base::{
    Benchmark, BenchmarkContext, BenchmarkProgressSink, BenchmarkResultRecord, BenchmarkStepStatus,
    BenchmarkStepUpdate,
};
use crate::benchmarks::metrics::{generation_metrics, preview};
use crate::benchmarks::registry::BenchmarkSuite;
use crate::prompts::SHORT_PROMPT;
use crate::providers::ProviderClient;
use crate::utils::error_chain;

pub struct BasicGenerationLatencyBenchmark;

impl Benchmark for BasicGenerationLatencyBenchmark {
    fn id(&self) -> &str {
        "chat-generation"
    }

    fn name(&self) -> &str {
        "Basic generation latency"
    }

    fn description(&self) -> &str {
        "Measures wall time, time to first token, usage fields, and output token throughput."
    }

    fn suite(&self) -> BenchmarkSuite {
        BenchmarkSuite::Llm
    }

    fn planned_steps(&self, context: &BenchmarkContext) -> u32 {
        context.runs
    }

    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
        progress: &mut dyn BenchmarkProgressSink,
    ) -> Vec<BenchmarkResultRecord> {
        let mut records = Vec::new();
        let options = context.request_options(Some(0.0), None);
        let messages = serde_json::json!([
            {"role": "user", "content": SHORT_PROMPT}
        ]);
        let total_steps = self.planned_steps(context);

        for run_index in 1..=context.runs {
            progress.on_step(BenchmarkStepUpdate {
                status: BenchmarkStepStatus::Started,
                step_index: run_index,
                total_steps,
                run_index: Some(run_index),
                prompt_name: Some("short".to_string()),
                message: "Issuing chat completion request".to_string(),
            });
            let record = match client.chat_completion(
                model,
                messages.clone(),
                context.max_tokens,
                0.0,
                true,
                Some(&options),
            ) {
                Ok(result) => BenchmarkResultRecord {
                    benchmark_id: self.id().to_string(),
                    benchmark_name: self.name().to_string(),
                    model: model.to_string(),
                    run_index: Some(run_index),
                    prompt_name: Some("short".to_string()),
                    metrics: generation_metrics(&result),
                    response_preview: Some(preview(&result.response_text, 180)),
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
                    error: Some(error_chain(&*e)),
                    metadata: Some({
                        let mut m = HashMap::new();
                        m.insert("options".to_string(), options.clone());
                        m
                    }),
                },
            };
            records.push(record);
            progress.on_step(BenchmarkStepUpdate {
                status: BenchmarkStepStatus::Completed,
                step_index: run_index,
                total_steps,
                run_index: Some(run_index),
                prompt_name: Some("short".to_string()),
                message: "Completed chat completion request".to_string(),
            });
        }
        records
    }
}
