use std::process::{Command, Stdio};

use llmeter::cli::{parse_positive_u32, parse_temperature};
use tempfile::TempDir;

#[test]
fn non_tty_invocation_prints_help_and_returns_usage_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run llmeter without a terminal");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout).expect("help is UTF-8");
    assert!(stdout.contains("Usage:"), "{stdout}");
    assert!(!stdout.contains("Main menu"), "{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn explicit_menu_also_requires_a_terminal() {
    let output = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .arg("menu")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run explicit menu without a terminal");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
}

#[test]
fn cli_version_comes_from_cargo_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .arg("--version")
        .output()
        .expect("run version command");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("llmeter {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn invalid_configuration_uses_documented_error_exit() {
    let output = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .args(["--timeout", "0", "providers", "list"])
        .output()
        .expect("run with invalid configuration");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--timeout"));
    assert!(output.stdout.is_empty());
}

#[test]
fn malformed_persisted_configuration_uses_documented_error_exit() {
    let temp = TempDir::new().expect("temporary config directory");
    std::fs::write(temp.path().join("config.json"), "{not-json")
        .expect("write malformed configuration");

    let output = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .env("LLMETER_CONFIG_DIR", temp.path())
        .args(["providers", "list"])
        .output()
        .expect("run llmeter with malformed configuration");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid persisted configuration"));
    assert!(output.stdout.is_empty());
}

#[test]
fn benchmark_cli_parsers_reject_invalid_numeric_values() {
    for value in ["0", "-1", "not-a-number"] {
        assert!(parse_positive_u32(value).is_err(), "accepted {value}");
    }
    for value in ["-0.1", "NaN", "inf", "-inf", "not-a-number"] {
        assert!(parse_temperature(value).is_err(), "accepted {value}");
    }
    assert_eq!(parse_positive_u32("3"), Ok(3));
    assert_eq!(parse_temperature("0.25"), Ok(0.25));
}
