use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

struct ResilienceProvider {
    base_url: String,
    address: String,
    stop: Arc<AtomicBool>,
    delay_ms: Arc<AtomicU64>,
    active_requests: Arc<AtomicUsize>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ResilienceProvider {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind resilience provider");
        let address = listener
            .local_addr()
            .expect("resilience provider address")
            .to_string();
        let base_url = format!("http://{address}/v1");
        let stop = Arc::new(AtomicBool::new(false));
        let delay_ms = Arc::new(AtomicU64::new(0));
        let active_requests = Arc::new(AtomicUsize::new(0));
        let thread_stop = Arc::clone(&stop);
        let thread_delay_ms = Arc::clone(&delay_ms);
        let thread_active_requests = Arc::clone(&active_requests);
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        handle_connection(&mut stream, &thread_delay_ms, &thread_active_requests)
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            base_url,
            address,
            stop,
            delay_ms,
            active_requests,
            handle: Some(handle),
        }
    }

    fn set_chat_delay(&self, delay: Duration) {
        self.delay_ms.store(
            delay.as_millis().min(u64::MAX as u128) as u64,
            Ordering::SeqCst,
        );
    }

    fn has_active_request(&self) -> bool {
        self.active_requests.load(Ordering::SeqCst) > 0
    }
}

impl Drop for ResilienceProvider {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(&self.address);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn handle_connection(stream: &mut TcpStream, delay_ms: &AtomicU64, active_requests: &AtomicUsize) {
    let path = read_request_path(stream);
    if path.ends_with("/v1/models") {
        write_json(
            stream,
            200,
            r#"{"object":"list","data":[{"id":"mock-model","object":"model"}]}"#,
        );
        return;
    }

    if path.ends_with("/v1/chat/completions") {
        active_requests.fetch_add(1, Ordering::SeqCst);
        let delay = Duration::from_millis(delay_ms.load(Ordering::SeqCst));
        if !delay.is_zero() {
            thread::sleep(delay);
        }
        write_json(
            stream,
            200,
            r#"{"choices":[{"message":{"content":"resilience-ok"}}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#,
        );
        active_requests.fetch_sub(1, Ordering::SeqCst);
        return;
    }

    write_json(stream, 404, r#"{"error":{"message":"not found"}}"#);
}

fn read_request_path(stream: &mut TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    while !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(read) => buffer.extend_from_slice(&chunk[..read]),
        }
    }
    String::from_utf8_lossy(&buffer)
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_string()
}

fn write_json(stream: &mut TcpStream, status: u16, body: &str) {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Error",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn llmeter_command(home: &Path, output: &Path, base_url: &str) -> Command {
    let executable =
        std::env::var_os("LLMETER_BIN").unwrap_or_else(|| env!("CARGO_BIN_EXE_llmeter").into());
    let mut command = Command::new(executable);
    command
        .env("LLMETER_HOME", home)
        .env("LLMETER_OUTPUT_DIR", output)
        .env_remove("LLMETER_PROVIDER")
        .env_remove("LLMETER_BASE_URL")
        .env_remove("LLMETER_TIMEOUT")
        .env_remove("LLMETER_API_KEY")
        .arg("--provider")
        .arg("openai-compatible")
        .arg("--base-url")
        .arg(base_url)
        .arg("--timeout")
        .arg("30");
    command
}

fn performance_command(
    home: &Path,
    output: &Path,
    base_url: &str,
    runs: &str,
    export: &str,
    report: &str,
) -> Command {
    let mut command = llmeter_command(home, output, base_url);
    command.args([
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
        runs,
        "--no-stream",
        "--load-measurement",
        "off",
        "--telemetry",
        "off",
        "--detail",
        "full",
        "--export",
        export,
        "--report",
        report,
    ]);
    command
}

fn json_result_files(output_dir: &Path) -> Vec<PathBuf> {
    if !output_dir.exists() {
        return Vec::new();
    }

    let mut files = fs::read_dir(output_dir)
        .expect("read resilience output directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn directory_snapshot(output_dir: &Path) -> BTreeMap<String, Vec<u8>> {
    if !output_dir.exists() {
        return BTreeMap::new();
    }

    fs::read_dir(output_dir)
        .expect("read resilience output snapshot")
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            (
                path.file_name()
                    .expect("snapshot file name")
                    .to_string_lossy()
                    .into_owned(),
                fs::read(path).expect("read snapshot file"),
            )
        })
        .collect()
}

fn assert_no_temporary_files(output_dir: &Path) {
    let temporary = directory_snapshot(output_dir)
        .keys()
        .filter(|name| name.contains(".tmp-"))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        temporary.is_empty(),
        "temporary result files remain: {temporary:?}"
    );
}

fn assert_valid_result(path: &Path) -> Value {
    let result: Value =
        serde_json::from_slice(&fs::read(path).expect("read result")).expect("parse result JSON");
    assert_eq!(result["schema_version"], "3.0");
    assert_eq!(result["run_kind"], "performance");
    result
}

fn wait_for_active_request(provider: &ResilienceProvider) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !provider.has_active_request() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        provider.has_active_request(),
        "delayed provider did not observe an active benchmark request"
    );
}

fn wait_for_exit(child: &mut Child) -> std::process::ExitStatus {
    child.wait().expect("wait for llmeter process")
}

#[test]
fn abrupt_termination_preserves_completed_results_and_allows_restart_recovery() {
    let provider = ResilienceProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let home = temp.path().join("home");
    let output = temp.path().join("results");

    let first = performance_command(&home, &output, &provider.base_url, "1", "json", "both")
        .output()
        .expect("run initial benchmark");
    assert!(
        first.status.success(),
        "initial benchmark failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let first_files = json_result_files(&output);
    assert_eq!(first_files.len(), 1, "initial canonical result count");
    let first_result = assert_valid_result(&first_files[0]);
    let first_snapshot = directory_snapshot(&output);
    let first_result_bytes = fs::read(&first_files[0]).expect("read initial result bytes");
    let first_name = first_files[0]
        .file_name()
        .expect("initial result name")
        .to_string_lossy()
        .into_owned();

    provider.set_chat_delay(Duration::from_millis(750));
    let mut interrupted =
        performance_command(&home, &output, &provider.base_url, "10", "both", "both")
            .spawn()
            .expect("spawn delayed benchmark");
    wait_for_active_request(&provider);
    interrupted.kill().expect("terminate delayed benchmark");
    let interrupted_status = wait_for_exit(&mut interrupted);
    assert!(
        !interrupted_status.success(),
        "abruptly terminated benchmark unexpectedly succeeded"
    );

    assert_eq!(
        directory_snapshot(&output),
        first_snapshot,
        "interruption changed the completed result or created a partial artifact"
    );
    assert_eq!(
        fs::read(&first_files[0]).expect("re-read completed result"),
        first_result_bytes,
        "completed result changed after later process termination"
    );
    assert_no_temporary_files(&output);
    assert_eq!(json_result_files(&output).len(), 1);

    provider.set_chat_delay(Duration::ZERO);
    let listed = llmeter_command(&home, &output, &provider.base_url)
        .args(["report", "list"])
        .output()
        .expect("list results after abrupt termination");
    assert!(
        listed.status.success(),
        "report list failed after restart: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    assert!(
        String::from_utf8_lossy(&listed.stdout).contains(&first_name),
        "report list omitted the completed result: {}",
        String::from_utf8_lossy(&listed.stdout)
    );

    let shown = llmeter_command(&home, &output, &provider.base_url)
        .args(["report", "show"])
        .arg(&first_files[0])
        .output()
        .expect("show result after abrupt termination");
    assert!(
        shown.status.success(),
        "report show failed after restart: {}",
        String::from_utf8_lossy(&shown.stderr)
    );
    assert_eq!(first_result["run_id"], first_name.trim_end_matches(".json"));

    let third = performance_command(&home, &output, &provider.base_url, "1", "json", "none")
        .output()
        .expect("run benchmark after restart recovery");
    assert!(
        third.status.success(),
        "recovery benchmark failed: {}",
        String::from_utf8_lossy(&third.stderr)
    );

    let recovered_files = json_result_files(&output);
    assert_eq!(recovered_files.len(), 2, "completed results after recovery");
    let mut run_ids = Vec::new();
    for path in recovered_files {
        let result = assert_valid_result(&path);
        run_ids.push(
            result["run_id"]
                .as_str()
                .expect("recovered run id")
                .to_string(),
        );
    }
    run_ids.sort();
    run_ids.dedup();
    assert_eq!(run_ids.len(), 2, "recovered runs must have distinct IDs");
    assert_no_temporary_files(&output);
}
