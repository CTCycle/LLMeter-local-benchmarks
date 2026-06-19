use std::path::PathBuf;

use serde_json::Value;

use crate::benchmarks::base::{
    Benchmark, BenchmarkContext, BenchmarkProgressSink, BenchmarkStepStatus, BenchmarkStepUpdate,
};
use crate::benchmarks::registry::{default_registry, BenchmarkSuite};
use crate::cli::ReportCommands;
use crate::config::AppConfig;
use crate::errors::LLMeterError;
use crate::progress::{ProgressEventKind, ProgressPhase, ProgressSink, ProgressUpdate};
use crate::providers::ProviderClient;
use crate::reporting::{save_html_report, save_markdown_report};
use crate::results::{BenchmarkRun, BenchmarkRunKind, ResultStore};
use crate::utils::utc_now_iso;

pub struct BenchmarkRunRequest<'a> {
    pub suite: BenchmarkSuite,
    pub model_names: &'a [String],
    pub benchmark_ids: Option<&'a [String]>,
    pub all_benchmarks: bool,
    pub runs: u32,
    pub max_tokens: u32,
    pub temperature: f64,
    pub extra_options: &'a std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkExecutionPlan {
    pub total_models: usize,
    pub total_benchmarks: usize,
    pub total_steps: u32,
}

impl BenchmarkExecutionPlan {
    pub fn new(
        total_models: usize,
        benchmarks: &[&dyn Benchmark],
        context: &BenchmarkContext,
    ) -> Self {
        let total_benchmarks = benchmarks.len();
        let total_steps = (total_models as u32)
            * benchmarks
                .iter()
                .map(|benchmark| benchmark.planned_steps(context))
                .sum::<u32>();
        Self {
            total_models,
            total_benchmarks,
            total_steps,
        }
    }
}

struct RunnerProgressAdapter<'a> {
    sink: &'a mut dyn ProgressSink,
    total_units: u32,
    completed_units: u32,
    model_name: &'a str,
    model_index: usize,
    total_models: usize,
    benchmark_id: &'a str,
    benchmark_name: &'a str,
    benchmark_index: usize,
    total_benchmarks: usize,
}

impl<'a> RunnerProgressAdapter<'a> {
    fn update(&mut self, step: BenchmarkStepUpdate) {
        if matches!(step.status, BenchmarkStepStatus::Completed) {
            self.completed_units = self.completed_units.saturating_add(1);
        }

        self.sink.on_update(ProgressUpdate {
            kind: match step.status {
                BenchmarkStepStatus::Started => ProgressEventKind::StepStarted,
                BenchmarkStepStatus::Completed => ProgressEventKind::StepCompleted,
            },
            phase: ProgressPhase::Running,
            message: step.message,
            completed_units: self.completed_units,
            total_units: self.total_units,
            model_name: Some(self.model_name.to_string()),
            model_index: Some(self.model_index),
            total_models: Some(self.total_models),
            benchmark_id: Some(self.benchmark_id.to_string()),
            benchmark_name: Some(self.benchmark_name.to_string()),
            benchmark_index: Some(self.benchmark_index),
            total_benchmarks: Some(self.total_benchmarks),
            step_index: Some(step.step_index),
            total_steps: Some(step.total_steps),
            run_index: step.run_index,
            prompt_name: step.prompt_name,
        });
    }
}

impl BenchmarkProgressSink for RunnerProgressAdapter<'_> {
    fn on_step(&mut self, update: BenchmarkStepUpdate) {
        self.update(update);
    }
}

pub fn installed_model_names(client: &ProviderClient) -> anyhow::Result<Vec<String>> {
    client.model_names()
}

pub fn validate_models(
    client: &ProviderClient,
    selected_models: &[String],
) -> anyhow::Result<Vec<String>> {
    let available: std::collections::HashSet<String> =
        installed_model_names(client)?.into_iter().collect();
    let mut requested: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    for model in selected_models {
        let trimmed = model.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        if available.contains(&trimmed) {
            requested.push(trimmed);
        } else {
            missing.push(trimmed);
        }
    }

    if !missing.is_empty() {
        return Err(LLMeterError::ModelNotFound(format!(
            "Model(s) not exposed by {} at {}: {}",
            client.provider().display_name(),
            client.base_url(),
            missing.join(", ")
        ))
        .into());
    }
    if requested.is_empty() {
        return Err(LLMeterError::ModelNotFound("No models selected.".to_string()).into());
    }
    Ok(requested)
}

pub fn run_benchmarks(
    client: &ProviderClient,
    config: &AppConfig,
    request: BenchmarkRunRequest<'_>,
    tail_progress_units: u32,
    progress: Option<&mut dyn ProgressSink>,
) -> anyhow::Result<BenchmarkRun> {
    let mut null_sink = crate::progress::NullProgressSink;
    let sink = match progress {
        Some(sink) => sink,
        None => &mut null_sink,
    };

    sink.on_update(ProgressUpdate {
        kind: ProgressEventKind::Phase,
        phase: ProgressPhase::Validating,
        message: "Checking provider reachability and selected models".to_string(),
        completed_units: 0,
        total_units: 0,
        model_name: None,
        model_index: None,
        total_models: None,
        benchmark_id: None,
        benchmark_name: None,
        benchmark_index: None,
        total_benchmarks: None,
        step_index: None,
        total_steps: None,
        run_index: None,
        prompt_name: None,
    });

    if !client.is_running() {
        return Err(LLMeterError::Provider(format!(
            "{} provider is not reachable at {}.",
            config.provider.display_name(),
            config.base_url
        ))
        .into());
    }

    let models = validate_models(client, request.model_names)?;
    let registry = default_registry();
    let benchmarks =
        registry.select(request.benchmark_ids, request.all_benchmarks, request.suite)?;

    let context = BenchmarkContext {
        runs: request.runs,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        timeout: config.timeout,
        options: request.extra_options.clone(),
    };

    let plan = BenchmarkExecutionPlan::new(models.len(), &benchmarks, &context);
    let overall_total_units = plan.total_steps + tail_progress_units;
    sink.on_update(ProgressUpdate {
        kind: ProgressEventKind::Phase,
        phase: ProgressPhase::Planning,
        message: format!(
            "Prepared {} model(s), {} benchmark(s), {} planned step(s)",
            plan.total_models, plan.total_benchmarks, plan.total_steps
        ),
        completed_units: 0,
        total_units: overall_total_units,
        model_name: None,
        model_index: None,
        total_models: Some(plan.total_models),
        benchmark_id: None,
        benchmark_name: None,
        benchmark_index: None,
        total_benchmarks: Some(plan.total_benchmarks),
        step_index: None,
        total_steps: Some(overall_total_units),
        run_index: None,
        prompt_name: None,
    });

    execute_benchmark_plan(
        client,
        config,
        &models,
        &benchmarks,
        request.suite,
        context,
        tail_progress_units,
        sink,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_benchmark_plan(
    client: &ProviderClient,
    config: &AppConfig,
    models: &[String],
    benchmarks: &[&dyn Benchmark],
    suite: BenchmarkSuite,
    context: BenchmarkContext,
    tail_progress_units: u32,
    sink: &mut dyn ProgressSink,
) -> anyhow::Result<BenchmarkRun> {
    let plan = BenchmarkExecutionPlan::new(models.len(), benchmarks, &context);
    let store = ResultStore::new(&config.output_dir);
    let run_id = store.new_run_id(models);

    let mut run = BenchmarkRun {
        run_id,
        created_at: utc_now_iso(),
        models: models.to_vec(),
        benchmark_ids: benchmarks.iter().map(|b| b.id().to_string()).collect(),
        config: {
            let mut c = std::collections::HashMap::new();
            c.insert(
                "provider".to_string(),
                Value::from(config.provider.to_string()),
            );
            c.insert("base_url".to_string(), Value::from(config.base_url.clone()));
            c.insert("runs".to_string(), Value::from(context.runs));
            c.insert("max_tokens".to_string(), Value::from(context.max_tokens));
            c.insert("temperature".to_string(), Value::from(context.temperature));
            c.insert("timeout".to_string(), Value::from(config.timeout));
            c.insert("suite".to_string(), Value::from(suite.label()));
            c
        },
        results: Vec::new(),
        schema_version: "2.0".to_string(),
        run_kind: Some(BenchmarkRunKind::Benchmark),
        environment: None,
        performance_plan: None,
        quality_plan: None,
        provider_capabilities: None,
        model_load_measurements: None,
        model_inventory_measurements: None,
        telemetry_summary: None,
    };

    let mut completed_units = 0u32;
    for (model_offset, model) in models.iter().enumerate() {
        for (benchmark_offset, benchmark) in benchmarks.iter().enumerate() {
            let mut benchmark_progress = RunnerProgressAdapter {
                sink,
                total_units: plan.total_steps + tail_progress_units,
                completed_units,
                model_name: model,
                model_index: model_offset + 1,
                total_models: plan.total_models,
                benchmark_id: benchmark.id(),
                benchmark_name: benchmark.name(),
                benchmark_index: benchmark_offset + 1,
                total_benchmarks: plan.total_benchmarks,
            };
            run.results
                .extend(benchmark.run(client, model, &context, &mut benchmark_progress));
            completed_units = benchmark_progress.completed_units;
        }
    }

    Ok(run)
}

pub fn save_outputs(
    config: &AppConfig,
    run: &BenchmarkRun,
    export: &str,
    report: &str,
    progress: Option<&mut dyn ProgressSink>,
) -> anyhow::Result<Vec<PathBuf>> {
    validate_output_choice(export, &["json", "csv", "both", "none"], "--export")?;
    validate_output_choice(report, &["md", "html", "both", "none"], "--report")?;

    let mut null_sink = crate::progress::NullProgressSink;
    let sink = match progress {
        Some(sink) => sink,
        None => &mut null_sink,
    };

    let store = ResultStore::new(&config.output_dir);
    let mut saved: Vec<PathBuf> = Vec::new();
    let benchmark_units = planned_steps_for_run(run);
    let total_units = benchmark_units + 2u32;
    let mut completed_units = benchmark_units;

    sink.on_update(ProgressUpdate {
        kind: ProgressEventKind::Phase,
        phase: ProgressPhase::SavingResults,
        message: format!("Saving raw result export setting: {export}"),
        completed_units,
        total_units,
        model_name: None,
        model_index: None,
        total_models: None,
        benchmark_id: None,
        benchmark_name: None,
        benchmark_index: None,
        total_benchmarks: None,
        step_index: None,
        total_steps: None,
        run_index: None,
        prompt_name: None,
    });

    if matches!(export, "json" | "both") {
        saved.push(store.save_json(run)?);
    }
    if matches!(export, "csv" | "both") {
        saved.push(store.save_csv(run)?);
    }

    completed_units += 1;
    sink.on_update(ProgressUpdate {
        kind: ProgressEventKind::Phase,
        phase: ProgressPhase::GeneratingReports,
        message: format!("Generating formatted report setting: {report}"),
        completed_units,
        total_units,
        model_name: None,
        model_index: None,
        total_models: None,
        benchmark_id: None,
        benchmark_name: None,
        benchmark_index: None,
        total_benchmarks: None,
        step_index: None,
        total_steps: None,
        run_index: None,
        prompt_name: None,
    });

    if matches!(report, "md" | "both") {
        saved.push(save_markdown_report(run, &config.output_dir)?);
    }
    if matches!(report, "html" | "both") {
        saved.push(save_html_report(run, &config.output_dir)?);
    }

    completed_units += 1;
    sink.on_update(ProgressUpdate {
        kind: ProgressEventKind::Finished,
        phase: ProgressPhase::Completed,
        message: "Benchmark run completed".to_string(),
        completed_units,
        total_units,
        model_name: None,
        model_index: None,
        total_models: None,
        benchmark_id: None,
        benchmark_name: None,
        benchmark_index: None,
        total_benchmarks: None,
        step_index: None,
        total_steps: None,
        run_index: None,
        prompt_name: None,
    });
    Ok(saved)
}

fn validate_output_choice(value: &str, allowed: &[&str], flag: &str) -> anyhow::Result<()> {
    if allowed.contains(&value) {
        return Ok(());
    }

    Err(LLMeterError::InvalidOption(format!(
        "Invalid {flag} '{value}'. Use one of: {}.",
        allowed.join(", ")
    ))
    .into())
}

pub fn command_report(config: &AppConfig, report_cmd: &ReportCommands) -> anyhow::Result<()> {
    let store = ResultStore::new(&config.output_dir);

    match report_cmd {
        ReportCommands::List => {
            let json_files = store.latest_json_files(10);
            let report_files = store.latest_report_files(10);
            crate::ui::print_file_list(&json_files, "Saved JSON results");
            crate::ui::print_file_list(&report_files, "Generated reports");
        }
        ReportCommands::Show { result } => {
            let path = resolve_result_file(config, result.as_deref())?;
            let run = store.load_json(&path)?;
            let markdown = crate::reporting::render_markdown_report(&run);
            crate::ui::print_terminal_report(&markdown);
        }
        ReportCommands::Generate { result, format } => {
            let path = resolve_result_file(config, result.as_deref())?;
            let run = store.load_json(&path)?;
            let saved = save_outputs(config, &run, "none", format, None)?;
            crate::ui::print_saved_paths(&saved);
        }
    }
    Ok(())
}

fn resolve_result_file(config: &AppConfig, maybe_path: Option<&str>) -> anyhow::Result<PathBuf> {
    if let Some(path_str) = maybe_path {
        let path = PathBuf::from(path_str);
        if !path.exists() {
            return Err(LLMeterError::Io(format!(
                "Result file does not exist: {}",
                path.display()
            ))
            .into());
        }
        return Ok(path);
    }

    let store = ResultStore::new(&config.output_dir);
    let latest = store.latest_json_files(1);
    match latest.into_iter().next() {
        Some(path) => Ok(path),
        None => Err(LLMeterError::Io(format!(
            "No JSON result files found in {}",
            config.output_dir.display()
        ))
        .into()),
    }
}

fn planned_steps_for_run(run: &BenchmarkRun) -> u32 {
    let registry = default_registry();
    let context = BenchmarkContext {
        runs: run
            .config
            .get("runs")
            .and_then(|value| value.as_u64())
            .map(|value| value as u32)
            .unwrap_or(1),
        max_tokens: run
            .config
            .get("max_tokens")
            .and_then(|value| value.as_u64())
            .map(|value| value as u32)
            .unwrap_or(1),
        temperature: run
            .config
            .get("temperature")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        timeout: run
            .config
            .get("timeout")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        options: std::collections::HashMap::new(),
    };

    run.benchmark_ids
        .iter()
        .filter_map(|benchmark_id| registry.get(benchmark_id))
        .map(|benchmark| benchmark.planned_steps(&context))
        .sum::<u32>()
        * run.models.len() as u32
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use super::{execute_benchmark_plan, BenchmarkExecutionPlan};
    use crate::benchmarks::base::{
        Benchmark, BenchmarkContext, BenchmarkProgressSink, BenchmarkResultRecord,
        BenchmarkStepStatus, BenchmarkStepUpdate,
    };
    use crate::benchmarks::registry::BenchmarkSuite;
    use crate::config::AppConfig;
    use crate::progress::{ProgressEventKind, ProgressSink, ProgressUpdate};
    use crate::providers::{ProviderClient, ProviderKind};
    use crate::results::{BenchmarkRun, BenchmarkRunKind};

    struct StubBenchmark {
        id: &'static str,
        name: &'static str,
        total_steps: u32,
        fail_on_step: Option<u32>,
    }

    impl Benchmark for StubBenchmark {
        fn id(&self) -> &str {
            self.id
        }

        fn name(&self) -> &str {
            self.name
        }

        fn description(&self) -> &str {
            "stub"
        }

        fn suite(&self) -> BenchmarkSuite {
            BenchmarkSuite::Llm
        }

        fn planned_steps(&self, _context: &BenchmarkContext) -> u32 {
            self.total_steps
        }

        fn run(
            &self,
            _client: &ProviderClient,
            model: &str,
            _context: &BenchmarkContext,
            progress: &mut dyn BenchmarkProgressSink,
        ) -> Vec<BenchmarkResultRecord> {
            let mut records = Vec::new();
            for step_index in 1..=self.total_steps {
                progress.on_step(BenchmarkStepUpdate {
                    status: BenchmarkStepStatus::Started,
                    step_index,
                    total_steps: self.total_steps,
                    run_index: Some(step_index),
                    prompt_name: Some("stub".to_string()),
                    message: format!("starting step {step_index}"),
                });
                records.push(BenchmarkResultRecord {
                    benchmark_id: self.id.to_string(),
                    benchmark_name: self.name.to_string(),
                    model: model.to_string(),
                    run_index: Some(step_index),
                    prompt_name: Some("stub".to_string()),
                    metrics: HashMap::new(),
                    response_preview: None,
                    error: self
                        .fail_on_step
                        .filter(|value| *value == step_index)
                        .map(|_| "boom".to_string()),
                    metadata: Some({
                        let mut metadata = HashMap::new();
                        metadata.insert("ok".to_string(), json!(true));
                        metadata
                    }),
                });
                progress.on_step(BenchmarkStepUpdate {
                    status: BenchmarkStepStatus::Completed,
                    step_index,
                    total_steps: self.total_steps,
                    run_index: Some(step_index),
                    prompt_name: Some("stub".to_string()),
                    message: format!("completed step {step_index}"),
                });
            }
            records
        }
    }

    #[derive(Default)]
    struct RecordingProgressSink {
        updates: Vec<ProgressUpdate>,
    }

    impl ProgressSink for RecordingProgressSink {
        fn on_update(&mut self, update: ProgressUpdate) {
            self.updates.push(update);
        }
    }

    fn test_config() -> AppConfig {
        AppConfig {
            provider: ProviderKind::Ollama,
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            timeout: 30.0,
            output_dir: std::env::temp_dir(),
            default_runs: 1,
            default_max_tokens: 128,
            default_temperature: 0.0,
            explicit_base_url: false,
        }
    }

    #[test]
    fn benchmark_execution_plan_counts_steps() {
        let context = BenchmarkContext {
            runs: 3,
            max_tokens: 128,
            temperature: 0.0,
            timeout: 30.0,
            options: HashMap::new(),
        };
        let first = StubBenchmark {
            id: "one",
            name: "One",
            total_steps: 2,
            fail_on_step: None,
        };
        let second = StubBenchmark {
            id: "two",
            name: "Two",
            total_steps: 5,
            fail_on_step: None,
        };
        let benchmarks: Vec<&dyn Benchmark> = vec![&first, &second];

        let plan = BenchmarkExecutionPlan::new(2, &benchmarks, &context);
        assert_eq!(plan.total_models, 2);
        assert_eq!(plan.total_benchmarks, 2);
        assert_eq!(plan.total_steps, 14);
    }

    #[test]
    fn execute_benchmark_plan_reports_all_steps() {
        let client = ProviderClient::new(ProviderKind::Ollama, "http://127.0.0.1:1/v1", 0.1);
        let config = test_config();
        let context = BenchmarkContext {
            runs: 1,
            max_tokens: 128,
            temperature: 0.0,
            timeout: 30.0,
            options: HashMap::new(),
        };
        let first = StubBenchmark {
            id: "one",
            name: "One",
            total_steps: 2,
            fail_on_step: None,
        };
        let second = StubBenchmark {
            id: "two",
            name: "Two",
            total_steps: 1,
            fail_on_step: None,
        };
        let benchmarks: Vec<&dyn Benchmark> = vec![&first, &second];
        let models = vec!["model-a".to_string(), "model-b".to_string()];
        let mut sink = RecordingProgressSink::default();

        let run = execute_benchmark_plan(
            &client,
            &config,
            &models,
            &benchmarks,
            BenchmarkSuite::Llm,
            context,
            0,
            &mut sink,
        )
        .unwrap();

        assert_eq!(run.results.len(), 6);
        let completed: Vec<&ProgressUpdate> = sink
            .updates
            .iter()
            .filter(|update| matches!(update.kind, ProgressEventKind::StepCompleted))
            .collect();
        assert_eq!(completed.len(), 6);
        assert_eq!(
            completed.last().map(|update| update.completed_units),
            Some(6)
        );
        assert_eq!(completed.last().map(|update| update.total_units), Some(6));
    }

    #[test]
    fn execute_benchmark_plan_advances_progress_when_records_fail() {
        let client = ProviderClient::new(ProviderKind::Ollama, "http://127.0.0.1:1/v1", 0.1);
        let config = test_config();
        let context = BenchmarkContext {
            runs: 1,
            max_tokens: 128,
            temperature: 0.0,
            timeout: 30.0,
            options: HashMap::new(),
        };
        let benchmark = StubBenchmark {
            id: "failing",
            name: "Failing",
            total_steps: 3,
            fail_on_step: Some(2),
        };
        let benchmarks: Vec<&dyn Benchmark> = vec![&benchmark];
        let models = vec!["model-a".to_string()];
        let mut sink = RecordingProgressSink::default();

        let run = execute_benchmark_plan(
            &client,
            &config,
            &models,
            &benchmarks,
            BenchmarkSuite::Llm,
            context,
            0,
            &mut sink,
        )
        .unwrap();

        assert_eq!(run.results.len(), 3);
        assert_eq!(
            run.results
                .iter()
                .filter(|record| record.error.is_some())
                .count(),
            1
        );
        let completed: Vec<&ProgressUpdate> = sink
            .updates
            .iter()
            .filter(|update| matches!(update.kind, ProgressEventKind::StepCompleted))
            .collect();
        assert_eq!(completed.len(), 3);
        assert_eq!(
            completed.last().map(|update| update.percent_complete()),
            Some(100)
        );
    }

    #[test]
    fn execute_benchmark_plan_records_suite_in_metadata() {
        let client = ProviderClient::new(ProviderKind::Ollama, "http://127.0.0.1:1/v1", 0.1);
        let config = test_config();
        let context = BenchmarkContext {
            runs: 1,
            max_tokens: 128,
            temperature: 0.0,
            timeout: 30.0,
            options: HashMap::new(),
        };
        let benchmark = StubBenchmark {
            id: "embedding-stub",
            name: "Embedding Stub",
            total_steps: 1,
            fail_on_step: None,
        };
        let benchmarks: Vec<&dyn Benchmark> = vec![&benchmark];
        let models = vec!["model-a".to_string()];
        let mut sink = RecordingProgressSink::default();

        let run = execute_benchmark_plan(
            &client,
            &config,
            &models,
            &benchmarks,
            BenchmarkSuite::Embeddings,
            context,
            0,
            &mut sink,
        )
        .unwrap();

        assert_eq!(
            run.config.get("suite").and_then(|value| value.as_str()),
            Some("embeddings")
        );
    }

    #[test]
    fn save_outputs_rejects_invalid_export_choice() {
        let config = test_config();
        let run = BenchmarkRun {
            run_id: "test-run".to_string(),
            created_at: "2026-06-15T13:00:00Z".to_string(),
            models: vec!["qwen3.5:2b".to_string()],
            benchmark_ids: vec!["chat-generation".to_string()],
            config: HashMap::new(),
            results: Vec::new(),
            schema_version: "2.0".to_string(),
            run_kind: Some(BenchmarkRunKind::Benchmark),
            environment: None,
            performance_plan: None,
            quality_plan: None,
            provider_capabilities: None,
            model_load_measurements: None,
            model_inventory_measurements: None,
            telemetry_summary: None,
        };

        let error = super::save_outputs(&config, &run, "raw", "none", None)
            .expect_err("expected invalid export choice to fail");
        assert!(error
            .to_string()
            .contains("Invalid --export 'raw'. Use one of: json, csv, both, none."));
    }

    #[test]
    fn save_outputs_rejects_invalid_report_choice() {
        let config = test_config();
        let run = BenchmarkRun {
            run_id: "test-run".to_string(),
            created_at: "2026-06-15T13:00:00Z".to_string(),
            models: vec!["qwen3.5:2b".to_string()],
            benchmark_ids: vec!["chat-generation".to_string()],
            config: HashMap::new(),
            results: Vec::new(),
            schema_version: "2.0".to_string(),
            run_kind: Some(BenchmarkRunKind::Benchmark),
            environment: None,
            performance_plan: None,
            quality_plan: None,
            provider_capabilities: None,
            model_load_measurements: None,
            model_inventory_measurements: None,
            telemetry_summary: None,
        };

        let error = super::save_outputs(&config, &run, "none", "pdf", None)
            .expect_err("expected invalid report choice to fail");
        assert!(error
            .to_string()
            .contains("Invalid --report 'pdf'. Use one of: md, html, both, none."));
    }
}
