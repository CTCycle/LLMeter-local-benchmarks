use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::HashMap;

use crate::errors::LLMeterError;

#[derive(Parser)]
#[command(
    name = "llmeter",
    about = "Benchmark local Ollama models from a modern interactive CLI.",
    version = "0.2.0"
)]
pub struct Cli {
    #[arg(long, help = "Ollama host, default from OLLAMA_HOST or http://localhost:11434")]
    pub host: Option<String>,

    #[arg(long, help = "HTTP request timeout in seconds")]
    pub timeout: Option<f64>,

    #[arg(long, help = "Directory for result and report files")]
    pub output_dir: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Show Ollama installation and server status")]
    Status,

    #[command(about = "Manage the local Ollama server")]
    Server {
        #[command(subcommand)]
        server_command: ServerCommands,
    },

    #[command(about = "List installed local Ollama models")]
    Models {
        #[arg(long, help = "Print raw JSON")]
        json: bool,
    },

    #[command(about = "Show model metadata from Ollama")]
    Show {
        model: String,
    },

    #[command(about = "Benchmark menu and commands")]
    Bench {
        #[command(subcommand)]
        bench_command: BenchCommands,
    },

    #[command(about = "View and generate formatted benchmark reports")]
    Report {
        #[command(subcommand)]
        report_command: ReportCommands,
    },

    #[command(about = "Open the interactive main menu")]
    Menu,
}

#[derive(Subcommand)]
pub enum ServerCommands {
    #[command(about = "Show server status")]
    Status,

    #[command(about = "Start 'ollama serve' if not running")]
    Start,

    #[command(about = "Stop a server started by this CLI")]
    Stop {
        #[arg(long, help = "Force-stop ollama server processes, not only the tracked process")]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum BenchCommands {
    #[command(about = "List available benchmarks")]
    List,

    #[command(about = "Run benchmarks")]
    Run {
        #[arg(long, help = "Comma-separated model names, or 'all'")]
        models: Option<String>,

        #[arg(long, help = "Comma-separated benchmark ids, or 'all'")]
        benchmarks: Option<String>,

        #[arg(long, help = "Repeated runs per benchmark")]
        runs: Option<u32>,

        #[arg(long, help = "Ollama num_predict option")]
        num_predict: Option<u32>,

        #[arg(long, help = "Ollama temperature option")]
        temperature: Option<f64>,

        #[arg(long, default_value = "both", help = "Raw result export format")]
        export: String,

        #[arg(long, default_value = "both", help = "Formatted report export format")]
        report: String,

        #[arg(long, help = "Start Ollama if the server is not running")]
        start_server: bool,

        #[arg(long = "option", action = clap::ArgAction::Append, help = "Extra Ollama option as key=value, repeatable")]
        option: Vec<String>,
    },

    #[command(about = "Open the interactive benchmark menu")]
    Menu,
}

#[derive(Subcommand)]
pub enum ReportCommands {
    #[command(about = "List recent JSON result files and generated reports")]
    List,

    #[command(about = "Render a saved JSON result as a terminal report")]
    Show {
        result: Option<String>,
    },

    #[command(about = "Generate Markdown and/or HTML reports from a saved JSON result")]
    Generate {
        result: Option<String>,

        #[arg(long, default_value = "both", help = "Report format")]
        format: String,
    },
}

pub const EXPORT_CHOICES: &[&str] = &["json", "csv", "both", "none"];
pub const REPORT_CHOICES: &[&str] = &["md", "html", "both", "none"];

pub fn parse_options(values: &[String]) -> anyhow::Result<HashMap<String, Value>> {
    let mut parsed: HashMap<String, Value> = HashMap::new();
    for item in values {
        let eq_pos = item.find('=').ok_or_else(|| {
            LLMeterError::InvalidOption(format!("Invalid --option '{item}'. Use key=value."))
        })?;
        let key = item[..eq_pos].trim().to_string();
        let value_str = item[eq_pos + 1..].trim().to_string();
        if key.is_empty() {
            return Err(LLMeterError::InvalidOption(format!("Invalid --option '{item}'. Empty key.")).into());
        }
        let value: Value = serde_json::from_str(&value_str).unwrap_or(Value::String(value_str));
        parsed.insert(key, value);
    }
    Ok(parsed)
}
