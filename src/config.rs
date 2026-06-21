use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::errors::LLMeterError;
use crate::providers::ProviderKind;

const DEFAULT_OUTPUT_DIR: &str = "benchmark_results";
const CONFIG_FILE_NAME: &str = "config.json";
const CONFIG_DIR_NAME: &str = "config";
const DEFAULT_HOME_DIR_NAME: &str = ".llmeter";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub provider: ProviderKind,
    pub base_url: String,
    pub timeout: f64,
    pub output_dir: PathBuf,
    pub default_runs: u32,
    pub default_max_tokens: u32,
    pub default_temperature: f64,
    pub explicit_base_url: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct PersistedConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<ProviderKind>,
}

impl AppConfig {
    pub fn from_env(cli: &Cli) -> Self {
        Self::from_env_with_provider(cli, None)
    }

    pub fn from_env_with_provider(cli: &Cli, provider_override: Option<ProviderKind>) -> Self {
        let provider = provider_override
            .or(cli.provider)
            .or_else(provider_from_env)
            .or_else(load_persisted_provider)
            .unwrap_or(ProviderKind::Ollama);

        let explicit_base_url = cli.base_url.is_some();
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
            .map(PathBuf::from)
            .unwrap_or_else(default_output_dir);

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
            output_dir,
            default_runs,
            default_max_tokens,
            default_temperature,
            explicit_base_url,
        }
    }

    pub fn with_provider(&self, provider: ProviderKind) -> Self {
        let base_url = if self.explicit_base_url {
            self.base_url.clone()
        } else {
            provider_env_url(provider).unwrap_or_else(|| provider.default_base_url().to_string())
        };

        Self {
            provider,
            base_url: normalize_base_url(&base_url),
            timeout: self.timeout,
            output_dir: self.output_dir.clone(),
            default_runs: self.default_runs,
            default_max_tokens: self.default_max_tokens,
            default_temperature: self.default_temperature,
            explicit_base_url: self.explicit_base_url,
        }
    }
}

pub fn save_global_provider(provider: ProviderKind) -> anyhow::Result<PathBuf> {
    let path = default_config_path().ok_or_else(|| {
        LLMeterError::Io("Unable to determine the LLMeter config directory.".to_string())
    })?;
    save_global_provider_to_path(provider, &path)?;
    Ok(path)
}

pub fn save_global_provider_to_path(provider: ProviderKind, path: &Path) -> anyhow::Result<()> {
    let config = PersistedConfig {
        provider: Some(provider),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            LLMeterError::Io(format!(
                "Failed to create config directory {}: {}",
                parent.display(),
                error
            ))
        })?;
    }
    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(path, content).map_err(|error| {
        LLMeterError::Io(format!(
            "Failed to write config file {}: {}",
            path.display(),
            error
        ))
    })?;
    Ok(())
}

fn load_persisted_provider() -> Option<ProviderKind> {
    let path = default_config_path()?;
    load_persisted_provider_from_path(&path)
}

fn load_persisted_provider_from_path(path: &Path) -> Option<ProviderKind> {
    let content = std::fs::read_to_string(path).ok()?;
    let config: PersistedConfig = serde_json::from_str(&content).ok()?;
    config.provider
}

fn default_config_path() -> Option<PathBuf> {
    config_root_dir().map(|dir| dir.join(CONFIG_FILE_NAME))
}

fn config_root_dir() -> Option<PathBuf> {
    std::env::var_os("LLMETER_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| llmeter_home_dir().map(|dir| dir.join(CONFIG_DIR_NAME)))
}

fn default_output_dir() -> PathBuf {
    llmeter_home_dir()
        .map(|dir| dir.join(DEFAULT_OUTPUT_DIR))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_DIR))
}

pub fn llmeter_home_dir() -> Option<PathBuf> {
    std::env::var_os("LLMETER_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|dir| dir.join(DEFAULT_HOME_DIR_NAME)))
}

fn provider_from_env() -> Option<ProviderKind> {
    std::env::var("LLMETER_PROVIDER")
        .ok()
        .and_then(|value| value.parse().ok())
}

fn provider_env_url(provider: ProviderKind) -> Option<String> {
    match provider {
        ProviderKind::Ollama => std::env::var("OLLAMA_HOST")
            .ok()
            .map(|host| format!("{}/v1", host.trim_end_matches('/'))),
        ProviderKind::Lmstudio => std::env::var("LMSTUDIO_BASE_URL").ok(),
        ProviderKind::LlamaCpp => std::env::var("LLAMA_CPP_BASE_URL").ok(),
        ProviderKind::OpenaiCompatible => None,
        ProviderKind::Vllm => std::env::var("VLLM_BASE_URL").ok(),
        ProviderKind::Sglang => std::env::var("SGLANG_BASE_URL").ok(),
        ProviderKind::Localai => std::env::var("LOCALAI_BASE_URL").ok(),
        ProviderKind::Litellm => std::env::var("LITELLM_BASE_URL").ok(),
        ProviderKind::Tgi => std::env::var("TGI_BASE_URL").ok(),
        ProviderKind::TextGenerationWebui => std::env::var("TEXT_GENERATION_WEBUI_BASE_URL").ok(),
        ProviderKind::Jan => std::env::var("JAN_BASE_URL").ok(),
        ProviderKind::MlxLm => std::env::var("MLX_LM_BASE_URL").ok(),
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
    use clap::Parser;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    use tempfile::tempdir;

    use super::{
        load_persisted_provider_from_path, normalize_base_url, save_global_provider,
        save_global_provider_to_path, AppConfig,
    };
    use crate::cli::Cli;
    use crate::providers::ProviderKind;

    fn clear_env() {
        for key in [
            "LLMETER_HOME",
            "LLMETER_PROVIDER",
            "LLMETER_BASE_URL",
            "OLLAMA_HOST",
            "LMSTUDIO_BASE_URL",
            "LLAMA_CPP_BASE_URL",
            "LLMETER_CONFIG_DIR",
            "LLMETER_OUTPUT_DIR",
        ] {
            std::env::remove_var(key);
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

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

    #[test]
    fn persists_global_provider() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.json");
        save_global_provider_to_path(ProviderKind::Lmstudio, &path).unwrap();
        assert_eq!(
            load_persisted_provider_from_path(&path),
            Some(ProviderKind::Lmstudio)
        );
    }

    #[test]
    fn provider_resolution_precedence_is_cli_then_env_then_persisted_then_default() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let temp = tempdir().unwrap();
        std::env::set_var("LLMETER_CONFIG_DIR", temp.path());
        let path = temp.path().join("config.json");
        save_global_provider_to_path(ProviderKind::LlamaCpp, &path).unwrap();

        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli);
        assert_eq!(config.provider, ProviderKind::LlamaCpp);

        std::env::set_var("LLMETER_PROVIDER", "lmstudio");
        let config = AppConfig::from_env(&cli);
        assert_eq!(config.provider, ProviderKind::Lmstudio);

        let cli_override = Cli::parse_from(["llmeter", "--provider", "ollama"]);
        let config = AppConfig::from_env(&cli_override);
        assert_eq!(config.provider, ProviderKind::Ollama);

        clear_env();
        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli);
        assert_eq!(config.provider, ProviderKind::Ollama);
    }

    #[test]
    fn default_output_dir_uses_llmeter_home_when_present() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let temp = tempdir().unwrap();
        std::env::set_var("LLMETER_HOME", temp.path());

        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli);

        assert_eq!(config.output_dir, temp.path().join("benchmark_results"));
    }

    #[test]
    fn output_dir_env_override_wins_over_llmeter_home_default() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let temp = tempdir().unwrap();
        std::env::set_var("LLMETER_HOME", temp.path());
        std::env::set_var("LLMETER_OUTPUT_DIR", "custom-results");

        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli);

        assert_eq!(config.output_dir, PathBuf::from("custom-results"));
    }

    #[test]
    fn persisted_provider_uses_llmeter_home_config_root_by_default() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let temp = tempdir().unwrap();
        std::env::set_var("LLMETER_HOME", temp.path());

        save_global_provider(ProviderKind::Lmstudio).unwrap();

        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli);

        assert_eq!(config.provider, ProviderKind::Lmstudio);
    }

    #[test]
    fn provider_override_recalculates_default_base_url() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli).with_provider(ProviderKind::Lmstudio);
        assert_eq!(config.provider, ProviderKind::Lmstudio);
        assert_eq!(config.base_url, "http://localhost:1234/v1");
    }

    #[test]
    fn explicit_base_url_is_retained_when_provider_changes() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let cli = Cli::parse_from(["llmeter", "--base-url", "http://localhost:9999/v1"]);
        let config = AppConfig::from_env(&cli).with_provider(ProviderKind::Lmstudio);
        assert_eq!(config.base_url, "http://localhost:9999/v1");
    }
}
