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
}
```

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

## File formats

Raw files are saved as JSON and CSV. Formatted reports are generated as Markdown and HTML from the same JSON-compatible data model.

| Format | Extension | Content |
|---|---|---|
| JSON | `.json` | Full `BenchmarkRun` serialized via `serde`. |
| CSV | `.csv` | Flattened result records for spreadsheet analysis. |
| Markdown | `.report.md` | Human-readable summary with aggregated tables and interpretation notes. |
| HTML | `.report.html` | Self-contained browser report with summary cards, sortable tables, and dark mode support. |

## Output directory

Default: `benchmark_results/` in the current working directory.

Overridable via `--output-dir` flag or `LLMETER_OUTPUT_DIR` environment variable.

Last updated: 2026-06-12
