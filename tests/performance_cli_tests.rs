use clap::Parser;
use llmeter::cli::{BenchCommands, Cli, Commands};
use llmeter::performance::config::{PerformancePlan, PerformanceProfile};
use llmeter::providers::ProviderKind;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn bench_perf_cli_parses_sweep_inputs() {
    let cli = Cli::parse_from([
        "llmeter",
        "bench",
        "perf",
        "--models",
        "all",
        "--profile",
        "latency",
        "--concurrency",
        "1",
        "--warmup",
        "1",
        "--runs",
        "3",
    ]);

    match cli.command {
        Some(Commands::Bench {
            bench_command:
                BenchCommands::Perf {
                    models,
                    profile,
                    concurrency,
                    warmup,
                    runs,
                    ..
                },
        }) => {
            assert_eq!(models.as_deref(), Some("all"));
            assert_eq!(profile, PerformanceProfile::Latency);
            assert_eq!(concurrency.as_deref(), Some("1"));
            assert_eq!(warmup, Some(1));
            assert_eq!(runs, Some(3));
        }
        _ => panic!("expected bench perf command"),
    }
}

#[test]
fn performance_plan_normalizes_and_validates_csv_values() {
    let plan = PerformancePlan::from_cli(
        ProviderKind::Ollama,
        vec!["llama3.1".to_string()],
        PerformanceProfile::Sweep,
        Some("512,128,512"),
        Some("64,128"),
        Some("4,1,2"),
        Some(1),
        Some(3),
        true,
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(plan.prompt_sizes.estimated_tokens, vec![128, 512]);
    assert_eq!(plan.concurrency.levels, vec![1, 2, 4]);
}

#[test]
fn performance_plan_rejects_oversized_prompt_without_override() {
    let error = PerformancePlan::from_cli(
        ProviderKind::Ollama,
        vec!["llama3.1".to_string()],
        PerformanceProfile::Sweep,
        Some("65536"),
        Some("64"),
        Some("1"),
        Some(1),
        Some(1),
        true,
        None,
        HashMap::new(),
    )
    .unwrap_err();

    assert!(error.to_string().contains("unsafe_large_prompt=true"));

    let mut params = HashMap::new();
    params.insert("unsafe_large_prompt".to_string(), json!(true));
    PerformancePlan::from_cli(
        ProviderKind::Ollama,
        vec!["llama3.1".to_string()],
        PerformanceProfile::Sweep,
        Some("65536"),
        Some("64"),
        Some("1"),
        Some(1),
        Some(1),
        true,
        None,
        params,
    )
    .unwrap();
}
