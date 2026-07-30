use serde::{Deserialize, Serialize};

use crate::providers::ProviderKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTiming {
    pub wall_time_ms: f64,
    pub ttft_ms: Option<f64>,
    pub itl_ms: Option<f64>,
    pub inter_chunk_latency_ms: Option<f64>,
    pub generation_wall_ms: Option<f64>,
    pub output_tokens_per_second_including_ttft: Option<f64>,
    pub output_tokens_per_second_excluding_ttft: Option<f64>,
    pub ttlt_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkTiming {
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
    pub chunk_timings: Vec<ChunkTiming>,
    pub timing: RequestTiming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySummary {
    pub request_count: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub error_rate: f64,
    pub successful_latency_sample_count: usize,
    pub percentile_estimator: String,
    pub standard_deviation_kind: String,
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
    pub itl_ms_p50: Option<f64>,
    pub itl_ms_p90: Option<f64>,
    pub itl_ms_p95: Option<f64>,
    pub itl_ms_p99: Option<f64>,
    pub inter_chunk_latency_ms_p50: Option<f64>,
    pub generation_wall_ms_p50: Option<f64>,
    pub generation_wall_ms_p95: Option<f64>,
    pub generation_wall_ms_p99: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputSummary {
    pub scenario_wall_time_ms: f64,
    pub output_tokens_per_second: Option<f64>,
    pub input_tokens_per_second: Option<f64>,
    pub requests_per_second: Option<f64>,
    pub successful_requests_per_second: Option<f64>,
    pub output_tokens_per_second_including_ttft: Option<f64>,
    pub output_tokens_per_second_excluding_ttft: Option<f64>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub input_token_sample_count: usize,
    pub output_token_sample_count: usize,
    pub input_token_coverage: f64,
    pub output_token_coverage: f64,
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
    if values.is_empty() || !p.is_finite() || !(0.0..=100.0).contains(&p) {
        return None;
    }
    let mut sorted = values
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect::<Vec<_>>();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(f64::total_cmp);
    let rank = (((p / 100.0) * sorted.len() as f64).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len().saturating_sub(1));
    sorted.get(rank).copied()
}

/// Calculates inter-token latency from provider-reported usage and streaming timing.
/// Chunk arrival timing is intentionally not used as a token metric.
pub fn inter_token_latency_ms(
    wall_time_ms: f64,
    ttft_ms: Option<f64>,
    output_tokens: Option<u64>,
    streaming: bool,
) -> Option<f64> {
    if !streaming || !wall_time_ms.is_finite() || wall_time_ms < 0.0 {
        return None;
    }
    let tokens = output_tokens?;
    if tokens < 2 {
        return None;
    }
    let ttft = ttft_ms?;
    if !ttft.is_finite() || ttft < 0.0 || ttft > wall_time_ms {
        return None;
    }
    Some((wall_time_ms - ttft) / (tokens - 1) as f64)
}

fn percentile_if_supported(values: &[f64], p: f64) -> Option<f64> {
    let minimum_samples = if p >= 99.0 {
        100
    } else if p >= 95.0 {
        20
    } else {
        1
    };
    (values.len() >= minimum_samples)
        .then(|| percentile(values, p))
        .flatten()
}

pub fn summarize_traces(traces: &[RequestTrace], scenario_wall_time_ms: f64) -> PerformanceSummary {
    let request_count = traces.len();
    let success_count = traces.iter().filter(|trace| trace.success).count();
    let error_count = request_count.saturating_sub(success_count);
    // Latency distributions describe completed requests only. Failed attempts remain
    // explicitly represented in ErrorSummary and the scenario-wide request rate.
    let successful: Vec<&RequestTrace> = traces.iter().filter(|trace| trace.success).collect();
    let wall: Vec<f64> = successful
        .iter()
        .map(|trace| trace.timing.wall_time_ms)
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect();
    let ttft: Vec<f64> = successful
        .iter()
        .filter_map(|trace| trace.timing.ttft_ms)
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect();
    let itl: Vec<f64> = successful
        .iter()
        .filter_map(|trace| trace.timing.itl_ms)
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect();
    let inter_chunk: Vec<f64> = successful
        .iter()
        .filter_map(|trace| trace.timing.inter_chunk_latency_ms)
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect();
    let generation_wall: Vec<f64> = successful
        .iter()
        .filter_map(|trace| trace.timing.generation_wall_ms)
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect();
    let including_ttft_tps: Vec<f64> = successful
        .iter()
        .filter_map(|trace| trace.timing.output_tokens_per_second_including_ttft)
        .collect();
    let excluding_ttft_tps: Vec<f64> = successful
        .iter()
        .filter_map(|trace| trace.timing.output_tokens_per_second_excluding_ttft)
        .collect();
    let total_input_tokens = successful
        .iter()
        .filter_map(|trace| trace.input_tokens)
        .sum::<u64>();
    let total_output_tokens = successful
        .iter()
        .filter_map(|trace| trace.output_tokens)
        .sum::<u64>();
    let input_token_sample_count = successful
        .iter()
        .filter(|trace| trace.input_tokens.is_some())
        .count();
    let output_token_sample_count = successful
        .iter()
        .filter(|trace| trace.output_tokens.is_some())
        .count();
    let input_token_coverage = coverage(input_token_sample_count, success_count);
    let output_token_coverage = coverage(output_token_sample_count, success_count);

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
            successful_latency_sample_count: wall.len(),
            percentile_estimator: "nearest-rank".to_string(),
            standard_deviation_kind: "population".to_string(),
            wall_time_ms_min: wall.iter().copied().reduce(f64::min),
            wall_time_ms_mean: mean(&wall),
            wall_time_ms_max: wall.iter().copied().reduce(f64::max),
            wall_time_ms_stddev: stddev(&wall),
            wall_time_ms_p50: percentile(&wall, 50.0),
            wall_time_ms_p90: percentile(&wall, 90.0),
            wall_time_ms_p95: percentile_if_supported(&wall, 95.0),
            wall_time_ms_p99: percentile_if_supported(&wall, 99.0),
            ttft_ms_mean: mean(&ttft),
            ttft_ms_min: ttft.iter().copied().reduce(f64::min),
            ttft_ms_max: ttft.iter().copied().reduce(f64::max),
            ttft_ms_p50: percentile(&ttft, 50.0),
            ttft_ms_p95: percentile_if_supported(&ttft, 95.0),
            ttft_ms_p99: percentile_if_supported(&ttft, 99.0),
            itl_ms_p50: percentile(&itl, 50.0),
            itl_ms_p90: percentile(&itl, 90.0),
            itl_ms_p95: percentile_if_supported(&itl, 95.0),
            itl_ms_p99: percentile_if_supported(&itl, 99.0),
            inter_chunk_latency_ms_p50: percentile(&inter_chunk, 50.0),
            generation_wall_ms_p50: percentile(&generation_wall, 50.0),
            generation_wall_ms_p95: percentile_if_supported(&generation_wall, 95.0),
            generation_wall_ms_p99: percentile_if_supported(&generation_wall, 99.0),
        },
        throughput: ThroughputSummary {
            scenario_wall_time_ms,
            output_tokens_per_second: (output_token_sample_count == success_count)
                .then(|| rate(total_output_tokens as f64, scenario_wall_time_ms))
                .flatten(),
            input_tokens_per_second: (input_token_sample_count == success_count)
                .then(|| rate(total_input_tokens as f64, scenario_wall_time_ms))
                .flatten(),
            requests_per_second: rate(request_count as f64, scenario_wall_time_ms),
            successful_requests_per_second: rate(success_count as f64, scenario_wall_time_ms),
            output_tokens_per_second_including_ttft: (output_token_sample_count == success_count)
                .then(|| mean(&including_ttft_tps))
                .flatten(),
            output_tokens_per_second_excluding_ttft: (output_token_sample_count == success_count)
                .then(|| mean(&excluding_ttft_tps))
                .flatten(),
            total_input_tokens,
            total_output_tokens,
            input_token_sample_count,
            output_token_sample_count,
            input_token_coverage,
            output_token_coverage,
            mean_input_tokens_per_request: if input_token_sample_count == success_count
                && success_count > 0
            {
                Some(total_input_tokens as f64 / input_token_sample_count as f64)
            } else {
                None
            },
            mean_output_tokens_per_request: if output_token_sample_count == success_count
                && success_count > 0
            {
                Some(total_output_tokens as f64 / output_token_sample_count as f64)
            } else {
                None
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
    let valid = values
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect::<Vec<_>>();
    if valid.is_empty() {
        None
    } else {
        Some(valid.iter().sum::<f64>() / valid.len() as f64)
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
    if !units.is_finite() || !wall_time_ms.is_finite() || wall_time_ms <= 0.0 {
        None
    } else {
        Some(units / (wall_time_ms / 1000.0))
    }
}

fn coverage(sample_count: usize, success_count: usize) -> f64 {
    if success_count == 0 {
        0.0
    } else {
        sample_count as f64 / success_count as f64
    }
}
