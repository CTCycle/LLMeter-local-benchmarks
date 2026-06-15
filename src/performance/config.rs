use std::collections::{BTreeSet, HashMap};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::LLMeterError;
use crate::providers::ProviderKind;

pub const DOCUMENTED_MAX_PROMPT_TOKENS: u32 = 32768;
pub const DOCUMENTED_MAX_OUTPUT_TOKENS: u32 = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum PerformanceProfile {
    Smoke,
    Latency,
    Throughput,
    Sweep,
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
}

impl std::str::FromStr for PerformanceProfile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "smoke" => Ok(Self::Smoke),
            "latency" => Ok(Self::Latency),
            "throughput" => Ok(Self::Throughput),
            "sweep" => Ok(Self::Sweep),
            other => Err(format!("Unknown performance profile '{other}'.")),
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PerformanceExportRequest {
    Json,
    Csv,
    Both,
    None,
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
    ) -> anyhow::Result<Self> {
        let unsafe_large_prompt = extra_params
            .get("unsafe_large_prompt")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);

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
                warmup: WarmupConfig { requests: 1 },
                runs: 3,
                stream,
                workload_jsonl,
                extra_params,
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
                warmup: WarmupConfig { requests: 1 },
                runs: 5,
                stream,
                workload_jsonl,
                extra_params,
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
                warmup: WarmupConfig { requests: 1 },
                runs: 4,
                stream,
                workload_jsonl,
                extra_params,
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
                warmup: WarmupConfig { requests: 1 },
                runs: 3,
                stream,
                workload_jsonl,
                extra_params,
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

        plan.validate(unsafe_large_prompt)?;
        Ok(plan)
    }

    pub fn validate(&self, unsafe_large_prompt: bool) -> anyhow::Result<()> {
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
        if !unsafe_large_prompt
            && self
                .prompt_sizes
                .estimated_tokens
                .iter()
                .any(|value| *value > DOCUMENTED_MAX_PROMPT_TOKENS)
        {
            return Err(LLMeterError::InvalidOption(format!(
                "Prompt token values above {DOCUMENTED_MAX_PROMPT_TOKENS} require --param unsafe_large_prompt=true."
            ))
            .into());
        }
        if !unsafe_large_prompt
            && self
                .output_sizes
                .estimated_tokens
                .iter()
                .any(|value| *value > DOCUMENTED_MAX_OUTPUT_TOKENS)
        {
            return Err(LLMeterError::InvalidOption(format!(
                "Output token values above {DOCUMENTED_MAX_OUTPUT_TOKENS} require --param unsafe_large_prompt=true."
            ))
            .into());
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
