# Report generation

## Inputs and outputs

Formatted reports are generated from a saved-or-in-memory `BenchmarkRun`.

Supported formatted outputs:

- Markdown: `<run-id>.report.md`
- HTML: `<run-id>.report.html`

Raw exports remain separate:

- JSON: `<run-id>.json`
- CSV: `<run-id>.csv`

## Aggregation model

`src/reporting.rs` groups records by:

- model
- benchmark ID
- prompt name

For each group it computes summary rows that can include:

- average wall time
- average time to first token
- average tokens per second
- average input and output tokens
- average embedding dimensions
- mean pairwise similarity
- structured-output success ratio
- tool-calling success ratio
- record count and error count

## Markdown report

The Markdown renderer produces these sections:

1. Run metadata
2. Executive summary
3. Run configuration
4. Aggregated benchmark results
5. Detailed records
6. Errors, when present
7. Interpretation notes

This is also the format used for terminal report viewing through `llmeter report show` and the interactive "View latest result as terminal report" flow.

## HTML report

The HTML renderer uses the same aggregated data model as Markdown and emits a self-contained report intended for browser viewing. The report includes:

- summary cards
- run configuration
- aggregated benchmark table
- detailed records table
- error section when needed

The HTML file is generated locally and does not depend on an external web app or asset pipeline.

## Save flow

`runner::save_outputs()` applies persistence in this order:

1. Save raw results according to `--export`
2. Save formatted reports according to `--report`

Supported export choices:

- `json`
- `csv`
- `both`
- `none`

Supported report choices:

- `md`
- `html`
- `both`
- `none`

The saved file list is returned to the caller and then shown in the terminal UI.

Last updated: 2026-06-15
