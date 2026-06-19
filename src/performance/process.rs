use serde::{Deserialize, Serialize};
use sysinfo::{ProcessesToUpdate, System};

use crate::providers::ProviderKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub cpu_usage_percent: f32,
}

pub fn discover_provider_processes(
    provider: ProviderKind,
    explicit_hint: Option<&str>,
) -> Vec<ProcessSnapshot> {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let needles = process_needles(provider, explicit_hint);
    system
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let name = process.name().to_string_lossy().to_string();
            let command = process
                .cmd()
                .iter()
                .map(|part| part.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            let haystack = format!("{name} {command}").to_ascii_lowercase();
            if needles.iter().any(|needle| haystack.contains(needle)) {
                Some(ProcessSnapshot {
                    pid: pid.as_u32(),
                    name,
                    command,
                    memory_bytes: process.memory(),
                    virtual_memory_bytes: process.virtual_memory(),
                    cpu_usage_percent: process.cpu_usage(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn process_needles(provider: ProviderKind, explicit_hint: Option<&str>) -> Vec<String> {
    let mut needles = Vec::new();
    if let Some(hint) = explicit_hint {
        if !hint.trim().is_empty() {
            needles.push(hint.trim().to_ascii_lowercase());
        }
    }
    needles.push(
        match provider {
            ProviderKind::Ollama => "ollama",
            ProviderKind::Lmstudio => "lm studio",
            ProviderKind::LlamaCpp => "llama",
            ProviderKind::OpenaiCompatible => "openai",
            ProviderKind::Vllm => "vllm",
            ProviderKind::Sglang => "sglang",
            ProviderKind::Localai => "localai",
            ProviderKind::Litellm => "litellm",
            ProviderKind::Tgi => "text-generation",
            ProviderKind::TextGenerationWebui => "text-generation-webui",
            ProviderKind::Jan => "jan",
            ProviderKind::MlxLm => "mlx",
        }
        .to_string(),
    );
    needles
}
