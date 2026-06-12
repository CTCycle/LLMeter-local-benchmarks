use llmeter::benchmarks::registry::default_registry;

#[test]
fn test_default_registry_contains_initial_benchmarks() {
    let registry = default_registry();
    let ids = registry.ids();
    assert_eq!(ids.len(), 7);
    assert_eq!(ids[0], "chat-generation");
    assert_eq!(ids[1], "responses-generation");
    assert_eq!(ids[2], "consistency");
    assert_eq!(ids[3], "prompt-sizes");
    assert_eq!(ids[4], "structured-output");
    assert_eq!(ids[5], "tool-calling");
    assert_eq!(ids[6], "embeddings");
}

#[test]
fn test_select_specific_benchmark() {
    let registry = default_registry();
    let selected = registry
        .select(Some(&["consistency".to_string()]), false)
        .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id(), "consistency");
}

#[test]
fn test_select_all_benchmarks() {
    let registry = default_registry();
    let all = registry.select(None, true).unwrap();
    assert_eq!(all.len(), 7);
}

#[test]
fn test_select_nonexistent_benchmark_returns_error() {
    let registry = default_registry();
    let result = registry.select(Some(&["nonexistent".to_string()]), false);
    assert!(result.is_err());
}

#[test]
fn test_get_benchmark_by_id() {
    let registry = default_registry();
    let bench = registry.get("chat-generation");
    assert!(bench.is_some());
    assert_eq!(bench.unwrap().name(), "Basic generation latency");
}

#[test]
fn test_get_nonexistent_benchmark() {
    let registry = default_registry();
    assert!(registry.get("nope").is_none());
}
