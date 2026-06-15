use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize;
use inquire::{CustomType, MultiSelect, Select, Text};
use serde_json::Value;
use tabled::{
    builder::Builder,
    settings::{Panel, Style},
};

use crate::benchmarks::registry::{default_registry, BenchmarkSuite};
use crate::cli::{EXPORT_CHOICES, REPORT_CHOICES};
use crate::config::AppConfig;
use crate::progress::TerminalProgressRenderer;
use crate::providers::{ProviderClient, ProviderKind, ProviderStatus};
use crate::quality::catalog::default_catalog;
use crate::reporting::{build_summary_rows, render_markdown_report};
use crate::results::BenchmarkRun;
use crate::runner;

const APP_TITLE: &str = "LLMeter";
const APP_SUBTITLE: &str = "Local LLM benchmarking for OpenAI-compatible providers";

pub fn print_header() {
    let subtitle = APP_SUBTITLE.dimmed();
    println!("\n{}", APP_TITLE.bold());
    println!("{}\n", subtitle);
}

pub fn print_status_panel(status: &ProviderStatus) {
    let mut builder = Builder::new();
    builder.push_record(vec!["Check", "Value"]);
    builder.push_record(vec!["Provider", status.provider.display_name()]);
    builder.push_record(vec!["Base URL", &status.base_url]);
    builder.push_record(vec![
        "API reachable",
        if status.running { "yes" } else { "no" },
    ]);
    builder.push_record(vec!["Models exposed", &status.models.to_string()]);
    if let Some(error) = &status.error {
        builder.push_record(vec!["Error", error]);
    }
    let mut table = builder.build();
    table.with(Panel::header("Provider status"));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn print_provider_catalog() {
    let mut builder = Builder::new();
    builder.push_record(vec!["Provider", "Default /v1 base URL", "Notes"]);
    for provider in [
        ProviderKind::Ollama,
        ProviderKind::Lmstudio,
        ProviderKind::LlamaCpp,
        ProviderKind::OpenaiCompatible,
    ] {
        let notes = match provider {
            ProviderKind::Ollama => "Ollama OpenAI-compatible API",
            ProviderKind::Lmstudio => "LM Studio local server",
            ProviderKind::LlamaCpp => "llama.cpp server",
            ProviderKind::OpenaiCompatible => "Any local OpenAI-compatible server",
        };
        builder.push_record(vec![
            provider.label().to_string(),
            provider.default_base_url().to_string(),
            notes.to_string(),
        ]);
    }
    let mut table = builder.build();
    table.with(Panel::header("Supported providers"));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn print_models(models: &[Value], provider: ProviderKind) {
    if models.is_empty() {
        println!(
            "No models found from {}. Start the provider server and load or expose a model.",
            provider.display_name()
        );
        return;
    }

    let mut builder = Builder::new();
    builder.push_record(vec!["#", "Model", "Owner", "Object"]);

    for (index, model) in models.iter().enumerate() {
        builder.push_record(vec![
            (index + 1).to_string(),
            model
                .get("id")
                .or_else(|| model.get("name"))
                .or_else(|| model.get("model"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            model
                .get("owned_by")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            model
                .get("object")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Panel::header(format!("{} models", provider.display_name())));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn print_benchmark_catalog(suite: Option<BenchmarkSuite>) {
    let registry = default_registry();
    let mut builder = Builder::new();
    builder.push_record(vec!["#", "Suite", "ID", "Name", "Description"]);

    let benchmarks: Vec<_> = match suite {
        Some(selected_suite) => registry.all_in_suite(selected_suite),
        None => registry
            .all()
            .iter()
            .map(|benchmark| benchmark.as_ref())
            .collect(),
    };

    for (index, benchmark) in benchmarks.iter().enumerate() {
        builder.push_record(vec![
            (index + 1).to_string(),
            benchmark.suite().label().to_string(),
            benchmark.id().to_string(),
            benchmark.name().to_string(),
            benchmark.description().to_string(),
        ]);
    }

    let mut table = builder.build();
    let title = suite
        .map(|selected_suite| format!("{} benchmark catalog", selected_suite.label()))
        .unwrap_or_else(|| "Benchmark catalog".to_string());
    table.with(Panel::header(title));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn print_quality_catalog() {
    let catalog = default_catalog();
    let mut builder = Builder::new();
    builder.push_record(vec![
        "ID",
        "Name",
        "Family",
        "Framework",
        "Metric",
        "Code exec",
    ]);
    for entry in catalog {
        builder.push_record(vec![
            entry.id,
            entry.display_name,
            format!("{:?}", entry.family),
            entry.framework_hint.label().to_string(),
            entry.default_metric,
            if entry.requires_code_execution {
                "yes".to_string()
            } else {
                "no".to_string()
            },
        ]);
    }
    let mut table = builder.build();
    table.with(Panel::header("Quality benchmark catalog"));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn summarize_run(run: &BenchmarkRun) {
    let errors: Vec<_> = run.results.iter().filter(|r| r.error.is_some()).collect();
    let ok = run.results.len() - errors.len();
    let models_str = run.models.join(", ");
    let records_str = format!(
        "{} total, {ok} ok, {} errors",
        run.results.len(),
        errors.len()
    );

    println!("\n{}", "Benchmark run".bold());
    println!("  {}: {}", "Run ID".dimmed(), run.run_id);
    println!("  {}: {}", "Created".dimmed(), run.created_at);
    println!("  {}: {}", "Models".dimmed(), models_str);
    println!("  {}: {}", "Records".dimmed(), records_str);
    println!();

    let summary_rows = build_summary_rows(run);
    if summary_rows.is_empty() {
        println!("{}", "No results.".dimmed());
        return;
    }

    let mut builder = Builder::new();
    builder.push_record(vec![
        "Model",
        "Benchmark",
        "Prompt",
        "Runs",
        "Errors",
        "Avg wall ms",
        "Avg TTFT ms",
        "Avg tok/s",
        "Similarity",
    ]);

    for row in &summary_rows {
        builder.push_record(vec![
            &row.model,
            &row.benchmark_id,
            row.prompt_name.as_deref().unwrap_or("all"),
            &row.records.to_string(),
            &row.errors.to_string(),
            &fmt(row.avg_wall_time_ms),
            &fmt(row.avg_time_to_first_token_ms),
            &fmt(row.avg_tokens_per_second),
            &fmt_digits(row.mean_similarity, 4),
        ]);
    }

    let mut table = builder.build();
    table.with(Panel::header("Aggregated summary"));
    table.with(Style::rounded());
    println!("{table}");

    if !errors.is_empty() {
        let mut err_builder = Builder::new();
        err_builder.push_record(vec!["Model", "Benchmark", "Prompt", "Error"]);
        for record in errors.iter().take(20) {
            err_builder.push_record(vec![
                &record.model,
                &record.benchmark_id,
                record.prompt_name.as_deref().unwrap_or(""),
                record.error.as_deref().unwrap_or(""),
            ]);
        }
        let mut err_table = err_builder.build();
        err_table.with(Panel::header("Errors"));
        err_table.with(Style::rounded());
        println!("{err_table}");
    }
}

pub fn print_terminal_report(markdown: &str) {
    println!("{markdown}");
}

pub fn menu(prompt: &str, choices: &[&str]) -> Result<usize> {
    let selected = Select::new(prompt, choices.to_vec())
        .prompt()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(choices.iter().position(|&c| c == selected).unwrap_or(0) + 1)
}

pub fn choose_from_menu(
    prompt: &str,
    choices: &[String],
    multi: bool,
    all_option: bool,
) -> Result<Vec<String>> {
    if choices.is_empty() {
        return Ok(Vec::new());
    }

    let mut values = choices.to_vec();
    if all_option {
        values.insert(0, "all".to_string());
    }

    if multi {
        let selected = MultiSelect::new(prompt, values)
            .prompt()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if selected.iter().any(|s| s == "all") {
            return Ok(choices.to_vec());
        }
        Ok(selected)
    } else {
        let selected = Select::new(prompt, values)
            .prompt()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if selected == "all" {
            Ok(choices.to_vec())
        } else {
            Ok(vec![selected])
        }
    }
}

pub fn choose_number(prompt: &str, default: u32, min: u32, max: Option<u32>) -> Result<u32> {
    CustomType::<u32>::new(prompt)
        .with_default(default)
        .with_validator(move |value: &u32| {
            if *value < min {
                return Ok(inquire::validator::Validation::Invalid(
                    format!("Must be at least {min}.").into(),
                ));
            }
            if let Some(max_value) = max {
                if *value > max_value {
                    return Ok(inquire::validator::Validation::Invalid(
                        format!("Must be at most {max_value}.").into(),
                    ));
                }
            }
            Ok(inquire::validator::Validation::Valid)
        })
        .prompt()
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn ask_choice(prompt: &str, choices: &[&str], default: &str) -> Result<String> {
    Select::new(prompt, choices.to_vec())
        .with_starting_cursor(choices.iter().position(|&c| c == default).unwrap_or(0))
        .prompt()
        .map_err(|e| anyhow::anyhow!("{e}"))
        .map(|s| s.to_string())
}

pub fn pause() {
    let _ = Text::new("Press Enter to continue").prompt();
}

pub fn print_saved_paths(paths: &[PathBuf]) {
    if paths.is_empty() {
        return;
    }
    let mut builder = Builder::new();
    builder.push_record(vec!["Type", "Path"]);
    for path in paths {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        builder.push_record(vec![ext, path.to_string_lossy().to_string()]);
    }
    let mut table = builder.build();
    table.with(Panel::header("Saved files"));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn print_file_list(paths: &[PathBuf], title: &str) {
    if paths.is_empty() {
        println!("{} No files found.", "Info:".dimmed());
        return;
    }
    let mut builder = Builder::new();
    builder.push_record(vec!["#", "File", "Modified"]);
    for (index, path) in paths.iter().enumerate() {
        let modified = path
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                use std::time::SystemTime;
                let duration = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
                duration.as_secs().to_string()
            })
            .unwrap_or_default();
        builder.push_record(vec![
            (index + 1).to_string(),
            path.to_string_lossy().to_string(),
            modified,
        ]);
    }
    let mut table = builder.build();
    table.with(Panel::header(title));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn main_menu(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    loop {
        print_header();
        print_status_panel(&client.status());
        let choice = menu(
            "Main menu",
            &[
                "List providers",
                "List models",
                "Benchmark workspace",
                "Reports",
                "Help",
                "Exit",
            ],
        )?;

        match choice {
            1 => {
                print_provider_catalog();
                pause();
            }
            2 => {
                match client.list_models() {
                    Ok(models) => print_models(&models, config.provider),
                    Err(e) => println!("{} {e}", "Error:".red()),
                }
                pause();
            }
            3 => benchmark_menu(config, client)?,
            4 => report_menu(config)?,
            5 => {
                print_help_topic(None);
                pause();
            }
            6 => return Ok(()),
            _ => {}
        }
    }
}

pub fn benchmark_menu(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    loop {
        print_header();
        let choice = menu(
            "Benchmark workspace",
            &[
                "View available benchmark tests",
                "Run a guided benchmark",
                "View latest result as terminal report",
                "Generate report from saved result",
                "Back",
            ],
        )?;

        match choice {
            1 => {
                print_benchmark_catalog(None);
                pause();
            }
            2 => guided_benchmark_run(config, client)?,
            3 => {
                show_latest_report(config);
                pause();
            }
            4 => generate_report_interactive(config)?,
            5 => return Ok(()),
            _ => {}
        }
    }
}

fn guided_benchmark_run(config: &AppConfig, _client: &ProviderClient) -> Result<()> {
    let provider_choice = ask_choice(
        "Provider for this benchmark run",
        &["ollama", "lmstudio", "llama-cpp", "openai-compatible"],
        config.provider.label(),
    )?;
    let provider = provider_choice.parse().unwrap_or(config.provider);
    let run_config = config.with_provider(provider);
    let run_client = ProviderClient::new(
        run_config.provider,
        &run_config.base_url,
        run_config.timeout,
    );

    let suite_choice = ask_choice(
        "Benchmark suite",
        &["llm", "embeddings"],
        BenchmarkSuite::Llm.label(),
    )?;
    let suite = if suite_choice == "embeddings" {
        BenchmarkSuite::Embeddings
    } else {
        BenchmarkSuite::Llm
    };

    let models = match runner::installed_model_names(&run_client) {
        Ok(models) => models,
        Err(error) => {
            println!("{} {error}", "Error:".red());
            return Ok(());
        }
    };
    let selected_models = choose_from_menu("Choose model or models", &models, true, true)?;
    if selected_models.is_empty() {
        println!("{} No models selected.", "Warning:".yellow());
        return Ok(());
    }

    let registry = default_registry();
    let benchmark_ids: Vec<String> = registry
        .ids_for_suite(suite)
        .iter()
        .map(|s| s.to_string())
        .collect();
    let selected_benchmark_ids =
        choose_from_menu("Choose benchmark or benchmarks", &benchmark_ids, true, true)?;
    if selected_benchmark_ids.is_empty() {
        println!("{} No benchmarks selected.", "Warning:".yellow());
        return Ok(());
    }

    let all_benchmarks = {
        let selected_set: std::collections::HashSet<&str> =
            selected_benchmark_ids.iter().map(|s| s.as_str()).collect();
        let ids_set: std::collections::HashSet<&str> =
            benchmark_ids.iter().map(|s| s.as_str()).collect();
        selected_set == ids_set
    };

    let runs = choose_number("Runs per benchmark", config.default_runs, 1, None)?;
    let max_tokens = choose_number("Max output tokens", config.default_max_tokens, 1, None)?;
    let raw_temperature = Text::new("temperature")
        .with_default(&config.default_temperature.to_string())
        .prompt()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let temperature: f64 = raw_temperature
        .parse()
        .unwrap_or(config.default_temperature);

    let export = ask_choice("Save raw results", EXPORT_CHOICES, "both")?;
    let report = ask_choice("Generate formatted report", REPORT_CHOICES, "both")?;

    let selected_ids: Option<Vec<String>> = if all_benchmarks {
        None
    } else {
        Some(selected_benchmark_ids.clone())
    };

    let extra_options = std::collections::HashMap::new();
    let mut progress = TerminalProgressRenderer::new();
    let run = runner::run_benchmarks(
        &run_client,
        &run_config,
        runner::BenchmarkRunRequest {
            suite,
            model_names: &selected_models,
            benchmark_ids: selected_ids.as_deref(),
            all_benchmarks,
            runs,
            max_tokens,
            temperature,
            extra_options: &extra_options,
        },
        2,
        Some(&mut progress),
    )?;

    let saved = runner::save_outputs(&run_config, &run, &export, &report, Some(&mut progress))?;
    summarize_run(&run);
    print_saved_paths(&saved);
    pause();
    Ok(())
}

fn show_latest_report(config: &AppConfig) {
    use crate::results::ResultStore;
    let store = ResultStore::new(&config.output_dir);
    let files = store.latest_json_files(10);
    if files.is_empty() {
        println!("{} No JSON benchmark result files found.", "Info:".dimmed());
        return;
    }
    let selected = choose_from_menu(
        "Select a result file",
        &files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>(),
        false,
        false,
    );
    match selected {
        Ok(s) if !s.is_empty() => {
            let path = PathBuf::from(&s[0]);
            match store.load_json(&path) {
                Ok(run) => {
                    let markdown = render_markdown_report(&run);
                    print_terminal_report(&markdown);
                }
                Err(e) => println!("{} {e}", "Error:".red()),
            }
        }
        _ => {}
    }
}

fn generate_report_interactive(config: &AppConfig) -> Result<()> {
    use crate::results::ResultStore;
    let store = ResultStore::new(&config.output_dir);
    let files = store.latest_json_files(10);
    if files.is_empty() {
        println!("{} No JSON benchmark result files found.", "Info:".dimmed());
        return Ok(());
    }
    let selected = choose_from_menu(
        "Select a result file",
        &files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>(),
        false,
        false,
    )?;

    if selected.is_empty() {
        return Ok(());
    }

    let path = PathBuf::from(&selected[0]);
    let report_format = ask_choice("Report format", REPORT_CHOICES, "both")?;
    if report_format == "none" {
        println!("{} No report generated.", "Warning:".yellow());
        return Ok(());
    }

    let run = store.load_json(&path)?;
    let saved = runner::save_outputs(config, &run, "none", &report_format, None)?;
    print_saved_paths(&saved);
    Ok(())
}

fn report_menu(config: &AppConfig) -> Result<()> {
    loop {
        print_header();
        let choice = menu(
            "Reports",
            &[
                "List saved result and report files",
                "View latest result as terminal report",
                "Generate Markdown or HTML report",
                "Back",
            ],
        )?;

        match choice {
            1 => {
                use crate::results::ResultStore;
                let store = ResultStore::new(&config.output_dir);
                print_file_list(&store.latest_json_files(10), "Saved JSON results");
                print_file_list(&store.latest_report_files(10), "Generated reports");
                pause();
            }
            2 => {
                show_latest_report(config);
                pause();
            }
            3 => {
                generate_report_interactive(config)?;
                pause();
            }
            4 => return Ok(()),
            _ => {}
        }
    }
}

pub fn print_help_topic(topic: Option<&str>) {
    match topic.unwrap_or("overview").to_ascii_lowercase().as_str() {
        "providers" => {
            println!("Providers:");
            println!("  --provider ollama             default http://localhost:11434/v1");
            println!("  --provider lmstudio           default http://localhost:1234/v1");
            println!("  --provider llama-cpp          default http://localhost:8080/v1");
            println!("  --provider openai-compatible  use with --base-url");
        }
        "bench" | "benchmarks" => {
            println!("Benchmark examples:");
            println!("  llmeter bench list --suite llm");
            println!(
                "  llmeter --provider lmstudio bench run --suite llm --models all --benchmarks all"
            );
            println!("  llmeter bench run --provider ollama --suite embeddings --models all --benchmarks all");
            println!("  llmeter bench perf --models all --profile smoke --export json --report md");
        }
        "reports" => {
            println!("Reports:");
            println!("  llmeter report list");
            println!("  llmeter report show");
            println!("  llmeter report generate --format both");
        }
        "examples" => {
            println!("Examples:");
            println!("  llmeter status");
            println!("  llmeter providers list");
            println!("  llmeter providers set ollama");
            println!("  llmeter models");
            println!("  llmeter bench run --suite llm --models all --benchmarks all --runs 3 --max-tokens 128");
            println!("  llmeter quality list");
            println!("  llmeter quality plan --framework lighteval --task leaderboard|mmlu|5 --model llama3.1");
        }
        _ => {
            println!("LLMeter help topics: providers, bench, reports, examples");
            println!("Use `llmeter help <topic>` or normal CLI help with `llmeter --help`.");
        }
    }
}

fn fmt(value: Option<f64>) -> String {
    match value {
        Some(v) if v.is_finite() => format!("{:.2}", v),
        _ => String::new(),
    }
}

fn fmt_digits(value: Option<f64>, digits: usize) -> String {
    match value {
        Some(v) if v.is_finite() => format!("{:.digits$}", v),
        _ => String::new(),
    }
}
