# Error handling

## Error layers

LLMeter uses two main error layers:

- `LLMeterError` in `src/errors.rs` for domain-specific failures.
- `anyhow::Result` for CLI entry points, orchestration code, and context-rich propagation.

`LLMeterError` currently covers:

- provider failures
- model lookup failures
- benchmark selection failures
- invalid option parsing
- I/O failures

## Exit behavior

`src/main.rs` maps failures to process exit codes:

- `2` for recognized `LLMeterError` values
- `1` for other unexpected errors

This keeps scriptable usage able to distinguish product-level validation failures from generic runtime failures.

## Provider failure rules

- Fail fast when provider reachability or model discovery is required and unavailable.
- Attach endpoint and request-path context when HTTP operations fail.
- Normalize non-success HTTP responses into readable provider errors instead of passing through raw transport details alone.

## Benchmark failure rules

- A benchmark capability failure should become an error-bearing `BenchmarkResultRecord` when the overall run can still proceed.
- One unsupported capability must not erase successful records from other benchmarks in the same run.
- Reports must surface benchmark errors explicitly rather than silently dropping them.

## Option and input validation

- Reject malformed repeated `--param` values unless they follow `key=value`.
- Reject unknown benchmark IDs early through registry selection.
- Reject missing or unavailable selected models before the run starts.

## I/O rules

- Add path context when saving or loading result files fails.
- Create the configured output directory on demand before writing artifacts.
- Treat invalid saved JSON as a hard error for report regeneration and terminal report rendering.

Last updated: 2026-06-15
