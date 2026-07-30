use llmeter::benchmarks::metrics::pairwise_similarity;
use llmeter::providers::ApiResult;
use serde_json::json;

fn make_result(output_tokens: u64, wall_time_ns: u128) -> ApiResult {
    ApiResult {
        endpoint: "/v1/chat/completions".to_string(),
        response_text: "hello world".to_string(),
        raw: json!({
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": output_tokens,
                "total_tokens": output_tokens + 10
            },
            "choices": [{"finish_reason": "stop"}]
        }),
        wall_time_ns,
        time_to_first_token_ns: Some(300_000_000),
        chunk_timings_ns: Vec::new(),
        http_status: Some(200),
    }
}

#[test]
fn test_tokens_per_second() {
    let result = make_result(100, 1_000_000_000);
    let tps = result.tokens_per_second().unwrap();
    assert!((tps - 100.0).abs() < 0.1, "Expected ~100 tps, got {tps}");
}

#[test]
fn test_tokens_per_second_zero_duration_returns_none() {
    let result = make_result(10, 0);
    assert!(result.tokens_per_second().is_none());
}

#[test]
fn test_pairwise_similarity_identical_texts() {
    let texts = vec![
        "hello".to_string(),
        "hello".to_string(),
        "hello".to_string(),
    ];
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
