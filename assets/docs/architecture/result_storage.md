# Result storage

## BenchmarkRun model

Each benchmark run becomes one `BenchmarkRun` object:

```text
run_id
created_at
models
benchmark_ids
config
results[]
```

## Result records

Each result record contains:

```text
benchmark_id
benchmark_name
model
run_index
prompt_name
metrics
response_preview
error
metadata
```

## File formats

Raw files are saved as JSON and CSV. Formatted reports are generated as Markdown and HTML from the same JSON-compatible data model.

| Format | Extension | Content |
|---|---|---|
| JSON | `.json` | Full `BenchmarkRun` serialized. |
| CSV | `.csv` | Flattened result records for spreadsheet analysis. |
| Markdown | `.report.md` | Human-readable summary with aggregated tables and interpretation notes. |
| HTML | `.report.html` | Self-contained browser report with summary cards, sortable tables, and dark mode support. |

## Output directory

Default: `benchmark_results/` in the current working directory.

Overridable via `--output-dir` flag or `LLMETER_OUTPUT_DIR` environment variable.

## State directory

Default: `~/.llmeter/`

Stores the tracked PID file (`ollama-server.pid.json`) and server logs (`ollama-server.log`).

Overridable via `LLMETER_STATE_DIR` environment variable.

Last updated: 2026-06-12
