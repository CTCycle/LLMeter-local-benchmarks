use llmeter::performance::config::{
    LoadMeasurementMode, PerformancePlan, PerformanceProfile, PerformanceSafetyOptions,
    ReportDetailLevel, TelemetryLevel, DEFAULT_MAX_PERFORMANCE_REQUESTS,
};
use llmeter::providers::ProviderKind;
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
        LoadMeasurementMode::FirstRequestEstimate,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
        None,
    )
    .unwrap();

    assert_eq!(plan.prompt_sizes.estimated_tokens, vec![128, 512]);
    assert_eq!(plan.concurrency.levels, vec![1, 2, 4]);
}

#[test]
fn performance_plan_rejects_oversized_prompt_without_explicit_safety_override() {
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
        LoadMeasurementMode::FirstRequestEstimate,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
        None,
    )
    .unwrap_err();

    assert!(error.to_string().contains("--allow-large-prompt"));

    PerformancePlan::from_cli_with_safety(
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
        PerformanceSafetyOptions {
            allow_large_prompt: true,
            allow_large_matrix: false,
        },
        LoadMeasurementMode::FirstRequestEstimate,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
        None,
    )
    .unwrap();
}

#[test]
fn legacy_unsafe_parameter_does_not_bypass_prompt_safety() {
    let mut params = HashMap::new();
    params.insert("unsafe_large_prompt".to_string(), true.into());
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
        params,
        LoadMeasurementMode::FirstRequestEstimate,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
        None,
    )
    .unwrap_err();
    assert!(error.to_string().contains("--allow-large-prompt"));
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
        LoadMeasurementMode::FirstRequestEstimate,
        2,
        TelemetryLevel::Detailed,
        50,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
        None,
    )
    .unwrap_err();

    assert!(error.to_string().contains("at least 100 ms"));
}

#[test]
fn performance_plan_estimates_request_matrix() {
    let plan = PerformancePlan::from_cli(
        ProviderKind::Ollama,
        vec!["model-a".to_string(), "model-b".to_string()],
        PerformanceProfile::Sweep,
        Some("128,512"),
        Some("64,128"),
        Some("1,2"),
        Some(1),
        Some(3),
        true,
        None,
        HashMap::new(),
        LoadMeasurementMode::FirstRequestEstimate,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
        Some(DEFAULT_MAX_PERFORMANCE_REQUESTS),
    )
    .unwrap();

    assert_eq!(plan.scenario_count(), 16);
    assert_eq!(plan.total_warmup_requests(), 16);
    assert_eq!(plan.total_measured_requests(), 48);
    assert_eq!(plan.total_requests(), 64);
}

#[test]
fn performance_plan_rejects_large_request_matrix_without_explicit_safety_override() {
    let error = PerformancePlan::from_cli(
        ProviderKind::Ollama,
        vec!["model-a".to_string(), "model-b".to_string()],
        PerformanceProfile::Sweep,
        Some("128,512"),
        Some("64,128"),
        Some("1,2"),
        Some(1),
        Some(3),
        true,
        None,
        HashMap::new(),
        LoadMeasurementMode::FirstRequestEstimate,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
        Some(10),
    )
    .unwrap_err();

    assert!(error.to_string().contains("--max-requests 10"));

    PerformancePlan::from_cli_with_safety(
        ProviderKind::Ollama,
        vec!["model-a".to_string(), "model-b".to_string()],
        PerformanceProfile::Sweep,
        Some("128,512"),
        Some("64,128"),
        Some("1,2"),
        Some(1),
        Some(3),
        true,
        None,
        HashMap::new(),
        PerformanceSafetyOptions {
            allow_large_prompt: false,
            allow_large_matrix: true,
        },
        LoadMeasurementMode::FirstRequestEstimate,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        None,
        false,
        ReportDetailLevel::Detailed,
        Some(10),
    )
    .unwrap();
}
