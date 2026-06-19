use llmeter::performance::provider_probe::{endpoint_probe_from_result, parse_model_capabilities};
use serde_json::json;

#[test]
fn parse_model_capabilities_extracts_common_metadata() {
    let models = vec![json!({
        "id": "llama3.1",
        "owned_by": "local",
        "context_length": 8192,
        "max_output_tokens": 2048,
        "architecture": "llama",
        "quantization": "q4"
    })];

    let parsed = parse_model_capabilities(&models);

    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].id, "llama3.1");
    assert_eq!(parsed[0].owned_by.as_deref(), Some("local"));
    assert_eq!(parsed[0].context_length, Some(8192));
    assert_eq!(parsed[0].max_output_tokens, Some(2048));
    assert_eq!(parsed[0].architecture.as_deref(), Some("llama"));
    assert_eq!(parsed[0].quantization.as_deref(), Some("q4"));
}

#[test]
fn endpoint_probe_marks_errors_as_unsupported() {
    let probe = endpoint_probe_from_result(
        "Models",
        "GET",
        "/v1/models",
        12.5,
        None,
        Some("connection refused".to_string()),
    );

    assert!(!probe.supported);
    assert_eq!(probe.latency_ms, Some(12.5));
    assert_eq!(probe.error.as_deref(), Some("connection refused"));
}
