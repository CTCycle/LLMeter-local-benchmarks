use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sysinfo::{Disks, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSample {
    pub elapsed_ms: u64,
    pub memory_used_ratio: f64,
    pub swap_used_ratio: f64,
    pub process_memory_bytes: Option<u64>,
    pub cpu_usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySummary {
    pub sample_count: usize,
    pub max_memory_used_ratio: Option<f64>,
    pub max_swap_used_ratio: Option<f64>,
    pub max_process_memory_bytes: Option<u64>,
    pub max_cpu_usage_percent: Option<f32>,
    pub disk_count: usize,
    pub disk_available_bytes: u64,
    pub warnings: Vec<String>,
}

pub struct TelemetrySampler {
    stop: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<SystemSample>>>,
    handle: Option<JoinHandle<()>>,
}

impl TelemetrySampler {
    pub fn start(sample_interval_ms: u64) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let samples = Arc::new(Mutex::new(Vec::new()));
        let thread_stop = Arc::clone(&stop);
        let thread_samples = Arc::clone(&samples);
        let interval = Duration::from_millis(sample_interval_ms.max(100));
        let handle = thread::spawn(move || {
            let started = Instant::now();
            let mut system = System::new_all();
            while !thread_stop.load(Ordering::Relaxed) {
                system.refresh_memory();
                system.refresh_cpu_usage();
                system.refresh_processes(ProcessesToUpdate::All, true);
                let pid = sysinfo::Pid::from_u32(std::process::id());
                let process_memory = system.process(pid).map(|process| process.memory());
                let memory_used = system
                    .total_memory()
                    .saturating_sub(system.available_memory());
                let memory_used_ratio = ratio(memory_used, system.total_memory());
                let swap_used_ratio = ratio(system.used_swap(), system.total_swap());
                let cpu_usage_percent = system.global_cpu_usage();
                if let Ok(mut locked) = thread_samples.lock() {
                    locked.push(SystemSample {
                        elapsed_ms: started.elapsed().as_millis() as u64,
                        memory_used_ratio,
                        swap_used_ratio,
                        process_memory_bytes: process_memory,
                        cpu_usage_percent,
                    });
                }
                thread::sleep(interval);
            }
        });
        Self {
            stop,
            samples,
            handle: Some(handle),
        }
    }

    pub fn stop(mut self) -> Vec<SystemSample> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        self.samples
            .lock()
            .map(|samples| samples.clone())
            .unwrap_or_default()
    }
}

pub fn summarize_samples(samples: &[SystemSample]) -> TelemetrySummary {
    let disks = Disks::new_with_refreshed_list();
    let disk_available_bytes = disks.iter().map(|disk| disk.available_space()).sum();
    let max_swap_used_ratio = samples
        .iter()
        .map(|sample| sample.swap_used_ratio)
        .reduce(f64::max);
    let mut warnings = Vec::new();
    if max_swap_used_ratio.unwrap_or_default() > 0.20 {
        warnings.push("Swap used ratio exceeded 0.20 during telemetry sampling.".to_string());
    }

    TelemetrySummary {
        sample_count: samples.len(),
        max_memory_used_ratio: samples
            .iter()
            .map(|sample| sample.memory_used_ratio)
            .reduce(f64::max),
        max_swap_used_ratio,
        max_process_memory_bytes: samples
            .iter()
            .filter_map(|sample| sample.process_memory_bytes)
            .max(),
        max_cpu_usage_percent: samples
            .iter()
            .map(|sample| sample.cpu_usage_percent)
            .reduce(f32::max),
        disk_count: disks.len(),
        disk_available_bytes,
        warnings,
    }
}

fn ratio(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        used as f64 / total as f64
    }
}
