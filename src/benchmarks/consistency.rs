use std::collections::HashMap;
use std::time::Instant;

use serde_json::Value;

use crate::benchmarks::base::{Benchmark, BenchmarkContext, BenchmarkResultRecord};
use crate::benchmarks::metrics::{generation_metrics, pairwise_similarity, preview};
use crate::ollama::client::OllamaClient;
use crate::prompts::CONSISTENCY_PROMPT;
use crate::utils::ns_to_ms;

pub struct ResponseConsistencyBenchmark;

impl Benchmark for ResponseConsistencyBenchmark {
    fn id(&self) -> &str {
        "consistency"
    }

    fn name(&self) -> &str {
        "Response consistency"
    }

    fn description(&self) -> &str {
        "Repeats the same prompt and reports exact-match and pairwise text similarity."
    }

    fn run(
        &self,
        client: &OllamaClient,
        model: &str,
        context: &BenchmarkContext,
    ) -> Vec<BenchmarkResultRecord> {
        let runs = context.runs.max(2);
        let options = context.generation_options(Some(context.temperature), None);
        let mut responses: Vec<String> = Vec::new();
        let mut per_run_metrics_list: Vec<HashMap<String, Value>> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let started = Instant::now();

        for _ in 0..runs {
            match client.generate(
                model,
                CONSISTENCY_PROMPT,
                Some(&options),
                Some(&context.keep_alive),
            ) {
                Ok(result) => {
                    responses.push(result.response.trim().to_string());
                    per_run_metrics_list.push(generation_metrics(&result));
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }

        let ended = Instant::now();
        let scores = pairwise_similarity(&responses);
        let unique_count = {
            let set: std::collections::HashSet<&str> =
                responses.iter().map(|s| s.as_str()).collect();
            set.len()
        };
        let exact_match_ratio = if responses.len() <= 1 {
            1.0
        } else {
            let ratio =
                1.0 - ((unique_count as f64 - 1.0) / (responses.len() as f64 - 1.0).max(1.0));
            (ratio * 1000.0).round() / 1000.0
        };

        let output_token_rates: Vec<f64> = per_run_metrics_list
            .iter()
            .filter_map(|m| m.get("tokens_per_second").and_then(|v| v.as_f64()))
            .collect();
        let mean_tps = if output_token_rates.is_empty() {
            None
        } else {
            Some(
                (output_token_rates.iter().sum::<f64>() / output_token_rates.len() as f64 * 1000.0)
                    .round()
                    / 1000.0,
            )
        };

        let mut metrics = HashMap::new();
        metrics.insert("requested_runs".to_string(), Value::from(runs));
        metrics.insert("successful_runs".to_string(), Value::from(responses.len() as u64));
        metrics.insert("failed_runs".to_string(), Value::from(errors.len() as u64));
        metrics.insert("unique_responses".to_string(), Value::from(unique_count as u64));
        metrics.insert("exact_match_ratio".to_string(), Value::from(exact_match_ratio));

        if !scores.is_empty() {
            let mean = scores.iter().sum::<f64>() / scores.len() as f64;
            let min = scores.iter().cloned().fold(f64::MAX, f64::min);
            let max = scores.iter().cloned().fold(f64::MIN, f64::max);
            metrics.insert(
                "mean_pairwise_similarity".to_string(),
                Value::from((mean * 10000.0).round() / 10000.0),
            );
            metrics.insert(
                "min_pairwise_similarity".to_string(),
                Value::from((min * 10000.0).round() / 10000.0),
            );
            metrics.insert(
                "max_pairwise_similarity".to_string(),
                Value::from((max * 10000.0).round() / 10000.0),
            );
        }

        if let Some(tps) = mean_tps {
            metrics.insert("mean_tokens_per_second".to_string(), Value::from(tps));
        }

        metrics.insert(
            "wall_time_ms".to_string(),
            Value::from(ns_to_ms(Some(ended.duration_since(started).as_nanos())).unwrap_or(0.0)),
        );

        let response_preview = if responses.is_empty() {
            None
        } else {
            let previews: Vec<String> = responses
                .iter()
                .take(3)
                .map(|t| preview(t, 80))
                .collect();
            Some(previews.join(" | "))
        };

        let error_str = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };

        vec![BenchmarkResultRecord {
            benchmark_id: self.id().to_string(),
            benchmark_name: self.name().to_string(),
            model: model.to_string(),
            run_index: None,
            prompt_name: Some("consistency".to_string()),
            metrics,
            response_preview,
            error: error_str,
            metadata: Some({
                let mut m = HashMap::new();
                m.insert("options".to_string(), options);
                m
            }),
        }]
    }
}
