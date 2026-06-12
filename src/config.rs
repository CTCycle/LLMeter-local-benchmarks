use std::path::PathBuf;

use crate::cli::Cli;
use crate::providers::ProviderKind;

const DEFAULT_OUTPUT_DIR: &str = "benchmark_results";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub provider: ProviderKind,
    pub base_url: String,
    pub timeout: f64,
    pub output_dir: PathBuf,
    pub default_runs: u32,
    pub default_max_tokens: u32,
    pub default_temperature: f64,
}

impl AppConfig {
    pub fn from_env(cli: &Cli) -> Self {
        let provider = cli
            .provider
            .or_else(|| {
                std::env::var("LLMETER_PROVIDER")
                    .ok()
                    .and_then(|v| v.parse().ok())
            })
            .unwrap_or(ProviderKind::Ollama);

        let base_url = cli
            .base_url
            .clone()
            .or_else(|| std::env::var("LLMETER_BASE_URL").ok())
            .or_else(|| provider_env_url(provider))
            .unwrap_or_else(|| provider.default_base_url().to_string());

        let timeout = cli
            .timeout
            .or_else(|| {
                std::env::var("LLMETER_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
            })
            .unwrap_or(120.0);

        let output_dir = cli
            .output_dir
            .clone()
            .or_else(|| std::env::var("LLMETER_OUTPUT_DIR").ok())
            .unwrap_or_else(|| DEFAULT_OUTPUT_DIR.to_string());

        let default_runs = std::env::var("LLMETER_RUNS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        let default_max_tokens = std::env::var("LLMETER_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(128);

        let default_temperature = std::env::var("LLMETER_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.2);

        AppConfig {
            provider,
            base_url: normalize_base_url(&base_url),
            timeout,
            output_dir: PathBuf::from(output_dir),
            default_runs,
            default_max_tokens,
            default_temperature,
        }
    }
}

fn provider_env_url(provider: ProviderKind) -> Option<String> {
    match provider {
        ProviderKind::Ollama => std::env::var("OLLAMA_HOST")
            .ok()
            .map(|host| format!("{}/v1", host.trim_end_matches('/'))),
        ProviderKind::Lmstudio => std::env::var("LMSTUDIO_BASE_URL").ok(),
        ProviderKind::LlamaCpp => std::env::var("LLAMA_CPP_BASE_URL").ok(),
        ProviderKind::OpenaiCompatible => None,
    }
}

fn normalize_base_url(value: &str) -> String {
    let trimmed = value.trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_base_url;

    #[test]
    fn normalizes_base_url_to_v1() {
        assert_eq!(
            normalize_base_url("http://localhost:1234"),
            "http://localhost:1234/v1"
        );
        assert_eq!(
            normalize_base_url("http://localhost:1234/v1/"),
            "http://localhost:1234/v1"
        );
    }
}
