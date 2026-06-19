use llmeter::config::AppConfig;
use llmeter::performance::resource::capture_environment_snapshot;
use llmeter::providers::ProviderKind;

fn test_config() -> AppConfig {
    AppConfig {
        provider: ProviderKind::Ollama,
        base_url: "http://localhost:11434/v1".to_string(),
        explicit_base_url: false,
        timeout: 1.0,
        output_dir: std::env::temp_dir(),
        default_runs: 1,
        default_max_tokens: 16,
        default_temperature: 0.0,
    }
}

#[test]
fn resource_snapshot_reports_memory_swap_and_disk_ratios() {
    let snapshot = capture_environment_snapshot(&test_config(), Some(1), Some(2), None);

    assert!(snapshot.cpu_count > 0);
    assert!(snapshot.total_memory >= snapshot.available_memory);
    assert!((0.0..=1.0).contains(&snapshot.memory_used_ratio));
    assert!((0.0..=1.0).contains(&snapshot.swap_used_ratio));
    assert_eq!(snapshot.process_memory_before, Some(1));
    assert_eq!(snapshot.process_memory_after, Some(2));
}
