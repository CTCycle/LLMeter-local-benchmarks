use std::path::Path;
use std::time::SystemTime;

use chrono::{DateTime, Utc};

pub fn utc_now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub fn utc_now_run_id_stamp() -> String {
    Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string()
}

pub fn format_system_time_utc(value: SystemTime) -> String {
    let timestamp: DateTime<Utc> = value.into();
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn ns_to_ms(value: Option<u128>) -> Option<f64> {
    value.map(|v| (v as f64 / 1_000_000.0 * 1000.0).round() / 1000.0)
}

pub fn ns_to_ms_u64(value: Option<u64>) -> Option<f64> {
    value.map(|v| (v as f64 / 1_000_000.0 * 1000.0).round() / 1000.0)
}

pub fn ns_to_seconds(value: u128) -> f64 {
    value as f64 / 1_000_000_000.0
}

pub fn slugify(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut prev_dash = false;
    for c in value.trim().chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
            result.push(c);
            prev_dash = false;
        } else if !prev_dash {
            result.push('-');
            prev_dash = true;
        }
    }
    let trimmed = result.trim_matches(&['-', '.', '_'][..]);
    if trimmed.is_empty() {
        "value".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

pub fn preview(text: &str, limit: usize) -> String {
    let compact: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.len() <= limit {
        compact
    } else {
        let mut s: String = compact.chars().take(limit - 1).collect();
        s.push('…');
        s
    }
}

pub fn error_chain(error: &dyn std::error::Error) -> String {
    let mut parts = Vec::new();
    let mut current = Some(error);

    while let Some(err) = current {
        let message = err.to_string();
        if !message.is_empty() && parts.last() != Some(&message) {
            parts.push(message);
        }
        current = err.source();
    }

    parts.join(": ")
}
