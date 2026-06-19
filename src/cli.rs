use clap::builder::PossibleValuesParser;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::HashMap;

use crate::benchmarks::registry::BenchmarkSuite;
use crate::errors::LLMeterError;
use crate::performance::config::{
    LoadMeasurementMode, PerformanceProfile, ReportDetailLevel, TelemetryLevel,
};
use crate::providers::ProviderKind;
use crate::quality::catalog::QualityFramework;

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

    #[command(about = "Prepare quality benchmark plans and external tool adapters")]
    Quality {
        #[command(subcommand)]
        quality_command: QualityCommands,
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

    #[command(about = "Persist the default provider for future runs")]
    Set { provider: ProviderKind },
}

#[derive(Subcommand)]
pub enum BenchCommands {
    #[command(about = "List available benchmarks")]
    List {
        #[arg(long, value_enum, help = "Filter benchmarks by suite")]
        suite: Option<BenchmarkSuite>,
    },

    #[command(about = "Run benchmarks")]
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

        #[arg(long, help = "Repeated runs per benchmark")]
        runs: Option<u32>,

        #[arg(long, help = "Maximum output tokens for generation-style requests")]
        max_tokens: Option<u32>,

        #[arg(long, help = "Sampling temperature")]
        temperature: Option<f64>,

        #[arg(
            long,
            default_value = "both",
            value_parser = PossibleValuesParser::new(EXPORT_CHOICES),
            help = "Raw result export format"
        )]
        export: String,

        #[arg(
            long,
            default_value = "both",
            value_parser = PossibleValuesParser::new(REPORT_CHOICES),
            help = "Formatted report export format"
        )]
        report: String,

        #[arg(long = "param", action = clap::ArgAction::Append, help = "Extra provider request parameter as key=value, repeatable")]
        param: Vec<String>,
    },

    #[command(
        about = "Run native performance benchmark scenarios",
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

        #[arg(long, help = "Optional JSONL workload path")]
        jsonl: Option<String>,

        #[arg(
            long,
            default_value = "both",
            value_parser = PossibleValuesParser::new(EXPORT_CHOICES),
            help = "Raw result export format"
        )]
        export: String,

        #[arg(
            long,
            default_value = "both",
            value_parser = PossibleValuesParser::new(REPORT_CHOICES),
            help = "Formatted report export format"
        )]
        report: String,

        #[arg(long = "param", action = clap::ArgAction::Append, help = "Extra provider request parameter as key=value, repeatable")]
        param: Vec<String>,

        #[arg(long, value_enum, default_value_t = LoadMeasurementMode::WarmBaseline)]
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

        #[arg(
            long,
            default_value = "both",
            value_parser = PossibleValuesParser::new(REPORT_CHOICES),
            help = "Report format"
        )]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum QualityCommands {
    #[command(about = "List built-in quality benchmark catalog entries")]
    List,

    #[command(about = "Build a dry-run external quality benchmark plan")]
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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{BenchCommands, Cli, Commands, ProviderCommands, QualityCommands};
    use crate::benchmarks::registry::BenchmarkSuite;
    use crate::performance::config::PerformanceProfile;
    use crate::providers::ProviderKind;
    use crate::quality::catalog::QualityFramework;

    #[test]
    fn parses_provider_set_command() {
        let cli = Cli::parse_from(["llmeter", "providers", "set", "lmstudio"]);
        match cli.command {
            Some(Commands::Providers {
                provider_command: ProviderCommands::Set { provider },
            }) => {
                assert_eq!(provider, ProviderKind::Lmstudio);
            }
            _ => panic!("expected providers set command"),
        }
    }

    #[test]
    fn parses_bench_run_provider_and_suite() {
        let cli = Cli::parse_from([
            "llmeter",
            "bench",
            "run",
            "--provider",
            "ollama",
            "--suite",
            "embeddings",
            "--models",
            "all",
        ]);
        match cli.command {
            Some(Commands::Bench {
                bench_command:
                    BenchCommands::Run {
                        provider,
                        suite,
                        models,
                        ..
                    },
            }) => {
                assert_eq!(provider, Some(ProviderKind::Ollama));
                assert_eq!(suite, BenchmarkSuite::Embeddings);
                assert_eq!(models.as_deref(), Some("all"));
            }
            _ => panic!("expected bench run command"),
        }
    }

    #[test]
    fn rejects_invalid_export_choice() {
        let result = Cli::try_parse_from([
            "llmeter", "bench", "run", "--models", "all", "--export", "raw",
        ]);

        let error = result.err().expect("expected clap validation error");
        assert!(error.to_string().contains("raw"));
        assert!(error.to_string().contains("possible values"));
    }

    #[test]
    fn rejects_invalid_report_choice() {
        let result = Cli::try_parse_from(["llmeter", "report", "generate", "--format", "pdf"]);

        let error = result.err().expect("expected clap validation error");
        assert!(error.to_string().contains("pdf"));
        assert!(error.to_string().contains("possible values"));
    }

    #[test]
    fn parses_bench_perf_command() {
        let cli = Cli::parse_from([
            "llmeter",
            "bench",
            "perf",
            "--models",
            "llama3.1",
            "--profile",
            "sweep",
            "--prompt-tokens",
            "128,512",
            "--concurrency",
            "1,2,4",
        ]);

        match cli.command {
            Some(Commands::Bench {
                bench_command:
                    BenchCommands::Perf {
                        models,
                        profile,
                        prompt_tokens,
                        concurrency,
                        ..
                    },
            }) => {
                assert_eq!(models.as_deref(), Some("llama3.1"));
                assert_eq!(profile, PerformanceProfile::Sweep);
                assert_eq!(prompt_tokens.as_deref(), Some("128,512"));
                assert_eq!(concurrency.as_deref(), Some("1,2,4"));
            }
            _ => panic!("expected bench perf command"),
        }
    }

    #[test]
    fn parses_quality_plan_command() {
        let cli = Cli::parse_from([
            "llmeter",
            "quality",
            "plan",
            "--framework",
            "lighteval",
            "--task",
            "leaderboard|mmlu|5",
            "--model",
            "llama3.1",
        ]);

        match cli.command {
            Some(Commands::Quality {
                quality_command:
                    QualityCommands::Plan {
                        framework,
                        task,
                        model,
                    },
            }) => {
                assert_eq!(framework, QualityFramework::LightEval);
                assert_eq!(task, "leaderboard|mmlu|5");
                assert_eq!(model, "llama3.1");
            }
            _ => panic!("expected quality plan command"),
        }
    }
}
