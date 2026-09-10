use std::collections::{BTreeSet, HashMap};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::LLMeterError;
use crate::providers::ProviderKind;

pub const DOCUMENTED_MAX_PROMPT_TOKENS: u32 = 32768;
pub const DOCUMENTED_MAX_OUTPUT_TOKENS: u32 = 8192;
pub const DEFAULT_MAX_PERFORMANCE_REQUESTS: u32 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum PerformanceProfile {
    Smoke,
    Latency,
    Throughput,
    Sweep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum LoadMeasurementMode {
    Off,
    FirstRequestEstimate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum TelemetryLevel {
    Off,
    Standard,
    Detailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ReportDetailLevel {
    Summary,
    Detailed,
    Full,
}

impl PerformanceProfile {
    pub fn label(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Latency => "latency",
            Self::Throughput => "throughput",
            Self::Sweep => "sweep",
        }
    }

    pub const fn default_runs(self) -> u32 {
        match self {
            Self::Smoke => 3,
            Self::Latency => 5,
            Self::Throughput => 4,
            Self::Sweep => 3,
        }
    }

    pub const fn default_warmup_requests(self) -> u32 {
        1
    }
}

impl std::str::FromStr for PerformanceProfile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let canonical = value.trim().to_ascii_lowercase();
        Self::value_variants()
            .iter()
            .copied()
            .find(|profile| profile.label() == canonical.as_str())
            .ok_or_else(|| format!("Unknown performance profile '{canonical}'."))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptSizeSpec {
    pub estimated_tokens: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputSizeSpec {
    pub estimated_tokens: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcurrencySpec {
    pub levels: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarmupConfig {
    pub requests: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PerformanceSafetyOptions {
    pub allow_large_prompt: bool,
    pub allow_large_matrix: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerformancePlan {
    pub provider: ProviderKind,
    pub models: Vec<String>,
    pub profile: PerformanceProfile,
    pub prompt_sizes: PromptSizeSpec,
    pub output_sizes: OutputSizeSpec,
    pub concurrency: ConcurrencySpec,
    pub warmup: WarmupConfig,
    pub runs: u32,
    pub stream: bool,
    pub workload_jsonl: Option<String>,
    pub extra_params: HashMap<String, Value>,
    pub safety: PerformanceSafetyOptions,
    pub load_measurement: LoadMeasurementMode,
    pub load_probe_runs: u32,
    pub telemetry: TelemetryLevel,
    pub sample_interval_ms: u64,
    pub provider_process: Option<String>,
    pub probe_capabilities: bool,
    pub probe_all_endpoints: bool,
    pub model_cache_dir: Option<String>,
    pub scan_model_cache: bool,
    pub detail: ReportDetailLevel,
}

impl PerformancePlan {
    #[allow(clippy::too_many_arguments)]
    pub fn from_cli(
        provider: ProviderKind,
        models: Vec<String>,
        profile: PerformanceProfile,
        prompt_tokens: Option<&str>,
        output_tokens: Option<&str>,
        concurrency: Option<&str>,
        warmup: Option<u32>,
        runs: Option<u32>,
        stream: bool,
        workload_jsonl: Option<String>,
        extra_params: HashMap<String, Value>,
        load_measurement: LoadMeasurementMode,
        load_probe_runs: u32,
        telemetry: TelemetryLevel,
        sample_interval_ms: u64,
        provider_process: Option<String>,
        probe_capabilities: bool,
        probe_all_endpoints: bool,
        model_cache_dir: Option<String>,
        scan_model_cache: bool,
        detail: ReportDetailLevel,
        max_requests: Option<u32>,
    ) -> anyhow::Result<Self> {
        Self::from_cli_with_safety(
            provider,
            models,
            profile,
            prompt_tokens,
            output_tokens,
            concurrency,
            warmup,
            runs,
            stream,
            workload_jsonl,
            extra_params,
            PerformanceSafetyOptions::default(),
            load_measurement,
            load_probe_runs,
            telemetry,
            sample_interval_ms,
            provider_process,
            probe_capabilities,
            probe_all_endpoints,
            model_cache_dir,
            scan_model_cache,
            detail,
            max_requests,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_cli_with_safety(
        provider: ProviderKind,
        models: Vec<String>,
        profile: PerformanceProfile,
        prompt_tokens: Option<&str>,
        output_tokens: Option<&str>,
        concurrency: Option<&str>,
        warmup: Option<u32>,
        runs: Option<u32>,
        stream: bool,
        workload_jsonl: Option<String>,
        extra_params: HashMap<String, Value>,
        safety: PerformanceSafetyOptions,
        load_measurement: LoadMeasurementMode,
        load_probe_runs: u32,
        telemetry: TelemetryLevel,
        sample_interval_ms: u64,
        provider_process: Option<String>,
        probe_capabilities: bool,
        probe_all_endpoints: bool,
        model_cache_dir: Option<String>,
        scan_model_cache: bool,
        detail: ReportDetailLevel,
        max_requests: Option<u32>,
    ) -> anyhow::Result<Self> {
        let mut plan = match profile {
            PerformanceProfile::Smoke => Self {
                provider,
                models,
                profile,
                prompt_sizes: PromptSizeSpec {
                    estimated_tokens: vec![128, 512],
                },
                output_sizes: OutputSizeSpec {
                    estimated_tokens: vec![128],
                },
                concurrency: ConcurrencySpec { levels: vec![1] },
                warmup: WarmupConfig {
                    requests: profile.default_warmup_requests(),
                },
                runs: profile.default_runs(),
                stream,
                workload_jsonl,
                extra_params,
                safety: safety.clone(),
                load_measurement,
                load_probe_runs,
                telemetry,
                sample_interval_ms,
                provider_process: provider_process.clone(),
                probe_capabilities,
                probe_all_endpoints,
                model_cache_dir: model_cache_dir.clone(),
                scan_model_cache,
                detail,
            },
            PerformanceProfile::Latency => Self {
                provider,
                models,
                profile,
                prompt_sizes: PromptSizeSpec {
                    estimated_tokens: vec![128, 512, 2048],
                },
                output_sizes: OutputSizeSpec {
                    estimated_tokens: vec![128],
                },
                concurrency: ConcurrencySpec { levels: vec![1] },
                warmup: WarmupConfig {
                    requests: profile.default_warmup_requests(),
                },
                runs: profile.default_runs(),
                stream,
                workload_jsonl,
                extra_params,
                safety: safety.clone(),
                load_measurement,
                load_probe_runs,
                telemetry,
                sample_interval_ms,
                provider_process: provider_process.clone(),
                probe_capabilities,
                probe_all_endpoints,
                model_cache_dir: model_cache_dir.clone(),
                scan_model_cache,
                detail,
            },
            PerformanceProfile::Throughput => Self {
                provider,
                models,
                profile,
                prompt_sizes: PromptSizeSpec {
                    estimated_tokens: vec![512],
                },
                output_sizes: OutputSizeSpec {
                    estimated_tokens: vec![256],
                },
                concurrency: ConcurrencySpec {
                    levels: vec![1, 2, 4, 8],
                },
                warmup: WarmupConfig {
                    requests: profile.default_warmup_requests(),
                },
                runs: profile.default_runs(),
                stream,
                workload_jsonl,
                extra_params,
                safety: safety.clone(),
                load_measurement,
                load_probe_runs,
                telemetry,
                sample_interval_ms,
                provider_process: provider_process.clone(),
                probe_capabilities,
                probe_all_endpoints,
                model_cache_dir: model_cache_dir.clone(),
                scan_model_cache,
                detail,
            },
            PerformanceProfile::Sweep => Self {
                provider,
                models,
                profile,
                prompt_sizes: PromptSizeSpec {
                    estimated_tokens: vec![128, 512, 2048],
                },
                output_sizes: OutputSizeSpec {
                    estimated_tokens: vec![64, 128, 256],
                },
                concurrency: ConcurrencySpec {
                    levels: vec![1, 2, 4],
                },
                warmup: WarmupConfig {
                    requests: profile.default_warmup_requests(),
                },
                runs: profile.default_runs(),
                stream,
                workload_jsonl,
                extra_params,
                safety: safety.clone(),
                load_measurement,
                load_probe_runs,
                telemetry,
                sample_interval_ms,
                provider_process,
                probe_capabilities,
                probe_all_endpoints,
                model_cache_dir,
                scan_model_cache,
                detail,
            },
        };

        if let Some(value) = prompt_tokens {
            plan.prompt_sizes.estimated_tokens = parse_csv_u32(value)?;
        }
        if let Some(value) = output_tokens {
            plan.output_sizes.estimated_tokens = parse_csv_u32(value)?;
        }
        if let Some(value) = concurrency {
            plan.concurrency.levels = parse_csv_u32(value)?;
        }
        if let Some(value) = warmup {
            plan.warmup.requests = value;
        }
        if let Some(value) = runs {
            plan.runs = value;
        }

        plan.validate(
            safety.allow_large_prompt,
            max_requests,
            safety.allow_large_matrix,
        )?;
        Ok(plan)
    }

    pub fn scenario_count(&self) -> u32 {
        self.models
            .len()
            .saturating_mul(self.prompt_sizes.estimated_tokens.len())
            .saturating_mul(self.output_sizes.estimated_tokens.len())
            .saturating_mul(self.concurrency.levels.len()) as u32
    }

    pub fn total_warmup_requests(&self) -> u32 {
        self.scenario_count().saturating_mul(self.warmup.requests)
    }

    pub fn total_measured_requests(&self) -> u32 {
        self.scenario_count().saturating_mul(self.runs)
    }

    pub fn total_requests(&self) -> u32 {
        self.total_warmup_requests()
            .saturating_add(self.total_measured_requests())
    }

    pub fn validate(
        &self,
        allow_large_prompt: bool,
        max_requests: Option<u32>,
        allow_large_matrix: bool,
    ) -> anyhow::Result<()> {
        if self.runs == 0 {
            return Err(LLMeterError::InvalidOption(
                "Performance runs must be greater than zero.".to_string(),
            )
            .into());
        }
        if self.concurrency.levels.contains(&0) {
            return Err(LLMeterError::InvalidOption(
                "Performance concurrency values must be greater than zero.".to_string(),
            )
            .into());
        }
        if self.load_measurement != LoadMeasurementMode::Off && self.load_probe_runs == 0 {
            return Err(LLMeterError::InvalidOption(
                "Load probe runs must be greater than zero when load measurement is enabled."
                    .to_string(),
            )
            .into());
        }
        if self.sample_interval_ms < 100 {
            return Err(LLMeterError::InvalidOption(
                "Telemetry sample interval must be at least 100 ms.".to_string(),
            )
            .into());
        }
        if !allow_large_prompt
            && self
                .prompt_sizes
                .estimated_tokens
                .iter()
                .any(|value| *value > DOCUMENTED_MAX_PROMPT_TOKENS)
        {
            return Err(LLMeterError::InvalidOption(format!(
                "Prompt token values above {DOCUMENTED_MAX_PROMPT_TOKENS} require --allow-large-prompt."
            ))
            .into());
        }
        if !allow_large_prompt
            && self
                .output_sizes
                .estimated_tokens
                .iter()
                .any(|value| *value > DOCUMENTED_MAX_OUTPUT_TOKENS)
        {
            return Err(LLMeterError::InvalidOption(format!(
                "Output token values above {DOCUMENTED_MAX_OUTPUT_TOKENS} require --allow-large-prompt."
            ))
            .into());
        }
        if let Some(limit) = max_requests {
            if self.total_requests() > limit && !allow_large_matrix {
                return Err(LLMeterError::InvalidOption(format!(
                    "Performance plan requests {} exceed --max-requests {limit}. Reduce the matrix, raise --max-requests, use --dry-run to inspect it, or pass --allow-large-matrix.",
                    self.total_requests()
                ))
                .into());
            }
        }
        Ok(())
    }
}

fn parse_csv_u32(value: &str) -> anyhow::Result<Vec<u32>> {
    let mut items = BTreeSet::new();
    for chunk in value.split(',') {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed: u32 = trimmed.parse().map_err(|_| {
            LLMeterError::InvalidOption(format!("Invalid numeric value '{trimmed}'."))
        })?;
        items.insert(parsed);
    }
    if items.is_empty() {
        return Err(LLMeterError::InvalidOption(
            "Expected at least one numeric value.".to_string(),
        )
        .into());
    }
    Ok(items.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use clap::ValueEnum;

    use super::PerformanceProfile;

    #[test]
    fn performance_profile_cli_names_match_canonical_labels() {
        for profile in PerformanceProfile::value_variants().iter().copied() {
            let possible = profile.to_possible_value().expect("profile value");
            assert_eq!(possible.get_name(), profile.label());
            assert_eq!(profile.label().parse::<PerformanceProfile>(), Ok(profile));
        }
    }

    #[test]
    fn performance_profile_defaults_are_canonical() {
        assert_eq!(PerformanceProfile::Smoke.default_runs(), 3);
        assert_eq!(PerformanceProfile::Latency.default_runs(), 5);
        assert_eq!(PerformanceProfile::Throughput.default_runs(), 4);
        assert_eq!(PerformanceProfile::Sweep.default_runs(), 3);
        for profile in [
            PerformanceProfile::Smoke,
            PerformanceProfile::Latency,
            PerformanceProfile::Throughput,
            PerformanceProfile::Sweep,
        ] {
            assert_eq!(profile.default_warmup_requests(), 1);
        }
    }
}
