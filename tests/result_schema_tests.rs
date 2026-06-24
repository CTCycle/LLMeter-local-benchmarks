use llmeter::results::BenchmarkRun;
use serde_json::json;

#[test]
fn old_result_shape_deserializes_with_optional_new_fields() {
    let legacy = json!({
        "run_id": "legacy-run",
        "created_at": "2026-06-15T12:00:00Z",
        "models": ["llama3.1"],
        "benchmark_ids": ["chat-generation"],
        "config": {"runs": 1},
        "results": [{
            "benchmark_id": "chat-generation",
            "benchmark_name": "Basic generation latency",
            "model": "llama3.1",
            "run_index": 1,
            "prompt_name": "short",
            "metrics": {"wall_time_ms": 42.0}
        }]
    });

    let run: BenchmarkRun = serde_json::from_value(legacy).unwrap();
    assert_eq!(run.schema_version, "2.1");
    assert!(run.environment.is_none());
    assert!(run.performance_plan.is_none());
    assert!(run.quality_plan.is_none());
    assert!(run.provider_capabilities.is_none());
    assert!(run.model_load_measurements.is_none());
    assert!(run.telemetry_summary.is_none());
}
