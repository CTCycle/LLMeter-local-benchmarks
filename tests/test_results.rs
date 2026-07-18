use llmeter::benchmarks::base::BenchmarkResultRecord;
use llmeter::results::{
    prepare_run_for_output, BenchmarkRun, BenchmarkRunKind, OutputPrivacyPolicy, ResultStore,
};
use serde_json::json;

fn sample_run() -> BenchmarkRun {
    BenchmarkRun {
        run_id: "test-run-id".to_string(),
        created_at: "2026-06-12T12:00:00".to_string(),
        models: vec!["llama3".to_string()],
        benchmark_ids: vec!["chat-generation".to_string()],
        config: {
            let mut c = std::collections::HashMap::new();
            c.insert("runs".to_string(), json!(3));
            c
        },
        results: vec![BenchmarkResultRecord {
            benchmark_id: "chat-generation".to_string(),
            benchmark_name: "Basic generation latency".to_string(),
            model: "llama3".to_string(),
            run_index: Some(1),
            prompt_name: Some("short".to_string()),
            metrics: {
                let mut m = std::collections::HashMap::new();
                m.insert("wall_time_ms".to_string(), json!(150.0));
                m.insert("tokens_per_second".to_string(), json!(45.2));
                m
            },
            response_preview: Some("hello".to_string()),
            error: None,
            metadata: None,
        }],
        schema_version: "2.0".to_string(),
        run_kind: Some(BenchmarkRunKind::Benchmark),
        environment: None,
        performance_plan: None,
        quality_plan: None,
        provider_capabilities: None,
        model_load_measurements: None,
        model_inventory_measurements: None,
        telemetry_summary: None,
    }
}

#[test]
fn test_result_store_saves_json_and_csv() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResultStore::new(dir.path());
    let run = sample_run();

    let json_path = store.save_json(&run).unwrap();
    assert!(json_path.exists());
    let content = std::fs::read_to_string(&json_path).unwrap();
    assert!(content.contains("test-run-id"));
    assert!(content.contains("chat-generation"));

    let csv_path = store.save_csv(&run).unwrap();
    assert!(csv_path.exists());
    let csv_content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(csv_content.contains("run_id"));
    assert!(csv_content.contains("test-run-id"));
    assert!(csv_content.contains("45.2"));
}

#[test]
fn test_csv_neutralizes_formula_like_text_fields() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResultStore::new(dir.path());
    let mut run = sample_run();
    run.models = vec!["=MODEL()".to_string()];
    run.results[0].benchmark_name = "+BENCHMARK()".to_string();
    run.results[0].model = "-MODEL()".to_string();
    run.results[0].prompt_name = Some("@PROMPT()".to_string());
    run.results[0].response_preview = Some("=PREVIEW()".to_string());
    run.results[0]
        .metrics
        .insert("text_metric".to_string(), json!("+METRIC()"));

    let csv_path = store.save_csv(&run).unwrap();
    let mut reader = csv::Reader::from_path(csv_path).unwrap();
    let headers = reader.headers().unwrap().clone();
    let record = reader.records().next().unwrap().unwrap();

    for (column, expected) in [
        ("benchmark_name", "'+BENCHMARK()"),
        ("model", "'-MODEL()"),
        ("prompt_name", "'@PROMPT()"),
        ("response_preview", "'=PREVIEW()"),
        ("text_metric", "'+METRIC()"),
    ] {
        let index = headers.iter().position(|header| header == column).unwrap();
        assert_eq!(&record[index], expected, "column {column}");
    }
}

#[test]
fn test_load_json_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResultStore::new(dir.path());
    let run = sample_run();

    store.save_json(&run).unwrap();
    let json_path = dir.path().join("test-run-id.json");
    let loaded = store.load_json(&json_path).unwrap();

    assert_eq!(loaded.run_id, "test-run-id");
    assert_eq!(loaded.models, vec!["llama3"]);
    assert_eq!(loaded.results.len(), 1);
    assert_eq!(loaded.results[0].benchmark_id, "chat-generation");
}

#[test]
fn output_privacy_omits_previews_and_redacts_secrets_by_default() {
    let mut run = sample_run();
    run.config
        .insert("api_key".to_string(), json!("super-secret"));
    run.results[0].error = Some("Bearer abc123 token=query-secret".to_string());

    let prepared = prepare_run_for_output(&run, OutputPrivacyPolicy::default());

    assert_eq!(run.results[0].response_preview.as_deref(), Some("hello"));
    assert!(prepared.results[0].response_preview.is_none());
    assert_eq!(prepared.config["api_key"], json!("[redacted]"));
    let error = prepared.results[0].error.as_deref().unwrap();
    assert!(!error.contains("abc123"));
    assert!(!error.contains("query-secret"));
    assert_eq!(prepared.config["response_previews_included"], json!(false));
    assert_eq!(prepared.config["sensitive_values_redacted"], json!(true));
}

#[test]
fn response_previews_require_explicit_opt_in() {
    let run = sample_run();
    let prepared = prepare_run_for_output(
        &run,
        OutputPrivacyPolicy {
            include_response_preview: true,
            redact_sensitive_values: true,
        },
    );
    assert_eq!(
        prepared.results[0].response_preview.as_deref(),
        Some("hello")
    );
}
