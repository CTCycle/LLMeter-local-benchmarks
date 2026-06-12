use llmeter::benchmarks::metrics::{generation_metrics, pairwise_similarity, preview};
use llmeter::ollama::client::GenerateResult;
use serde_json::json;

fn make_result(eval_count: u64, eval_duration_ns: u64) -> GenerateResult {
    GenerateResult {
        model: "test-model".to_string(),
        prompt: "test prompt".to_string(),
        response: "hello world".to_string(),
        raw: json!({
            "eval_count": eval_count,
            "eval_duration": eval_duration_ns,
            "total_duration": 1_000_000_000,
            "load_duration": 100_000_000,
            "prompt_eval_count": 10,
            "prompt_eval_duration": 200_000_000,
            "done_reason": "stop"
        }),
        wall_time_ns: 1_500_000_000,
        time_to_first_token_ns: Some(300_000_000),
    }
}

#[test]
fn test_tokens_per_second() {
    let result = make_result(100, 1_000_000_000);
    let tps = result.tokens_per_second().unwrap();
    assert!((tps - 100.0).abs() < 0.1, "Expected ~100 tps, got {tps}");
}

#[test]
fn test_generation_metrics_contains_all_keys() {
    let result = make_result(50, 500_000_000);
    let metrics = generation_metrics(&result);
    assert!(metrics.contains_key("wall_time_ms"));
    assert!(metrics.contains_key("time_to_first_token_ms"));
    assert!(metrics.contains_key("api_total_duration_ms"));
    assert!(metrics.contains_key("tokens_per_second"));
    assert!(metrics.contains_key("eval_count"));
    assert!(metrics.contains_key("done_reason"));
}

#[test]
fn test_tokens_per_second_zero_duration_returns_none() {
    let result = make_result(10, 0);
    assert!(result.tokens_per_second().is_none());
}

#[test]
fn test_pairwise_similarity_identical_texts() {
    let texts = vec!["hello".to_string(), "hello".to_string(), "hello".to_string()];
    let scores = pairwise_similarity(&texts);
    assert_eq!(scores.len(), 3);
    for score in &scores {
        assert!((*score - 1.0).abs() < 0.01, "Expected ~1.0, got {score}");
    }
}

#[test]
fn test_pairwise_similarity_different_texts() {
    let texts = vec!["abc".to_string(), "xyz".to_string()];
    let scores = pairwise_similarity(&texts);
    assert_eq!(scores.len(), 1);
    assert!(scores[0] < 1.0, "Different texts should have ratio < 1.0");
}

#[test]
fn test_preview_short_text() {
    let text = "hello world";
    assert_eq!(preview(text, 180), "hello world");
}

#[test]
fn test_preview_long_text() {
    let text = "a".repeat(200);
    let result = preview(&text, 180);
    assert_eq!(result.chars().count(), 180);
    assert!(result.ends_with('…'));
}
