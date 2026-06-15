use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use futures_util::stream::{FuturesUnordered, StreamExt};
use serde_json::{json, Value};
use tokio::sync::Semaphore;

use crate::benchmarks::base::BenchmarkResultRecord;
use crate::config::AppConfig;
use crate::errors::LLMeterError;
use crate::performance::config::PerformancePlan;
use crate::performance::metrics::{
    summarize_traces, PerformanceSummary, RequestTiming, RequestTrace, TokenTiming,
};
use crate::performance::resource::capture_environment_snapshot;
use crate::performance::workload::{
    load_jsonl_workload, synthetic_prompt_for_tokens, PerformancePrompt,
};
use crate::progress::{ProgressEventKind, ProgressPhase, ProgressSink, ProgressUpdate};
use crate::providers::ProviderClient;
use crate::results::{BenchmarkRun, BenchmarkRunKind, ResultStore};
use crate::utils::{ns_to_ms, utc_now_iso};

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

    let total_units = planned_units(&plan);
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
                    let traces = run_measured_requests(
                        client,
                        model,
                        prompt,
                        output_tokens,
                        concurrency,
                        &plan,
                    )?;
                    let summary = summarize_traces(&traces);
                    results.push(summary_record(
                        model,
                        &scenario_name,
                        prompt,
                        output_tokens,
                        concurrency,
                        &summary,
                        &traces,
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

    let environment =
        capture_environment_snapshot(config, process_memory_before, current_process_memory());

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
        schema_version: "2.0".to_string(),
        run_kind: Some(BenchmarkRunKind::Performance),
        environment: Some(environment),
        performance_plan: Some(plan),
        quality_plan: None,
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
    client: &ProviderClient,
    model: &str,
    prompt: &PerformancePrompt,
    output_tokens: u32,
    concurrency: u32,
    plan: &PerformancePlan,
) -> anyhow::Result<Vec<RequestTrace>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let traces = runtime.block_on(async {
        let semaphore = Arc::new(Semaphore::new(concurrency as usize));
        let mut futures = FuturesUnordered::new();
        let client = client.clone();

        for run_index in 0..plan.runs {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            let client = client.clone();
            let prompt = prompt.clone();
            let params = plan.clone();
            let model_name = model.to_string();
            futures.push(tokio::task::spawn_blocking(move || {
                let _permit = permit;
                execute_request(
                    &client,
                    &model_name,
                    &prompt,
                    output_tokens,
                    concurrency,
                    run_index + 1,
                    &params,
                )
            }));
        }

        let mut traces = Vec::new();
        while let Some(result) = futures.next().await {
            traces.push(result.map_err(|error| anyhow::anyhow!(error.to_string()))??);
        }
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
            let token_timings = result
                .token_timings_ns
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let since_start_ms = ns_to_ms(Some(*value)).unwrap_or_default();
                    let delta_ms = if index == 0 {
                        None
                    } else {
                        ns_to_ms(Some(
                            value.saturating_sub(result.token_timings_ns[index - 1]),
                        ))
                    };
                    TokenTiming {
                        index,
                        since_start_ms,
                        delta_ms,
                    }
                })
                .collect::<Vec<_>>();

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
                output_tokens: result.output_tokens(),
                token_timings,
                timing: RequestTiming {
                    wall_time_ms: ns_to_ms(Some(result.wall_time_ns)).unwrap_or_default(),
                    ttft_ms: ns_to_ms(result.time_to_first_token_ns),
                    tpot_ms: average_delta_ms(&result.token_timings_ns),
                    itl_ms: average_delta_ms(&result.token_timings_ns),
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
            error: Some(error.to_string()),
            endpoint: "/v1/chat/completions".to_string(),
            http_status: None,
            input_tokens: None,
            output_tokens: None,
            token_timings: Vec::new(),
            timing: RequestTiming {
                wall_time_ms: started.elapsed().as_secs_f64() * 1000.0,
                ttft_ms: None,
                tpot_ms: None,
                itl_ms: None,
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

fn summary_record(
    model: &str,
    scenario_name: &str,
    prompt: &PerformancePrompt,
    output_tokens: u32,
    concurrency: u32,
    summary: &PerformanceSummary,
    traces: &[RequestTrace],
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
    insert_opt(
        &mut metrics,
        "wall_time_ms_min",
        summary.latency.wall_time_ms_min,
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
    insert_opt(&mut metrics, "ttft_ms_p95", summary.latency.ttft_ms_p95);
    insert_opt(&mut metrics, "ttft_ms_p99", summary.latency.ttft_ms_p99);
    insert_opt(&mut metrics, "tpot_ms_p50", summary.latency.tpot_ms_p50);
    insert_opt(&mut metrics, "tpot_ms_p95", summary.latency.tpot_ms_p95);
    insert_opt(&mut metrics, "itl_ms_p50", summary.latency.itl_ms_p50);
    insert_opt(&mut metrics, "itl_ms_p95", summary.latency.itl_ms_p95);
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
    metrics.insert(
        "total_input_tokens".to_string(),
        json!(summary.throughput.total_input_tokens),
    );
    metrics.insert(
        "total_output_tokens".to_string(),
        json!(summary.throughput.total_output_tokens),
    );
    metrics.insert(
        "estimated_prompt_tokens".to_string(),
        json!(prompt.estimated_prompt_tokens),
    );
    metrics.insert("requested_output_tokens".to_string(), json!(output_tokens));
    metrics.insert("concurrency".to_string(), json!(concurrency));

    BenchmarkResultRecord {
        benchmark_id: "performance-scenario".to_string(),
        benchmark_name: "Performance scenario".to_string(),
        model: model.to_string(),
        run_index: None,
        prompt_name: Some(scenario_name.to_string()),
        metrics,
        response_preview: None,
        error: summary.errors.error_messages.first().cloned(),
        metadata: Some(HashMap::from([
            ("scenario".to_string(), json!(scenario_name)),
            ("prompt_id".to_string(), json!(prompt.id)),
            ("request_traces".to_string(), json!(traces)),
        ])),
    }
}

fn insert_opt(metrics: &mut HashMap<String, Value>, key: &str, value: Option<f64>) {
    if let Some(value) = value {
        metrics.insert(key.to_string(), json!(value));
    }
}

fn current_process_memory() -> Option<u64> {
    let mut system = sysinfo::System::new_all();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let process_id = sysinfo::Pid::from_u32(std::process::id());
    system.process(process_id).map(|process| process.memory())
}
