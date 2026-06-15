use llmeter::benchmarks::base::BenchmarkContext;
use llmeter::benchmarks::registry::{default_registry, BenchmarkSuite};

fn context(runs: u32) -> BenchmarkContext {
    BenchmarkContext {
        runs,
        max_tokens: 128,
        temperature: 0.5,
        timeout: 30.0,
        options: std::collections::HashMap::new(),
    }
}

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
    assert_eq!(registry.ids_for_suite(BenchmarkSuite::Llm).len(), 6);
    assert_eq!(
        registry.ids_for_suite(BenchmarkSuite::Embeddings),
        vec!["embeddings"]
    );
}

#[test]
fn test_select_specific_benchmark() {
    let registry = default_registry();
    let selected = registry
        .select(
            Some(&["consistency".to_string()]),
            false,
            BenchmarkSuite::Llm,
        )
        .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id(), "consistency");
}

#[test]
fn test_select_all_benchmarks() {
    let registry = default_registry();
    let all = registry.select(None, true, BenchmarkSuite::Llm).unwrap();
    assert_eq!(all.len(), 6);
}

#[test]
fn test_select_nonexistent_benchmark_returns_error() {
    let registry = default_registry();
    let result = registry.select(
        Some(&["nonexistent".to_string()]),
        false,
        BenchmarkSuite::Llm,
    );
    assert!(result.is_err());
}

#[test]
fn test_selecting_benchmark_from_other_suite_returns_error() {
    let registry = default_registry();
    let result = registry.select(
        Some(&["embeddings".to_string()]),
        false,
        BenchmarkSuite::Llm,
    );
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

#[test]
fn test_prompt_sizes_planned_steps_match_prompt_count_times_runs() {
    let registry = default_registry();
    let benchmark = registry.get("prompt-sizes").unwrap();
    assert_eq!(benchmark.planned_steps(&context(2)), 6);
}

#[test]
fn test_consistency_planned_steps_has_minimum_of_two() {
    let registry = default_registry();
    let benchmark = registry.get("consistency").unwrap();
    assert_eq!(benchmark.planned_steps(&context(1)), 2);
    assert_eq!(benchmark.planned_steps(&context(4)), 4);
}
