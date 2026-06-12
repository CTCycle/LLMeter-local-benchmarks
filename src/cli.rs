use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::HashMap;

use crate::errors::LLMeterError;
use crate::providers::ProviderKind;

#[derive(Parser)]
#[command(
    name = "llmeter",
    about = "Benchmark local OpenAI-compatible LLM providers from a modern CLI.",
    version = "0.3.0",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(
        long,
        value_enum,
        help = "Provider preset: ollama, lmstudio, llama-cpp, or openai-compatible"
    )]
    pub provider: Option<ProviderKind>,

    #[arg(long, help = "OpenAI-compatible /v1 base URL")]
    pub base_url: Option<String>,

    #[arg(long, help = "HTTP request timeout in seconds")]
    pub timeout: Option<f64>,

    #[arg(long, help = "Directory for result and report files")]
    pub output_dir: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Show provider API status")]
    Status,

    #[command(about = "List supported provider presets")]
    Providers {
        #[command(subcommand)]
        provider_command: ProviderCommands,
    },

    #[command(about = "List local models exposed by the selected provider")]
    Models {
        #[arg(long, help = "Print raw JSON")]
        json: bool,
    },

    #[command(about = "Show model metadata from the selected provider")]
    Show { model: String },

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

    #[command(
        about = "Show built-in help. Use a topic such as providers, bench, reports, or examples",
        visible_alias = "/help"
    )]
    Help { topic: Option<String> },

    #[command(about = "Open the interactive main menu")]
    Menu,
}

#[derive(Subcommand)]
pub enum ProviderCommands {
    #[command(about = "List provider presets and default base URLs")]
    List,
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

        #[arg(long, help = "Maximum output tokens for generation-style requests")]
        max_tokens: Option<u32>,

        #[arg(long, help = "Sampling temperature")]
        temperature: Option<f64>,

        #[arg(long, default_value = "both", help = "Raw result export format")]
        export: String,

        #[arg(long, default_value = "both", help = "Formatted report export format")]
        report: String,

        #[arg(long = "param", action = clap::ArgAction::Append, help = "Extra provider request parameter as key=value, repeatable")]
        param: Vec<String>,
    },

    #[command(about = "Open the interactive benchmark menu")]
    Menu,
}

#[derive(Subcommand)]
pub enum ReportCommands {
    #[command(about = "List recent JSON result files and generated reports")]
    List,

    #[command(about = "Render a saved JSON result as a terminal report")]
    Show { result: Option<String> },

    #[command(about = "Generate Markdown and/or HTML reports from a saved JSON result")]
    Generate {
        result: Option<String>,

        #[arg(long, default_value = "both", help = "Report format")]
        format: String,
    },
}

pub const EXPORT_CHOICES: &[&str] = &["json", "csv", "both", "none"];
pub const REPORT_CHOICES: &[&str] = &["md", "html", "both", "none"];

pub fn parse_params(values: &[String]) -> anyhow::Result<HashMap<String, Value>> {
    let mut parsed: HashMap<String, Value> = HashMap::new();
    for item in values {
        let eq_pos = item.find('=').ok_or_else(|| {
            LLMeterError::InvalidOption(format!("Invalid --param '{item}'. Use key=value."))
        })?;
        let key = item[..eq_pos].trim().to_string();
        let value_str = item[eq_pos + 1..].trim().to_string();
        if key.is_empty() {
            return Err(LLMeterError::InvalidOption(format!(
                "Invalid --param '{item}'. Empty key."
            ))
            .into());
        }
        let value: Value = serde_json::from_str(&value_str).unwrap_or(Value::String(value_str));
        parsed.insert(key, value);
    }
    Ok(parsed)
}
