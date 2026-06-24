use llmeter::performance::config::{
    LoadMeasurementMode, PerformancePlan, PerformanceProfile, ReportDetailLevel, TelemetryLevel,
};
use llmeter::providers::ProviderKind;
use serde_json::json;
use std::collections::HashMap;

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
        LoadMeasurementMode::WarmBaseline,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
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
        LoadMeasurementMode::WarmBaseline,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
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
        LoadMeasurementMode::WarmBaseline,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
    )
    .unwrap();
}

#[test]
fn performance_plan_rejects_too_fast_telemetry_sampling() {
    let error = PerformancePlan::from_cli(
        ProviderKind::Ollama,
        vec!["llama3.1".to_string()],
        PerformanceProfile::Smoke,
        None,
        None,
        None,
        Some(1),
        Some(1),
        false,
        None,
        HashMap::new(),
        LoadMeasurementMode::WarmBaseline,
        2,
        TelemetryLevel::Full,
        50,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
    )
    .unwrap_err();

    assert!(error.to_string().contains("at least 100 ms"));
}
