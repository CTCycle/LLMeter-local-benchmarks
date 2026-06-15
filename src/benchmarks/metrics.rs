use std::collections::HashMap;

use serde_json::Value;
use similar::TextDiff;

use crate::providers::ApiResult;
use crate::utils::ns_to_ms;

pub use crate::utils::preview;

pub fn generation_metrics(result: &ApiResult) -> HashMap<String, Value> {
    let mut metrics = HashMap::new();
    insert_f64(
        &mut metrics,
        "wall_time_ms",
        ns_to_ms(Some(result.wall_time_ns)),
    );
    insert_f64(
        &mut metrics,
        "time_to_first_token_ms",
        ns_to_ms(result.time_to_first_token_ns),
    );
    insert_u64(
        &mut metrics,
        "input_tokens",
        result
            .raw
            .pointer("/usage/prompt_tokens")
            .or_else(|| result.raw.pointer("/usage/input_tokens"))
            .and_then(|v| v.as_u64()),
    );
    insert_u64(&mut metrics, "output_tokens", result.output_tokens());
    insert_u64(
        &mut metrics,
        "total_tokens",
        result
            .raw
            .pointer("/usage/total_tokens")
            .and_then(|v| v.as_u64()),
    );
    insert_f64(
        &mut metrics,
        "tokens_per_second",
        result.tokens_per_second(),
    );
    metrics.insert("endpoint".to_string(), Value::from(result.endpoint.clone()));
    metrics.insert(
        "response_chars".to_string(),
        Value::from(result.response_text.len() as u64),
    );
    if let Some(reason) = result
        .raw
        .pointer("/choices/0/finish_reason")
        .and_then(|v| v.as_str())
    {
        metrics.insert("finish_reason".to_string(), Value::from(reason));
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

pub fn percentile(values: &[f64], p: f64) -> Option<f64> {
    crate::performance::metrics::percentile(values, p)
}

pub fn rate_per_second(units: f64, wall_time_ms: f64) -> Option<f64> {
    if wall_time_ms <= 0.0 {
        None
    } else {
        Some(units / (wall_time_ms / 1000.0))
    }
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
