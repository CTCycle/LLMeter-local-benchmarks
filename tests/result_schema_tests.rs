use llmeter::benchmarks::base::BenchmarkResultRecord;
use llmeter::results::BenchmarkRun;
use serde_json::json;
use std::collections::HashMap;

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

#[test]
fn new_result_shape_serializes_schema_metadata() {
    let run = BenchmarkRun {
        run_id: "schema-run".to_string(),
        created_at: "2026-06-15T12:00:00Z".to_string(),
        models: vec!["llama3.1".to_string()],
        benchmark_ids: vec!["chat-generation".to_string()],
        config: HashMap::new(),
        results: vec![BenchmarkResultRecord {
            benchmark_id: "chat-generation".to_string(),
            benchmark_name: "Basic generation latency".to_string(),
            model: "llama3.1".to_string(),
            run_index: Some(1),
            prompt_name: Some("short".to_string()),
            metrics: HashMap::new(),
            response_preview: None,
            error: None,
            metadata: None,
        }],
        schema_version: "2.1".to_string(),
        run_kind: None,
        environment: None,
        performance_plan: None,
        quality_plan: None,
        provider_capabilities: None,
        model_load_measurements: None,
        model_inventory_measurements: None,
        telemetry_summary: None,
    };

    let value = serde_json::to_value(&run).unwrap();
    assert_eq!(value["schema_version"], "2.1");
}
