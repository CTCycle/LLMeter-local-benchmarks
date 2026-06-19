use std::process::Command;

use serde::{Deserialize, Serialize};
use sysinfo::{Disks, ProcessesToUpdate, System};

use crate::config::AppConfig;
use crate::performance::process::{discover_provider_processes, ProcessSnapshot};
use crate::providers::ProviderKind;
use crate::utils::error_chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub os: String,
    pub cpu_count: usize,
    pub total_memory: u64,
    pub available_memory: u64,
    pub used_memory: u64,
    pub memory_used_ratio: f64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub swap_used_ratio: f64,
    pub disk_count: usize,
    pub disk_available_bytes: u64,
    pub process_memory_before: Option<u64>,
    pub process_memory_after: Option<u64>,
    pub provider_processes: Vec<ProcessSnapshot>,
    pub provider_base_url: String,
    pub provider_kind: ProviderKind,
    pub llmeter_version: String,
    pub gpu_probe_output: Option<String>,
    pub gpu_probe_error: Option<String>,
}

pub fn capture_environment_snapshot(
    config: &AppConfig,
    process_memory_before: Option<u64>,
    process_memory_after: Option<u64>,
    provider_process_hint: Option<&str>,
) -> EnvironmentSnapshot {
    let mut system = System::new_all();
    system.refresh_memory();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let process_id = sysinfo::Pid::from_u32(std::process::id());
    let current_process_memory = system.process(process_id).map(|process| process.memory());
    let used_memory = system
        .total_memory()
        .saturating_sub(system.available_memory());
    let disks = Disks::new_with_refreshed_list();
    let disk_available_bytes = disks.iter().map(|disk| disk.available_space()).sum();

    let (gpu_probe_output, gpu_probe_error) = match Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,memory.used,utilization.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        Ok(output) if output.status.success() => (
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
            None,
        ),
        Ok(output) => (
            None,
            Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        ),
        Err(error) => (None, Some(error_chain(&error))),
    };

    EnvironmentSnapshot {
        os: System::name().unwrap_or_else(|| std::env::consts::OS.to_string()),
        cpu_count: system.cpus().len(),
        total_memory: system.total_memory(),
        available_memory: system.available_memory(),
        used_memory,
        memory_used_ratio: ratio(used_memory, system.total_memory()),
        total_swap: system.total_swap(),
        used_swap: system.used_swap(),
        swap_used_ratio: ratio(system.used_swap(), system.total_swap()),
        disk_count: disks.len(),
        disk_available_bytes,
        process_memory_before: process_memory_before.or(current_process_memory),
        process_memory_after: process_memory_after.or(current_process_memory),
        provider_processes: discover_provider_processes(config.provider, provider_process_hint),
        provider_base_url: config.base_url.clone(),
        provider_kind: config.provider,
        llmeter_version: env!("CARGO_PKG_VERSION").to_string(),
        gpu_probe_output,
        gpu_probe_error,
    }
}

fn ratio(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        used as f64 / total as f64
    }
}
