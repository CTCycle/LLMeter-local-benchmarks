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
- user interruption and user cancellation

## Exit behavior

`src/main.rs` maps failures to process exit codes:

- `0` for an intentional user cancellation
- `130` for an interrupted interactive operation
- `2` for other recognized `LLMeterError` values
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
- Performance failures remain visible in scenario counts and error rates; they are not included in successful latency or token-throughput denominators.

## Option and input validation

- Reject malformed repeated `--param` values unless they follow `key=value`.
- Reject unknown benchmark IDs early through registry selection.
- Reject missing or unavailable selected models before the run starts.
- Keep performance safety controls on dedicated flags (`--allow-large-prompt` and `--allow-large-matrix`) rather than allowing provider parameters to bypass them.

## I/O rules

- Add path context when saving or loading result files fails.
- Create the configured output directory on demand before writing artifacts.
- Treat invalid saved JSON as a hard error for report regeneration and terminal report rendering.
- Write persisted configuration and output artifacts through atomic same-directory temporary files.

Last updated: 2026-08-02
