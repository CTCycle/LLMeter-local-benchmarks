# Result storage

## BenchmarkRun model

Each benchmark run becomes one `BenchmarkRun` struct (defined in `results.rs`):

```text
BenchmarkRun {
    run_id: String,
    created_at: String,
    models: Vec<String>,
    benchmark_ids: Vec<String>,
    config: HashMap<String, Value>,
    results: Vec<BenchmarkResultRecord>,
    schema_version: String,
    run_kind: BenchmarkRunKind,
    environment: Option<EnvironmentSnapshot>,
    performance_plan: Option<PerformancePlan>,
    provider_capabilities: Option<ProviderCapabilityReport>,
    model_load_measurements: Option<Vec<ModelLoadMeasurement>>,
    model_inventory_measurements: Option<Vec<ModelInventoryMeasurement>>,
    telemetry_summary: Option<TelemetrySummary>,
}
```

`schema_version` is persisted as `3.0`. Schema identity and `run_kind` are mandatory. Files from older result schemas are rejected on load instead of being silently normalized into the current model. Performance latency aggregates use successful requests only; failure counts and rates stay explicit in each scenario record. Scenario metrics persist the successful latency sample count, nearest-rank percentile estimator, and population-standard-deviation label. Unsupported high percentiles are omitted rather than repeated from undersized samples.

External quality benchmark plans are dry-run planning output and are not stored as an alternate `BenchmarkRun` shape. Persisted benchmark results therefore have one canonical run model, with `run_kind` distinguishing standard and performance runs.

The current output policy is applied to a clone of the in-memory run. Response previews are omitted by default, credential-shaped values and explicitly secret-named parameters are redacted, and local process/cache selectors are removed from persisted performance data. Non-secret measurement keys such as `max_tokens` remain intact because they are required to reproduce and validate saved benchmark metadata. JSON, CSV, Markdown, HTML, and provider configuration writes use same-directory temporary files followed by an atomic rename.

## Result records

Each `BenchmarkResultRecord` contains:

```text
BenchmarkResultRecord {
    benchmark_id: String,
    benchmark_name: String,
    model: String,
    run_index: Option<u32>,
    prompt_name: Option<String>,
    metrics: HashMap<String, Value>,
    response_preview: Option<String>,
    error: Option<String>,
    metadata: Option<HashMap<String, Value>>,
}
```

Performance runs reuse `BenchmarkResultRecord` and place scenario-level request traces inside `metadata["request_traces"]`.

## File formats

Raw files are saved as JSON and CSV. Formatted reports are generated as Markdown and HTML from the same JSON-compatible data model. Before any artifact is written, LLMeter clones the in-memory run and applies the output privacy policy: response previews are omitted by default, common credential-shaped values are redacted from errors and nested metadata, secret-named provider parameters are replaced, and local process/cache selectors are removed. The in-memory measurement data is not mutated.

Use `--include-response-preview` on benchmark or report-generation commands only when model output is safe to persist. Saved configuration records whether previews were included and whether sensitive-value redaction was applied.

JSON, CSV, Markdown, HTML, and persisted provider configuration are written through a same-directory temporary file that is flushed, synchronized, and renamed into place. This prevents interrupted serialization from leaving a partially written final artifact.

CSV text cells beginning with `=`, `+`, `-`, or `@` receive a leading apostrophe so spreadsheet applications do not interpret untrusted result text as a formula. Numeric and boolean metric values remain unchanged.

| Format | Extension | Content |
|---|---|---|
| JSON | `.json` | Privacy-filtered `BenchmarkRun` serialized via `serde`; complete measurement metadata, but response previews only when explicitly requested. |
| CSV | `.csv` | Flattened result records for spreadsheet analysis. |
| Markdown | `.report.md` | Human-readable summary with aggregated tables and interpretation notes. |
| HTML | `.report.html` | Self-contained browser report with summary cards, sortable tables, and dark mode support. |

JSON is the only format that preserves the full scenario metadata, provider capability probe, model load overhead estimates, model inventory measurements, telemetry summary, environment snapshot, and per-request performance traces.

CSV keeps one row per result record. It includes fixed high-value columns such as schema version, run kind, provider, base URL, profile, telemetry level, load measurement mode, estimated load overhead, memory ratio, swap ratio, and GPU probe text before dynamic metric columns. Full request traces remain JSON-only.

## Output directory

Default: `<LLMETER_HOME>/benchmark_results/`.

If `LLMETER_HOME` is unset, the effective home is `%USERPROFILE%\\.llmeter` on Windows and `~/.llmeter` on Unix.

Overridable via `--output-dir` flag or `LLMETER_OUTPUT_DIR` environment variable.

Last updated: 2026-09-10