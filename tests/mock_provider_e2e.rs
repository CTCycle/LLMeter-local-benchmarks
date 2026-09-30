use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use llmeter::providers::{ProviderClient, ProviderKind};
use llmeter::runner::validate_models;
use serde_json::Value;
use tempfile::TempDir;

const T1_04_SENTINEL_API_KEY: &str = "llmeter-t1-04-synthetic-token";

#[derive(Clone, Debug)]
#[cfg_attr(not(windows), allow(dead_code))]
struct MockRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

#[derive(Clone, Copy)]
#[cfg_attr(not(windows), allow(dead_code))]
enum MockScenario {
    Standard,
    CreatedResponses,
    RedirectModels,
    OversizedModels,
    OversizedStream,
    UnauthorizedEcho,
}

struct MockProvider {
    base_url: String,
    requests: Arc<Mutex<Vec<MockRequest>>>,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockProvider {
    fn start() -> Self {
        Self::start_with_scenario(MockScenario::Standard)
    }

    fn start_with_scenario(scenario: MockScenario) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock provider");
        let port = listener.local_addr().expect("mock provider address").port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_requests = Arc::clone(&requests);
        let thread_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        handle_connection(&mut stream, &thread_requests, scenario)
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            base_url: format!("http://127.0.0.1:{port}/v1"),
            requests,
            stop,
            handle: Some(handle),
        }
    }

    fn requests(&self) -> Vec<MockRequest> {
        self.requests.lock().expect("mock requests").clone()
    }

    fn request_paths(&self) -> Vec<String> {
        self.requests()
            .into_iter()
            .map(|request| request.path)
            .collect()
    }
}

impl Drop for MockProvider {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let address = self
            .base_url
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or_default();
        let _ = TcpStream::connect(address);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn handle_connection(
    stream: &mut TcpStream,
    requests: &Arc<Mutex<Vec<MockRequest>>>,
    scenario: MockScenario,
) {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    loop {
        let read = match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => read,
            Err(_) => break,
        };
        buffer.extend_from_slice(&chunk[..read]);
        if request_complete(&buffer) {
            break;
        }
    }

    let request_text = String::from_utf8_lossy(&buffer);
    let header_end = request_text.find("\r\n\r\n").unwrap_or(request_text.len());
    let mut request_lines = request_text[..header_end].split("\r\n");
    let mut first_line = request_lines.next().unwrap_or_default().split_whitespace();
    let method = first_line.next().unwrap_or_default().to_string();
    let path = first_line.next().unwrap_or_default().to_string();
    let headers = request_lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        .collect::<HashMap<_, _>>();
    let body_start = header_end.saturating_add(4).min(buffer.len());
    let body = String::from_utf8_lossy(&buffer[body_start..]).into_owned();
    requests.lock().expect("mock requests").push(MockRequest {
        method: method.clone(),
        path: path.clone(),
        headers,
        body: body.clone(),
    });

    match (method.as_str(), path.as_str()) {
        ("GET", path) if path.ends_with("/v1/models") => match scenario {
            MockScenario::RedirectModels => write_redirect(stream, "/v1/redirect-target"),
            MockScenario::OversizedModels => {
                let padding = "x".repeat(10 * 1024 * 1024 + 1);
                let body = format!(
                    r#"{{"object":"list","data":[{{"id":"mock-model"}}],"padding":"{padding}"}}"#
                );
                write_json(stream, 200, &body);
            }
            _ => write_json(
                stream,
                200,
                r#"{"object":"list","data":[{"id":"mock-model","object":"model","owned_by":"mock"}]}"#,
            ),
        },
        ("GET", path) if path.ends_with("/v1/redirect-target") => write_json(
            stream,
            200,
            r#"{"object":"list","data":[{"id":"should-not-follow"}]}"#,
        ),
        ("POST", path)
            if path.ends_with("/v1/chat/completions") && body.contains(r#""stream":true"#) =>
        {
            match scenario {
                MockScenario::OversizedStream => {
                    let content = "x".repeat(1024 * 1024 + 1);
                    let event =
                        format!(r#"data: {{"choices":[{{"delta":{{"content":"{content}"}}}}]}}"#);
                    write_sse_body(stream, &format!("{event}\n\n"));
                }
                MockScenario::UnauthorizedEcho => write_json(
                    stream,
                    401,
                    &format!(
                        r#"{{"error":{{"message":"Unauthorized Bearer {T1_04_SENTINEL_API_KEY} {}"}}}}"#,
                        "x".repeat(64 * 1024 + 1)
                    ),
                ),
                _ => write_sse(
                    stream,
                    &[
                        &[
                            r#"data: {"choices":["#,
                            r#"data: {"delta":{"content":"Hello"}}]}"#,
                        ],
                        &[r#"data: {"choices":[{"delta":{"content":" from mock"}}]}"#],
                        &[
                            r#"data: {"usage":{"prompt_tokens":4,"completion_tokens":3,"total_tokens":7},"choices":[{"delta":{}}]}"#,
                        ],
                        &["data: [DONE]"],
                    ],
                ),
            }
        }
        ("POST", path) if path.ends_with("/v1/chat/completions") => write_json(
            stream,
            201,
            r#"{"choices":[{"message":{"content":"{\"summary\":\"mock\",\"metrics\":[\"latency\",\"ttft\",\"throughput\"],\"recommendation\":\"ok\"}"}}],"usage":{"prompt_tokens":4,"completion_tokens":5,"total_tokens":9}}"#,
        ),
        ("POST", path) if path.ends_with("/v1/responses") => match scenario {
            MockScenario::CreatedResponses => write_json(
                stream,
                201,
                r#"{"output_text":"Created response","usage":{"input_tokens":4,"output_tokens":3,"total_tokens":7}}"#,
            ),
            _ => write_json(
                stream,
                501,
                r#"{"error":{"message":"responses endpoint unsupported by mock"}}"#,
            ),
        },
        ("POST", path) if path.ends_with("/v1/embeddings") => write_json(
            stream,
            501,
            r#"{"error":{"message":"embeddings endpoint unsupported by mock"}}"#,
        ),
        _ => write_json(
            stream,
            404,
            r#"{"error":{"message":"mock route not found"}}"#,
        ),
    }
}

fn request_complete(buffer: &[u8]) -> bool {
    let request = String::from_utf8_lossy(buffer);
    let Some(header_end) = request.find("\r\n\r\n") else {
        return false;
    };
    let content_length = request[..header_end]
        .split("\r\n")
        .skip(1)
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    buffer.len() >= header_end + 4 + content_length
}

fn write_json(stream: &mut TcpStream, status: u16, body: &str) {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        401 => "Unauthorized",
        _ => "Error",
    };
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn write_redirect(stream: &mut TcpStream, location: &str) {
    let response = format!(
        "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let _ = stream.write_all(response.as_bytes());
}

fn write_sse(stream: &mut TcpStream, events: &[&[&str]]) {
    let body = format!(
        "{}\n\n",
        events
            .iter()
            .map(|lines| lines.join("\n"))
            .collect::<Vec<_>>()
            .join("\n\n")
    );
    write_sse_body(stream, &body);
}

fn write_sse_body(stream: &mut TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn llmeter_command(home: &Path, output: &Path, base_url: &str) -> Command {
    llmeter_command_for_provider(home, output, ProviderKind::OpenaiCompatible, base_url)
}

fn llmeter_command_for_provider(
    home: &Path,
    output: &Path,
    provider: ProviderKind,
    base_url: &str,
) -> Command {
    let executable =
        std::env::var_os("LLMETER_BIN").unwrap_or_else(|| env!("CARGO_BIN_EXE_llmeter").into());
    let mut command = Command::new(executable);
    command
        .env("LLMETER_HOME", home)
        .env("LLMETER_OUTPUT_DIR", output)
        .arg("--provider")
        .arg(provider.label())
        .arg("--base-url")
        .arg(base_url)
        .arg("--timeout")
        .arg("3");
    command
}

fn json_result_files(output_dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(output_dir)
        .expect("read output dir")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension().and_then(|value| value.to_str()) == Some("json")).then_some(path)
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn assert_no_temporary_output_files(output_dir: &Path) {
    if !output_dir.exists() {
        return;
    }

    let temporary_files = fs::read_dir(output_dir)
        .expect("read output directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains(".tmp-"))
        })
        .collect::<Vec<_>>();
    assert!(
        temporary_files.is_empty(),
        "temporary atomic-write files remain: {temporary_files:?}"
    );
}

#[cfg(windows)]
struct WindowsAppHarness {
    root: TempDir,
    script: PathBuf,
    home: PathBuf,
}

#[cfg(windows)]
impl WindowsAppHarness {
    fn new() -> Self {
        let qa_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("QA");
        let root = TempDir::new_in(qa_dir).expect("create isolated app harness");
        let script = root.path().join("run_llmeter.ps1");
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("run_llmeter.ps1"),
            &script,
        )
        .expect("copy official launcher into fixture");
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"llmeter-t1-04-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .expect("write fixture manifest");

        let binary = root
            .path()
            .join("target")
            .join("release")
            .join("llmeter.exe");
        fs::create_dir_all(binary.parent().expect("binary parent"))
            .expect("create fixture binary directory");
        fs::copy(env!("CARGO_BIN_EXE_llmeter"), &binary)
            .expect("stage real llmeter executable in launcher fixture");
        fs::OpenOptions::new()
            .append(true)
            .open(&binary)
            .expect("open staged llmeter executable")
            .set_modified(std::time::SystemTime::now())
            .expect("set staged executable timestamp");

        let home = root.path().join("home");
        fs::create_dir_all(&home).expect("create isolated app home");
        Self { root, script, home }
    }

    fn output_dir(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    fn command(&self, base_url: &str, output_dir: &Path) -> Command {
        let mut command = Command::new("powershell.exe");
        command
            .arg("-NoProfile")
            .arg("-File")
            .arg(&self.script)
            .arg("--provider")
            .arg("openai-compatible")
            .arg("--base-url")
            .arg(base_url)
            .arg("--timeout")
            .arg("3")
            .env("LLMETER_HOME", &self.home)
            .env("LLMETER_OUTPUT_DIR", output_dir)
            .env_remove("LLMETER_PROVIDER")
            .env_remove("LLMETER_BASE_URL")
            .env_remove("LLMETER_TIMEOUT")
            .env_remove("LLMETER_API_KEY");
        command
    }
}

#[cfg(windows)]
fn output_text(output: &std::process::Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn cli_models_json_reads_mock_provider_catalog() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");

    let output_process = llmeter_command(temp.path(), &output, &provider.base_url)
        .arg("models")
        .arg("--json")
        .output()
        .expect("run llmeter models");

    assert!(
        output_process.status.success(),
        "{}",
        String::from_utf8_lossy(&output_process.stderr)
    );
    let models: Value = serde_json::from_slice(&output_process.stdout).expect("models json");
    assert_eq!(models[0]["id"], "mock-model");
    assert!(output_process.stderr.is_empty());
    assert!(provider.request_paths().contains(&"/v1/models".to_string()));
}

#[test]
fn cli_provider_catalog_lists_compatibility_tiers() {
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");
    let catalog = llmeter_command(temp.path(), &output, "http://127.0.0.1:1/v1")
        .args(["providers", "list"])
        .output()
        .expect("run llmeter provider catalog");

    assert!(
        catalog.status.success(),
        "{}",
        String::from_utf8_lossy(&catalog.stderr)
    );
    let stdout = String::from_utf8_lossy(&catalog.stdout);
    assert!(stdout.contains("first-class"));
    assert!(stdout.contains("openai-compatible"));
    assert!(stdout.contains("Provider presets"));
}

#[test]
fn cli_status_and_model_details_read_from_mock_provider() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");

    let status = llmeter_command(temp.path(), &output, &provider.base_url)
        .arg("status")
        .output()
        .expect("run llmeter status");
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(String::from_utf8_lossy(&status.stdout).contains("API reachable"));

    let details = llmeter_command(temp.path(), &output, &provider.base_url)
        .args(["show", "mock-model"])
        .output()
        .expect("run llmeter show");
    assert!(
        details.status.success(),
        "{}",
        String::from_utf8_lossy(&details.stderr)
    );
    let model: Value = serde_json::from_slice(&details.stdout).expect("model details json");
    assert_eq!(model["id"], "mock-model");
    assert!(provider.request_paths().contains(&"/v1/models".to_string()));
}

#[test]
fn cli_handles_unavailable_provider_and_missing_model() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");

    let missing = llmeter_command(temp.path(), &output, &provider.base_url)
        .args(["show", "missing-model"])
        .output()
        .expect("run llmeter show for missing model");
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("Model not found"),
        "{}",
        String::from_utf8_lossy(&missing.stderr)
    );

    let closed_listener = TcpListener::bind("127.0.0.1:0").expect("bind unavailable endpoint");
    let closed_address = closed_listener
        .local_addr()
        .expect("closed endpoint address");
    drop(closed_listener);
    let unavailable_url = format!("http://{closed_address}/v1");
    let unavailable = llmeter_command(temp.path(), &output, &unavailable_url)
        .arg("status")
        .output()
        .expect("run llmeter status for unavailable provider");
    assert_eq!(unavailable.status.code(), Some(1));
    let unavailable_stdout = String::from_utf8_lossy(&unavailable.stdout);
    assert!(unavailable_stdout.contains("API reachable"));
    assert!(unavailable_stdout.contains("no"));
}

#[test]
fn cli_bench_run_streams_and_generates_report_from_saved_json() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");

    let bench = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "run",
            "--suite",
            "llm",
            "--models",
            "mock-model",
            "--benchmarks",
            "chat-generation",
            "--runs",
            "1",
            "--export",
            "both",
            "--report",
            "both",
        ])
        .output()
        .expect("run llmeter bench");

    assert!(
        bench.status.success(),
        "{}",
        String::from_utf8_lossy(&bench.stderr)
    );
    let result_files = json_result_files(&output);
    assert_eq!(result_files.len(), 1);
    let run: Value =
        serde_json::from_slice(&fs::read(&result_files[0]).expect("read run")).expect("run json");
    let run_id = run["run_id"].as_str().expect("run id");
    assert_eq!(
        result_files[0].file_stem().and_then(|stem| stem.to_str()),
        Some(run_id)
    );
    assert_eq!(run["schema_version"], "3.0");
    assert_eq!(run["results"][0]["error"], Value::Null);
    assert_eq!(run["config"]["response_previews_included"], false);
    assert_eq!(run["config"]["sensitive_values_redacted"], true);
    assert!(run["results"][0]["response_preview"].is_null());
    let csv_path = result_files[0].with_extension("csv");
    assert!(csv_path.exists());
    let csv_content = fs::read_to_string(csv_path).expect("read CSV result");
    assert!(!csv_content.contains("Hello from mock"));
    assert!(result_files[0].with_extension("report.md").exists());
    assert!(result_files[0].with_extension("report.html").exists());

    let generated_report = llmeter_command(temp.path(), &output, &provider.base_url)
        .arg("report")
        .arg("generate")
        .arg(&result_files[0])
        .arg("--format")
        .arg("html")
        .output()
        .expect("generate report");

    assert!(
        generated_report.status.success(),
        "{}",
        String::from_utf8_lossy(&generated_report.stderr)
    );
    let html_report = result_files[0].with_extension("report.html");
    assert!(html_report.exists());
    assert!(provider
        .request_paths()
        .iter()
        .any(|path| path == "/v1/chat/completions"));
}

#[test]
fn repeated_cli_runs_keep_distinct_results_and_reload_after_process_restart() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");
    let args = [
        "bench",
        "run",
        "--suite",
        "llm",
        "--models",
        "mock-model",
        "--benchmarks",
        "chat-generation",
        "--runs",
        "1",
        "--export",
        "json",
        "--report",
        "none",
    ];

    for _ in 0..2 {
        let run = llmeter_command(temp.path(), &output, &provider.base_url)
            .args(args)
            .output()
            .expect("run llmeter benchmark in a fresh process");
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let result_files = json_result_files(&output);
    assert_eq!(result_files.len(), 2);
    let run_ids = result_files
        .iter()
        .map(|path| {
            let run: Value =
                serde_json::from_slice(&fs::read(path).expect("read repeated run")).unwrap();
            assert_eq!(run["schema_version"], "3.0");
            run["run_id"].as_str().unwrap().to_string()
        })
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        run_ids.len(),
        2,
        "repeated runs must not overwrite each other"
    );

    let listed = llmeter_command(temp.path(), &output, &provider.base_url)
        .args(["report", "list"])
        .output()
        .expect("list repeated results");
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let listing = String::from_utf8_lossy(&listed.stdout);
    for path in &result_files {
        assert!(
            listing.contains(path.file_name().unwrap().to_str().unwrap()),
            "missing {} from report listing: {listing}",
            path.display()
        );
        let shown = llmeter_command(temp.path(), &output, &provider.base_url)
            .args(["report", "show"])
            .arg(path)
            .output()
            .expect("reload repeated result");
        assert!(
            shown.status.success(),
            "{}",
            String::from_utf8_lossy(&shown.stderr)
        );
    }
}

#[test]
fn unsupported_endpoint_benchmark_records_controlled_errors() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");

    let bench = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "run",
            "--suite",
            "embeddings",
            "--models",
            "mock-model",
            "--benchmarks",
            "embeddings",
            "--runs",
            "1",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .output()
        .expect("run llmeter embeddings bench");

    assert!(
        bench.status.success(),
        "{}",
        String::from_utf8_lossy(&bench.stderr)
    );
    let result_files = json_result_files(&output);
    assert_eq!(result_files.len(), 1);
    let run: Value =
        serde_json::from_slice(&fs::read(&result_files[0]).expect("read run")).expect("run json");
    assert!(run["results"][0]["error"]
        .as_str()
        .unwrap_or_default()
        .contains("embeddings endpoint unsupported by mock"));
}

#[test]
fn jsonl_performance_accounting_matches_planning_requests_progress_and_results() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");
    let workload = temp.path().join("workload.jsonl");
    fs::write(
        &workload,
        concat!(
            "{\"id\":\"prompt-a\",\"prompt\":\"first prompt\"}\n",
            "{\"id\":\"prompt-b\",\"prompt\":\"second prompt\"}\n",
            "{\"id\":\"prompt-c\",\"prompt\":\"third prompt\"}\n",
        ),
    )
    .expect("write JSONL workload");

    let over_budget = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "perf",
            "--profile",
            "smoke",
            "--models",
            "mock-model",
            "--output-tokens",
            "1",
            "--concurrency",
            "1",
            "--warmup",
            "0",
            "--runs",
            "1",
            "--jsonl",
        ])
        .arg(&workload)
        .args(["--max-requests", "2", "--dry-run"])
        .output()
        .expect("reject an undercounted JSONL performance plan");
    assert!(!over_budget.status.success());
    assert!(
        String::from_utf8_lossy(&over_budget.stderr).contains("requests 3 exceed --max-requests 2")
    );
    assert!(
        provider
            .requests()
            .iter()
            .all(|request| request.method != "POST"),
        "a rejected request budget must not issue workload requests"
    );

    let dry_run = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "perf",
            "--profile",
            "smoke",
            "--models",
            "mock-model",
            "--output-tokens",
            "1",
            "--concurrency",
            "1",
            "--warmup",
            "0",
            "--runs",
            "1",
            "--jsonl",
        ])
        .arg(&workload)
        .args(["--max-requests", "3", "--dry-run"])
        .output()
        .expect("inspect JSONL performance plan");
    assert!(
        dry_run.status.success(),
        "{}",
        String::from_utf8_lossy(&dry_run.stderr)
    );
    let estimate = String::from_utf8_lossy(&dry_run.stdout);
    assert!(estimate.contains("Scenarios: 3"), "{estimate}");
    assert!(estimate.contains("Measured requests: 3"), "{estimate}");
    assert!(estimate.contains("Total requests: 3"), "{estimate}");
    assert!(
        provider
            .requests()
            .iter()
            .all(|request| request.method != "POST"),
        "dry-run must not issue workload requests"
    );

    let run = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "perf",
            "--profile",
            "smoke",
            "--models",
            "mock-model",
            "--output-tokens",
            "1",
            "--concurrency",
            "1",
            "--warmup",
            "0",
            "--runs",
            "1",
            "--jsonl",
        ])
        .arg(&workload)
        .args([
            "--max-requests",
            "3",
            "--load-measurement",
            "off",
            "--telemetry",
            "off",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .output()
        .expect("run JSONL performance scenarios");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let progress = String::from_utf8_lossy(&run.stderr);
    let scenario_progress = progress
        .lines()
        .filter(|line| line.contains("Running scenario"))
        .collect::<Vec<_>>();
    assert_eq!(scenario_progress.len(), 3, "{progress}");
    let scenario_steps = scenario_progress
        .iter()
        .filter_map(|line| {
            let step = line.rsplit_once("Step ")?.1.split_whitespace().next()?;
            let (current, total) = step.split_once('/')?;
            Some((current.parse::<u32>().ok()?, total.parse::<u32>().ok()?))
        })
        .collect::<Vec<_>>();
    assert_eq!(scenario_steps, [(2, 5), (3, 5), (4, 5)], "{progress}");
    assert!(progress.contains("100% Completed"), "{progress}");

    let chat_requests = provider
        .requests()
        .into_iter()
        .filter(|request| request.method == "POST" && request.path == "/v1/chat/completions")
        .collect::<Vec<_>>();
    assert_eq!(chat_requests.len(), 3);
    let request_prompts = chat_requests
        .iter()
        .map(|request| {
            serde_json::from_str::<Value>(&request.body).expect("chat request JSON")["messages"][0]
                ["content"]
                .as_str()
                .expect("request prompt")
                .to_string()
        })
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        request_prompts,
        ["first prompt", "second prompt", "third prompt"]
            .into_iter()
            .map(str::to_string)
            .collect()
    );

    let result_files = json_result_files(&output);
    assert_eq!(result_files.len(), 1);
    let saved: Value =
        serde_json::from_slice(&fs::read(&result_files[0]).expect("read performance result"))
            .expect("performance result JSON");
    assert_eq!(saved["results"].as_array().expect("scenario rows").len(), 3);
    assert_eq!(
        saved["performance_plan"]["prompt_sizes"]["estimated_tokens"]
            .as_array()
            .expect("planned prompt sizes")
            .len(),
        3
    );
    let persisted_prompt_ids = saved["results"]
        .as_array()
        .expect("scenario rows")
        .iter()
        .filter_map(|record| record["metadata"]["prompt_id"].as_str().map(str::to_string))
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        persisted_prompt_ids,
        ["prompt-a", "prompt-b", "prompt-c"]
            .into_iter()
            .map(str::to_string)
            .collect()
    );
}

#[test]
fn benchmark_surfaces_output_path_failures() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output_file = temp.path().join("results-file");
    fs::write(&output_file, "not a directory").expect("create output file");

    let bench = llmeter_command(temp.path(), &output_file, &provider.base_url)
        .args([
            "bench",
            "run",
            "--suite",
            "llm",
            "--models",
            "mock-model",
            "--benchmarks",
            "chat-generation",
            "--runs",
            "1",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .output()
        .expect("run llmeter with invalid output path");

    assert!(!bench.status.success());
    assert!(String::from_utf8_lossy(&bench.stderr).contains("Failed to create output directory"));
}

#[test]
fn provider_client_reuses_a_model_catalog_snapshot() {
    let provider = MockProvider::start();
    let client = ProviderClient::new(ProviderKind::OpenaiCompatible, &provider.base_url, 2.0)
        .expect("build fixture client");

    let names = client.model_names().expect("fixture model names");
    let model = client
        .show_model("mock-model")
        .expect("fixture model lookup");

    assert_eq!(names, vec!["mock-model"]);
    assert_eq!(model["id"], "mock-model");
    assert_eq!(
        provider
            .request_paths()
            .iter()
            .filter(|path| path.as_str() == "/v1/models")
            .count(),
        1
    );

    client
        .list_models_fresh()
        .expect("fresh fixture model discovery");
    client
        .refresh_model_catalog()
        .expect("refresh fixture model discovery");
    client
        .invalidate_model_catalog()
        .expect("invalidate fixture model catalog");
    client
        .list_models_cached()
        .expect("refreshed fixture model discovery");
    assert_eq!(
        provider
            .request_paths()
            .iter()
            .filter(|path| path.as_str() == "/v1/models")
            .count(),
        4
    );
}

#[test]
fn benchmark_model_validation_bypasses_cached_catalog() {
    let provider = MockProvider::start();
    let client = ProviderClient::new(ProviderKind::OpenaiCompatible, &provider.base_url, 2.0)
        .expect("build fixture client");

    client
        .list_models_cached()
        .expect("seed fixture model catalog cache");
    assert_eq!(
        validate_models(&client, &["mock-model".to_string()]).unwrap(),
        ["mock-model"]
    );
    assert_eq!(
        provider
            .request_paths()
            .iter()
            .filter(|path| path.as_str() == "/v1/models")
            .count(),
        2,
        "benchmark validation must bypass the cached catalog"
    );
}

#[test]
fn performance_load_estimate_runs_before_capability_chat_probes() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results");

    let performance = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "perf",
            "--profile",
            "smoke",
            "--models",
            "mock-model",
            "--prompt-tokens",
            "1",
            "--output-tokens",
            "2",
            "--concurrency",
            "1",
            "--warmup",
            "0",
            "--runs",
            "1",
            "--load-measurement",
            "first-request-estimate",
            "--load-probe-runs",
            "1",
            "--probe-capabilities",
            "--telemetry",
            "off",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .output()
        .expect("run performance estimate");

    assert!(
        performance.status.success(),
        "{}",
        String::from_utf8_lossy(&performance.stderr)
    );

    let chat_positions = provider
        .request_paths()
        .iter()
        .enumerate()
        .filter_map(|(index, path)| (path == "/v1/chat/completions").then_some(index))
        .collect::<Vec<_>>();
    assert!(
        chat_positions.len() >= 3,
        "expected load, capability, and measured chat requests"
    );
    assert_eq!(
        chat_positions[0], 3,
        "first load probe must follow cached selection, fresh validation, and fresh status"
    );
    assert_eq!(
        chat_positions[1], 4,
        "warm load probe must follow the first load probe"
    );

    let result_files = json_result_files(&output);
    assert_eq!(result_files.len(), 1);
    let run: Value =
        serde_json::from_slice(&fs::read(&result_files[0]).expect("read performance result"))
            .expect("performance result json");
    assert_eq!(
        run["model_load_measurements"][0]["mode"],
        "first-request-estimate"
    );
    assert!(run["model_load_measurements"][0]["notes"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|note| note
            .as_str()
            .unwrap_or_default()
            .contains("does not measure provider restart")));
}

#[test]
fn performance_profiles_persist_bounded_scenarios_telemetry_and_capabilities() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");

    for profile in ["smoke", "latency", "throughput", "sweep"] {
        let output = temp.path().join(format!("results-{profile}"));
        let mut command = llmeter_command(temp.path(), &output, &provider.base_url);
        command.args([
            "bench",
            "perf",
            "--profile",
            profile,
            "--models",
            "mock-model",
            "--prompt-tokens",
            "1",
            "--output-tokens",
            "2",
            "--concurrency",
            "1,2",
            "--warmup",
            "0",
            "--runs",
            "2",
            "--load-measurement",
            "off",
            "--telemetry",
            "standard",
            "--sample-interval-ms",
            "100",
            "--export",
            "json",
            "--report",
            "none",
        ]);
        if profile == "smoke" {
            command.arg("--probe-all-endpoints");
        }
        let run_output = command.output().expect("run bounded performance profile");
        assert!(
            run_output.status.success(),
            "{profile} profile failed: {}",
            String::from_utf8_lossy(&run_output.stderr)
        );

        let result_files = json_result_files(&output);
        assert_eq!(result_files.len(), 1, "{profile} result file");
        let run: Value =
            serde_json::from_slice(&fs::read(&result_files[0]).expect("read performance result"))
                .expect("performance result JSON");
        assert_eq!(run["run_kind"], "performance");
        assert_eq!(run["performance_plan"]["profile"], profile);
        assert_eq!(
            run["performance_plan"]["concurrency"]["levels"]
                .as_array()
                .expect("planned concurrency levels"),
            &[serde_json::json!(1), serde_json::json!(2)]
        );
        assert_eq!(run["performance_plan"]["runs"], 2);

        let scenarios = run["results"].as_array().expect("scenario records");
        assert_eq!(scenarios.len(), 2, "{profile} scenario count");
        for (scenario, concurrency) in scenarios.iter().zip([1, 2]) {
            assert_eq!(scenario["metrics"]["concurrency"], concurrency);
            assert_eq!(scenario["metrics"]["request_count"], 2);
            assert_eq!(scenario["metrics"]["success_count"], 2);
            let traces = scenario["metadata"]["request_traces"]
                .as_array()
                .expect("request traces");
            assert_eq!(traces.len(), 2);
            assert!(traces.iter().all(|trace| {
                trace["concurrency"] == concurrency
                    && trace["success"] == true
                    && trace["http_status"] == 200
            }));
        }

        let telemetry = run["telemetry_summary"]
            .as_object()
            .expect("telemetry summary");
        assert!(telemetry["sample_count"].as_u64().unwrap_or_default() >= 2);
        assert!(telemetry["max_memory_used_ratio"].as_f64().is_some());
        assert_eq!(
            run["model_inventory_measurements"][0]["model"],
            "mock-model"
        );
        assert!(run["environment"]["cpu_count"].as_u64().unwrap_or_default() > 0);

        if profile == "smoke" {
            let endpoints = run["provider_capabilities"]["endpoints"]
                .as_array()
                .expect("capability endpoints");
            assert_eq!(endpoints.len(), 5);
            for (name, supported) in [
                ("Models", true),
                ("Chat completions", true),
                ("Chat completions streaming", true),
                ("Embeddings", false),
                ("Responses", false),
            ] {
                let endpoint = endpoints
                    .iter()
                    .find(|endpoint| endpoint["name"] == name)
                    .unwrap_or_else(|| panic!("missing {name} capability result"));
                assert_eq!(endpoint["supported"], supported, "{name} capability");
            }
        }
    }

    let measured_and_probe_chat_requests = provider
        .requests()
        .iter()
        .filter(|request| request.method == "POST" && request.path == "/v1/chat/completions")
        .count();
    assert_eq!(measured_and_probe_chat_requests, 18);
}

#[test]
fn performance_profiles_execute_default_matrices_through_fixture() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let expected = [
        ("smoke", &[128, 512][..], &[128][..], &[1][..], 1, 3, 2),
        (
            "latency",
            &[128, 512, 2048][..],
            &[128][..],
            &[1][..],
            1,
            5,
            3,
        ),
        (
            "throughput",
            &[512][..],
            &[256][..],
            &[1, 2, 4, 8][..],
            1,
            4,
            4,
        ),
        (
            "sweep",
            &[128, 512, 2048][..],
            &[64, 128, 256][..],
            &[1, 2, 4][..],
            1,
            3,
            27,
        ),
    ];
    let mut sweep_request_ids = HashSet::new();

    for (profile, prompt_tokens, output_tokens, concurrency, warmup, runs, scenario_count) in
        expected
    {
        let output = temp.path().join(format!("results-default-{profile}"));
        let run_output = llmeter_command(temp.path(), &output, &provider.base_url)
            .args([
                "bench",
                "perf",
                "--profile",
                profile,
                "--models",
                "mock-model",
                "--load-measurement",
                "off",
                "--telemetry",
                "off",
                "--export",
                "json",
                "--report",
                "none",
            ])
            .output()
            .expect("run default performance profile");
        assert!(
            run_output.status.success(),
            "{profile} profile failed: {}",
            String::from_utf8_lossy(&run_output.stderr)
        );

        let result_files = json_result_files(&output);
        assert_eq!(result_files.len(), 1, "{profile} result file");
        let run: Value =
            serde_json::from_slice(&fs::read(&result_files[0]).expect("read default result"))
                .expect("default performance result JSON");
        assert_eq!(run["performance_plan"]["profile"], profile);
        assert_eq!(
            run["performance_plan"]["prompt_sizes"]["estimated_tokens"],
            serde_json::json!(prompt_tokens)
        );
        assert_eq!(
            run["performance_plan"]["output_sizes"]["estimated_tokens"],
            serde_json::json!(output_tokens)
        );
        assert_eq!(
            run["performance_plan"]["concurrency"]["levels"],
            serde_json::json!(concurrency)
        );
        assert_eq!(run["performance_plan"]["warmup"]["requests"], warmup);
        assert_eq!(run["performance_plan"]["runs"], runs);

        let scenarios = run["results"].as_array().expect("default scenario records");
        assert_eq!(scenarios.len(), scenario_count, "{profile} scenario count");
        assert!(scenarios.iter().all(|scenario| {
            scenario["metrics"]["request_count"] == runs
                && scenario["metrics"]["success_count"] == runs
                && scenario["metadata"]["request_traces"]
                    .as_array()
                    .is_some_and(|traces| traces.len() == runs as usize)
                && scenario["metadata"]["request_traces"]
                    .as_array()
                    .is_some_and(|traces| {
                        traces.iter().all(|trace| {
                            trace["success"] == true
                                && matches!(trace["http_status"].as_u64(), Some(200 | 201))
                        })
                    })
        }));
        if profile == "sweep" {
            for scenario in scenarios {
                for trace in scenario["metadata"]["request_traces"]
                    .as_array()
                    .expect("sweep request traces")
                {
                    assert!(
                        sweep_request_ids.insert(
                            trace["request_id"]
                                .as_str()
                                .expect("sweep request trace id")
                                .to_string()
                        ),
                        "duplicate sweep request ID: {}",
                        trace["request_id"]
                    );
                }
            }
            assert_eq!(sweep_request_ids.len(), 81, "unique sweep request IDs");
        }
    }

    let measured_and_warmup_chat_requests = provider
        .requests()
        .iter()
        .filter(|request| request.method == "POST" && request.path == "/v1/chat/completions")
        .count();
    assert_eq!(measured_and_warmup_chat_requests, 154);
}

#[test]
fn performance_profiles_handle_bounded_high_concurrency_matrix() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results-scale");
    let run_output = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "perf",
            "--profile",
            "throughput",
            "--models",
            "mock-model",
            "--prompt-tokens",
            "1,2",
            "--output-tokens",
            "1",
            "--concurrency",
            "1,2,4,8,16",
            "--warmup",
            "1",
            "--runs",
            "16",
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
        .output()
        .expect("run bounded scale performance profile");
    assert!(
        run_output.status.success(),
        "bounded scale profile failed: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let result_files = json_result_files(&output);
    assert_eq!(result_files.len(), 1, "bounded scale result file");
    let run: Value =
        serde_json::from_slice(&fs::read(&result_files[0]).expect("read bounded scale result"))
            .expect("bounded scale result JSON");
    assert_eq!(
        run["performance_plan"]["prompt_sizes"]["estimated_tokens"],
        serde_json::json!([1, 2])
    );
    assert_eq!(
        run["performance_plan"]["output_sizes"]["estimated_tokens"],
        serde_json::json!([1])
    );
    assert_eq!(
        run["performance_plan"]["concurrency"]["levels"],
        serde_json::json!([1, 2, 4, 8, 16])
    );
    assert_eq!(run["performance_plan"]["warmup"]["requests"], 1);
    assert_eq!(run["performance_plan"]["runs"], 16);

    let scenarios = run["results"]
        .as_array()
        .expect("bounded scale scenario records");
    assert_eq!(scenarios.len(), 10);
    for (scenario, concurrency) in scenarios.iter().zip([1, 2, 4, 8, 16, 1, 2, 4, 8, 16]) {
        assert_eq!(scenario["metrics"]["concurrency"], concurrency);
        assert_eq!(scenario["metrics"]["request_count"], 16);
        assert_eq!(scenario["metrics"]["success_count"], 16);
        let traces = scenario["metadata"]["request_traces"]
            .as_array()
            .expect("bounded scale request traces");
        assert_eq!(traces.len(), 16);
        assert!(traces.iter().all(|trace| {
            trace["concurrency"] == concurrency
                && trace["success"] == true
                && matches!(trace["http_status"].as_u64(), Some(200 | 201))
        }));
    }

    let measured_and_warmup_chat_requests = provider
        .requests()
        .iter()
        .filter(|request| request.method == "POST" && request.path == "/v1/chat/completions")
        .count();
    assert_eq!(measured_and_warmup_chat_requests, 170);
}

#[test]
fn performance_reaches_configured_request_ceiling_with_complete_accounting() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results-ceiling");

    let run_output = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "perf",
            "--profile",
            "throughput",
            "--models",
            "mock-model",
            "--prompt-tokens",
            "1,2,3,4,5",
            "--output-tokens",
            "1",
            "--concurrency",
            "1,2,4,8,16",
            "--warmup",
            "1",
            "--runs",
            "19",
            "--no-stream",
            "--load-measurement",
            "off",
            "--telemetry",
            "off",
            "--detail",
            "full",
            "--max-requests",
            "500",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .output()
        .expect("run performance plan at the configured request ceiling");
    assert!(
        run_output.status.success(),
        "ceiling run failed: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let estimate = String::from_utf8_lossy(&run_output.stdout);
    assert!(estimate.contains("Scenarios: 25"), "{estimate}");
    assert!(estimate.contains("Warmup requests: 25"), "{estimate}");
    assert!(estimate.contains("Measured requests: 475"), "{estimate}");
    assert!(estimate.contains("Total requests: 500"), "{estimate}");
    assert!(estimate.contains("Max requests: 500"), "{estimate}");

    let chat_requests = provider
        .requests()
        .into_iter()
        .filter(|request| request.method == "POST" && request.path == "/v1/chat/completions")
        .collect::<Vec<_>>();
    assert_eq!(chat_requests.len(), 500, "provider request accounting");

    let result_files = json_result_files(&output);
    assert_eq!(result_files.len(), 1, "canonical ceiling result");
    let result_content = fs::read_to_string(&result_files[0]).expect("read ceiling result");
    let run: Value = serde_json::from_str(&result_content).expect("parse ceiling result");
    assert_eq!(run["schema_version"], "3.0");
    assert_eq!(run["run_kind"], "performance");
    assert_eq!(run["models"], serde_json::json!(["mock-model"]));
    assert_eq!(run["performance_plan"]["profile"], "throughput");
    assert_eq!(
        run["performance_plan"]["prompt_sizes"]["estimated_tokens"],
        serde_json::json!([1, 2, 3, 4, 5])
    );
    assert_eq!(
        run["performance_plan"]["concurrency"]["levels"],
        serde_json::json!([1, 2, 4, 8, 16])
    );
    assert_eq!(run["performance_plan"]["warmup"]["requests"], 1);
    assert_eq!(run["performance_plan"]["runs"], 19);
    assert_eq!(run["config"]["response_previews_included"], false);
    assert_eq!(run["config"]["sensitive_values_redacted"], true);

    let scenarios = run["results"].as_array().expect("ceiling scenario records");
    assert_eq!(scenarios.len(), 25, "persisted scenario count");
    let mut request_ids = HashSet::new();
    let mut persisted_trace_count = 0usize;
    for scenario in scenarios {
        assert_eq!(scenario["error"], Value::Null);
        assert!(scenario["response_preview"].is_null());
        assert_eq!(scenario["metrics"]["request_count"], 19);
        assert_eq!(scenario["metrics"]["success_count"], 19);
        assert_eq!(scenario["metrics"]["error_count"], 0);
        let traces = scenario["metadata"]["request_traces"]
            .as_array()
            .expect("persisted measured request traces");
        assert_eq!(traces.len(), 19);
        persisted_trace_count += traces.len();
        for (index, trace) in traces.iter().enumerate() {
            assert_eq!(trace["run_index"], (index + 1) as u64);
            assert_eq!(trace["success"], true);
            assert!(matches!(trace["http_status"].as_u64(), Some(200 | 201)));
            assert!(
                request_ids.insert(
                    trace["request_id"]
                        .as_str()
                        .expect("request trace id")
                        .to_string()
                ),
                "duplicate request index/id in persisted traces: {}",
                trace["request_id"]
            );
        }
    }
    assert_eq!(persisted_trace_count, 475, "measured request accounting");
    assert_eq!(request_ids.len(), 475, "unique measured request IDs");
    assert_no_temporary_output_files(&output);

    let shown = llmeter_command(temp.path(), &output, &provider.base_url)
        .args(["report", "show"])
        .arg(&result_files[0])
        .output()
        .expect("reload ceiling result through report command");
    assert!(
        shown.status.success(),
        "report reload failed: {}",
        String::from_utf8_lossy(&shown.stderr)
    );
}

#[test]
fn performance_above_configured_request_ceiling_is_rejected_before_execution() {
    let provider = MockProvider::start();
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join("results-over-ceiling");

    let run_output = llmeter_command(temp.path(), &output, &provider.base_url)
        .args([
            "bench",
            "perf",
            "--profile",
            "throughput",
            "--models",
            "mock-model",
            "--prompt-tokens",
            "1,2,3,4,5",
            "--output-tokens",
            "1",
            "--concurrency",
            "1,2,4,8,16",
            "--warmup",
            "1",
            "--runs",
            "20",
            "--no-stream",
            "--load-measurement",
            "off",
            "--telemetry",
            "off",
            "--max-requests",
            "500",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .output()
        .expect("run above-ceiling performance plan");
    assert!(!run_output.status.success());
    assert!(
        String::from_utf8_lossy(&run_output.stderr)
            .contains("requests 525 exceed --max-requests 500"),
        "{}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let chat_requests = provider
        .requests()
        .into_iter()
        .filter(|request| request.method == "POST" && request.path == "/v1/chat/completions")
        .count();
    assert_eq!(
        chat_requests, 0,
        "above-ceiling plan reached the chat endpoint"
    );
    let json_count = if output.exists() {
        json_result_files(&output).len()
    } else {
        0
    };
    assert_eq!(json_count, 0, "above-ceiling plan wrote a result artifact");
    assert_no_temporary_output_files(&output);
}

#[test]
fn every_registered_preset_obeys_the_baseline_openai_contract_fixture() {
    let provider = MockProvider::start();

    for entry in ProviderKind::catalog() {
        assert_eq!(
            entry.provider.label().parse::<ProviderKind>().unwrap(),
            entry.provider
        );
        assert_eq!(entry.provider.compatibility_tier(), entry.tier);
        assert_eq!(entry.provider.default_base_url(), entry.default_base_url);

        let client = ProviderClient::new(entry.provider, &provider.base_url, 2.0)
            .expect("build fixture client");
        assert_eq!(client.provider(), entry.provider);
        assert_eq!(client.base_url(), provider.base_url);
        let models = client
            .list_models_cached()
            .expect("fixture model discovery");
        assert_eq!(models[0]["id"], "mock-model", "{}", entry.provider);

        let response = client
            .chat_completion(
                "mock-model",
                serde_json::json!([{"role":"user","content":"fixture"}]),
                8,
                0.0,
                true,
                None,
            )
            .expect("fixture streamed chat");
        assert_eq!(
            response.response_text, "Hello from mock",
            "{}",
            entry.provider
        );
        assert_eq!(response.output_tokens(), Some(3), "{}", entry.provider);
        assert!(
            response.time_to_first_token_ns.is_some(),
            "{}",
            entry.provider
        );
        assert_eq!(response.http_status, Some(200), "{}", entry.provider);

        let non_streaming = client
            .chat_completion(
                "mock-model",
                serde_json::json!([{"role":"user","content":"fixture"}]),
                8,
                0.0,
                false,
                None,
            )
            .expect("fixture non-streamed chat");
        assert_eq!(non_streaming.http_status, Some(201), "{}", entry.provider);

        let responses_error = client
            .responses("mock-model", serde_json::json!("fixture"), 8, 0.0, None)
            .expect_err("unsupported responses endpoint must remain a controlled error")
            .to_string();
        assert!(responses_error.contains("HTTP 501"), "{responses_error}");

        let embeddings_error = client
            .embeddings("mock-model", serde_json::json!("fixture"))
            .expect_err("unsupported embeddings endpoint must remain a controlled error")
            .to_string();
        assert!(embeddings_error.contains("HTTP 501"), "{embeddings_error}");
    }

    let requests = provider.requests();
    let model_requests = requests
        .iter()
        .filter(|request| request.method == "GET" && request.path == "/v1/models")
        .count();
    assert_eq!(model_requests, ProviderKind::catalog().len());

    let chat_requests = requests
        .iter()
        .filter(|request| request.method == "POST" && request.path == "/v1/chat/completions")
        .collect::<Vec<_>>();
    assert_eq!(chat_requests.len(), ProviderKind::catalog().len() * 2);
    assert!(chat_requests.iter().any(|request| {
        serde_json::from_str::<Value>(&request.body)
            .map(|body| body["stream"] == true && body["stream_options"]["include_usage"] == true)
            .unwrap_or(false)
    }));
    assert!(chat_requests.iter().any(|request| {
        serde_json::from_str::<Value>(&request.body)
            .map(|body| body["stream"] == false && body["stream_options"].is_null())
            .unwrap_or(false)
    }));

    assert_eq!(
        requests
            .iter()
            .filter(|request| request.method == "POST" && request.path == "/v1/responses")
            .count(),
        ProviderKind::catalog().len()
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.method == "POST" && request.path == "/v1/embeddings")
            .count(),
        ProviderKind::catalog().len()
    );
}

#[test]
fn every_registered_preset_is_cli_selectable_with_an_explicit_base_url() {
    let provider = MockProvider::start();
    let synthetic_key = "tier4-contract-synthetic-key";

    for entry in ProviderKind::catalog() {
        let temp = TempDir::new().expect("create preset CLI fixture home");
        let output = temp.path().join("results");
        let status =
            llmeter_command_for_provider(temp.path(), &output, entry.provider, &provider.base_url)
                .env("LLMETER_API_KEY", synthetic_key)
                .arg("status")
                .output()
                .expect("run preset status against fixture");

        let stdout = String::from_utf8_lossy(&status.stdout);
        let stderr = String::from_utf8_lossy(&status.stderr);
        assert!(
            status.status.success(),
            "{}: {stdout}\n{stderr}",
            entry.provider
        );
        assert!(stdout.contains(entry.provider.display_name()), "{stdout}");
        assert!(stdout.contains(&provider.base_url), "{stdout}");
        assert!(!stdout.contains(synthetic_key), "{stdout}");
        assert!(!stderr.contains(synthetic_key), "{stderr}");

        for root in [temp.path(), output.as_path()] {
            if !root.exists() {
                continue;
            }
            for file in fs::read_dir(root).expect("read fixture output root") {
                let path = file.expect("read fixture output entry").path();
                if path.is_file() {
                    let contents = fs::read_to_string(&path).unwrap_or_default();
                    assert!(!contents.contains(synthetic_key), "{}", path.display());
                }
            }
        }
    }
}

#[cfg(windows)]
#[test]
fn t1_04_official_launcher_captures_multiline_sse_and_accepts_created_responses() {
    let harness = WindowsAppHarness::new();
    let provider = MockProvider::start();
    let output_dir = harness.output_dir("t1-04-stream-output");

    let chat = harness
        .command(&provider.base_url, &output_dir)
        .args([
            "bench",
            "run",
            "--suite",
            "llm",
            "--models",
            "mock-model",
            "--benchmarks",
            "chat-generation",
            "--runs",
            "1",
            "--export",
            "json",
            "--report",
            "none",
            "--include-response-preview",
        ])
        .output()
        .expect("run streaming benchmark through official launcher");
    assert_eq!(chat.status.code(), Some(0), "{}", output_text(&chat));
    let chat_files = json_result_files(&output_dir);
    assert_eq!(chat_files.len(), 1);
    let chat_run: Value =
        serde_json::from_slice(&fs::read(&chat_files[0]).expect("read launcher streaming result"))
            .expect("parse launcher streaming result");
    assert_eq!(chat_run["results"][0]["error"], Value::Null);
    assert!(chat_run["results"][0]["response_preview"]
        .as_str()
        .unwrap_or_default()
        .contains("Hello from mock"));

    let chat_request = provider
        .requests()
        .into_iter()
        .find(|request| request.path.ends_with("/v1/chat/completions"))
        .expect("capture chat request");
    assert_eq!(chat_request.method, "POST");
    let chat_body: Value = serde_json::from_str(&chat_request.body).expect("parse chat request");
    assert_eq!(chat_body["model"], "mock-model");
    assert!(chat_body["messages"].as_array().is_some());
    assert_eq!(chat_body["stream"], true);

    let response_provider = MockProvider::start_with_scenario(MockScenario::CreatedResponses);
    let response_output = harness.output_dir("t1-04-created-response-output");
    let responses = harness
        .command(&response_provider.base_url, &response_output)
        .args([
            "bench",
            "run",
            "--suite",
            "llm",
            "--models",
            "mock-model",
            "--benchmarks",
            "responses-generation",
            "--runs",
            "1",
            "--export",
            "json",
            "--report",
            "none",
            "--include-response-preview",
        ])
        .output()
        .expect("run responses benchmark through official launcher");
    assert_eq!(
        responses.status.code(),
        Some(0),
        "{}",
        output_text(&responses)
    );
    let response_files = json_result_files(&response_output);
    assert_eq!(response_files.len(), 1);
    let response_run: Value = serde_json::from_slice(
        &fs::read(&response_files[0]).expect("read launcher responses result"),
    )
    .expect("parse launcher responses result");
    assert_eq!(response_run["results"][0]["error"], Value::Null);
    assert!(response_run["results"][0]["response_preview"]
        .as_str()
        .unwrap_or_default()
        .contains("Created response"));
    assert!(response_provider
        .requests()
        .iter()
        .any(|request| request.method == "POST" && request.path.ends_with("/v1/responses")));
}

#[cfg(windows)]
#[test]
fn t1_04_official_launcher_rejects_redirects_and_unsafe_urls() {
    let harness = WindowsAppHarness::new();
    let redirect_provider = MockProvider::start_with_scenario(MockScenario::RedirectModels);
    let redirect = harness
        .command(
            &redirect_provider.base_url,
            &harness.output_dir("t1-04-redirect-output"),
        )
        .arg("status")
        .output()
        .expect("check redirected provider through official launcher");
    assert_eq!(
        redirect.status.code(),
        Some(1),
        "{}",
        output_text(&redirect)
    );
    assert!(output_text(&redirect).contains("HTTP 302"));
    let redirect_requests = redirect_provider.requests();
    assert_eq!(redirect_requests.len(), 1);
    assert_eq!(redirect_requests[0].path, "/v1/models");

    let url_provider = MockProvider::start();
    let authority = url_provider
        .base_url
        .strip_prefix("http://")
        .expect("mock provider URL scheme");
    let unsafe_urls = [
        format!("{}?token=t1-04-url-query-secret", url_provider.base_url),
        format!("http://user:t1-04-url-user-secret@{authority}"),
        "file:///provider/v1".to_string(),
    ];
    for (index, base_url) in unsafe_urls.iter().enumerate() {
        let result = harness
            .command(
                base_url,
                &harness.output_dir(&format!("t1-04-unsafe-url-{index}")),
            )
            .arg("status")
            .output()
            .expect("run invalid URL through official launcher");
        assert!(!result.status.success(), "{}", output_text(&result));
        let text = output_text(&result);
        assert!(!text.contains("t1-04-url-query-secret"), "{text}");
        assert!(!text.contains("t1-04-url-user-secret"), "{text}");
    }
    assert!(url_provider.requests().is_empty());
}

#[cfg(windows)]
#[test]
fn t1_04_official_launcher_bounds_json_and_streaming_responses() {
    let harness = WindowsAppHarness::new();
    let json_provider = MockProvider::start_with_scenario(MockScenario::OversizedModels);
    let oversized_json = harness
        .command(
            &json_provider.base_url,
            &harness.output_dir("t1-04-oversized-json-output"),
        )
        .arg("status")
        .output()
        .expect("run oversized JSON status through official launcher");
    assert_eq!(
        oversized_json.status.code(),
        Some(1),
        "{}",
        output_text(&oversized_json)
    );
    assert!(output_text(&oversized_json).contains("response exceeded the 10485760 byte limit"));

    let stream_provider = MockProvider::start_with_scenario(MockScenario::OversizedStream);
    let stream_output = harness.output_dir("t1-04-oversized-stream-output");
    let oversized_stream = harness
        .command(&stream_provider.base_url, &stream_output)
        .args([
            "bench",
            "run",
            "--suite",
            "llm",
            "--models",
            "mock-model",
            "--benchmarks",
            "chat-generation",
            "--runs",
            "1",
            "--export",
            "json",
            "--report",
            "none",
        ])
        .output()
        .expect("run oversized SSE benchmark through official launcher");
    assert_eq!(
        oversized_stream.status.code(),
        Some(0),
        "{}",
        output_text(&oversized_stream)
    );
    let stream_files = json_result_files(&stream_output);
    assert_eq!(stream_files.len(), 1);
    let stream_run: Value =
        serde_json::from_slice(&fs::read(&stream_files[0]).expect("read oversized stream result"))
            .expect("parse oversized stream result");
    assert!(stream_run["results"][0]["error"]
        .as_str()
        .unwrap_or_default()
        .contains("Streaming event line exceeded the 1048576 byte limit"));
}

#[cfg(windows)]
#[test]
fn t1_04_official_launcher_auth_is_captured_and_secrets_are_not_persisted() {
    let harness = WindowsAppHarness::new();
    let provider = MockProvider::start_with_scenario(MockScenario::UnauthorizedEcho);
    let output_dir = harness.output_dir("t1-04-auth-output");
    let result = harness
        .command(&provider.base_url, &output_dir)
        .env("LLMETER_API_KEY", T1_04_SENTINEL_API_KEY)
        .args([
            "bench",
            "run",
            "--suite",
            "llm",
            "--models",
            "mock-model",
            "--benchmarks",
            "chat-generation",
            "--runs",
            "1",
            "--export",
            "both",
            "--report",
            "both",
        ])
        .output()
        .expect("run authenticated benchmark through official launcher");
    assert_eq!(result.status.code(), Some(0), "{}", output_text(&result));

    let requests = provider.requests();
    let chat_request = requests
        .iter()
        .find(|request| request.path.ends_with("/v1/chat/completions"))
        .expect("capture authenticated chat request");
    assert_eq!(
        chat_request
            .headers
            .get("authorization")
            .map(String::as_str),
        Some("Bearer llmeter-t1-04-synthetic-token")
    );

    let result_files = json_result_files(&output_dir);
    assert_eq!(result_files.len(), 1);
    let run: Value = serde_json::from_slice(&fs::read(&result_files[0]).expect("read auth result"))
        .expect("parse auth result");
    let error = run["results"][0]["error"].as_str().unwrap_or_default();
    assert!(error.contains("HTTP 401"), "{error}");
    assert!(error.contains("[body truncated]"), "{error}");
    assert!(error.contains("[redacted]"), "{error}");

    let saved_files = fs::read_dir(&output_dir)
        .expect("read all persisted outputs")
        .map(|entry| entry.expect("read persisted output entry").path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    for path in &saved_files {
        let contents = fs::read_to_string(path).expect("read persisted output text");
        assert!(
            !contents.contains(T1_04_SENTINEL_API_KEY),
            "raw API key persisted in {}",
            path.display()
        );
    }
    for extension in ["csv", "report.md", "report.html"] {
        assert!(result_files[0].with_extension(extension).is_file());
    }
}
