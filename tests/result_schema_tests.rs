// Copyright © 2026 CTCycle
// Licensed under the MIT License.

use llmeter::results::{BenchmarkRun, ResultStore, RESULT_SCHEMA_VERSION};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn unversioned_legacy_result_is_rejected() {
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

    assert!(serde_json::from_value::<BenchmarkRun>(legacy).is_err());
}

#[test]
fn unsupported_explicit_schema_is_rejected_by_result_store() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("old.json");
    let old = json!({
        "run_id": "old-run",
        "created_at": "2026-06-15T12:00:00Z",
        "models": ["llama3.1"],
        "benchmark_ids": ["chat-generation"],
        "config": {"runs": 1, "max_tokens": 128, "temperature": 0.0, "timeout": 30.0},
        "results": [],
        "schema_version": "2.4",
        "run_kind": "benchmark"
    });
    std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();

    let error = ResultStore::new(temp.path()).load_json(&path).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("Unsupported result schema"), "{message}");
    assert!(message.contains("2.4"), "{message}");
    assert!(message.contains(RESULT_SCHEMA_VERSION), "{message}");
}
