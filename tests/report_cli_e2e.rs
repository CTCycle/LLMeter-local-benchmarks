use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use llmeter::{
    benchmarks::base::BenchmarkResultRecord,
    performance::config::{
        ConcurrencySpec, LoadMeasurementMode, OutputSizeSpec, PerformancePlan, PerformanceProfile,
        PromptSizeSpec, ReportDetailLevel, TelemetryLevel, WarmupConfig,
    },
    providers::ProviderKind,
    results::{BenchmarkRun, BenchmarkRunKind},
};
use serde_json::{json, Value};
use tempfile::TempDir;

struct ReportHarness {
    root: TempDir,
    home: PathBuf,
    output: PathBuf,
}

impl ReportHarness {
    fn new() -> Self {
        let qa = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("QA");
        let root = tempfile::Builder::new()
            .prefix("t1-06-report-cli-")
            .tempdir_in(qa)
            .expect("create isolated report fixture");
        let home = root.path().join("home");
        let output = root.path().join("results");
        fs::create_dir_all(&home).expect("create isolated home");
        fs::create_dir_all(&output).expect("create isolated output directory");
        Self { root, home, output }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_llmeter"))
            .args(["--provider", "ollama"])
            .args(args)
            .env("LLMETER_HOME", &self.home)
            .env("LLMETER_OUTPUT_DIR", &self.output)
            .output()
            .expect("run LLMeter CLI")
    }
}

fn record(
    benchmark_id: &str,
    benchmark_name: &str,
    model: &str,
    prompt_name: Option<&str>,
    metrics: HashMap<String, Value>,
    response_preview: Option<&str>,
    error: Option<&str>,
) -> BenchmarkResultRecord {
    BenchmarkResultRecord {
        benchmark_id: benchmark_id.to_string(),
        benchmark_name: benchmark_name.to_string(),
        model: model.to_string(),
        run_index: Some(1),
        prompt_name: prompt_name.map(str::to_string),
        metrics,
        response_preview: response_preview.map(str::to_string),
        error: error.map(str::to_string),
        metadata: None,
    }
}

fn performance_plan() -> PerformancePlan {
    PerformancePlan {
        provider: ProviderKind::Ollama,
        models: vec!["fixture-model".to_string()],
        profile: PerformanceProfile::Smoke,
        prompt_sizes: PromptSizeSpec {
            estimated_tokens: vec![128],
        },
        output_sizes: OutputSizeSpec {
            estimated_tokens: vec![64],
        },
        concurrency: ConcurrencySpec { levels: vec![1] },
        warmup: WarmupConfig { requests: 1 },
        runs: 1,
        stream: true,
        workload_jsonl: None,
        extra_params: HashMap::new(),
        safety: Default::default(),
        load_measurement: LoadMeasurementMode::FirstRequestEstimate,
        load_probe_runs: 1,
        telemetry: TelemetryLevel::Standard,
        sample_interval_ms: 1000,
        provider_process: None,
        probe_capabilities: false,
        probe_all_endpoints: false,
        model_cache_dir: None,
        scan_model_cache: false,
        detail: ReportDetailLevel::Detailed,
    }
}

fn save_fixture(output: &Path, run: &BenchmarkRun) -> PathBuf {
    let path = output.join(format!("{}.json", run.run_id));
    fs::write(
        &path,
        serde_json::to_vec_pretty(run).expect("serialize schema 3.0 result"),
    )
    .expect("write result fixture");
    path
}

fn text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn report_cli_lists_shows_and_generates_standard_performance_and_adversarial_runs() {
    let harness = ReportHarness::new();
    let standard = BenchmarkRun::new(
        "t1-06-standard".to_string(),
        "2026-09-23T12:00:00Z".to_string(),
        vec!["bad|model<script>".to_string()],
        vec!["chat-generation".to_string()],
        HashMap::from([
            ("api_key".to_string(), json!("T1_06_SECRET_SENTINEL")),
            ("runs".to_string(), json!(2)),
            ("max_tokens".to_string(), json!(128)),
            ("temperature".to_string(), json!(0.0)),
            ("timeout".to_string(), json!(30.0)),
        ]),
        vec![
            record(
                "chat-generation",
                "Generation|unsafe <script>alert(1)</script>",
                "bad|model<script>",
                Some("prompt|one"),
                HashMap::from([("wall_time_ms".to_string(), json!(150.0))]),
                Some("T1_06_PREVIEW_SENTINEL"),
                None,
            ),
            record(
                "chat-generation",
                "Generation|unsafe <script>alert(1)</script>",
                "bad|model<script>",
                Some("prompt|failure"),
                HashMap::new(),
                None,
                Some("T1_06_ERROR_SENTINEL <script>alert(2)</script>|failure"),
            ),
        ],
        BenchmarkRunKind::Benchmark,
    );
    let standard_path = save_fixture(&harness.output, &standard);

    let mut performance = BenchmarkRun::new(
        "t1-06-performance".to_string(),
        "2026-09-23T12:01:00Z".to_string(),
        vec!["fixture-model".to_string()],
        vec!["performance-scenario".to_string()],
        HashMap::from([
            ("runs".to_string(), json!(1)),
            ("max_tokens".to_string(), json!(128)),
            ("temperature".to_string(), json!(0.0)),
            ("timeout".to_string(), json!(30.0)),
        ]),
        vec![record(
            "performance-scenario",
            "Performance scenario",
            "fixture-model",
            Some("128p-64o-c1"),
            HashMap::from([
                ("wall_time_ms_p50".to_string(), json!(150.0)),
                ("wall_time_ms_p95".to_string(), json!(165.0)),
                ("wall_time_ms_p99".to_string(), json!(170.0)),
                ("ttft_ms_p50".to_string(), json!(40.0)),
                ("output_tokens_per_second".to_string(), json!(45.2)),
                ("requests_per_second".to_string(), json!(4.0)),
                ("error_count".to_string(), json!(0)),
            ]),
            None,
            None,
        )],
        BenchmarkRunKind::Performance,
    );
    performance.performance_plan = Some(performance_plan());
    let performance_path = save_fixture(&harness.output, &performance);

    let listed = harness.run(&["report", "list"]);
    assert!(listed.status.success(), "{}", text(&listed));
    assert!(text(&listed).contains("t1-06-standard.json"));
    assert!(text(&listed).contains("t1-06-performance.json"));

    for (path, expected) in [
        (&standard_path, "t1-06-standard"),
        (&performance_path, "t1-06-performance"),
    ] {
        let shown = harness.run(&["report", "show", path.to_str().unwrap()]);
        let shown_text = text(&shown);
        assert!(shown.status.success(), "{shown_text}");
        assert!(shown_text.contains(expected), "{shown_text}");
    }
    let shown_standard = harness.run(&["report", "show", standard_path.to_str().unwrap()]);
    assert!(text(&shown_standard).contains("T1_06_ERROR_SENTINEL"));

    for path in [&standard_path, &performance_path] {
        let generated = harness.run(&[
            "report",
            "generate",
            path.to_str().unwrap(),
            "--format",
            "both",
        ]);
        assert!(generated.status.success(), "{}", text(&generated));
    }

    let standard_markdown = harness.output.join("t1-06-standard.report.md");
    let standard_html = harness.output.join("t1-06-standard.report.html");
    let performance_markdown = harness.output.join("t1-06-performance.report.md");
    let performance_html = harness.output.join("t1-06-performance.report.html");
    for path in [
        &standard_markdown,
        &standard_html,
        &performance_markdown,
        &performance_html,
    ] {
        assert!(path.is_file(), "missing {}", path.display());
    }

    let markdown = fs::read_to_string(&standard_markdown).unwrap();
    let html = fs::read_to_string(&standard_html).unwrap();
    let performance_md = fs::read_to_string(&performance_markdown).unwrap();
    let performance_html = fs::read_to_string(&performance_html).unwrap();
    assert!(markdown.contains("T1_06_ERROR_SENTINEL"));
    assert!(markdown.contains("bad\\|model"));
    assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    assert!(!html.contains("<script>alert(2)</script>"));
    for content in [&markdown, &html] {
        assert!(!content.contains("T1_06_SECRET_SENTINEL"));
        assert!(!content.contains("T1_06_PREVIEW_SENTINEL"));
    }
    assert!(performance_md.contains("## Performance Summary"));
    assert!(performance_md.contains("## Benchmark timing model"));
    assert!(performance_html.contains("Performance Summary"));

    let listed_after_generation = harness.run(&["report", "list"]);
    let listed_text = text(&listed_after_generation);
    assert!(listed_after_generation.status.success(), "{listed_text}");
    assert!(listed_text.contains("t1-06-standard.report.md"));
    assert!(listed_text.contains("t1-06-standard.report.html"));
    assert!(listed_text.contains("t1-06-performance.report.md"));
    assert!(listed_text.contains("t1-06-performance.report.html"));

    assert!(harness.root.path().join("home").is_dir());
}
