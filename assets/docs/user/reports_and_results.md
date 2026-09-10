# Reports and results

## Saved artifact types

LLMeter can save four artifact types per run:

| Type | Extension | Purpose |
|---|---|---|
| Raw run | `.json` | Canonical full-fidelity benchmark run data. |
| Flat export | `.csv` | Spreadsheet-friendly record export. |
| Markdown report | `.report.md` | Human-readable report and terminal-renderable summary. |
| HTML report | `.report.html` | Browser-friendly formatted report. |

New runs persist result schema `3.0`. Schema identity and run kind are mandatory. Older JSON result schemas are rejected rather than silently upgraded or interpreted through legacy compatibility paths.

Default output directory:

- `<LLMETER_HOME>/benchmark_results/`

## During benchmark runs

Both guided and scriptable benchmark execution can save:

- raw exports with `json`, `csv`, `both`, or `none`
- formatted reports with `md`, `html`, `both`, or `none`

Response previews are not persisted by default. Add `--include-response-preview` only for runs whose model output is safe to retain. Credential-shaped values in diagnostic text and explicitly secret-named provider parameters are redacted before JSON, CSV, Markdown, or HTML is written. Measurement metadata such as token-count settings remains intact.

The terminal prints a saved-file table after the run completes.

## Report commands

List recent outputs:

```bash
llmeter report list
```

Show the latest or a selected saved JSON result as a terminal report:

```bash
llmeter report show
llmeter report show <LLMETER_HOME>/benchmark_results/<run-id>.json
```

Generate Markdown and HTML from a saved JSON run:

```bash
llmeter report generate --format both
llmeter report generate <LLMETER_HOME>/benchmark_results/<run-id>.json --format html
llmeter report generate <LLMETER_HOME>/benchmark_results/<run-id>.json --format html --include-response-preview
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
- `ttft_ms_*` - client-observed time to the first non-empty streamed content chunk; this is not tokenizer-confirmed
- `generation_wall_ms_*` - wall time minus TTFT when streaming timing exists
- `inter_chunk_latency_ms_*` - inter-arrival latency for non-empty streamed response chunks; this is not a token metric
- `itl_ms_*` - inter-token latency calculated as `(wall_time_ms - ttft_ms) / (output_tokens - 1)` only for streamed requests with provider-reported output usage of at least two tokens
- `output_tokens_per_second` - aggregate output token throughput across the scenario
- `output_tokens_per_second_including_ttft` - per-request output throughput over full wall time
- `output_tokens_per_second_excluding_ttft` - per-request output throughput over generation wall time when TTFT exists
- `input_tokens_per_second` - aggregate input token throughput across the scenario
- `input_token_coverage` and `output_token_coverage` - the fraction of successful requests with provider-reported input/output usage; aggregate token throughput and means are omitted when coverage is incomplete
- Standard benchmark summary throughput is per-request output throughput; performance scenarios are reported separately as scenario-level throughput and are not merged into the standard summary column
- `request_traces_omitted` means the selected summary detail intentionally stored no traces; `request_traces_truncated` means a detailed trace set exceeded its configured cap
- `requests_per_second` - aggregate scenario request throughput
- `error_rate` - failed request count divided by total request count
- `estimated_load_overhead_ms` - client-side first-probe minus warm-probe estimate, clamped at zero

Latency percentiles use the nearest-rank estimator over successful, finite, non-negative samples. Every performance summary shows the successful latency sample count. P95 is omitted when fewer than 20 samples are available and P99 when fewer than 100 are available. `wall_time_ms_stddev` is population standard deviation. Smoke profiles are exploratory checks; use larger deliberate run counts for statistical interpretation.

Environment snapshots record the host OS, CPU count, memory and swap ratios, disk availability, provider endpoint, LLMeter version, provider-process candidates, and optional `nvidia-smi` output or probe error.

## Reading results correctly

- Compare runs from the same machine under similar load.
- Treat `estimated_load_overhead_ms` as a client-observed first-request versus warm-request estimate; it is not provider startup or model-loading time.
- Do not compare runs across provider versions, model quantizations, thermal states, or memory pressure without noting those differences.
- Use JSON when programmatic post-processing needs complete fidelity.
- Use CSV when slicing metrics in spreadsheets.
- Use Markdown or HTML when sharing human-readable summaries.

All persisted formats are written through same-directory temporary files and atomic rename. Output preparation omits response previews by default and redacts credential-shaped values before writing; use `--include-response-preview` only when the output is safe to retain.

Last updated: 2026-09-10