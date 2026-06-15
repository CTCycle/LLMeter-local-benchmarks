use serde::{Deserialize, Serialize};

use crate::quality::catalog::{QualityBenchmarkCatalogEntry, QualityFramework};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityPlan {
    pub framework: QualityFramework,
    pub task: String,
    pub model: String,
    pub dry_run: bool,
    pub command_preview: Vec<String>,
    pub catalog_entry: Option<QualityBenchmarkCatalogEntry>,
}
