use std::path::PathBuf;

use crate::cli::Cli;

const DEFAULT_HOST: &str = "http://localhost:11434";
const DEFAULT_OUTPUT_DIR: &str = "benchmark_results";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub timeout: f64,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub default_runs: u32,
    pub default_num_predict: u32,
    pub default_temperature: f64,
    pub api_base_url: String,
}

impl AppConfig {
    pub fn from_env(cli: &Cli) -> Self {
        let host = cli
            .host
            .clone()
            .or_else(|| std::env::var("OLLAMA_HOST").ok())
            .unwrap_or_else(|| DEFAULT_HOST.to_string());

        let timeout = cli
            .timeout
            .or_else(|| std::env::var("LLMETER_TIMEOUT").ok().and_then(|v| v.parse().ok()))
            .unwrap_or(120.0);

        let output_dir = cli
            .output_dir
            .clone()
            .or_else(|| std::env::var("LLMETER_OUTPUT_DIR").ok())
            .unwrap_or_else(|| DEFAULT_OUTPUT_DIR.to_string());

        let state_dir = std::env::var("LLMETER_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home.join(".llmeter")
            });

        let default_runs = std::env::var("LLMETER_RUNS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        let default_num_predict = std::env::var("LLMETER_NUM_PREDICT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(128);

        let default_temperature = std::env::var("LLMETER_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.2);

        let host_trimmed = host.trim_end_matches('/');
        let api_base_url = if host_trimmed.ends_with("/api") {
            host_trimmed.to_string()
        } else {
            format!("{host_trimmed}/api")
        };

        AppConfig {
            host,
            timeout,
            output_dir: PathBuf::from(output_dir),
            state_dir,
            default_runs,
            default_num_predict,
            default_temperature,
            api_base_url,
        }
    }
}
