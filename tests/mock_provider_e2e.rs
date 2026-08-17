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

struct MockProvider {
    base_url: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockProvider {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock provider");
        let port = listener.local_addr().expect("mock provider address").port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_requests = Arc::clone(&requests);
        let thread_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => handle_connection(&mut stream, &thread_requests),
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

    fn request_paths(&self) -> Vec<String> {
        self.requests.lock().expect("mock requests").clone()
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

fn handle_connection(stream: &mut TcpStream, requests: &Arc<Mutex<Vec<String>>>) {
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

    let request = String::from_utf8_lossy(&buffer);
    let mut first_line = request
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace();
    let method = first_line.next().unwrap_or_default();
    let path = first_line.next().unwrap_or_default();
    requests
        .lock()
        .expect("mock requests")
        .push(path.to_string());

    match method {
        "GET" if path.ends_with("/v1/models") => write_json(
            stream,
            200,
            r#"{"object":"list","data":[{"id":"mock-model","object":"model","owned_by":"mock"}]}"#,
        ),
        "POST"
            if path.ends_with("/v1/chat/completions") && request.contains(r#""stream":true"#) =>
        {
            write_sse(
                stream,
                &[
                    r#"data: {"choices":[{"delta":{"content":"Hello"}}]}"#,
                    r#"data: {"choices":[{"delta":{"content":" from mock"}}]}"#,
                    r#"data: {"usage":{"prompt_tokens":4,"completion_tokens":3,"total_tokens":7},"choices":[{"delta":{}}]}"#,
                    "data: [DONE]",
                ],
            )
        }
        "POST" if path.ends_with("/v1/chat/completions") => write_json(
            stream,
            201,
            r#"{"choices":[{"message":{"content":"{\"summary\":\"mock\",\"metrics\":[\"latency\",\"ttft\",\"throughput\"],\"recommendation\":\"ok\"}"}}],"usage":{"prompt_tokens":4,"completion_tokens":5,"total_tokens":9}}"#,
        ),
        "POST" if path.ends_with("/v1/responses") => write_json(
            stream,
            501,
            r#"{"error":{"message":"responses endpoint unsupported by mock"}}"#,
        ),
        "POST" if path.ends_with("/v1/embeddings") => write_json(
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
    let content_length = request
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    buffer.len() >= header_end + 4 + content_length
}

fn write_json(stream: &mut TcpStream, status: u16, body: &str) {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        _ => "Error",
    };
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn write_sse(stream: &mut TcpStream, lines: &[&str]) {
    let body = format!("{}\n\n", lines.join("\n\n"));
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
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
        .arg("--provider")
        .arg("openai-compatible")
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
            "--include-response-preview",
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
    assert_eq!(run["schema_version"], "2.4");
    assert_eq!(run["results"][0]["error"], Value::Null);
    assert!(run["results"][0]["response_preview"]
        .as_str()
        .unwrap_or_default()
        .contains("Hello from mock"));
    assert!(result_files[0].with_extension("csv").exists());
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
fn every_registered_preset_obeys_the_baseline_openai_contract_fixture() {
    let provider = MockProvider::start();

    for entry in ProviderKind::catalog() {
        let client = ProviderClient::new(entry.provider, &provider.base_url, 2.0)
            .expect("build fixture client");
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
    }
}
