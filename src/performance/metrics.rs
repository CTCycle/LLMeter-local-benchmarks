use serde::{Deserialize, Serialize};

use crate::providers::ProviderKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTiming {
    pub wall_time_ms: f64,
    pub ttft_ms: Option<f64>,
    pub tpot_ms: Option<f64>,
    pub itl_ms: Option<f64>,
    pub generation_wall_ms: Option<f64>,
    pub output_tokens_per_second_including_ttft: Option<f64>,
    pub output_tokens_per_second_excluding_ttft: Option<f64>,
    pub ttlt_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTiming {
    pub index: usize,
    pub since_start_ms: f64,
    pub delta_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTrace {
    pub request_id: String,
    pub model: String,
    pub provider: ProviderKind,
    pub prompt_id: String,
    pub estimated_prompt_tokens: u32,
    pub requested_output_tokens: u32,
    pub concurrency: u32,
    pub run_index: u32,
    pub stream: bool,
    pub success: bool,
    pub error: Option<String>,
    pub endpoint: String,
    pub http_status: Option<u16>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub token_timings: Vec<TokenTiming>,
    pub timing: RequestTiming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySummary {
    pub request_count: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub error_rate: f64,
    pub wall_time_ms_min: Option<f64>,
    pub wall_time_ms_mean: Option<f64>,
    pub wall_time_ms_max: Option<f64>,
    pub wall_time_ms_stddev: Option<f64>,
    pub wall_time_ms_p50: Option<f64>,
    pub wall_time_ms_p90: Option<f64>,
    pub wall_time_ms_p95: Option<f64>,
    pub wall_time_ms_p99: Option<f64>,
    pub ttft_ms_mean: Option<f64>,
    pub ttft_ms_min: Option<f64>,
    pub ttft_ms_max: Option<f64>,
    pub ttft_ms_p50: Option<f64>,
    pub ttft_ms_p95: Option<f64>,
    pub ttft_ms_p99: Option<f64>,
    pub tpot_ms_p50: Option<f64>,
    pub tpot_ms_p95: Option<f64>,
    pub itl_ms_p50: Option<f64>,
    pub itl_ms_p90: Option<f64>,
    pub itl_ms_p95: Option<f64>,
    pub itl_ms_p99: Option<f64>,
    pub generation_wall_ms_p50: Option<f64>,
    pub generation_wall_ms_p95: Option<f64>,
    pub generation_wall_ms_p99: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputSummary {
    pub output_tokens_per_second: Option<f64>,
    pub input_tokens_per_second: Option<f64>,
    pub requests_per_second: Option<f64>,
    pub successful_requests_per_second: Option<f64>,
    pub output_tokens_per_second_including_ttft: Option<f64>,
    pub output_tokens_per_second_excluding_ttft: Option<f64>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub mean_input_tokens_per_request: Option<f64>,
    pub mean_output_tokens_per_request: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub error_messages: Vec<String>,
    pub timeout_count: usize,
    pub http_error_count: usize,
    pub provider_error_count: usize,
    pub empty_response_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub latency: LatencySummary,
    pub throughput: ThroughputSummary,
    pub errors: ErrorSummary,
}

pub fn percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = (((p / 100.0) * sorted.len() as f64).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len().saturating_sub(1));
    sorted.get(rank).copied()
}

pub fn summarize_traces(traces: &[RequestTrace]) -> PerformanceSummary {
    let request_count = traces.len();
    let success_count = traces.iter().filter(|trace| trace.success).count();
    let error_count = request_count.saturating_sub(success_count);
    let wall: Vec<f64> = traces
        .iter()
        .map(|trace| trace.timing.wall_time_ms)
        .collect();
    let ttft: Vec<f64> = traces
        .iter()
        .filter_map(|trace| trace.timing.ttft_ms)
        .collect();
    let tpot: Vec<f64> = traces
        .iter()
        .filter_map(|trace| trace.timing.tpot_ms)
        .collect();
    let itl: Vec<f64> = traces
        .iter()
        .filter_map(|trace| trace.timing.itl_ms)
        .collect();
    let generation_wall: Vec<f64> = traces
        .iter()
        .filter_map(|trace| trace.timing.generation_wall_ms)
        .collect();
    let including_ttft_tps: Vec<f64> = traces
        .iter()
        .filter_map(|trace| trace.timing.output_tokens_per_second_including_ttft)
        .collect();
    let excluding_ttft_tps: Vec<f64> = traces
        .iter()
        .filter_map(|trace| trace.timing.output_tokens_per_second_excluding_ttft)
        .collect();
    let total_wall_ms = wall.iter().sum::<f64>();
    let total_input_tokens = traces
        .iter()
        .filter_map(|trace| trace.input_tokens)
        .sum::<u64>();
    let total_output_tokens = traces
        .iter()
        .filter_map(|trace| trace.output_tokens)
        .sum::<u64>();

    PerformanceSummary {
        latency: LatencySummary {
            request_count,
            success_count,
            error_count,
            error_rate: if request_count == 0 {
                0.0
            } else {
                error_count as f64 / request_count as f64
            },
            wall_time_ms_min: wall.iter().copied().reduce(f64::min),
            wall_time_ms_mean: mean(&wall),
            wall_time_ms_max: wall.iter().copied().reduce(f64::max),
            wall_time_ms_stddev: stddev(&wall),
            wall_time_ms_p50: percentile(&wall, 50.0),
            wall_time_ms_p90: percentile(&wall, 90.0),
            wall_time_ms_p95: percentile(&wall, 95.0),
            wall_time_ms_p99: percentile(&wall, 99.0),
            ttft_ms_mean: mean(&ttft),
            ttft_ms_min: ttft.iter().copied().reduce(f64::min),
            ttft_ms_max: ttft.iter().copied().reduce(f64::max),
            ttft_ms_p50: percentile(&ttft, 50.0),
            ttft_ms_p95: percentile(&ttft, 95.0),
            ttft_ms_p99: percentile(&ttft, 99.0),
            tpot_ms_p50: percentile(&tpot, 50.0),
            tpot_ms_p95: percentile(&tpot, 95.0),
            itl_ms_p50: percentile(&itl, 50.0),
            itl_ms_p90: percentile(&itl, 90.0),
            itl_ms_p95: percentile(&itl, 95.0),
            itl_ms_p99: percentile(&itl, 99.0),
            generation_wall_ms_p50: percentile(&generation_wall, 50.0),
            generation_wall_ms_p95: percentile(&generation_wall, 95.0),
            generation_wall_ms_p99: percentile(&generation_wall, 99.0),
        },
        throughput: ThroughputSummary {
            output_tokens_per_second: rate(total_output_tokens as f64, total_wall_ms),
            input_tokens_per_second: rate(total_input_tokens as f64, total_wall_ms),
            requests_per_second: rate(request_count as f64, total_wall_ms),
            successful_requests_per_second: rate(success_count as f64, total_wall_ms),
            output_tokens_per_second_including_ttft: mean(&including_ttft_tps),
            output_tokens_per_second_excluding_ttft: mean(&excluding_ttft_tps),
            total_input_tokens,
            total_output_tokens,
            mean_input_tokens_per_request: if request_count == 0 {
                None
            } else {
                Some(total_input_tokens as f64 / request_count as f64)
            },
            mean_output_tokens_per_request: if request_count == 0 {
                None
            } else {
                Some(total_output_tokens as f64 / request_count as f64)
            },
        },
        errors: ErrorSummary {
            error_messages: traces
                .iter()
                .filter_map(|trace| trace.error.clone())
                .collect(),
            timeout_count: count_errors(traces, "timed out"),
            http_error_count: traces
                .iter()
                .filter(|trace| {
                    trace
                        .http_status
                        .map(|status| status >= 400)
                        .unwrap_or(false)
                })
                .count(),
            provider_error_count: count_errors(traces, "provider"),
            empty_response_count: traces
                .iter()
                .filter(|trace| trace.success && trace.output_tokens == Some(0))
                .count(),
        },
    }
}

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn stddev(values: &[f64]) -> Option<f64> {
    let mean = mean(values)?;
    Some(
        (values
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / values.len() as f64)
            .sqrt(),
    )
}

fn count_errors(traces: &[RequestTrace], needle: &str) -> usize {
    traces
        .iter()
        .filter_map(|trace| trace.error.as_deref())
        .filter(|error| error.to_ascii_lowercase().contains(needle))
        .count()
}

fn rate(units: f64, wall_time_ms: f64) -> Option<f64> {
    if wall_time_ms <= 0.0 {
        None
    } else {
        Some(units / (wall_time_ms / 1000.0))
    }
}
