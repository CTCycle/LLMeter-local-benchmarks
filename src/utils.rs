use std::path::Path;
use std::time::SystemTime;
use std::{fs, io::Write};

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

/// Writes a file through a same-directory temporary file and renames it into place.
///
/// Keeping the temporary file beside the destination makes the final rename atomic on
/// filesystems that support atomic same-volume renames and prevents readers from seeing
/// partially serialized JSON, CSV, or report output.
pub fn atomic_write(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("llmeter-output");
    let temp_path = parent.join(format!(
        ".{file_name}.tmp-{}-{}",
        std::process::id(),
        utc_now_run_id_stamp().replace(':', "")
    ));

    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)?;
        file.write_all(contents)?;
        file.flush()?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp_path, path)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::atomic_write;

    #[test]
    fn atomic_write_replaces_complete_content_without_temp_files() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("result.json");

        atomic_write(&path, br#"{"version":1}"#).unwrap();
        atomic_write(&path, br#"{"version":2,"complete":true}"#).unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            r#"{"version":2,"complete":true}"#
        );
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .count(),
            1
        );
    }

    #[test]
    fn atomic_write_cleans_temporary_file_when_final_rename_fails() {
        let directory = tempdir().unwrap();
        let destination = directory.path().join("occupied");
        fs::create_dir(&destination).unwrap();

        atomic_write(&destination, b"content").unwrap_err();
        let entries = fs::read_dir(directory.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path(), destination);
    }
}
