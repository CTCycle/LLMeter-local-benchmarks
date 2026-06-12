use llmeter::benchmarks::base::BenchmarkResultRecord;
use llmeter::reporting::{
    build_summary_rows, render_html_report, render_markdown_report,
};
use llmeter::results::BenchmarkRun;
use serde_json::json;

fn sample_run() -> BenchmarkRun {
    BenchmarkRun {
        run_id: "test-run-1".to_string(),
        created_at: "2026-06-12T12:00:00".to_string(),
        models: vec!["llama3".to_string()],
        benchmark_ids: vec!["generation-latency".to_string()],
        config: std::collections::HashMap::new(),
        results: vec![
            BenchmarkResultRecord {
                benchmark_id: "generation-latency".to_string(),
                benchmark_name: "Basic generation latency".to_string(),
                model: "llama3".to_string(),
                run_index: Some(1),
                prompt_name: Some("short".to_string()),
                metrics: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("wall_time_ms".to_string(), json!(150.0));
                    m.insert("tokens_per_second".to_string(), json!(45.2));
                    m.insert("api_total_duration_ms".to_string(), json!(1200.0));
                    m
                },
                response_preview: Some("a short response".to_string()),
                error: None,
                metadata: None,
            },
            BenchmarkResultRecord {
                benchmark_id: "generation-latency".to_string(),
                benchmark_name: "Basic generation latency".to_string(),
                model: "llama3".to_string(),
                run_index: Some(2),
                prompt_name: Some("short".to_string()),
                metrics: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("wall_time_ms".to_string(), json!(160.0));
                    m.insert("tokens_per_second".to_string(), json!(42.1));
                    m
                },
                response_preview: Some("another response".to_string()),
                error: None,
                metadata: None,
            },
        ],
    }
}

#[test]
fn test_markdown_report_contains_summary() {
    let run = sample_run();
    let report = render_markdown_report(&run);
    assert!(report.contains("# LLMeter Report"));
    assert!(report.contains("test-run-1"));
    assert!(report.contains("generation-latency"));
    assert!(report.contains("llama3"));
    assert!(report.contains("tokens_per_second") || report.contains("tok/s"));
}

#[test]
fn test_html_report_contains_expected_sections() {
    let run = sample_run();
    let report = render_html_report(&run);
    assert!(report.contains("<!doctype html>"));
    assert!(report.contains("LLMeter Report"));
    assert!(report.contains("test-run-1"));
    assert!(report.contains("light dark") || report.contains("dark"));
    assert!(report.contains("Interpretation notes"));
}

#[test]
fn test_build_summary_rows() {
    let run = sample_run();
    let rows = build_summary_rows(&run);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.model, "llama3");
    assert_eq!(row.records, 2);
    assert_eq!(row.errors, 0);
    assert!(row.avg_wall_time_ms.is_some());
    let avg = row.avg_wall_time_ms.unwrap();
    assert!((avg - 155.0).abs() < 1.0, "Expected ~155.0, got {avg}");
}

#[test]
fn test_error_records_section_appears_in_markdown() {
    let mut run = sample_run();
    run.results.push(BenchmarkResultRecord {
        benchmark_id: "generation-latency".to_string(),
        benchmark_name: "Basic generation latency".to_string(),
        model: "llama3".to_string(),
        run_index: Some(3),
        prompt_name: Some("short".to_string()),
        metrics: std::collections::HashMap::new(),
        response_preview: None,
        error: Some("connection refused".to_string()),
        metadata: None,
    });

    let report = render_markdown_report(&run);
    assert!(report.contains("## Errors"));
    assert!(report.contains("connection refused"));
    assert!(report.contains("1"));
}
