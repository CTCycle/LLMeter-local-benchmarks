use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::performance::config::PerformancePlan;
use crate::providers::{ProviderClient, ProviderKind};
use crate::utils::error_chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInventoryMeasurement {
    pub model: String,
    pub provider: ProviderKind,
    pub metadata_latency_ms: Option<f64>,
    pub metadata_payload_bytes: Option<u64>,
    pub cache_dir: Option<String>,
    pub cache_bytes: Option<u64>,
    pub notes: Vec<String>,
}

pub fn measure_model_inventory(
    client: &ProviderClient,
    models: &[String],
    plan: &PerformancePlan,
) -> Vec<ModelInventoryMeasurement> {
    models
        .iter()
        .map(|model| measure_one_model(client, model, plan))
        .collect()
}

fn measure_one_model(
    client: &ProviderClient,
    model: &str,
    plan: &PerformancePlan,
) -> ModelInventoryMeasurement {
    let mut notes = Vec::new();
    let started = Instant::now();
    let (metadata_latency_ms, metadata_payload_bytes) = match client.show_model(model) {
        Ok(value) => (
            Some(started.elapsed().as_secs_f64() * 1000.0),
            serde_json::to_vec(&value)
                .ok()
                .map(|bytes| bytes.len() as u64),
        ),
        Err(error) => {
            notes.push(format!(
                "Model metadata probe failed: {}",
                error_chain(&*error)
            ));
            (None, None)
        }
    };

    let cache_dir = plan.model_cache_dir.clone();
    let cache_bytes = if plan.scan_model_cache {
        cache_dir
            .as_deref()
            .map(Path::new)
            .and_then(|path| match directory_size(path) {
                Ok(size) => Some(size),
                Err(error) => {
                    notes.push(format!("Cache scan failed: {error}"));
                    None
                }
            })
    } else {
        notes.push("Local cache scan disabled; pass --scan-model-cache to opt in.".to_string());
        None
    };

    ModelInventoryMeasurement {
        model: model.to_string(),
        provider: client.provider(),
        metadata_latency_ms,
        metadata_payload_bytes,
        cache_dir,
        cache_bytes,
        notes,
    }
}

fn directory_size(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Err(format!("{} does not exist", path.display()));
    }
    let mut total = 0u64;
    let mut stack = vec![PathBuf::from(path)];
    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(&current).map_err(|error| error.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let metadata = entry.metadata().map_err(|error| error.to_string())?;
            if metadata.is_dir() {
                stack.push(entry.path());
            } else {
                total = total.saturating_add(metadata.len());
            }
        }
    }
    Ok(total)
}
