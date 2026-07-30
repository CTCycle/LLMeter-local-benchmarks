use clap::{Parser, Subcommand, ValueEnum};
use serde_json::Value;
use std::collections::HashMap;

use crate::benchmarks::registry::BenchmarkSuite;
use crate::errors::LLMeterError;
use crate::performance::config::{
    LoadMeasurementMode, PerformanceProfile, ReportDetailLevel, TelemetryLevel,
    DEFAULT_MAX_PERFORMANCE_REQUESTS,
};
use crate::providers::ProviderKind;
use crate::quality::catalog::QualityFramework;

pub fn parse_positive_u32(value: &str) -> Result<u32, String> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| format!("'{value}' must be a positive integer"))?;
    if parsed == 0 {
        Err(format!("'{value}' must be greater than zero"))
    } else {
        Ok(parsed)
    }
}

pub fn parse_temperature(value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| format!("'{value}' must be a finite non-negative number"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        Err(format!("'{value}' must be a finite non-negative number"))
    } else {
        Ok(parsed)
    }
}

#[derive(Parser)]
#[command(
    name = "llmeter",
    about = "Benchmark local OpenAI-compatible LLM providers from a modern CLI.",
    version,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(
        long,
        value_enum,
        help = "Provider preset. Use `llmeter providers list` for the full catalog."
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
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    #[command(about = "Check provider /v1 endpoint reachability, health, and exposed model count")]
    Status,

    #[command(about = "List provider presets with compatibility tiers and default URLs")]
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

    #[command(
        about = "Run and manage LLM benchmarks — generation, latency, performance, and quality"
    )]
    Bench {
        #[command(subcommand)]
        bench_command: BenchCommands,
    },

    #[command(about = "View saved results and export formatted Markdown or HTML reports")]
    Report {
        #[command(subcommand)]
        report_command: ReportCommands,
    },

    #[command(
        about = "Quality benchmark plans for lighteval, inspect-ai, lm-eval-harness, and SWE-bench"
    )]
    Quality {
        #[command(subcommand)]
        quality_command: QualityCommands,
    },

    #[command(about = "Install LLMeter into the managed CLI home bin directory")]
    Install {
        #[arg(
            long,
            help = "Managed bin directory override. Defaults to <LLMETER_HOME>/bin"
        )]
        bin_dir: Option<String>,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Overwrite an existing managed install")]
        force: bool,
    },

    #[command(about = "Update the managed LLMeter install")]
    Update {
        #[arg(long, help = "Path to the replacement llmeter executable")]
        source: Option<String>,

        #[arg(
            long,
            help = "Managed bin directory override. Defaults to <LLMETER_HOME>/bin"
        )]
        bin_dir: Option<String>,
    },

    #[command(about = "Uninstall the managed LLMeter CLI")]
    Uninstall {
        #[arg(
            long,
            help = "Managed bin directory override. Defaults to <LLMETER_HOME>/bin"
        )]
        bin_dir: Option<String>,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Also remove <LLMETER_HOME> config and benchmark outputs")]
        purge_home: bool,
    },

    #[command(
        about = "Show built-in help. Use a topic such as providers, bench, reports, install, or examples",
        visible_alias = "/help"
    )]
    Help { topic: Option<String> },

    #[command(about = "Open the interactive main menu")]
    Menu,
}

#[derive(Subcommand)]
pub enum ProviderCommands {
    #[command(about = "List provider presets with compatibility tiers and default base URLs")]
    List,

    #[command(about = "Persist the default provider for future runs")]
    Set { provider: ProviderKind },
}

#[derive(Subcommand)]
pub enum BenchCommands {
    #[command(about = "List benchmark IDs, suites, names, and descriptions")]
    List {
        #[arg(long, value_enum, help = "Filter benchmarks by suite")]
        suite: Option<BenchmarkSuite>,
    },

    #[command(
        about = "Run standard benchmarks — generation, consistency, structured output, etc."
    )]
    Run {
        #[arg(
            long,
            value_enum,
            help = "Provider preset override for this benchmark run"
        )]
        provider: Option<ProviderKind>,

        #[arg(
            long,
            value_enum,
            default_value_t = BenchmarkSuite::Llm,
            help = "Benchmark suite to run"
        )]
        suite: BenchmarkSuite,

        #[arg(long, help = "Comma-separated model names, or 'all'")]
        models: Option<String>,

        #[arg(long, help = "Comma-separated benchmark ids, or 'all'")]
        benchmarks: Option<String>,

        #[arg(long, value_parser = parse_positive_u32, help = "Repeated runs per benchmark")]
        runs: Option<u32>,

        #[arg(long, value_parser = parse_positive_u32, help = "Maximum output tokens for generation-style requests")]
        max_tokens: Option<u32>,

        #[arg(long, value_parser = parse_temperature, help = "Sampling temperature")]
        temperature: Option<f64>,

        #[arg(long, default_value = "both", help = "Raw result export format")]
        export: ExportFormat,

        #[arg(long, default_value = "both", help = "Formatted report export format")]
        report: ReportFormat,

        #[arg(long = "param", action = clap::ArgAction::Append, help = "Extra provider request parameter as key=value, repeatable")]
        param: Vec<String>,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Include truncated model response previews in saved outputs and reports")]
        include_response_preview: bool,
    },

    #[command(
        about = "Measure latency, throughput, TTFT, and chunk timing under concurrent load",
        visible_alias = "performance"
    )]
    Perf {
        #[arg(
            long,
            value_enum,
            help = "Provider preset override for this performance run"
        )]
        provider: Option<ProviderKind>,

        #[arg(long, help = "Comma-separated model names, or 'all'")]
        models: Option<String>,

        #[arg(
            long,
            help = "Performance profile: smoke, latency, throughput, or sweep"
        )]
        profile: PerformanceProfile,

        #[arg(long, help = "Comma-separated estimated prompt token sizes")]
        prompt_tokens: Option<String>,

        #[arg(long, help = "Comma-separated estimated output token sizes")]
        output_tokens: Option<String>,

        #[arg(long, help = "Comma-separated concurrency levels")]
        concurrency: Option<String>,

        #[arg(long, help = "Warmup request count")]
        warmup: Option<u32>,

        #[arg(long, help = "Measured request count per scenario")]
        runs: Option<u32>,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Disable streaming requests")]
        no_stream: bool,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Allow prompt/output sizes above documented safety limits")]
        allow_large_prompt: bool,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Allow performance matrices above the request safety limit")]
        allow_large_matrix: bool,

        #[arg(long, help = "Optional JSONL workload path")]
        jsonl: Option<String>,

        #[arg(long, default_value = "both", help = "Raw result export format")]
        export: ExportFormat,

        #[arg(long, default_value = "both", help = "Formatted report export format")]
        report: ReportFormat,

        #[arg(long = "param", action = clap::ArgAction::Append, help = "Extra provider request parameter as key=value, repeatable")]
        param: Vec<String>,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Include truncated model response previews in saved outputs and reports")]
        include_response_preview: bool,

        #[arg(
            long,
            value_enum,
            default_value_t = LoadMeasurementMode::FirstRequestEstimate,
            help = "Client-observed first-request versus warm-request estimate; not true provider cold-start telemetry"
        )]
        load_measurement: LoadMeasurementMode,

        #[arg(long, default_value_t = 2)]
        load_probe_runs: u32,

        #[arg(long, value_enum, default_value_t = TelemetryLevel::Standard)]
        telemetry: TelemetryLevel,

        #[arg(long, default_value_t = 1000)]
        sample_interval_ms: u64,

        #[arg(long)]
        provider_process: Option<String>,

        #[arg(long, action = clap::ArgAction::SetTrue)]
        probe_capabilities: bool,

        #[arg(long, action = clap::ArgAction::SetTrue)]
        probe_all_endpoints: bool,

        #[arg(long)]
        model_cache_dir: Option<String>,

        #[arg(long, action = clap::ArgAction::SetTrue)]
        scan_model_cache: bool,

        #[arg(long, value_enum, default_value_t = ReportDetailLevel::Detailed)]
        detail: ReportDetailLevel,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Print the performance scenario and request estimate without running requests")]
        dry_run: bool,

        #[arg(long, default_value_t = DEFAULT_MAX_PERFORMANCE_REQUESTS, help = "Maximum warmup plus measured requests allowed before requiring an explicit override")]
        max_requests: u32,
    },

    #[command(about = "Open the interactive benchmark menu")]
    Menu,
}

#[derive(Subcommand)]
pub enum ReportCommands {
    #[command(about = "List recent JSON result files and generated reports")]
    List,

    #[command(about = "Display a saved JSON benchmark result as a formatted terminal report")]
    Show { result: Option<String> },

    #[command(about = "Export a saved JSON result as a Markdown or HTML report")]
    Generate {
        result: Option<String>,

        #[arg(long, default_value = "both", help = "Report format")]
        format: ReportFormat,

        #[arg(long, action = clap::ArgAction::SetTrue, help = "Include response previews already present in the saved result")]
        include_response_preview: bool,
    },
}

#[derive(Subcommand)]
pub enum QualityCommands {
    #[command(
        about = "List quality benchmark tasks (MMLU, HellaSwag, etc.) with frameworks and metrics"
    )]
    List,

    #[command(
        about = "Build a dry-run plan for an external quality benchmark (lighteval, inspect-ai, etc.)"
    )]
    Plan {
        #[arg(
            long,
            help = "Framework: lighteval, inspect-ai, lm-eval-harness, swe-bench"
        )]
        framework: QualityFramework,
        #[arg(long, help = "Catalog task id or framework-specific task expression")]
        task: String,
        #[arg(long, help = "Target model name")]
        model: String,
    },
}

pub const EXPORT_CHOICES: &[&str] = &["json", "csv", "both", "none"];
pub const REPORT_CHOICES: &[&str] = &["md", "html", "both", "none"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
    Both,
    None,
}

impl ExportFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Both => "both",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReportFormat {
    Md,
    Html,
    Both,
    None,
}

impl ReportFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Md => "md",
            Self::Html => "html",
            Self::Both => "both",
            Self::None => "none",
        }
    }
}

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
        if matches!(
            key.as_str(),
            "unsafe_large_prompt"
                | "unsafe_large_matrix"
                | "allow_large_prompt"
                | "allow_large_matrix"
        ) {
            return Err(LLMeterError::InvalidOption(format!(
                "Invalid --param key '{key}'. Safety controls must use their dedicated CLI flags."
            ))
            .into());
        }
        let value: Value = serde_json::from_str(&value_str).unwrap_or(Value::String(value_str));
        parsed.insert(key, value);
    }
    Ok(parsed)
}
