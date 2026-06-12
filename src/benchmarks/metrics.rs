use std::collections::HashMap;

use serde_json::Value;
use similar::TextDiff;

use crate::ollama::client::GenerateResult;
use crate::utils::{ns_to_ms, ns_to_ms_u64};

pub use crate::utils::preview;

pub fn generation_metrics(result: &GenerateResult) -> HashMap<String, Value> {
    let mut metrics = HashMap::new();
    insert_f64(&mut metrics, "wall_time_ms", ns_to_ms(Some(result.wall_time_ns)));
    insert_f64(
        &mut metrics,
        "time_to_first_token_ms",
        ns_to_ms(result.time_to_first_token_ns),
    );
    insert_f64(
        &mut metrics,
        "api_total_duration_ms",
        ns_to_ms_u64(result.raw.get("total_duration").and_then(|v| v.as_u64())),
    );
    insert_f64(
        &mut metrics,
        "api_load_duration_ms",
        ns_to_ms_u64(result.raw.get("load_duration").and_then(|v| v.as_u64())),
    );
    insert_f64(
        &mut metrics,
        "api_prompt_eval_duration_ms",
        ns_to_ms_u64(result.raw.get("prompt_eval_duration").and_then(|v| v.as_u64())),
    );
    insert_f64(
        &mut metrics,
        "api_eval_duration_ms",
        ns_to_ms_u64(result.raw.get("eval_duration").and_then(|v| v.as_u64())),
    );
    insert_u64(
        &mut metrics,
        "prompt_eval_count",
        result.raw.get("prompt_eval_count").and_then(|v| v.as_u64()),
    );
    insert_u64(
        &mut metrics,
        "eval_count",
        result.raw.get("eval_count").and_then(|v| v.as_u64()),
    );
    insert_f64(&mut metrics, "tokens_per_second", result.tokens_per_second());
    metrics.insert(
        "response_chars".to_string(),
        Value::from(result.response.len() as u64),
    );
    if let Some(reason) = result.raw.get("done_reason").and_then(|v| v.as_str()) {
        metrics.insert("done_reason".to_string(), Value::from(reason));
    }
    metrics
}

pub fn pairwise_similarity(values: &[String]) -> Vec<f64> {
    let mut scores = Vec::new();
    for i in 0..values.len() {
        for j in (i + 1)..values.len() {
            let diff = TextDiff::from_words(&values[i], &values[j]);
            let ratio = diff.ratio();
            scores.push(ratio);
        }
    }
    scores.into_iter().map(|s| s as f64).collect()
}

fn insert_f64(map: &mut HashMap<String, Value>, key: &str, value: Option<f64>) {
    if let Some(v) = value {
        map.insert(key.to_string(), Value::from(v));
    }
}

fn insert_u64(map: &mut HashMap<String, Value>, key: &str, value: Option<u64>) {
    if let Some(v) = value {
        map.insert(key.to_string(), Value::from(v));
    }
}
