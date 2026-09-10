use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::cli::Cli;
use crate::errors::LLMeterError;
use crate::providers::ProviderKind;

const DEFAULT_OUTPUT_DIR: &str = "benchmark_results";
const CONFIG_FILE_NAME: &str = "config.json";
const CONFIG_DIR_NAME: &str = "config";
const DEFAULT_HOME_DIR_NAME: &str = ".llmeter";
const DEFAULT_TIMEOUT_SECS: f64 = 120.0;
const MAX_TIMEOUT_SECS: f64 = 24.0 * 60.0 * 60.0;

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
    pub fn from_env(cli: &Cli) -> anyhow::Result<Self> {
        Self::from_env_with_provider(cli, None)
    }

    pub fn from_env_with_provider(
        cli: &Cli,
        provider_override: Option<ProviderKind>,
    ) -> anyhow::Result<Self> {
        let provider = if let Some(provider) = provider_override.or(cli.provider) {
            provider
        } else if let Some(provider) = provider_from_env()? {
            provider
        } else {
            load_persisted_provider()?.unwrap_or(ProviderKind::Ollama)
        };

        let explicit_base_url = if let Some(value) = cli.base_url.clone() {
            Some((value, "--base-url"))
        } else {
            read_env("LLMETER_BASE_URL")?.map(|value| (value, "LLMETER_BASE_URL"))
        };
        let has_explicit_base_url = explicit_base_url.is_some();
        let (base_url, base_url_source) = resolve_base_url(provider, explicit_base_url)?;

        let timeout = match cli.timeout {
            Some(value) => validate_timeout(value, "--timeout")?,
            None => match std::env::var("LLMETER_TIMEOUT") {
                Ok(value) => {
                    let parsed = value.parse::<f64>().map_err(|_| {
                        LLMeterError::InvalidOption(format!(
                            "Invalid LLMETER_TIMEOUT '{value}'. Expected a finite value greater than 0 and at most 86400 seconds."
                        ))
                    })?;
                    validate_timeout(parsed, "LLMETER_TIMEOUT")?
                }
                Err(_) => DEFAULT_TIMEOUT_SECS,
            },
        };

        let output_dir = cli
            .output_dir
            .clone()
            .or_else(|| std::env::var("LLMETER_OUTPUT_DIR").ok())
            .map(PathBuf::from)
            .unwrap_or_else(default_output_dir);

        let default_runs = parse_positive_env_u32("LLMETER_RUNS")?.unwrap_or(3);
        let default_max_tokens = parse_positive_env_u32("LLMETER_MAX_TOKENS")?.unwrap_or(128);
        let default_temperature = parse_temperature_env("LLMETER_TEMPERATURE")?.unwrap_or(0.2);

        Ok(AppConfig {
            provider,
            base_url: validate_base_url(&base_url, base_url_source)?,
            timeout,
            output_dir,
            default_runs,
            default_max_tokens,
            default_temperature,
            explicit_base_url: has_explicit_base_url,
        })
    }

    pub fn with_provider(&self, provider: ProviderKind) -> anyhow::Result<Self> {
        let explicit_base_url = self
            .explicit_base_url
            .then(|| (self.base_url.clone(), "explicit base URL"));
        let (base_url, base_url_source) = resolve_base_url(provider, explicit_base_url)?;

        Ok(Self {
            provider,
            base_url: validate_base_url(&base_url, base_url_source)?,
            timeout: self.timeout,
            output_dir: self.output_dir.clone(),
            default_runs: self.default_runs,
            default_max_tokens: self.default_max_tokens,
            default_temperature: self.default_temperature,
            explicit_base_url: self.explicit_base_url,
        })
    }
}

fn resolve_base_url(
    provider: ProviderKind,
    explicit_base_url: Option<(String, &'static str)>,
) -> anyhow::Result<(String, &'static str)> {
    if let Some(explicit) = explicit_base_url {
        return Ok(explicit);
    }
    if let Some(env_name) = provider.base_url_env() {
        if let Some(value) = read_env(env_name)? {
            return Ok((value, env_name));
        }
    }
    Ok((provider.default_base_url().to_string(), "provider default"))
}

fn validate_timeout(value: f64, source: &str) -> anyhow::Result<f64> {
    if !value.is_finite() || value <= 0.0 || value > MAX_TIMEOUT_SECS {
        return Err(LLMeterError::InvalidOption(format!(
            "Invalid {source} '{value}'. Expected a finite value greater than 0 and at most 86400 seconds."
        ))
        .into());
    }
    Ok(value)
}

fn parse_positive_env_u32(name: &str) -> anyhow::Result<Option<u32>> {
    let Ok(value) = std::env::var(name) else {
        return Ok(None);
    };
    let parsed = value.parse::<u32>().map_err(|_| {
        LLMeterError::InvalidOption(format!(
            "Invalid {name} '{value}'. Expected a positive integer."
        ))
    })?;
    if parsed == 0 {
        return Err(LLMeterError::InvalidOption(format!(
            "Invalid {name} '{value}'. Expected a positive integer."
        ))
        .into());
    }
    Ok(Some(parsed))
}

fn parse_temperature_env(name: &str) -> anyhow::Result<Option<f64>> {
    let Ok(value) = std::env::var(name) else {
        return Ok(None);
    };
    let parsed = value.parse::<f64>().map_err(|_| {
        LLMeterError::InvalidOption(format!(
            "Invalid {name} '{value}'. Expected a finite value greater than or equal to 0."
        ))
    })?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(LLMeterError::InvalidOption(format!(
            "Invalid {name} '{value}'. Expected a finite value greater than or equal to 0."
        ))
        .into());
    }
    Ok(Some(parsed))
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
    crate::utils::atomic_write(path, content.as_bytes()).map_err(|error| {
        LLMeterError::Io(format!(
            "Failed to write config file {}: {}",
            path.display(),
            error
        ))
    })?;
    Ok(())
}

fn load_persisted_provider() -> anyhow::Result<Option<ProviderKind>> {
    let Some(path) = default_config_path() else {
        return Ok(None);
    };
    load_persisted_provider_from_path(&path)
}

fn load_persisted_provider_from_path(path: &Path) -> anyhow::Result<Option<ProviderKind>> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(LLMeterError::Configuration(format!(
                "Failed to read persisted configuration {}: {error}",
                path.display()
            ))
            .into())
        }
    };
    let config: PersistedConfig = serde_json::from_str(&content).map_err(|error| {
        LLMeterError::Configuration(format!(
            "Invalid persisted configuration {}: {error}",
            path.display()
        ))
    })?;
    Ok(config.provider)
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

fn provider_from_env() -> anyhow::Result<Option<ProviderKind>> {
    let Some(value) = read_env("LLMETER_PROVIDER")? else {
        return Ok(None);
    };
    value.parse().map(Some).map_err(|_| {
        LLMeterError::Configuration(format!(
            "Invalid LLMETER_PROVIDER '{value}'. Use `llmeter providers list` for supported presets."
        ))
        .into()
    })
}

fn read_env(name: &str) -> anyhow::Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(LLMeterError::Configuration(format!(
            "Invalid {name} value: {error}. Expected valid Unicode text."
        ))
        .into()),
    }
}

fn validate_base_url(value: &str, source: &str) -> anyhow::Result<String> {
    let mut url = Url::parse(value).map_err(|error| {
        LLMeterError::Configuration(format!(
            "Invalid {source} URL '{value}': {error}. Expected an absolute HTTP or HTTPS URL."
        ))
    })?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(LLMeterError::Configuration(format!(
            "Invalid {source} URL '{value}'. Expected an absolute HTTP or HTTPS URL with a host."
        ))
        .into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(LLMeterError::Configuration(format!(
            "Invalid {source} URL. Embedded credentials are not supported."
        ))
        .into());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(LLMeterError::Configuration(format!(
            "Invalid {source} URL. Query strings and fragments are not supported in provider base URLs."
        ))
        .into());
    }

    let path = url.path().trim_end_matches('/');
    let normalized_path = if path.ends_with("/v1") {
        path.to_string()
    } else if path.is_empty() {
        "/v1".to_string()
    } else {
        format!("{path}/v1")
    };
    url.set_path(&normalized_path);
    Ok(url.to_string().trim_end_matches('/').to_string())
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    use tempfile::tempdir;

    use super::{
        load_persisted_provider_from_path, save_global_provider, save_global_provider_to_path,
        AppConfig,
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
            "VLLM_BASE_URL",
            "SGLANG_BASE_URL",
            "LOCALAI_BASE_URL",
            "LITELLM_BASE_URL",
            "TGI_BASE_URL",
            "TEXT_GENERATION_WEBUI_BASE_URL",
            "JAN_BASE_URL",
            "MLX_LM_BASE_URL",
            "LLMETER_CONFIG_DIR",
            "LLMETER_OUTPUT_DIR",
            "LLMETER_TIMEOUT",
            "LLMETER_RUNS",
            "LLMETER_MAX_TOKENS",
            "LLMETER_TEMPERATURE",
        ] {
            std::env::remove_var(key);
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn persists_global_provider() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.json");
        save_global_provider_to_path(ProviderKind::Lmstudio, &path).unwrap();
        assert_eq!(
            load_persisted_provider_from_path(&path).unwrap(),
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
        let config = AppConfig::from_env(&cli).unwrap();
        assert_eq!(config.provider, ProviderKind::LlamaCpp);

        std::env::set_var("LLMETER_PROVIDER", "lmstudio");
        let config = AppConfig::from_env(&cli).unwrap();
        assert_eq!(config.provider, ProviderKind::Lmstudio);

        let cli_override = Cli::parse_from(["llmeter", "--provider", "ollama"]);
        let config = AppConfig::from_env(&cli_override).unwrap();
        assert_eq!(config.provider, ProviderKind::Ollama);

        clear_env();
        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli).unwrap();
        assert_eq!(config.provider, ProviderKind::Ollama);
    }

    #[test]
    fn default_output_dir_uses_llmeter_home_when_present() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let temp = tempdir().unwrap();
        std::env::set_var("LLMETER_HOME", temp.path());

        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli).unwrap();

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
        let config = AppConfig::from_env(&cli).unwrap();

        assert_eq!(config.output_dir, PathBuf::from("custom-results"));
    }

    #[test]
    fn invalid_timeout_values_are_reported_instead_of_defaulted() {
        let _guard = env_lock().lock().unwrap();
        for value in ["-1", "0", "NaN", "inf", "86401", "not-a-number"] {
            clear_env();
            std::env::set_var("LLMETER_TIMEOUT", value);
            let cli = Cli::parse_from(["llmeter"]);
            let error = AppConfig::from_env(&cli).unwrap_err().to_string();
            assert!(error.contains("LLMETER_TIMEOUT"), "{error}");
        }

        clear_env();
        let cli = Cli::parse_from(["llmeter", "--timeout=-1"]);
        let error = AppConfig::from_env(&cli).unwrap_err().to_string();
        assert!(error.contains("--timeout"), "{error}");
    }

    #[test]
    fn invalid_numeric_defaults_are_reported_instead_of_defaulted() {
        let _guard = env_lock().lock().unwrap();
        for (name, value) in [
            ("LLMETER_RUNS", "not-a-number"),
            ("LLMETER_RUNS", "0"),
            ("LLMETER_MAX_TOKENS", "0"),
            ("LLMETER_TEMPERATURE", "NaN"),
            ("LLMETER_TEMPERATURE", "-1"),
        ] {
            clear_env();
            std::env::set_var(name, value);
            let cli = Cli::parse_from(["llmeter"]);
            let error = AppConfig::from_env(&cli).unwrap_err().to_string();
            assert!(error.contains(name), "{error}");
        }
        clear_env();
    }

    #[test]
    fn persisted_provider_uses_llmeter_home_config_root_by_default() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let temp = tempdir().unwrap();
        std::env::set_var("LLMETER_HOME", temp.path());

        save_global_provider(ProviderKind::Lmstudio).unwrap();

        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli).unwrap();

        assert_eq!(config.provider, ProviderKind::Lmstudio);
    }

    #[test]
    fn provider_override_recalculates_default_base_url() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli)
            .unwrap()
            .with_provider(ProviderKind::Lmstudio)
            .unwrap();
        assert_eq!(config.provider, ProviderKind::Lmstudio);
        assert_eq!(config.base_url, "http://localhost:1234/v1");
    }

    #[test]
    fn explicit_base_url_is_retained_when_provider_changes() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let cli = Cli::parse_from(["llmeter", "--base-url", "http://localhost:9999/v1"]);
        let config = AppConfig::from_env(&cli)
            .unwrap()
            .with_provider(ProviderKind::Lmstudio)
            .unwrap();
        assert_eq!(config.base_url, "http://localhost:9999/v1");
    }

    #[test]
    fn global_base_url_env_is_retained_when_provider_changes() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        std::env::set_var("LLMETER_BASE_URL", "http://localhost:9998/v1");
        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli)
            .unwrap()
            .with_provider(ProviderKind::Lmstudio)
            .unwrap();
        assert_eq!(config.base_url, "http://localhost:9998/v1");
        clear_env();
    }

    #[test]
    fn malformed_and_unknown_persisted_configuration_are_errors() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let temp = tempdir().unwrap();
        std::env::set_var("LLMETER_CONFIG_DIR", temp.path());
        let path = temp.path().join("config.json");
        std::fs::write(&path, "{not-json").unwrap();

        let cli = Cli::parse_from(["llmeter"]);
        let error = AppConfig::from_env(&cli).unwrap_err().to_string();
        assert!(error.contains("Invalid persisted configuration"), "{error}");
        assert!(error.contains(path.to_string_lossy().as_ref()), "{error}");

        std::fs::write(&path, r#"{"provider":"unknown-provider"}"#).unwrap();
        let error = AppConfig::from_env(&cli).unwrap_err().to_string();
        assert!(error.contains("unknown-provider"), "{error}");
        clear_env();
    }

    #[test]
    fn invalid_provider_environment_value_is_not_defaulted() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        std::env::set_var("LLMETER_PROVIDER", "unknown-provider");
        let cli = Cli::parse_from(["llmeter"]);
        let error = AppConfig::from_env(&cli).unwrap_err().to_string();
        assert!(error.contains("LLMETER_PROVIDER"), "{error}");
        assert!(error.contains("unknown-provider"), "{error}");
        clear_env();
    }

    #[test]
    fn base_urls_are_parsed_normalized_and_source_specific() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let cli = Cli::parse_from(["llmeter", "--base-url", "http://localhost:1234/api/"]);
        let config = AppConfig::from_env(&cli).unwrap();
        assert_eq!(config.base_url, "http://localhost:1234/api/v1");

        clear_env();
        std::env::set_var("OLLAMA_HOST", "http://localhost:11434/v1/");
        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli).unwrap();
        assert_eq!(config.base_url, "http://localhost:11434/v1");

        for value in ["localhost:1234", "file:///tmp/provider"] {
            clear_env();
            std::env::set_var("LLMETER_BASE_URL", value);
            let cli = Cli::parse_from(["llmeter"]);
            let error = AppConfig::from_env(&cli).unwrap_err().to_string();
            assert!(error.contains("LLMETER_BASE_URL"), "{error}");
            assert!(error.contains(value), "{error}");
        }

        clear_env();
        std::env::set_var("LLMETER_BASE_URL", "http://host/v1?token=secret");
        let cli = Cli::parse_from(["llmeter"]);
        let error = AppConfig::from_env(&cli).unwrap_err().to_string();
        assert!(error.contains("LLMETER_BASE_URL"), "{error}");
        assert!(!error.contains("secret"), "{error}");
        clear_env();
    }

    #[test]
    fn provider_switch_validates_provider_specific_url() {
        let _guard = env_lock().lock().unwrap();
        clear_env();
        let cli = Cli::parse_from(["llmeter"]);
        let config = AppConfig::from_env(&cli).unwrap();
        std::env::set_var("LMSTUDIO_BASE_URL", "not-a-url");
        let error = config
            .with_provider(ProviderKind::Lmstudio)
            .unwrap_err()
            .to_string();
        assert!(error.contains("LMSTUDIO_BASE_URL"), "{error}");
        clear_env();
    }
}
