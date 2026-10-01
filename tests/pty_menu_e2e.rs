// Copyright © 2026 CTCycle
// Licensed under the MIT License.

#![cfg(windows)]

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{
    thread,
    time::{Duration, Instant},
};

use expectrl::{spawn, ControlCode, Expect};
use serde_json::Value;
use tempfile::TempDir;

fn menu_command() -> String {
    format!("\"{}\" --timeout 0.1 menu", env!("CARGO_BIN_EXE_llmeter"))
}

fn menu_command_with_provider(base_url: &str) -> String {
    format!(
        "\"{}\" --provider openai-compatible --base-url {base_url} --timeout 0.1 menu",
        env!("CARGO_BIN_EXE_llmeter")
    )
}

struct MockProvider {
    base_url: String,
    address: String,
    stop: Arc<AtomicBool>,
    chat_requests: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockProvider {
    fn start() -> Self {
        Self::start_with_chat_delay(Duration::ZERO)
    }

    fn start_with_chat_delay(chat_delay: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind PTY mock provider");
        let address = listener
            .local_addr()
            .expect("PTY mock provider address")
            .to_string();
        let base_url = format!("http://{address}/v1");
        let stop = Arc::new(AtomicBool::new(false));
        let chat_requests = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread_chat_requests = Arc::clone(&chat_requests);
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                if let Ok((mut stream, _)) = listener.accept() {
                    handle_connection(&mut stream, chat_delay, &thread_chat_requests);
                }
            }
        });
        Self {
            base_url,
            address,
            stop,
            chat_requests,
            handle: Some(handle),
        }
    }
}

impl Drop for MockProvider {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(&self.address);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn handle_connection(stream: &mut TcpStream, chat_delay: Duration, chat_requests: &AtomicBool) {
    let mut buffer = [0_u8; 4096];
    let size = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..size]);
    let (status, body) = if request.starts_with("GET /v1/models") {
        (
            "200 OK",
            r#"{"object":"list","data":[{"id":"mock-model","object":"model"}]}"#,
        )
    } else if request.starts_with("POST /v1/chat/completions") {
        chat_requests.store(true, Ordering::SeqCst);
        thread::sleep(chat_delay);
        (
            "200 OK",
            r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#,
        )
    } else {
        ("404 Not Found", r#"{"error":{"message":"not found"}}"#)
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn json_result_files(output_dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(output_dir)
        .expect("read PTY output directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn assert_no_temporary_files(output_dir: &Path) {
    let temporary = fs::read_dir(output_dir)
        .expect("read PTY output directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains(".tmp-"))
        })
        .collect::<Vec<_>>();
    assert!(
        temporary.is_empty(),
        "PTY left temporary result files: {temporary:?}"
    );
}

#[test]
fn pty_menu_process_can_be_interrupted_without_leaking() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    // ConPTY launches a real raw-mode menu. `expectrl` 0.9 cannot reliably
    // match crossterm alternate-screen output on this Windows host, so visible
    // navigation assertions stay at the deterministic key-event boundary.
    thread::sleep(Duration::from_millis(500));
    session.send(ControlCode::ETX).expect("interrupt menu");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits");
    assert_eq!(status, 130);
}

#[test]
fn pty_menu_eof_exits_with_interrupt_code() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    thread::sleep(Duration::from_millis(500));
    session.send(ControlCode::EOT).expect("send EOF to menu");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits after EOF");
    assert_eq!(status, 130);
}

#[test]
fn pty_nested_selection_cancel_returns_to_menu() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    thread::sleep(Duration::from_millis(500));

    // Enter Provider setup, move to Set default provider, and open its nested
    // provider selection prompt. Escape must cancel that prompt and return to
    // the provider menu instead of terminating with a generic error.
    session.send("\r").expect("open provider setup");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\u{1b}[B\u{1b}[B\r")
        .expect("open nested provider prompt");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}")
        .expect("cancel nested provider prompt");
    thread::sleep(Duration::from_millis(250));

    session
        .send(ControlCode::ETX)
        .expect("interrupt after nested cancel");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits");
    assert_eq!(status, 130);
}

#[test]
fn pty_nested_menu_back_then_clean_exit() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    thread::sleep(Duration::from_millis(500));

    // Enter Provider setup, use Escape for the submenu Back action, then
    // select the top-level Exit item. The process should leave cleanly.
    session.send("\r").expect("open provider setup");
    thread::sleep(Duration::from_millis(250));
    session.send("\u{1b}").expect("back from provider setup");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\u{1b}[B\u{1b}[B\u{1b}[B\u{1b}[B\r")
        .expect("select top-level exit");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits cleanly");
    assert_eq!(status, 0);
}

#[test]
fn pty_pause_prompt_interrupt_returns_130() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Provider setup -> List supported provider presets -> the
    // pause prompt. Ctrl+C must propagate through pause() and exit 130.
    session.send("\r").expect("open provider setup");
    thread::sleep(Duration::from_millis(250));
    session.send("\u{1b}[B\r").expect("open provider catalog");
    thread::sleep(Duration::from_millis(300));
    session
        .send(ControlCode::ETX)
        .expect("interrupt pause prompt");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits after pause interruption");
    assert_eq!(status, 130);
}

#[test]
fn pty_reports_list_cancel_returns_to_menu() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let provider = MockProvider::start();
    let mut session =
        spawn(menu_command_with_provider(&provider.base_url)).expect("spawn mocked provider menu");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Reports and comparisons -> List saved files -> pause.
    // Escape cancels the pause and returns to the reports menu; Ctrl+C then
    // verifies the process can still terminate through the outer menu.
    session
        .send("\u{1b}[B\u{1b}[B\u{1b}[B\r")
        .expect("open reports menu");
    thread::sleep(Duration::from_millis(300));
    session.send("\r").expect("list report files");
    thread::sleep(Duration::from_millis(300));
    session.send("\u{1b}").expect("cancel report pause");
    thread::sleep(Duration::from_millis(300));
    session
        .send(ControlCode::ETX)
        .expect("interrupt after returning to reports menu");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits after report cancellation");
    assert_eq!(status, 130);
}

#[test]
fn pty_nested_model_selection_cancel_returns_to_menu() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let provider = MockProvider::start();
    let mut session =
        spawn(menu_command_with_provider(&provider.base_url)).expect("spawn mocked provider menu");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Model inventory -> Show raw model metadata -> model selector.
    session.send("\u{1b}[B\r").expect("open model inventory");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\u{1b}[B\r")
        .expect("open model selector");
    thread::sleep(Duration::from_millis(300));
    session.send("\u{1b}").expect("cancel model selector");
    thread::sleep(Duration::from_millis(250));

    session
        .send(ControlCode::ETX)
        .expect("interrupt after model selection cancel");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits");
    assert_eq!(status, 130);
}

#[test]
fn pty_numeric_prompt_cancel_returns_to_menu() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let provider = MockProvider::start();
    let mut session =
        spawn(menu_command_with_provider(&provider.base_url)).expect("spawn mocked provider menu");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Benchmark workspace -> Standard LLM benchmark. Accept the
    // provider, model, and benchmark selectors, then cancel the numeric runs
    // prompt. Cancellation must return to the benchmark menu, not exit 1.
    session
        .send("\u{1b}[B\u{1b}[B\r")
        .expect("open benchmark workspace");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\u{1b}[B\r")
        .expect("open standard benchmark");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept provider selector");
    thread::sleep(Duration::from_millis(500));
    session.send(" \r").expect("select all exposed models");
    thread::sleep(Duration::from_millis(250));
    session.send(" \r").expect("select all benchmarks");
    thread::sleep(Duration::from_millis(250));
    session.send("\u{1b}").expect("cancel numeric prompt");
    thread::sleep(Duration::from_millis(250));

    session
        .send(ControlCode::ETX)
        .expect("interrupt after numeric cancellation");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits");
    assert_eq!(status, 130);
}

#[test]
fn pty_performance_confirmation_interrupt_returns_130() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let provider = MockProvider::start();
    let mut session =
        spawn(menu_command_with_provider(&provider.base_url)).expect("spawn mocked provider menu");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Benchmark workspace -> Performance benchmark. Accept each
    // guided value through the plan preview, then interrupt the confirmation
    // prompt. The nested confirmation must use the documented exit code 130.
    session
        .send("\u{1b}[B\u{1b}[B\r")
        .expect("open benchmark workspace");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\r")
        .expect("open performance benchmark");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept performance provider");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept capability probe");
    thread::sleep(Duration::from_millis(500));
    session.send(" \r").expect("select all exposed models");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept performance profile");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept measured runs");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept warmup requests");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept streaming choice");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept raw export choice");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept report choice");
    thread::sleep(Duration::from_millis(500));
    session
        .send(ControlCode::ETX)
        .expect("interrupt performance confirmation");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits after confirmation interruption");
    assert_eq!(status, 130);
}

#[test]
fn pty_interrupted_performance_run_leaves_no_partial_result_and_allows_recovery() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let output = TempDir::new_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
        .expect("create isolated interrupted-run output");
    let home = output.path().join("home");
    fs::create_dir_all(&home).expect("create isolated PTY home");

    let initial_provider = MockProvider::start();
    let initial = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .args([
            "--provider",
            "openai-compatible",
            "--base-url",
            initial_provider.base_url.as_str(),
            "--timeout",
            "30",
            "--output-dir",
            output.path().to_str().expect("initial output path"),
            "bench",
            "perf",
            "--profile",
            "smoke",
            "--models",
            "mock-model",
            "--prompt-tokens",
            "1",
            "--output-tokens",
            "1",
            "--concurrency",
            "1",
            "--warmup",
            "0",
            "--runs",
            "1",
            "--no-stream",
            "--load-measurement",
            "off",
            "--telemetry",
            "off",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .env("LLMETER_HOME", &home)
        .env("LLMETER_OUTPUT_DIR", output.path())
        .output()
        .expect("create completed result before interruption");
    assert!(
        initial.status.success(),
        "initial PTY result failed: {}",
        String::from_utf8_lossy(&initial.stderr)
    );
    let initial_files = json_result_files(output.path());
    assert_eq!(initial_files.len(), 1, "initial PTY result count");
    let initial_path = initial_files[0].clone();
    let initial_bytes = fs::read(&initial_path).expect("read initial PTY result");
    drop(initial_provider);

    let provider = MockProvider::start_with_chat_delay(Duration::from_secs(3));
    let relative_output = output
        .path()
        .strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .expect("output is under the package directory")
        .display();
    let command = format!(
        "\"\"{}\" --provider openai-compatible --base-url {} --timeout 30 --output-dir {} bench perf --profile smoke --models mock-model --prompt-tokens 128 --output-tokens 1 --concurrency 1 --warmup 0 --runs 10 --no-stream --load-measurement off --telemetry off --export json --report none\"",
        env!("CARGO_BIN_EXE_llmeter"),
        provider.base_url,
        relative_output
    );
    let mut session = spawn(command).expect("spawn long-running performance command");
    let wait_started = Instant::now();
    while !provider.chat_requests.load(Ordering::SeqCst)
        && wait_started.elapsed() < Duration::from_secs(5)
    {
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        provider.chat_requests.load(Ordering::SeqCst),
        "the performance process must reach a delayed provider request before interruption"
    );
    session
        .send(ControlCode::ETX)
        .expect("interrupt performance run");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("interrupted performance process exits");
    assert_ne!(
        status, 0,
        "interrupted performance run must not report success"
    );

    assert_eq!(
        json_result_files(output.path()),
        vec![initial_path.clone()],
        "interruption created a second canonical result"
    );
    assert_eq!(
        fs::read(&initial_path).expect("re-read completed PTY result"),
        initial_bytes,
        "interruption changed the completed result"
    );
    assert_no_temporary_files(output.path());

    let listed = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .args([
            "--provider",
            "openai-compatible",
            "--base-url",
            provider.base_url.as_str(),
            "--timeout",
            "30",
            "--output-dir",
            output.path().to_str().expect("report output path"),
            "report",
            "list",
        ])
        .env("LLMETER_HOME", &home)
        .env("LLMETER_OUTPUT_DIR", output.path())
        .output()
        .expect("list PTY result after interruption");
    assert!(
        listed.status.success(),
        "report list failed after PTY interruption: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    assert!(
        String::from_utf8_lossy(&listed.stdout).contains(
            initial_path
                .file_name()
                .expect("initial PTY result file name")
                .to_string_lossy()
                .as_ref()
        ),
        "report list omitted the completed PTY result"
    );

    let shown = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .args([
            "--provider",
            "openai-compatible",
            "--base-url",
            provider.base_url.as_str(),
            "--timeout",
            "30",
            "--output-dir",
            output.path().to_str().expect("show output path"),
            "report",
            "show",
        ])
        .arg(&initial_path)
        .env("LLMETER_HOME", &home)
        .env("LLMETER_OUTPUT_DIR", output.path())
        .output()
        .expect("show PTY result after interruption");
    assert!(
        shown.status.success(),
        "report show failed after PTY interruption: {}",
        String::from_utf8_lossy(&shown.stderr)
    );

    let recovery = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .args([
            "--provider",
            "openai-compatible",
            "--base-url",
            provider.base_url.as_str(),
            "--timeout",
            "30",
            "--output-dir",
            output.path().to_str().expect("recovery output path"),
            "bench",
            "perf",
            "--profile",
            "smoke",
            "--models",
            "mock-model",
            "--prompt-tokens",
            "1",
            "--output-tokens",
            "1",
            "--concurrency",
            "1",
            "--warmup",
            "0",
            "--runs",
            "1",
            "--no-stream",
            "--load-measurement",
            "off",
            "--telemetry",
            "off",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .env("LLMETER_HOME", &home)
        .env("LLMETER_OUTPUT_DIR", output.path())
        .output()
        .expect("rerun performance benchmark after interruption");
    assert!(
        recovery.status.success(),
        "recovery run failed: {}",
        String::from_utf8_lossy(&recovery.stderr)
    );

    let recovered_files = json_result_files(output.path());
    assert_eq!(recovered_files.len(), 2, "recovery must retain two results");
    let initial_id = serde_json::from_slice::<Value>(&initial_bytes).expect("parse initial result")
        ["run_id"]
        .as_str()
        .expect("initial run id")
        .to_string();
    let mut run_ids = Vec::new();
    for path in &recovered_files {
        let recovered: Value =
            serde_json::from_slice(&fs::read(path).expect("read recovered result"))
                .expect("parse recovered result");
        assert_eq!(recovered["schema_version"], "3.0");
        assert_eq!(recovered["run_kind"], "performance");
        run_ids.push(
            recovered["run_id"]
                .as_str()
                .expect("recovered run id")
                .to_string(),
        );
    }
    run_ids.sort();
    run_ids.dedup();
    assert_eq!(run_ids.len(), 2, "recovery runs must have distinct IDs");
    assert!(run_ids.contains(&initial_id));
    assert_eq!(
        json_result_files(output.path()).len(),
        2,
        "recovery must retain both completed results"
    );
    assert_no_temporary_files(output.path());
}
