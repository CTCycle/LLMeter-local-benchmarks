use std::path::PathBuf;

use serde_json::Value;

use crate::benchmarks::base::BenchmarkContext;
use crate::benchmarks::registry::default_registry;
use crate::cli::ReportCommands;
use crate::config::AppConfig;
use crate::errors::LLMeterError;
use crate::providers::ProviderClient;
use crate::reporting::{save_html_report, save_markdown_report};
use crate::results::{BenchmarkRun, ResultStore};
use crate::utils::utc_now_iso;

pub struct BenchmarkRunRequest<'a> {
    pub model_names: &'a [String],
    pub benchmark_ids: Option<&'a [String]>,
    pub all_benchmarks: bool,
    pub runs: u32,
    pub max_tokens: u32,
    pub temperature: f64,
    pub extra_options: &'a std::collections::HashMap<String, Value>,
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
) -> anyhow::Result<BenchmarkRun> {
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
    let benchmarks = registry.select(request.benchmark_ids, request.all_benchmarks)?;

    let context = BenchmarkContext {
        runs: request.runs,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        timeout: config.timeout,
        options: request.extra_options.clone(),
    };

    let store = ResultStore::new(&config.output_dir);
    let run_id = store.new_run_id(&models);

    let mut run = BenchmarkRun {
        run_id,
        created_at: utc_now_iso(),
        models: models.clone(),
        benchmark_ids: benchmarks.iter().map(|b| b.id().to_string()).collect(),
        config: {
            let mut c = std::collections::HashMap::new();
            c.insert(
                "provider".to_string(),
                Value::from(config.provider.to_string()),
            );
            c.insert("base_url".to_string(), Value::from(config.base_url.clone()));
            c.insert("runs".to_string(), Value::from(request.runs));
            c.insert("max_tokens".to_string(), Value::from(request.max_tokens));
            c.insert("temperature".to_string(), Value::from(request.temperature));
            c.insert("timeout".to_string(), Value::from(config.timeout));
            c
        },
        results: Vec::new(),
    };

    for model in &models {
        for benchmark in &benchmarks {
            run.results.extend(benchmark.run(client, model, &context));
        }
    }

    Ok(run)
}

pub fn save_outputs(
    config: &AppConfig,
    run: &BenchmarkRun,
    export: &str,
    report: &str,
) -> anyhow::Result<Vec<PathBuf>> {
    let store = ResultStore::new(&config.output_dir);
    let mut saved: Vec<PathBuf> = Vec::new();

    if matches!(export, "json" | "both") {
        saved.push(store.save_json(run)?);
    }
    if matches!(export, "csv" | "both") {
        saved.push(store.save_csv(run)?);
    }
    if matches!(report, "md" | "both") {
        saved.push(save_markdown_report(run, &config.output_dir)?);
    }
    if matches!(report, "html" | "both") {
        saved.push(save_html_report(run, &config.output_dir)?);
    }
    Ok(saved)
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
            let saved = save_outputs(config, &run, "none", format)?;
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
