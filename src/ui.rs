use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, CustomType, MultiSelect, Select, Text};
use serde_json::Value;
use tabled::{
    builder::Builder,
    settings::{Panel, Style},
};

use crate::benchmarks::registry::default_registry;
use crate::cli::{EXPORT_CHOICES, REPORT_CHOICES};
use crate::config::AppConfig;

use crate::ollama::client::OllamaClient;
use crate::ollama::server::{OllamaServerManager, ServerStatus};
use crate::reporting::{build_summary_rows, render_markdown_report};
use crate::results::BenchmarkRun;
use crate::runner;

const APP_TITLE: &str = "LLMeter";
const APP_SUBTITLE: &str = "Local LLM benchmarking for Ollama";

pub fn print_header() {
    let subtitle = APP_SUBTITLE.dimmed();
    println!("\n{}", APP_TITLE.bold());
    println!("{}\n", subtitle);
}

pub fn print_status_panel(status: &ServerStatus, cli_version: &Option<String>) {
    let tracked_pid = status
        .tracked_pid
        .map(|p| p.to_string())
        .unwrap_or_else(|| "none".to_string());
    let mut rows: Vec<Vec<&str>> = vec![
        vec!["Ollama installed", if status.installed { "yes" } else { "no" }],
        vec![
            "Executable",
            status.executable.as_deref().unwrap_or("not found"),
        ],
        vec!["Server running", if status.running { "yes" } else { "no" }],
        vec![
            "API version",
            status.version.as_deref().unwrap_or("unknown"),
        ],
        vec!["Tracked PID", &tracked_pid],
    ];
    if let Some(version) = cli_version {
        rows.push(vec!["Ollama CLI", version]);
    }

    let mut builder = Builder::new();
    builder.push_record(vec!["Check", "Value"]);
    for row in &rows {
        builder.push_record(row.iter().map(|s| s.to_string()));
    }
    let mut table = builder.build();
    table.with(Panel::header("Status"));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn print_models(models: &[Value]) {
    if models.is_empty() {
        println!(
            "No local Ollama models found. Install one with: {} {}",
            "ollama pull".bold(),
            "<model>".dimmed()
        );
        return;
    }

    let mut builder = Builder::new();
    builder.push_record(vec![
        "#".to_string(),
        "Model".to_string(),
        "Size".to_string(),
        "Params".to_string(),
        "Quant".to_string(),
        "Family".to_string(),
        "Modified".to_string(),
    ]);

    for (index, model) in models.iter().enumerate() {
        let details = model.get("details").and_then(|v| v.as_object());
        builder.push_record(vec![
            (index + 1).to_string(),
            model
                .get("name")
                .or_else(|| model.get("model"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            model
                .get("size")
                .and_then(|v| v.as_u64())
                .map(human_size)
                .unwrap_or_default(),
            details
                .and_then(|d| d.get("parameter_size"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            details
                .and_then(|d| d.get("quantization_level"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            details
                .and_then(|d| d.get("family"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            model
                .get("modified_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Panel::header("Installed Ollama models"));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn print_benchmark_catalog() {
    let registry = default_registry();
    let mut builder = Builder::new();
    builder.push_record(vec!["#", "ID", "Name", "Description"]);

    for (index, benchmark) in registry.all().iter().enumerate() {
        builder.push_record(vec![
            (index + 1).to_string(),
            benchmark.id().to_string(),
            benchmark.name().to_string(),
            benchmark.description().to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Panel::header("Benchmark catalog"));
    table.with(Style::rounded());
    println!("{table}");
}

pub fn summarize_run(run: &BenchmarkRun) {
    let errors: Vec<_> = run.results.iter().filter(|r| r.error.is_some()).collect();
    let ok = run.results.len() - errors.len();
    let models_str = run.models.join(", ");
    let records_str = format!("{} total, {ok} ok, {} errors", run.results.len(), errors.len());

    let metadata = vec![
        ("Run ID", &run.run_id),
        ("Created", &run.created_at),
        ("Models", &models_str),
        ("Records", &records_str),
    ];

    println!("\n{}", "Benchmark run".bold());
    for (label, value) in &metadata {
        println!("  {}: {}", label.dimmed(), value);
    }
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
    // Use comrak to convert markdown to HTML, then strip tags for terminal
    let html = comrak::markdown_to_html(markdown, &comrak::ComrakOptions::default());
    // Simple HTML to plain-text rendering with basic formatting
    let text = strip_html_for_terminal(&html);
    println!("{text}");
}

fn strip_html_for_terminal(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_code = false;
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            // Check for </code> or <code>
            let rest: String = chars[i..].iter().collect();
            if rest.starts_with("<code>") {
                in_code = true;
                i += 6;
                continue;
            }
            if rest.starts_with("</code>") {
                in_code = false;
                i += 7;
                continue;
            }
            if rest.starts_with("<strong>") {
                i += 8;
                // Next text should be bold
                let end = rest[8..].find("</strong>").map(|p| p + 8).unwrap_or(rest.len());
                let content = &rest[8..end];
                let bold_text = strip_html_inner(content);
                result.push_str(&bold_text.bold().to_string());
                i = chars.len().min(i + end - 8 + 9); // +9 for </strong>
                continue;
            }
            if rest.starts_with("<h1>") {
                let end = rest[4..].find("</h1>").map(|p| p + 4).unwrap_or(rest.len());
                let content = strip_html_inner(&rest[4..end]);
                result.push_str(&format!("\n# {}\n", content.bold()));
                i += end + 5;
                continue;
            }
            if rest.starts_with("<h2>") {
                let end = rest[4..].find("</h2>").map(|p| p + 4).unwrap_or(rest.len());
                let content = strip_html_inner(&rest[4..end]);
                result.push_str(&format!("\n## {}\n", content.bold()));
                i += end + 5;
                continue;
            }
            if rest.starts_with("<h3>") {
                let end = rest[4..].find("</h3>").map(|p| p + 4).unwrap_or(rest.len());
                let content = strip_html_inner(&rest[4..end]);
                result.push_str(&format!("\n### {}\n", content.bold()));
                i += end + 5;
                continue;
            }
            if rest.starts_with("<li>") {
                result.push_str("  • ");
                i += 4;
                continue;
            }
            if rest.starts_with("</li>") {
                result.push('\n');
                i += 5;
                continue;
            }
            if rest.starts_with("<ul>") || rest.starts_with("</ul>") || rest.starts_with("<ol>") || rest.starts_with("</ol>") {
                result.push('\n');
                i += if rest.starts_with("</") { 5 } else { 4 };
                continue;
            }
            if rest.starts_with("<tr>") || rest.starts_with("</tr>") || rest.starts_with("<thead>") || rest.starts_with("</thead>") || rest.starts_with("<tbody>") || rest.starts_with("</tbody>") || rest.starts_with("<section>") || rest.starts_with("</section>") {
                let tag_end = if rest.starts_with("</") {
                    rest[2..].find('>').map(|p| p + 3).unwrap_or(rest.len())
                } else {
                    rest[1..].find('>').map(|p| p + 2).unwrap_or(rest.len())
                };
                i += tag_end;
                continue;
            }
            if rest.starts_with("<th>") {
                let end = rest[4..].find("</th>").map(|p| p + 4).unwrap_or(rest.len());
                let content = strip_html_inner(&rest[4..end]);
                result.push_str(&content.bold().to_string());
                result.push_str("  ");
                i += end + 5;
                continue;
            }
            if rest.starts_with("<td>") {
                let end = rest[4..].find("</td>").map(|p| p + 4).unwrap_or(rest.len());
                let content = strip_html_inner(&rest[4..end]);
                result.push_str(&content);
                result.push_str("  ");
                i += end + 5;
                continue;
            }
            if rest.starts_with("<table>") || rest.starts_with("</table>") {
                result.push('\n');
                i += if rest.starts_with("</") { 8 } else { 7 };
                continue;
            }
            if rest.starts_with("<br") || rest.starts_with("<hr") || rest.starts_with("<div") || rest.starts_with("<span") || rest.starts_with("<header") || rest.starts_with("<main") || rest.starts_with("<meta") || rest.starts_with("<link") || rest.starts_with("<title") || rest.starts_with("<style") {
                // Skip self-closing and block tags
                let tag_end = rest[1..].find('>').map(|p| p + 2).unwrap_or(rest.len());
                i += tag_end;
                continue;
            }
            in_tag = true;
            i += 1;
            continue;
        }

        if chars[i] == '>' && in_tag {
            in_tag = false;
            i += 1;
            continue;
        }

        if in_tag {
            i += 1;
            continue;
        }

        // Decode HTML entities
        if chars[i] == '&' {
            let rest: String = chars[i..].iter().take(10).collect();
            if rest.starts_with("&amp;") { result.push('&'); i += 5; continue; }
            if rest.starts_with("&lt;") { result.push('<'); i += 4; continue; }
            if rest.starts_with("&gt;") { result.push('>'); i += 4; continue; }
            if rest.starts_with("&quot;") { result.push('"'); i += 6; continue; }
            if rest.starts_with("&#x27;") { result.push('\''); i += 6; continue; }
        }

        if in_code {
            result.push(chars[i]);
        } else {
            result.push(chars[i]);
        }
        i += 1;
    }

    result
}

fn strip_html_inner(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => {
                result.push(c);
            }
            _ => {}
        }
    }
    result
}

pub fn menu(title: &str, options: &[&str]) -> Result<usize> {
    let selection = Select::new(title, options.to_vec())
        .prompt()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    options
        .iter()
        .position(|&o| o == selection)
        .map(|i| i + 1)
        .ok_or_else(|| anyhow::anyhow!("Invalid selection"))
}

pub fn choose_from_menu(
    prompt: &str,
    choices: &[String],
    allow_all: bool,
    multi: bool,
) -> Result<Vec<String>> {
    if choices.is_empty() {
        return Ok(Vec::new());
    }

    let mut display_choices: Vec<String> = choices.to_vec();
    if allow_all {
        display_choices.insert(0, "All".to_string());
    }

    if multi {
        let selected = MultiSelect::new(prompt, display_choices)
            .prompt()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if selected.is_empty() {
            return Ok(Vec::new());
        }
        if allow_all && selected[0] == "All" {
            return Ok(choices.to_vec());
        }
        Ok(selected)
    } else {
        let selected = Select::new(prompt, display_choices)
            .prompt()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if allow_all && selected == "All" {
            return Ok(choices.to_vec());
        }
        Ok(vec![selected])
    }
}

pub fn choose_number(prompt: &str, default: u32, minimum: u32, maximum: Option<u32>) -> Result<u32> {
    loop {
        let value = CustomType::new(prompt)
            .with_default(default)
            .with_error_message("Please enter a valid number.")
            .prompt()
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        if value < minimum {
            println!("{} Value must be at least {minimum}.", "Error:".red());
            continue;
        }
        if let Some(max) = maximum {
            if value > max {
                println!("{} Value must be at most {max}.", "Error:".red());
                continue;
            }
        }
        return Ok(value);
    }
}

pub fn ask_yes_no(prompt: &str, default: bool) -> Result<bool> {
    Confirm::new(prompt)
        .with_default(default)
        .prompt()
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn ask_choice(prompt: &str, choices: &[&str], default: &str) -> Result<String> {
    Select::new(prompt, choices.to_vec())
        .with_starting_cursor(
            choices
                .iter()
                .position(|&c| c == default)
                .unwrap_or(0),
        )
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
                let duration = t
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default();
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

pub fn main_menu(
    config: &AppConfig,
    client: &OllamaClient,
    manager: &OllamaServerManager,
) -> Result<()> {
    loop {
        print_header();
        let status = manager.status();
        print_status_panel(&status, &manager.installed_version_cli());
        let choice = menu(
            "Main menu",
            &[
                "Start Ollama server",
                "Stop Ollama server",
                "List installed models",
                "Benchmark workspace",
                "Reports",
                "Exit",
            ],
        )?;

        match choice {
            1 => {
                let status = manager.start(10.0)?;
                print_status_panel(&status, &manager.installed_version_cli());
                pause();
            }
            2 => {
                let msg = manager.stop(false)?;
                println!("{msg}");
                pause();
            }
            3 => {
                match client.list_models() {
                    Ok(models) => print_models(&models),
                    Err(e) => println!("{} {e}", "Error:".red()),
                }
                pause();
            }
            4 => benchmark_menu(config, client, manager)?,
            5 => report_menu(config)?,
            6 => return Ok(()),
            _ => {}
        }
    }
}

pub fn benchmark_menu(
    config: &AppConfig,
    client: &OllamaClient,
    manager: &OllamaServerManager,
) -> Result<()> {
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
                print_benchmark_catalog();
                pause();
            }
            2 => guided_benchmark_run(config, client, manager)?,
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

fn guided_benchmark_run(
    config: &AppConfig,
    client: &OllamaClient,
    manager: &OllamaServerManager,
) -> Result<()> {
    if !client.is_running() {
        let start = ask_yes_no("Ollama server is not running. Start it now?", true)?;
        if start {
            match manager.start(10.0) {
                Ok(_) => {}
                Err(e) => {
                    println!("{} Could not start Ollama: {e}", "Error:".red());
                    return Ok(());
                }
            }
        } else {
            println!("{} Benchmark cancelled.", "Warning:".yellow());
            return Ok(());
        }
    }

    let models = runner::installed_model_names(client)?;
    let selected_models = choose_from_menu("Choose model or models", &models, true, true)?;
    if selected_models.is_empty() {
        println!("{} No models selected.", "Warning:".yellow());
        return Ok(());
    }

    let registry = default_registry();
    let benchmark_ids: Vec<String> = registry.ids().iter().map(|s| s.to_string()).collect();
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
    let num_predict = choose_number("num_predict", config.default_num_predict, 1, None)?;
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

    let run = runner::run_benchmarks(
        client,
        config,
        &selected_models,
        selected_ids.as_deref(),
        all_benchmarks,
        runs,
        num_predict,
        temperature,
        &std::collections::HashMap::new(),
    )?;

    summarize_run(&run);
    let saved = runner::save_outputs(config, &run, &export, &report)?;
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
    let saved = runner::save_outputs(config, &run, "none", &report_format)?;
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

fn human_size(num_bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut value = num_bytes as f64;
    for (i, unit) in units.iter().enumerate() {
        if value < 1024.0 || i == units.len() - 1 {
            if *unit == "B" {
                return format!("{value:.0} B");
            }
            return format!("{value:.1} {unit}");
        }
        value /= 1024.0;
    }
    format!("{num_bytes} B")
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
