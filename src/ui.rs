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
use crate::performance::config::{
    LoadMeasurementMode, PerformancePlan, PerformanceProfile, ReportDetailLevel, TelemetryLevel,
};
use crate::performance::model_inventory::{
    measure_model_inventory_with_progress, planned_inventory_steps,
};
use crate::performance::provider_probe::{
    planned_probe_steps, probe_provider_capabilities_with_progress,
};
use crate::progress::{
    ProgressEventKind, ProgressPhase, ProgressSink, ProgressUpdate, TerminalProgressRenderer,
};
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
    builder.push_record(vec!["Provider", "Tier", "Default /v1 base URL", "Notes"]);
    for entry in ProviderKind::catalog() {
        builder.push_record(vec![
            entry.provider.label().to_string(),
            entry.tier.label().to_string(),
            entry.default_base_url.to_string(),
            entry.notes.to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Select(usize),
    Back,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptOutcome<T> {
    Selected(T),
    Back,
    Exit,
}

struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

fn menu_action_for_key(key: crossterm::event::KeyEvent) -> Option<MenuAction> {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    match key {
        KeyEvent {
            code: KeyCode::Enter,
            kind: KeyEventKind::Press,
            ..
        } => Some(MenuAction::Select(0)),
        KeyEvent {
            code: KeyCode::Left | KeyCode::Esc,
            kind: KeyEventKind::Press,
            ..
        } => Some(MenuAction::Back),
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            ..
        } => Some(MenuAction::Exit),
        _ => None,
    }
}

fn move_menu_selection(selected: usize, choices_len: usize, direction: i8) -> usize {
    if choices_len == 0 {
        return 0;
    }
    match direction {
        direction if direction < 0 => selected.saturating_sub(1),
        direction if direction > 0 => selected.saturating_add(1).min(choices_len - 1),
        _ => selected.min(choices_len - 1),
    }
}

pub fn menu(prompt: &str, choices: &[&str]) -> Result<MenuAction> {
    use crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        execute,
        terminal::{Clear, ClearType},
    };
    use std::io::{stdout, Write};

    let _raw_mode = RawModeGuard::enter()?;
    let mut stdout = stdout();
    let mut selected = 0usize;

    let result = loop {
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            Clear(ClearType::FromCursorDown)
        )?;
        writeln!(stdout, "{}", prompt.bold())?;
        writeln!(stdout)?;
        for (i, choice) in choices.iter().enumerate() {
            if i == selected {
                writeln!(stdout, "  {} {}", "▸".cyan(), choice.cyan())?;
            } else {
                writeln!(stdout, "    {}", choice)?;
            }
        }
        writeln!(stdout)?;
        write!(
            stdout,
            "{}",
            "Navigation: ↑↓ • Select: Enter • Back: ← / Esc".dimmed()
        )?;
        stdout.flush()?;

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            }) => {
                selected = move_menu_selection(selected, choices.len(), -1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            }) => {
                selected = move_menu_selection(selected, choices.len(), 1);
            }
            Event::Key(key) => match menu_action_for_key(key) {
                Some(MenuAction::Select(_)) => break MenuAction::Select(selected),
                Some(MenuAction::Back) => break MenuAction::Back,
                Some(MenuAction::Exit) => break MenuAction::Exit,
                None => {}
            },
            _ => {}
        }
    };

    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        Clear(ClearType::FromCursorDown)
    )?;
    Ok(result)
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
            .map(crate::utils::format_system_time_utc)
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
                "Provider setup",
                "Model inventory",
                "Benchmark workspace",
                "Reports and comparisons",
                "Help and examples",
                "Exit",
            ],
        )?;

        match choice {
            MenuAction::Select(0) => provider_setup_menu(config, client)?,
            MenuAction::Select(1) => model_inventory_menu(config, client)?,
            MenuAction::Select(2) => benchmark_menu(config, client)?,
            MenuAction::Select(3) => report_menu(config)?,
            MenuAction::Select(4) => {
                print_help_topic(None);
                pause();
            }
            MenuAction::Select(5) | MenuAction::Exit => return Ok(()),
            _ => {}
        }
    }
}

pub fn provider_setup_menu(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    loop {
        let choice = menu(
            "Provider setup",
            &[
                "Show current provider status",
                "List supported provider presets",
                "Probe provider capabilities",
                "Set default provider",
                "Back",
            ],
        )?;
        match choice {
            MenuAction::Select(0) => print_status_panel(&client.status()),
            MenuAction::Select(1) => print_provider_catalog(),
            MenuAction::Select(2) => probe_provider_capabilities_interactive(config, client)?,
            MenuAction::Select(3) => {
                let choices = ProviderKind::catalog()
                    .iter()
                    .map(|entry| entry.provider.label().to_string())
                    .collect::<Vec<_>>();
                let selected = choose_from_menu("Default provider", &choices, false, false)?;
                if let Some(label) = selected.first() {
                    let provider = label.parse().unwrap_or(config.provider);
                    let path = crate::config::save_global_provider(provider)?;
                    println!(
                        "Saved default provider '{}' to {}",
                        provider,
                        path.display()
                    );
                }
            }
            MenuAction::Back | MenuAction::Exit | MenuAction::Select(4) => return Ok(()),
            _ => {}
        }
        pause();
    }
}

pub fn model_inventory_menu(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    loop {
        let choice = menu(
            "Model inventory",
            &[
                "List exposed models",
                "Show raw model metadata",
                "Estimate model cache footprint",
                "Back",
            ],
        )?;
        match choice {
            MenuAction::Select(0) => match client.list_models() {
                Ok(models) => print_models(&models, config.provider),
                Err(error) => println!("{} {error}", "Error:".red()),
            },
            MenuAction::Select(1) => {
                let models = runner::installed_model_names(client)?;
                let selected = choose_from_menu("Choose model", &models, false, false)?;
                if let Some(model) = selected.first() {
                    match client.show_model(model) {
                        Ok(value) => println!("{}", serde_json::to_string_pretty(&value)?),
                        Err(error) => println!("{} {error}", "Error:".red()),
                    }
                }
            }
            MenuAction::Select(2) => estimate_model_inventory_interactive(config, client)?,
            MenuAction::Back | MenuAction::Exit | MenuAction::Select(3) => return Ok(()),
            _ => {}
        }
        pause();
    }
}

pub fn benchmark_menu(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    loop {
        print_header();
        let choice = menu(
            "Benchmark workspace",
            &[
                "Quick benchmark",
                "Performance benchmark",
                "Standard LLM benchmark",
                "Embeddings benchmark",
                "Quality benchmark plan",
                "View benchmark catalog",
                "View latest result as terminal report",
                "Generate report from saved result",
                "Back",
            ],
        )?;

        match choice {
            MenuAction::Select(0) => guided_quick_benchmark_run(config, client)?,
            MenuAction::Select(1) => guided_performance_run(config, client)?,
            MenuAction::Select(2) => guided_standard_llm_run(config, client)?,
            MenuAction::Select(3) => guided_embeddings_run(config, client)?,
            MenuAction::Select(4) => guided_quality_plan(),
            MenuAction::Select(5) => {
                print_benchmark_catalog(None);
                pause();
            }
            MenuAction::Select(6) => {
                show_latest_report(config);
                pause();
            }
            MenuAction::Select(7) => generate_report_interactive(config)?,
            MenuAction::Back | MenuAction::Exit | MenuAction::Select(8) => return Ok(()),
            _ => {}
        }
    }
}

fn provider_labels() -> Vec<String> {
    ProviderKind::catalog()
        .iter()
        .map(|entry| entry.provider.label().to_string())
        .collect()
}

fn guided_quick_benchmark_run(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    guided_performance_run_with_profile(config, client, PerformanceProfile::Smoke, true)
}

fn guided_standard_llm_run(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    guided_benchmark_run_for_suite(config, client, BenchmarkSuite::Llm)
}

fn guided_embeddings_run(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    guided_benchmark_run_for_suite(config, client, BenchmarkSuite::Embeddings)
}

fn guided_quality_plan() {
    print_quality_catalog();
    println!(
        "Use `llmeter quality plan --framework <name> --task <task> --model <model>` to build an executable dry-run plan."
    );
    pause();
}

fn guided_benchmark_run_for_suite(
    config: &AppConfig,
    _client: &ProviderClient,
    fixed_suite: BenchmarkSuite,
) -> Result<()> {
    guided_benchmark_run_inner(config, fixed_suite)
}

fn guided_benchmark_run_inner(config: &AppConfig, suite: BenchmarkSuite) -> Result<()> {
    let provider_choices = provider_labels();
    let provider_choice = ask_choice(
        "Provider for this benchmark run",
        &provider_choices
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        config.provider.label(),
    )?;
    let provider = provider_choice.parse().unwrap_or(config.provider);
    let run_config = config.with_provider(provider)?;
    let run_client = match ProviderClient::new(
        run_config.provider,
        &run_config.base_url,
        run_config.timeout,
    ) {
        Ok(client) => client,
        Err(error) => {
            println!("{} {error}", "Error:".red());
            return Ok(());
        }
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

    let saved = runner::save_outputs(
        &run_config,
        &run,
        &export,
        &report,
        crate::results::OutputPrivacyPolicy::default(),
        Some(&mut progress),
    )?;
    summarize_run(&run);
    print_saved_paths(&saved);
    pause();
    Ok(())
}

fn guided_performance_run(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    guided_performance_run_with_profile(config, client, PerformanceProfile::Latency, false)
}

fn guided_performance_run_with_profile(
    config: &AppConfig,
    _client: &ProviderClient,
    default_profile: PerformanceProfile,
    quick: bool,
) -> Result<()> {
    let provider_choices = provider_labels();
    let provider_choice = ask_choice(
        "Provider for this performance run",
        &provider_choices
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        config.provider.label(),
    )?;
    let provider = provider_choice.parse().unwrap_or(config.provider);
    let run_config = config.with_provider(provider)?;
    let run_client = match ProviderClient::new(
        run_config.provider,
        &run_config.base_url,
        run_config.timeout,
    ) {
        Ok(client) => client,
        Err(error) => {
            println!("{} {error}", "Error:".red());
            return Ok(());
        }
    };

    let probe_choice = ask_choice(
        "Capability probe",
        &["basic probe", "full endpoint probe", "skip probe"],
        "basic probe",
    )?;
    let models = match runner::installed_model_names(&run_client) {
        Ok(models) => models,
        Err(error) => {
            println!("{} {error}", "Error:".red());
            return Ok(());
        }
    };
    let selected_models = choose_from_menu("Choose model or models", &models, !quick, true)?;
    if selected_models.is_empty() {
        println!("{} No models selected.", "Warning:".yellow());
        return Ok(());
    }

    let profile_choice = if quick {
        "smoke".to_string()
    } else {
        ask_choice(
            "Performance profile",
            &["smoke", "latency", "throughput", "sweep"],
            default_profile.label(),
        )?
    };
    let profile = profile_choice.parse().unwrap_or(default_profile);
    let runs = choose_number(
        "Measured runs per scenario",
        if quick { 1 } else { 3 },
        1,
        None,
    )?;
    let warmup = choose_number("Warmup requests", 1, 0, None)?;
    let stream_choice = ask_choice("Streaming", &["yes", "no"], "yes")?;
    let export = ask_choice("Save raw results", EXPORT_CHOICES, "both")?;
    let report = ask_choice(
        "Generate formatted report",
        REPORT_CHOICES,
        if quick { "md" } else { "both" },
    )?;

    let plan = PerformancePlan::from_cli(
        run_config.provider,
        selected_models,
        profile,
        None,
        None,
        None,
        Some(warmup),
        Some(runs),
        stream_choice == "yes",
        None,
        std::collections::HashMap::new(),
        LoadMeasurementMode::WarmBaseline,
        2,
        TelemetryLevel::Standard,
        1000,
        None,
        probe_choice != "skip probe",
        probe_choice == "full endpoint probe",
        None,
        false,
        ReportDetailLevel::Detailed,
        None,
    )?;

    print_performance_plan_preview(&plan);
    let confirm = ask_choice("Run this benchmark plan", &["yes", "no"], "yes")?;
    if confirm != "yes" {
        return Ok(());
    }

    let mut progress = TerminalProgressRenderer::new();
    let run = crate::performance::runner::run_performance_plan(
        &run_config,
        &run_client,
        plan,
        Some(&mut progress),
    )?;
    let saved = runner::save_outputs(
        &run_config,
        &run,
        &export,
        &report,
        crate::results::OutputPrivacyPolicy::default(),
        Some(&mut progress),
    )?;
    summarize_run(&run);
    print_saved_paths(&saved);
    pause();
    Ok(())
}

fn print_performance_plan_preview(plan: &PerformancePlan) {
    let mut builder = Builder::new();
    builder.push_record(vec!["Setting", "Value"]);
    builder.push_record(vec!["Provider", plan.provider.label()]);
    builder.push_record(vec!["Models", &plan.models.join(", ")]);
    builder.push_record(vec!["Profile", plan.profile.label()]);
    builder.push_record(vec![
        "Prompt sizes",
        &format!("{:?}", plan.prompt_sizes.estimated_tokens),
    ]);
    builder.push_record(vec![
        "Output sizes",
        &format!("{:?}", plan.output_sizes.estimated_tokens),
    ]);
    builder.push_record(vec![
        "Concurrency",
        &format!("{:?}", plan.concurrency.levels),
    ]);
    builder.push_record(vec!["Warmup", &plan.warmup.requests.to_string()]);
    builder.push_record(vec!["Runs", &plan.runs.to_string()]);
    builder.push_record(vec!["Streaming", if plan.stream { "yes" } else { "no" }]);
    builder.push_record(vec![
        "Capability probe",
        if plan.probe_all_endpoints {
            "full"
        } else if plan.probe_capabilities {
            "basic"
        } else {
            "skip"
        },
    ]);
    let mut table = builder.build();
    table.with(Panel::header("Benchmark plan preview"));
    table.with(Style::rounded());
    println!("{table}");
}

fn probe_provider_capabilities_interactive(
    _config: &AppConfig,
    client: &ProviderClient,
) -> Result<()> {
    let full = ask_choice("Probe depth", &["basic", "full"], "basic")? == "full";
    let models = runner::installed_model_names(client).unwrap_or_default();
    let plan = PerformancePlan::from_cli(
        client.provider(),
        models,
        PerformanceProfile::Smoke,
        None,
        None,
        None,
        Some(0),
        Some(1),
        false,
        None,
        std::collections::HashMap::new(),
        LoadMeasurementMode::Off,
        1,
        TelemetryLevel::Standard,
        1000,
        None,
        true,
        full,
        None,
        false,
        ReportDetailLevel::Summary,
        None,
    )?;
    let total_units = planned_probe_steps(&plan);
    let mut progress = TerminalProgressRenderer::new();
    let report =
        probe_provider_capabilities_with_progress(client, &plan, &mut progress, 0, total_units);
    progress.on_update(ProgressUpdate {
        kind: ProgressEventKind::Finished,
        phase: ProgressPhase::Completed,
        message: "Capability probe complete".to_string(),
        completed_units: total_units,
        total_units,
        model_name: None,
        model_index: None,
        total_models: Some(plan.models.len()),
        benchmark_id: Some("provider-probe".to_string()),
        benchmark_name: Some("Provider capability probe".to_string()),
        benchmark_index: Some(total_units as usize),
        total_benchmarks: Some(total_units as usize),
        step_index: Some(total_units),
        total_steps: Some(total_units),
        run_index: None,
        prompt_name: None,
    });
    let mut builder = Builder::new();
    builder.push_record(vec![
        "Endpoint",
        "Method",
        "Path",
        "Supported",
        "Latency ms",
        "Error",
    ]);
    for endpoint in report.endpoints {
        builder.push_record(vec![
            endpoint.name,
            endpoint.method,
            endpoint.path,
            endpoint.supported.to_string(),
            endpoint
                .latency_ms
                .map(|value| format!("{value:.2}"))
                .unwrap_or_default(),
            endpoint.error.unwrap_or_default(),
        ]);
    }
    let mut table = builder.build();
    table.with(Panel::header(format!(
        "{} capability probe",
        client.provider().display_name()
    )));
    table.with(Style::rounded());
    println!("{table}");
    if !report.warnings.is_empty() {
        println!("{}", "Warnings".yellow().bold());
        for warning in report.warnings {
            println!("- {warning}");
        }
    }
    Ok(())
}

fn estimate_model_inventory_interactive(config: &AppConfig, client: &ProviderClient) -> Result<()> {
    let models = runner::installed_model_names(client)?;
    let selected = choose_from_menu("Choose model or models", &models, true, true)?;
    if selected.is_empty() {
        return Ok(());
    }
    let default_path = config
        .provider
        .default_cache_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let hint = "Model cache directory to scan (leave empty to skip)";
    let cache_dir = Text::new(hint)
        .with_default(&default_path)
        .prompt()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    let scan = !cache_dir.trim().is_empty();
    let plan = PerformancePlan::from_cli(
        config.provider,
        selected.clone(),
        PerformanceProfile::Smoke,
        None,
        None,
        None,
        Some(0),
        Some(1),
        true,
        None,
        std::collections::HashMap::new(),
        LoadMeasurementMode::Off,
        1,
        TelemetryLevel::Standard,
        1000,
        None,
        false,
        false,
        scan.then(|| cache_dir.trim().to_string()),
        scan,
        ReportDetailLevel::Summary,
        None,
    )?;
    let total_units = planned_inventory_steps(&plan, selected.len());
    let mut progress = TerminalProgressRenderer::new();
    let measurements = measure_model_inventory_with_progress(
        client,
        &selected,
        &plan,
        &mut progress,
        0,
        total_units,
    );
    progress.on_update(ProgressUpdate {
        kind: ProgressEventKind::Finished,
        phase: ProgressPhase::Completed,
        message: "Model inventory complete".to_string(),
        completed_units: total_units,
        total_units,
        model_name: None,
        model_index: None,
        total_models: Some(selected.len()),
        benchmark_id: Some("model-inventory".to_string()),
        benchmark_name: Some("Model inventory".to_string()),
        benchmark_index: Some(total_units as usize),
        total_benchmarks: Some(total_units as usize),
        step_index: Some(total_units),
        total_steps: Some(total_units),
        run_index: None,
        prompt_name: None,
    });
    let mut builder = Builder::new();
    builder.push_record(vec![
        "Model",
        "Metadata ms",
        "Metadata bytes",
        "Cache",
        "Notes",
    ]);
    for item in measurements {
        builder.push_record(vec![
            item.model,
            item.metadata_latency_ms
                .map(|value| format!("{value:.2}"))
                .unwrap_or_default(),
            item.metadata_payload_bytes
                .map(|value| value.to_string())
                .unwrap_or_default(),
            fmt_bytes(item.cache_bytes),
            item.notes.join("; "),
        ]);
    }
    let mut table = builder.build();
    table.with(Panel::header("Model cache and metadata"));
    table.with(Style::rounded());
    println!("{table}");
    if !scan {
        if let Some(path) = config.provider.default_cache_dir() {
            println!(
                "{} Enter a cache directory path to measure disk usage (e.g. {})",
                "Hint:".yellow(),
                path.display()
            );
        } else {
            println!(
                "{} Enter a cache directory path to measure disk usage.",
                "Hint:".yellow()
            );
        }
    }
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
    let mut progress = TerminalProgressRenderer::new();
    let saved = runner::save_outputs(
        config,
        &run,
        "none",
        &report_format,
        crate::results::OutputPrivacyPolicy::default(),
        Some(&mut progress),
    )?;
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
            MenuAction::Select(0) => {
                use crate::results::ResultStore;
                let store = ResultStore::new(&config.output_dir);
                print_file_list(&store.latest_json_files(10), "Saved JSON results");
                print_file_list(&store.latest_report_files(10), "Generated reports");
                pause();
            }
            MenuAction::Select(1) => {
                show_latest_report(config);
                pause();
            }
            MenuAction::Select(2) => {
                generate_report_interactive(config)?;
                pause();
            }
            MenuAction::Back | MenuAction::Exit | MenuAction::Select(3) => return Ok(()),
            _ => {}
        }
    }
}

pub fn print_help_topic(topic: Option<&str>) {
    match topic.unwrap_or("overview").to_ascii_lowercase().as_str() {
        "providers" => {
            println!("Providers — Supported OpenAI-compatible LLM backends:");
            println!(
                "  Use `llmeter providers list` for the full catalog with compatibility tiers"
            );
            println!("  and default /v1 base URLs (Ollama, LM Studio, llama.cpp, vLLM, etc.).");
            println!("  Use `llmeter providers set <name>` to persist a default provider.");
        }
        "bench" | "benchmarks" => {
            println!("Benchmarks — Generation latency, consistency, performance under load:");
            println!(
                "  Standard benchmarks  : llmeter bench run (chat, JSON, tool calls, embeddings)"
            );
            println!(
                "  Performance bench    : llmeter bench perf (latency/throughput/TTFT profiles)"
            );
            println!("  Interactive menu     : llmeter menu");
            println!();
            println!("Examples:");
            println!("  llmeter bench list --suite llm");
            println!(
                "  llmeter --provider lmstudio bench run --suite llm --models all --benchmarks all"
            );
            println!(
                "  llmeter bench run --provider ollama --suite embeddings --models all --benchmarks all"
            );
            println!("  llmeter bench perf --models all --profile smoke --export json --report md");
            println!(
                "  llmeter bench performance --models all --profile latency --probe-capabilities --telemetry full"
            );
        }
        "reports" => {
            println!("Reports — View and export saved benchmark results:");
            println!(
                "  llmeter report list       List recent JSON result files and generated reports"
            );
            println!("  llmeter report show       Display a saved result as a terminal report");
            println!("  llmeter report generate   Export a saved result as Markdown or HTML");
        }
        "install" | "lifecycle" => {
            println!("Install and lifecycle — Manage the LLMeter binary:");
            println!("  llmeter install           Install to the managed CLI home directory");
            println!("  llmeter update            Replace with a newer binary");
            println!("  llmeter uninstall         Remove the managed install");
            println!();
            println!("Examples:");
            println!("  llmeter install");
            println!("  llmeter install --force");
            println!("  llmeter update --source C:\\path\\to\\llmeter.exe");
            println!("  llmeter uninstall");
            println!("  llmeter uninstall --purge-home");
        }
        "examples" => {
            println!("Quick examples — Common workflows:");
            println!(
                "  llmeter status                                Check your provider is reachable"
            );
            println!("  llmeter providers set ollama                  Set Ollama as the default");
            println!("  llmeter models                                List local models");
            println!("  llmeter menu                                  Open the interactive menu");
            println!("  llmeter bench run --suite llm --models all \\");
            println!("    --benchmarks all --runs 3 --max-tokens 128  Run all LLM benchmarks");
            println!("  llmeter perf --models all --profile smoke     Quick performance check");
            println!("  llmeter quality list                          Browse quality tasks");
        }
        _ => {
            println!("LLMeter — Benchmark local OpenAI-compatible LLM providers.");
            println!("Help topics: providers, bench, reports, install, examples");
            println!(
                "Use `llmeter help <topic>` for details, or `llmeter --help` for CLI reference."
            );
        }
    }
}

fn fmt(value: Option<f64>) -> String {
    match value {
        Some(v) if v.is_finite() => format!("{:.2}", v),
        _ => String::new(),
    }
}

fn fmt_bytes(bytes: Option<u64>) -> String {
    match bytes {
        Some(b) if b >= 1_073_741_824 => format!("{:.1} GB", b as f64 / 1_073_741_824.0),
        Some(b) if b >= 1_048_576 => format!("{:.1} MB", b as f64 / 1_048_576.0),
        Some(b) if b >= 1_024 => format!("{:.1} KB", b as f64 / 1_024.0),
        Some(b) => format!("{b} B"),
        None => String::new(),
    }
}

fn fmt_digits(value: Option<f64>, digits: usize) -> String {
    match value {
        Some(v) if v.is_finite() => format!("{:.digits$}", v),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use super::{menu_action_for_key, move_menu_selection, MenuAction};

    #[test]
    fn enter_release_cannot_select_a_menu_item() {
        let release =
            KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(menu_action_for_key(release), None);
        let press =
            KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Press);
        assert_eq!(menu_action_for_key(press), Some(MenuAction::Select(0)));
    }

    #[test]
    fn navigation_back_and_interrupt_are_not_label_dependent() {
        assert_eq!(
            menu_action_for_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Some(MenuAction::Back)
        );
        assert_eq!(
            menu_action_for_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(MenuAction::Exit)
        );
    }

    #[test]
    fn navigation_clamps_for_each_workspace_menu_size() {
        for choices_len in [4, 5, 6, 9] {
            assert_eq!(move_menu_selection(0, choices_len, -1), 0);
            assert_eq!(
                move_menu_selection(choices_len - 1, choices_len, 1),
                choices_len - 1
            );
            assert_eq!(move_menu_selection(0, choices_len, 1), 1);
        }
        assert_eq!(move_menu_selection(4, 0, 1), 0);
    }
}
