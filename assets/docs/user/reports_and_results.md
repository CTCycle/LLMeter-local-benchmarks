# Reports and results

## Saved artifact types

LLMeter can save four artifact types per run:

| Type | Extension | Purpose |
|---|---|---|
| Raw run | `.json` | Canonical full-fidelity benchmark run data. |
| Flat export | `.csv` | Spreadsheet-friendly record export. |
| Markdown report | `.report.md` | Human-readable report and terminal-renderable summary. |
| HTML report | `.report.html` | Browser-friendly formatted report. |

Default output directory:

- `benchmark_results/`

## During benchmark runs

Both guided and scriptable benchmark execution can save:

- raw exports with `json`, `csv`, `both`, or `none`
- formatted reports with `md`, `html`, `both`, or `none`

The terminal prints a saved-file table after the run completes.

## Report commands

List recent outputs:

```bash
llmeter report list
```

Show the latest or a selected saved JSON result as a terminal report:

```bash
llmeter report show
llmeter report show benchmark_results/<run-id>.json
```

Generate Markdown and HTML from a saved JSON run:

```bash
llmeter report generate --format both
llmeter report generate benchmark_results/<run-id>.json --format html
```

## What reports contain

Reports summarize:

- run ID and creation time
- models and benchmark IDs
- selected run configuration
- grouped benchmark averages
- detailed per-record entries
- explicit error rows when some capabilities fail

Performance runs add dedicated sections for:

- benchmark timing model
- provider capability matrix
- model load overhead estimates
- performance summary
- scenario matrix
- resource and system snapshot
- swap and memory pressure when telemetry was sampled
- GPU snapshot or GPU probe warning
- model cache and metadata
- provider parameters

## Performance metric definitions

- `wall_time_ms_*` - end-to-end request wall-clock latency percentiles measured by the CLI
- `ttft_ms_*` - time to first streamed token percentiles when streaming is enabled and the provider emits token chunks
- `generation_wall_ms_*` - wall time minus TTFT when streaming timing exists
- `tpot_ms_*` - average time per output token derived from token arrival deltas
- `itl_ms_*` - average inter-token latency derived from token arrival deltas
- `output_tokens_per_second` - aggregate output token throughput across the scenario
- `output_tokens_per_second_including_ttft` - per-request output throughput over full wall time
- `output_tokens_per_second_excluding_ttft` - per-request output throughput over generation wall time when TTFT exists
- `input_tokens_per_second` - aggregate input token throughput across the scenario
- `requests_per_second` - aggregate scenario request throughput
- `error_rate` - failed request count divided by total request count
- `estimated_load_overhead_ms` - client-side first-probe minus warm-probe estimate, clamped at zero

Environment snapshots record the host OS, CPU count, memory and swap ratios, disk availability, provider endpoint, LLMeter version, provider-process candidates, and optional `nvidia-smi` output or probe error.

## Reading results correctly

- Compare runs from the same machine under similar load.
- Treat load overhead as an estimate unless the report explicitly says provider-native telemetry was used.
- Do not compare runs across provider versions, model quantizations, thermal states, or memory pressure without noting those differences.
- Use JSON when programmatic post-processing needs complete fidelity.
- Use CSV when slicing metrics in spreadsheets.
- Use Markdown or HTML when sharing human-readable summaries.

Last updated: 2026-06-18
