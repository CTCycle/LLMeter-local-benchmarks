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

## Reading results correctly

- Compare runs from the same machine under similar load.
- Use JSON when programmatic post-processing needs complete fidelity.
- Use CSV when slicing metrics in spreadsheets.
- Use Markdown or HTML when sharing human-readable summaries.

Last updated: 2026-06-15
