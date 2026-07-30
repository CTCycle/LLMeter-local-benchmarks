use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use futures_util::stream::{FuturesUnordered, StreamExt};
use serde_json::{json, Value};

use crate::benchmarks::base::BenchmarkResultRecord;
use crate::config::AppConfig;
use crate::errors::LLMeterError;
use crate::performance::config::{PerformancePlan, ReportDetailLevel, TelemetryLevel};
use crate::performance::load::{measure_model_load_with_progress, planned_load_steps};
use crate::performance::metrics::{
    inter_token_latency_ms, summarize_traces, ChunkTiming, PerformanceSummary, RequestTiming,
    RequestTrace,
};
use crate::performance::model_inventory::{
    measure_model_inventory_with_progress, planned_inventory_steps,
};
use crate::performance::provider_probe::{
    planned_probe_steps, probe_provider_capabilities_with_progress,
};
use crate::performance::resource::{
    capture_environment_snapshot_with_progress, ENVIRONMENT_SNAPSHOT_STEPS,
};
use crate::performance::telemetry::{summarize_samples, TelemetrySampler};
use crate::performance::workload::{
    load_jsonl_workload, synthetic_prompt_for_tokens, PerformancePrompt,
};
use crate::progress::{ProgressEventKind, ProgressPhase, ProgressSink, ProgressUpdate};
use crate::providers::ProviderClient;
use crate::results::{BenchmarkRun, BenchmarkRunKind, ResultStore, RESULT_SCHEMA_VERSION};
use crate::utils::{error_chain, ns_to_ms, utc_now_iso};

pub fn run_performance_plan(
    config: &AppConfig,
    client: &ProviderClient,
    plan: PerformancePlan,
    progress: Option<&mut dyn ProgressSink>,
) -> anyhow::Result<BenchmarkRun> {
    let mut null_sink = crate::progress::NullProgressSink;
    let sink = match progress {
        Some(sink) => sink,
        None => &mut null_sink,
    };

    if plan.models.is_empty() {
        return Err(LLMeterError::InvalidOption(
            "At least one model is required for performance benchmarking.".to_string(),
        )
        .into());
    }

    let probe_units = if plan.probe_capabilities || plan.probe_all_endpoints {
        planned_probe_steps(&plan)
    } else {
        0
    };
    let load_units = planned_load_steps(&plan, plan.models.len());
    let inventory_units = planned_inventory_steps(&plan, plan.models.len());
    let total_units = planned_units(&plan)
        + probe_units
        + load_units
        + inventory_units
        + ENVIRONMENT_SNAPSHOT_STEPS;
    sink.on_update(ProgressUpdate {
        kind: ProgressEventKind::Phase,
        phase: ProgressPhase::Planning,
        message: format!(
            "Prepared performance profile {} across {} model(s)",
            plan.profile.label(),
            plan.models.len()
        ),
        completed_units: 0,
        total_units,
        model_name: None,
        model_index: None,
        total_models: Some(plan.models.len()),
        benchmark_id: None,
        benchmark_name: None,
        benchmark_index: None,
        total_benchmarks: None,
        step_index: None,
        total_steps: Some(total_units),
        run_index: None,
        prompt_name: None,
    });

    let prompts = build_prompts(&plan)?;
    let available = crate::runner::validate_models(client, &plan.models)?;
    let store = ResultStore::new(&config.output_dir);
    let run_id = store.new_run_id(&available);
    let mut results = Vec::new();
    let mut completed_units = 0u32;
    let process_memory_before = current_process_memory();
    let model_load_measurements = measure_model_load_with_progress(
        client,
        &available,
        &plan,
        sink,
        completed_units,
        total_units,
    );
    completed_units += load_units;
    let provider_capabilities = if plan.probe_capabilities || plan.probe_all_endpoints {
        let report = probe_provider_capabilities_with_progress(
            client,
            &plan,
            sink,
            completed_units,
            total_units,
        );
        completed_units += probe_units;
        Some(report)
    } else {
        None
    };
    let model_inventory_measurements = measure_model_inventory_with_progress(
        client,
        &available,
        &plan,
        sink,
        completed_units,
        total_units,
    );
    completed_units += inventory_units;
    let mut telemetry_samples = Vec::new();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    for (model_index, model) in available.iter().enumerate() {
        for prompt in &prompts {
            for &output_tokens in &plan.output_sizes.estimated_tokens {
                for &concurrency in &plan.concurrency.levels {
                    let scenario_name = format!(
                        "{}p-{}o-c{}",
                        prompt.estimated_prompt_tokens, output_tokens, concurrency
                    );
                    sink.on_update(ProgressUpdate {
                        kind: ProgressEventKind::StepStarted,
                        phase: ProgressPhase::Running,
                        message: format!("Running scenario {scenario_name}"),
                        completed_units,
                        total_units,
                        model_name: Some(model.clone()),
                        model_index: Some(model_index + 1),
                        total_models: Some(available.len()),
                        benchmark_id: Some("performance-scenario".to_string()),
                        benchmark_name: Some("Performance scenario".to_string()),
                        benchmark_index: Some(1),
                        total_benchmarks: Some(1),
                        step_index: Some(completed_units + 1),
                        total_steps: Some(total_units),
                        run_index: None,
                        prompt_name: Some(prompt.id.clone()),
                    });

                    run_warmup_requests(client, model, prompt, output_tokens, &plan)?;
                    let sampler = match plan.telemetry {
                        TelemetryLevel::Off => None,
                        TelemetryLevel::Standard | TelemetryLevel::Detailed => {
                            Some(TelemetrySampler::start(plan.sample_interval_ms))
                        }
                    };
                    let scenario_started = Instant::now();
                    let traces = run_measured_requests(
                        &runtime,
                        client,
                        model,
                        prompt,
                        output_tokens,
                        concurrency,
                        &plan,
                    )?;
                    let scenario_wall_time_ms = scenario_started.elapsed().as_secs_f64() * 1000.0;
                    if let Some(sampler) = sampler {
                        telemetry_samples.extend(sampler.stop());
                    }
                    let summary = summarize_traces(&traces, scenario_wall_time_ms);
                    results.push(summary_record(
                        model,
                        &scenario_name,
                        prompt,
                        output_tokens,
                        concurrency,
                        &summary,
                        &traces,
                        plan.detail,
                    ));

                    completed_units += 1;
                    sink.on_update(ProgressUpdate {
                        kind: ProgressEventKind::StepCompleted,
                        phase: ProgressPhase::Running,
                        message: format!("Completed scenario {scenario_name}"),
                        completed_units,
                        total_units,
                        model_name: Some(model.clone()),
                        model_index: Some(model_index + 1),
                        total_models: Some(available.len()),
                        benchmark_id: Some("performance-scenario".to_string()),
                        benchmark_name: Some("Performance scenario".to_string()),
                        benchmark_index: Some(1),
                        total_benchmarks: Some(1),
                        step_index: Some(completed_units),
                        total_steps: Some(total_units),
                        run_index: None,
                        prompt_name: Some(prompt.id.clone()),
                    });
                }
            }
        }
    }

    let environment = capture_environment_snapshot_with_progress(
        config,
        process_memory_before,
        current_process_memory(),
        plan.provider_process.as_deref(),
        sink,
        completed_units,
        total_units,
    );
    let telemetry_summary = if telemetry_samples.is_empty() {
        None
    } else {
        Some(summarize_samples(&telemetry_samples))
    };

    Ok(BenchmarkRun {
        run_id,
        created_at: utc_now_iso(),
        models: available,
        benchmark_ids: vec!["performance-scenario".to_string()],
        config: {
            let mut config_map = HashMap::new();
            config_map.insert("provider".to_string(), json!(config.provider.to_string()));
            config_map.insert("base_url".to_string(), json!(config.base_url));
            config_map.insert("profile".to_string(), json!(plan.profile.label()));
            config_map.insert("stream".to_string(), json!(plan.stream));
            config_map
        },
        results,
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        run_kind: Some(BenchmarkRunKind::Performance),
        environment: Some(environment),
        performance_plan: Some(plan),
        quality_plan: None,
        provider_capabilities,
        model_load_measurements: if model_load_measurements.is_empty() {
            None
        } else {
            Some(model_load_measurements)
        },
        model_inventory_measurements: Some(model_inventory_measurements),
        telemetry_summary,
    })
}

fn build_prompts(plan: &PerformancePlan) -> anyhow::Result<Vec<PerformancePrompt>> {
    match plan.workload_jsonl.as_deref() {
        Some(path) => Ok(load_jsonl_workload(Path::new(path))?.prompts),
        None => Ok(plan
            .prompt_sizes
            .estimated_tokens
            .iter()
            .map(|value| synthetic_prompt_for_tokens(*value))
            .collect()),
    }
}

fn planned_units(plan: &PerformancePlan) -> u32 {
    (plan.models.len()
        * plan.prompt_sizes.estimated_tokens.len()
        * plan.output_sizes.estimated_tokens.len()
        * plan.concurrency.levels.len()) as u32
}

fn run_warmup_requests(
    client: &ProviderClient,
    model: &str,
    prompt: &PerformancePrompt,
    output_tokens: u32,
    plan: &PerformancePlan,
) -> anyhow::Result<()> {
    for _ in 0..plan.warmup.requests {
        let _ = execute_request(client, model, prompt, output_tokens, 1, 0, plan)?;
    }
    Ok(())
}

fn run_measured_requests(
    runtime: &tokio::runtime::Runtime,
    client: &ProviderClient,
    model: &str,
    prompt: &PerformancePrompt,
    output_tokens: u32,
    concurrency: u32,
    plan: &PerformancePlan,
) -> anyhow::Result<Vec<RequestTrace>> {
    let traces = runtime.block_on(async {
        let mut futures = FuturesUnordered::new();
        let client = client.clone();
        let shared_plan = Arc::new(plan.clone());
        let mut next_run_index = 0;
        let initial_requests = plan.runs.min(concurrency);

        for _ in 0..initial_requests {
            let client = client.clone();
            let prompt = prompt.clone();
            let params = Arc::clone(&shared_plan);
            let model_name = model.to_string();
            next_run_index += 1;
            let run_index = next_run_index;
            futures.push(tokio::task::spawn_blocking(move || {
                execute_request(
                    &client,
                    &model_name,
                    &prompt,
                    output_tokens,
                    concurrency,
                    run_index,
                    &params,
                )
            }));
        }

        let mut traces = Vec::new();
        while let Some(result) = futures.next().await {
            traces.push(result.map_err(|error| anyhow::anyhow!(error_chain(&error)))??);
            if next_run_index < plan.runs {
                let client = client.clone();
                let prompt = prompt.clone();
                let params = Arc::clone(&shared_plan);
                let model_name = model.to_string();
                next_run_index += 1;
                let run_index = next_run_index;
                futures.push(tokio::task::spawn_blocking(move || {
                    execute_request(
                        &client,
                        &model_name,
                        &prompt,
                        output_tokens,
                        concurrency,
                        run_index,
                        &params,
                    )
                }));
            }
        }
        traces.sort_by_key(|trace| trace.run_index);
        Ok::<Vec<RequestTrace>, anyhow::Error>(traces)
    })?;
    Ok(traces)
}
fn execute_request(
    client: &ProviderClient,
    model: &str,
    prompt: &PerformancePrompt,
    output_tokens: u32,
    concurrency: u32,
    run_index: u32,
    plan: &PerformancePlan,
) -> anyhow::Result<RequestTrace> {
    let request_id = format!("{model}-{}-c{concurrency}-r{run_index}", prompt.id);
    let extra = if plan.extra_params.is_empty() {
        None
    } else {
        Some(serde_json::json!(plan.extra_params.clone()))
    };
    let started = Instant::now();
    match client.chat_completion(
        model,
        json!([{ "role": "user", "content": prompt.prompt }]),
        output_tokens,
        0.0,
        plan.stream,
        extra.as_ref(),
    ) {
        Ok(result) => {
            let chunk_timings = result
                .chunk_timings_ns
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let since_start_ms = ns_to_ms(Some(*value)).unwrap_or_default();
                    let delta_ms = if index == 0 {
                        None
                    } else {
                        ns_to_ms(Some(
                            value.saturating_sub(result.chunk_timings_ns[index - 1]),
                        ))
                    };
                    ChunkTiming {
                        index,
                        since_start_ms,
                        delta_ms,
                    }
                })
                .collect::<Vec<_>>();

            let wall_time_ms = ns_to_ms(Some(result.wall_time_ns)).unwrap_or_default();
            let ttft_ms = ns_to_ms(result.time_to_first_token_ns);
            let actual_output_tokens = result.output_tokens();
            let generation_wall_ms = ttft_ms.map(|ttft| (wall_time_ms - ttft).max(0.0));
            let including_ttft = actual_output_tokens.and_then(|tokens| rate(tokens, wall_time_ms));
            let excluding_ttft = actual_output_tokens
                .and_then(|tokens| generation_wall_ms.and_then(|wall| rate(tokens, wall)));
            let itl_ms =
                inter_token_latency_ms(wall_time_ms, ttft_ms, actual_output_tokens, plan.stream);

            Ok(RequestTrace {
                request_id,
                model: model.to_string(),
                provider: client.provider(),
                prompt_id: prompt.id.clone(),
                estimated_prompt_tokens: prompt.estimated_prompt_tokens,
                requested_output_tokens: output_tokens,
                concurrency,
                run_index,
                stream: plan.stream,
                success: true,
                error: None,
                endpoint: result.endpoint.clone(),
                http_status: result.http_status,
                input_tokens: result
                    .raw
                    .pointer("/usage/prompt_tokens")
                    .or_else(|| result.raw.pointer("/usage/input_tokens"))
                    .and_then(|value| value.as_u64()),
                output_tokens: actual_output_tokens,
                chunk_timings,
                timing: RequestTiming {
                    wall_time_ms,
                    ttft_ms,
                    itl_ms,
                    inter_chunk_latency_ms: average_delta_ms(&result.chunk_timings_ns),
                    generation_wall_ms,
                    output_tokens_per_second_including_ttft: including_ttft,
                    output_tokens_per_second_excluding_ttft: excluding_ttft,
                    ttlt_ms: if plan.stream {
                        Some(wall_time_ms)
                    } else {
                        None
                    },
                },
            })
        }
        Err(error) => Ok(RequestTrace {
            request_id,
            model: model.to_string(),
            provider: client.provider(),
            prompt_id: prompt.id.clone(),
            estimated_prompt_tokens: prompt.estimated_prompt_tokens,
            requested_output_tokens: output_tokens,
            concurrency,
            run_index,
            stream: plan.stream,
            success: false,
            error: Some(error_chain(&*error)),
            endpoint: "/v1/chat/completions".to_string(),
            http_status: None,
            input_tokens: None,
            output_tokens: None,
            chunk_timings: Vec::new(),
            timing: RequestTiming {
                wall_time_ms: started.elapsed().as_secs_f64() * 1000.0,
                ttft_ms: None,
                itl_ms: None,
                inter_chunk_latency_ms: None,
                generation_wall_ms: None,
                output_tokens_per_second_including_ttft: None,
                output_tokens_per_second_excluding_ttft: None,
                ttlt_ms: None,
            },
        }),
    }
}

fn average_delta_ms(values: &[u128]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let deltas = values
        .windows(2)
        .map(|window| window[1].saturating_sub(window[0]) as f64 / 1_000_000.0)
        .collect::<Vec<_>>();
    Some(deltas.iter().sum::<f64>() / deltas.len() as f64)
}

#[allow(clippy::too_many_arguments)]
fn summary_record(
    model: &str,
    scenario_name: &str,
    prompt: &PerformancePrompt,
    output_tokens: u32,
    concurrency: u32,
    summary: &PerformanceSummary,
    traces: &[RequestTrace],
    detail: ReportDetailLevel,
) -> BenchmarkResultRecord {
    let mut metrics = HashMap::new();
    metrics.insert(
        "request_count".to_string(),
        json!(summary.latency.request_count),
    );
    metrics.insert(
        "success_count".to_string(),
        json!(summary.latency.success_count),
    );
    metrics.insert(
        "error_count".to_string(),
        json!(summary.latency.error_count),
    );
    metrics.insert("error_rate".to_string(), json!(summary.latency.error_rate));
    metrics.insert(
        "partial_failure".to_string(),
        json!(summary.latency.error_count > 0 && summary.latency.success_count > 0),
    );
    metrics.insert(
        "successful_latency_sample_count".to_string(),
        json!(summary.latency.successful_latency_sample_count),
    );
    metrics.insert(
        "percentile_estimator".to_string(),
        json!(summary.latency.percentile_estimator),
    );
    metrics.insert(
        "standard_deviation_kind".to_string(),
        json!(summary.latency.standard_deviation_kind),
    );
    insert_opt(
        &mut metrics,
        "wall_time_ms_min",
        summary.latency.wall_time_ms_min,
    );
    insert_opt(
        &mut metrics,
        "wall_time_ms_mean",
        summary.latency.wall_time_ms_mean,
    );
    insert_opt(
        &mut metrics,
        "wall_time_ms_max",
        summary.latency.wall_time_ms_max,
    );
    insert_opt(
        &mut metrics,
        "wall_time_ms_stddev",
        summary.latency.wall_time_ms_stddev,
    );
    insert_opt(
        &mut metrics,
        "wall_time_ms_p50",
        summary.latency.wall_time_ms_p50,
    );
    insert_opt(
        &mut metrics,
        "wall_time_ms_p90",
        summary.latency.wall_time_ms_p90,
    );
    insert_opt(
        &mut metrics,
        "wall_time_ms_p95",
        summary.latency.wall_time_ms_p95,
    );
    insert_opt(
        &mut metrics,
        "wall_time_ms_p99",
        summary.latency.wall_time_ms_p99,
    );
    insert_opt(&mut metrics, "ttft_ms_p50", summary.latency.ttft_ms_p50);
    insert_opt(&mut metrics, "ttft_ms_mean", summary.latency.ttft_ms_mean);
    insert_opt(
        &mut metrics,
        "time_to_first_token_ms",
        summary.latency.ttft_ms_mean,
    );
    insert_opt(&mut metrics, "ttft_ms_min", summary.latency.ttft_ms_min);
    insert_opt(&mut metrics, "ttft_ms_max", summary.latency.ttft_ms_max);
    insert_opt(&mut metrics, "ttft_ms_p95", summary.latency.ttft_ms_p95);
    insert_opt(&mut metrics, "ttft_ms_p99", summary.latency.ttft_ms_p99);
    insert_opt(&mut metrics, "itl_ms_p50", summary.latency.itl_ms_p50);
    insert_opt(&mut metrics, "itl_ms_p90", summary.latency.itl_ms_p90);
    insert_opt(&mut metrics, "itl_ms_p95", summary.latency.itl_ms_p95);
    insert_opt(&mut metrics, "itl_ms_p99", summary.latency.itl_ms_p99);
    insert_opt(
        &mut metrics,
        "inter_chunk_latency_ms_p50",
        summary.latency.inter_chunk_latency_ms_p50,
    );
    insert_opt(
        &mut metrics,
        "generation_wall_ms_p50",
        summary.latency.generation_wall_ms_p50,
    );
    insert_opt(
        &mut metrics,
        "generation_wall_ms_p95",
        summary.latency.generation_wall_ms_p95,
    );
    insert_opt(
        &mut metrics,
        "generation_wall_ms_p99",
        summary.latency.generation_wall_ms_p99,
    );
    insert_opt(
        &mut metrics,
        "output_tokens_per_second",
        summary.throughput.output_tokens_per_second,
    );
    insert_opt(
        &mut metrics,
        "input_tokens_per_second",
        summary.throughput.input_tokens_per_second,
    );
    insert_opt(
        &mut metrics,
        "requests_per_second",
        summary.throughput.requests_per_second,
    );
    insert_opt(
        &mut metrics,
        "successful_requests_per_second",
        summary.throughput.successful_requests_per_second,
    );
    insert_opt(
        &mut metrics,
        "output_tokens_per_second_including_ttft",
        summary.throughput.output_tokens_per_second_including_ttft,
    );
    insert_opt(
        &mut metrics,
        "output_tokens_per_second_excluding_ttft",
        summary.throughput.output_tokens_per_second_excluding_ttft,
    );
    insert_opt(
        &mut metrics,
        "mean_input_tokens_per_request",
        summary.throughput.mean_input_tokens_per_request,
    );
    insert_opt(
        &mut metrics,
        "mean_output_tokens_per_request",
        summary.throughput.mean_output_tokens_per_request,
    );
    metrics.insert(
        "total_input_tokens".to_string(),
        json!(summary.throughput.total_input_tokens),
    );
    metrics.insert(
        "total_output_tokens".to_string(),
        json!(summary.throughput.total_output_tokens),
    );
    metrics.insert(
        "input_token_sample_count".to_string(),
        json!(summary.throughput.input_token_sample_count),
    );
    metrics.insert(
        "output_token_sample_count".to_string(),
        json!(summary.throughput.output_token_sample_count),
    );
    metrics.insert(
        "input_token_coverage".to_string(),
        json!(summary.throughput.input_token_coverage),
    );
    metrics.insert(
        "output_token_coverage".to_string(),
        json!(summary.throughput.output_token_coverage),
    );
    metrics.insert(
        "estimated_prompt_tokens".to_string(),
        json!(prompt.estimated_prompt_tokens),
    );
    metrics.insert("requested_output_tokens".to_string(), json!(output_tokens));
    metrics.insert("concurrency".to_string(), json!(concurrency));
    metrics.insert(
        "timeout_count".to_string(),
        json!(summary.errors.timeout_count),
    );
    metrics.insert(
        "http_error_count".to_string(),
        json!(summary.errors.http_error_count),
    );
    metrics.insert(
        "provider_error_count".to_string(),
        json!(summary.errors.provider_error_count),
    );
    metrics.insert(
        "empty_response_count".to_string(),
        json!(summary.errors.empty_response_count),
    );

    BenchmarkResultRecord {
        benchmark_id: "performance-scenario".to_string(),
        benchmark_name: "Performance scenario".to_string(),
        model: model.to_string(),
        run_index: None,
        prompt_name: Some(scenario_name.to_string()),
        metrics,
        response_preview: None,
        error: (summary.latency.success_count == 0)
            .then(|| summary.errors.error_messages.first().cloned())
            .flatten(),
        metadata: Some(performance_metadata(scenario_name, prompt, traces, detail)),
    }
}

const DETAILED_TRACE_LIMIT: usize = 20;

fn performance_metadata(
    scenario_name: &str,
    prompt: &PerformancePrompt,
    traces: &[RequestTrace],
    detail: ReportDetailLevel,
) -> HashMap<String, Value> {
    let trace_count = traces.len();
    let mut metadata = HashMap::from([
        ("scenario".to_string(), json!(scenario_name)),
        ("prompt_id".to_string(), json!(prompt.id)),
        ("request_trace_count".to_string(), json!(trace_count)),
    ]);
    let persisted = match detail {
        ReportDetailLevel::Summary => None,
        ReportDetailLevel::Detailed => Some(&traces[..trace_count.min(DETAILED_TRACE_LIMIT)]),
        ReportDetailLevel::Full => Some(traces),
    };
    let traces_omitted = matches!(detail, ReportDetailLevel::Summary);
    let traces_truncated =
        matches!(detail, ReportDetailLevel::Detailed) && trace_count > DETAILED_TRACE_LIMIT;
    if let Some(persisted) = persisted {
        metadata.insert("request_traces".to_string(), json!(persisted));
    }
    metadata.insert(
        "request_traces_truncated".to_string(),
        json!(traces_truncated),
    );
    metadata.insert("request_traces_omitted".to_string(), json!(traces_omitted));
    metadata
}
fn insert_opt(metrics: &mut HashMap<String, Value>, key: &str, value: Option<f64>) {
    if let Some(value) = value {
        metrics.insert(key.to_string(), json!(value));
    }
}

fn rate(tokens: u64, wall_time_ms: f64) -> Option<f64> {
    if wall_time_ms <= 0.0 {
        None
    } else {
        Some(tokens as f64 / (wall_time_ms / 1000.0))
    }
}

fn current_process_memory() -> Option<u64> {
    let mut system = sysinfo::System::new_all();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let process_id = sysinfo::Pid::from_u32(std::process::id());
    system.process(process_id).map(|process| process.memory())
}

#[cfg(test)]
mod tests {
    use super::performance_metadata;
    use crate::performance::config::ReportDetailLevel;
    use crate::performance::metrics::{RequestTiming, RequestTrace};
    use crate::performance::workload::PerformancePrompt;
    use crate::providers::ProviderKind;

    fn prompt() -> PerformancePrompt {
        PerformancePrompt {
            id: "test-prompt".to_string(),
            prompt: "hello".to_string(),
            tags: Vec::new(),
            estimated_prompt_tokens: 1,
        }
    }

    fn trace(index: u32) -> RequestTrace {
        RequestTrace {
            request_id: format!("request-{index}"),
            model: "model".to_string(),
            provider: ProviderKind::Ollama,
            prompt_id: "test-prompt".to_string(),
            estimated_prompt_tokens: 1,
            requested_output_tokens: 1,
            concurrency: 1,
            run_index: index,
            stream: true,
            success: true,
            error: None,
            endpoint: "/v1/chat/completions".to_string(),
            http_status: Some(200),
            input_tokens: Some(1),
            output_tokens: Some(1),
            chunk_timings: Vec::new(),
            timing: RequestTiming {
                wall_time_ms: 1.0,
                ttft_ms: Some(1.0),
                itl_ms: None,
                inter_chunk_latency_ms: None,
                generation_wall_ms: Some(0.0),
                output_tokens_per_second_including_ttft: Some(1_000.0),
                output_tokens_per_second_excluding_ttft: None,
                ttlt_ms: Some(1.0),
            },
        }
    }

    #[test]
    fn trace_detail_distinguishes_omission_from_truncation() {
        let empty = Vec::new();
        let summary =
            performance_metadata("scenario", &prompt(), &empty, ReportDetailLevel::Summary);
        assert_eq!(summary["request_traces_omitted"], true);
        assert_eq!(summary["request_traces_truncated"], false);

        let detailed =
            performance_metadata("scenario", &prompt(), &empty, ReportDetailLevel::Detailed);
        assert_eq!(detailed["request_traces_omitted"], false);
        assert_eq!(detailed["request_traces_truncated"], false);

        let traces = (0..21).map(trace).collect::<Vec<_>>();
        let truncated =
            performance_metadata("scenario", &prompt(), &traces, ReportDetailLevel::Detailed);
        assert_eq!(truncated["request_traces_omitted"], false);
        assert_eq!(truncated["request_traces_truncated"], true);
    }
}
