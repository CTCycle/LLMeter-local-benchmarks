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
    run_kind: Option<BenchmarkRunKind>,
    environment: Option<EnvironmentSnapshot>,
    performance_plan: Option<PerformancePlan>,
    quality_plan: Option<QualityPlan>,
    provider_capabilities: Option<ProviderCapabilityReport>,
    model_load_measurements: Option<Vec<ModelLoadMeasurement>>,
    model_inventory_measurements: Option<Vec<ModelInventoryMeasurement>>,
    telemetry_summary: Option<TelemetrySummary>,
}
```

`schema_version` is now persisted as `2.3`. Older JSON files that omit newer fields still deserialize because added fields default to `None`. Performance latency aggregates use successful requests only; failure counts/rates stay explicit in each scenario record.

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

Raw files are saved as JSON and CSV. Formatted reports are generated as Markdown and HTML from the same JSON-compatible data model.

JSON, CSV, Markdown, HTML, and persisted provider configuration are written through a same-directory temporary file that is flushed, synchronized, and renamed into place. This prevents interrupted serialization from leaving a partially written final artifact.

| Format | Extension | Content |
|---|---|---|
| JSON | `.json` | Full `BenchmarkRun` serialized via `serde`. |
| CSV | `.csv` | Flattened result records for spreadsheet analysis. |
| Markdown | `.report.md` | Human-readable summary with aggregated tables and interpretation notes. |
| HTML | `.report.html` | Self-contained browser report with summary cards, sortable tables, and dark mode support. |

JSON is the only format that preserves the full scenario metadata, provider capability probe, model load overhead estimates, model inventory measurements, telemetry summary, environment snapshot, quality plan previews, and per-request performance traces.

CSV keeps one row per result record. It includes fixed high-value columns such as schema version, run kind, provider, base URL, profile, telemetry level, load measurement mode, estimated load overhead, memory ratio, swap ratio, and GPU probe text before dynamic metric columns. Full request traces remain JSON-only.

## Output directory

Default: `<LLMETER_HOME>/benchmark_results/`.

If `LLMETER_HOME` is unset, the effective home is `%USERPROFILE%\\.llmeter` on Windows and `~/.llmeter` on Unix.

Overridable via `--output-dir` flag or `LLMETER_OUTPUT_DIR` environment variable.

Last updated: 2026-07-18
