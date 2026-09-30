#![cfg(unix)]

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

use expectrl::{
    process::unix::{Signal, WaitStatus},
    session::OsSession,
    ControlCode, Expect, Session,
};
use serde_json::Value;
use tempfile::TempDir;

const PROCESS_EXIT_TIMEOUT: Duration = Duration::from_secs(5);

fn wait_for_exit(session: &mut OsSession) -> WaitStatus {
    let deadline = Instant::now() + PROCESS_EXIT_TIMEOUT;
    loop {
        let status = session
            .get_process_mut()
            .status()
            .expect("read Unix PTY process status");
        if !matches!(status, WaitStatus::StillAlive) {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = session.get_process_mut().kill(Signal::SIGKILL);
            panic!("Unix PTY process did not exit within {PROCESS_EXIT_TIMEOUT:?}");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn assert_interrupt_status(status: WaitStatus) {
    assert!(
        matches!(status, WaitStatus::Exited(_, 130))
            || matches!(status, WaitStatus::Signaled(_, Signal::SIGINT, _)),
        "expected Unix PTY interrupt status equivalent to exit code 130, got {status:?}"
    );
}

fn assert_nonzero_exit(status: WaitStatus) {
    assert!(
        !matches!(status, WaitStatus::Exited(_, 0)),
        "expected Unix PTY process to exit unsuccessfully, got {status:?}"
    );
}

fn configured_command(home: &Path, output: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_llmeter"));
    command
        .env("LLMETER_HOME", home)
        .env("LLMETER_OUTPUT_DIR", output)
        .env("LLMETER_CONFIG_DIR", output.join("config"));
    command
}

fn menu_command(home: &Path, output: &Path) -> Command {
    let mut command = configured_command(home, output);
    command.args(["--timeout", "0.1", "menu"]);
    command
}

fn menu_command_with_provider(home: &Path, output: &Path, base_url: &str) -> Command {
    let mut command = configured_command(home, output);
    command.args([
        "--provider",
        "openai-compatible",
        "--base-url",
        base_url,
        "--timeout",
        "3",
        "menu",
    ]);
    command
}

fn expect_prompt(session: &mut OsSession, prompt: &str) {
    session.set_expect_timeout(Some(PROCESS_EXIT_TIMEOUT));
    session
        .expect(prompt)
        .unwrap_or_else(|error| panic!("expected Unix PTY prompt {prompt:?}: {error}"));
}

fn performance_command(home: &Path, output: &Path, base_url: &str) -> Command {
    let mut command = configured_command(home, output);
    command.args([
        "--provider",
        "openai-compatible",
        "--base-url",
        base_url,
        "--timeout",
        "30",
        "--output-dir",
        output.to_str().expect("performance output path"),
        "bench",
        "perf",
        "--profile",
        "smoke",
        "--models",
        "mock-model",
        "--prompt-tokens",
        "128",
        "--output-tokens",
        "1",
        "--concurrency",
        "1",
        "--warmup",
        "0",
        "--runs",
        "10",
        "--no-stream",
        "--load-measurement",
        "off",
        "--telemetry",
        "off",
        "--export",
        "json",
        "--report",
        "none",
    ]);
    command
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
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind Unix PTY mock provider");
        let address = listener
            .local_addr()
            .expect("Unix PTY mock provider address")
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
    if !output_dir.exists() {
        return Vec::new();
    }
    let mut files = fs::read_dir(output_dir)
        .expect("read Unix PTY output directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn assert_no_temporary_files(output_dir: &Path) {
    if !output_dir.exists() {
        return;
    }
    let temporary = fs::read_dir(output_dir)
        .expect("read Unix PTY output directory")
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
        "Unix PTY left temporary files: {temporary:?}"
    );
}

#[test]
fn unix_pty_menu_ctrl_c_exits_with_interrupt_status() {
    let temp = TempDir::new().expect("tempdir");
    let home = temp.path().join("home");
    let output = temp.path().join("results");
    let mut session = Session::spawn(menu_command(&home, &output)).expect("spawn Unix PTY menu");
    thread::sleep(Duration::from_millis(500));
    session
        .send(ControlCode::ETX)
        .expect("interrupt Unix PTY menu");
    let status = wait_for_exit(&mut session);
    assert_interrupt_status(status);
}

#[test]
fn unix_pty_nested_cancel_and_back_navigation_do_not_hang() {
    let temp = TempDir::new().expect("tempdir");
    let home = temp.path().join("home");
    let output = temp.path().join("results");
    let mut session = Session::spawn(menu_command(&home, &output)).expect("spawn Unix PTY menu");
    thread::sleep(Duration::from_millis(500));

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
        .send("\u{1b}")
        .expect("navigate back from provider setup");
    thread::sleep(Duration::from_millis(250));
    session
        .send(ControlCode::ETX)
        .expect("interrupt after nested cancellation and back navigation");

    let status = wait_for_exit(&mut session);
    assert_interrupt_status(status);
}

#[test]
fn unix_pty_performance_confirmation_interrupt_exits_cleanly() {
    let temp = TempDir::new().expect("tempdir");
    let home = temp.path().join("home");
    let output = temp.path().join("results");
    let provider = MockProvider::start();
    let mut session = Session::spawn(menu_command_with_provider(
        &home,
        &output,
        &provider.base_url,
    ))
    .expect("spawn Unix PTY provider menu");
    thread::sleep(Duration::from_millis(500));

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
    expect_prompt(&mut session, "Run this benchmark plan");
    session
        .send(ControlCode::ETX)
        .expect("interrupt performance confirmation");

    let status = wait_for_exit(&mut session);
    assert_interrupt_status(status);
}

#[test]
fn unix_pty_interrupted_performance_run_leaves_no_partial_result_and_recovers() {
    let temp = TempDir::new().expect("tempdir");
    let home = temp.path().join("home");
    let output = temp.path().join("results");
    let provider = MockProvider::start_with_chat_delay(Duration::from_secs(2));
    let mut session = Session::spawn(performance_command(&home, &output, &provider.base_url))
        .expect("spawn delayed Unix PTY performance run");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !provider.chat_requests.load(Ordering::SeqCst) && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        provider.chat_requests.load(Ordering::SeqCst),
        "Unix PTY run did not reach a delayed provider request"
    );
    session
        .send(ControlCode::ETX)
        .expect("interrupt delayed Unix PTY performance run");
    let status = wait_for_exit(&mut session);
    assert_nonzero_exit(status);

    assert!(
        json_result_files(&output).is_empty(),
        "interrupted Unix PTY run left a canonical result"
    );
    assert_no_temporary_files(&output);
    drop(provider);

    let recovery_provider = MockProvider::start();
    let recovery = Command::new(env!("CARGO_BIN_EXE_llmeter"))
        .args([
            "--provider",
            "openai-compatible",
            "--base-url",
            recovery_provider.base_url.as_str(),
            "--timeout",
            "30",
            "--output-dir",
            output.to_str().expect("recovery output path"),
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
        .env("LLMETER_OUTPUT_DIR", &output)
        .env("LLMETER_CONFIG_DIR", output.join("config"))
        .output()
        .expect("run fresh Unix PTY recovery process");
    assert!(
        recovery.status.success(),
        "Unix PTY recovery failed: {}",
        String::from_utf8_lossy(&recovery.stderr)
    );

    let recovered = json_result_files(&output);
    assert_eq!(recovered.len(), 1);
    let run: Value =
        serde_json::from_slice(&fs::read(&recovered[0]).expect("read recovery result"))
            .expect("parse recovery result");
    assert_eq!(run["schema_version"], "3.0");
    assert_eq!(run["run_kind"], "performance");
    assert_no_temporary_files(&output);
}
