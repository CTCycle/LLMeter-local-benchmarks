use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrompt {
    pub id: String,
    pub prompt: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub estimated_prompt_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceWorkload {
    pub prompts: Vec<PerformancePrompt>,
}

#[derive(Debug, Deserialize)]
struct JsonlPrompt {
    id: String,
    prompt: String,
    #[serde(default)]
    tags: Vec<String>,
}

pub fn load_jsonl_workload(path: &Path) -> anyhow::Result<PerformanceWorkload> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open JSONL workload {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut prompts = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to read JSONL line {}", index + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let prompt: JsonlPrompt = serde_json::from_str(&line)
            .with_context(|| format!("Invalid JSONL prompt line {}", index + 1))?;
        prompts.push(PerformancePrompt {
            id: prompt.id,
            estimated_prompt_tokens: estimate_tokens(&prompt.prompt),
            prompt: prompt.prompt,
            tags: prompt.tags,
        });
    }

    Ok(PerformanceWorkload { prompts })
}

pub fn synthetic_prompt_for_tokens(target_tokens: u32) -> PerformancePrompt {
    let base = "Summarize the system behavior, identify bottlenecks, and explain your reasoning in detail.";
    let mut prompt = String::new();
    while estimate_tokens(&prompt) < target_tokens {
        if !prompt.is_empty() {
            prompt.push(' ');
        }
        prompt.push_str(base);
    }

    PerformancePrompt {
        id: format!("synthetic-{target_tokens}"),
        prompt,
        tags: vec!["synthetic".to_string()],
        estimated_prompt_tokens: target_tokens,
    }
}

fn estimate_tokens(text: &str) -> u32 {
    let words = text.split_whitespace().count() as u32;
    words.saturating_mul(4) / 3 + 1
}
