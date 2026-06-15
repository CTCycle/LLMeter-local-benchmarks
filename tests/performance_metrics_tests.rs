use llmeter::performance::metrics::{percentile, summarize_traces, RequestTiming, RequestTrace};
use llmeter::providers::ProviderKind;

fn trace(
    run_index: u32,
    wall_time_ms: f64,
    ttft_ms: Option<f64>,
    output_tokens: Option<u64>,
) -> RequestTrace {
    RequestTrace {
        request_id: format!("req-{run_index}"),
        model: "llama3.1".to_string(),
        provider: ProviderKind::Ollama,
        prompt_id: "synthetic-128".to_string(),
        estimated_prompt_tokens: 128,
        requested_output_tokens: 64,
        concurrency: 1,
        run_index,
        stream: true,
        success: true,
        error: None,
        endpoint: "/v1/chat/completions".to_string(),
        http_status: Some(200),
        input_tokens: Some(128),
        output_tokens,
        token_timings: Vec::new(),
        timing: RequestTiming {
            wall_time_ms,
            ttft_ms,
            tpot_ms: Some(12.0),
            itl_ms: Some(9.0),
        },
    }
}

#[test]
fn percentile_sorts_and_picks_requested_rank() {
    let values = vec![90.0, 10.0, 70.0, 50.0];
    assert_eq!(percentile(&values, 50.0), Some(50.0));
    assert_eq!(percentile(&values, 95.0), Some(90.0));
}

#[test]
fn summarize_traces_computes_rates_and_counts() {
    let summary = summarize_traces(&[
        trace(1, 100.0, Some(20.0), Some(40)),
        trace(2, 200.0, Some(30.0), Some(60)),
    ]);

    assert_eq!(summary.latency.request_count, 2);
    assert_eq!(summary.latency.success_count, 2);
    assert_eq!(summary.latency.error_count, 0);
    assert_eq!(summary.throughput.total_input_tokens, 256);
    assert_eq!(summary.throughput.total_output_tokens, 100);
    assert!(summary.throughput.requests_per_second.unwrap() > 6.0);
    assert_eq!(summary.latency.wall_time_ms_p50, Some(100.0));
}
