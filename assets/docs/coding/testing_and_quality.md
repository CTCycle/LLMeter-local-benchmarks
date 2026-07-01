# Testing and quality

## Test framework

Rust's built-in `#[test]` attribute is the test framework. Integration tests live in `tests/` and import the library crate via `use llmeter::...`.

Run all tests:

```bash
cargo test -- --test-threads=1
```

Use a single test thread because configuration tests mutate process environment variables.

Run a specific test:

```bash
cargo test test_name
```

## Clippy

Clippy runs as a linter with `-D warnings` (deny mode) in CI.

Check lint:

```bash
cargo clippy -- -D warnings
```

## Formatting

`cargo fmt` ensures consistent code style.

Check formatting:

```bash
cargo fmt --check
```

## CI

GitHub Actions runs on push and pull request. Steps:
1. Install stable Rust toolchain with clippy.
2. Run `cargo fmt --check`.
3. Run `cargo clippy -- -D warnings`.
4. Run `cargo test -- --test-threads=1`.

## Test coverage

Currently no coverage threshold enforced. Each test file in `tests/` mirrors a source module:

| Test file | Module under test |
|---|---|
| `tests/test_metrics.rs` | `llmeter::benchmarks::metrics` |
| `tests/test_results.rs` | `llmeter::results` |
| `tests/test_registry.rs` | `llmeter::benchmarks::registry` |
| `tests/test_reporting.rs` | `llmeter::reporting` |
| `tests/mock_provider_e2e.rs` | Real CLI execution against a local mock OpenAI-compatible `/v1` provider |
| `tests/performance_cli_tests.rs` | `llmeter::performance::config`, request matrix estimation, safety guards, and CLI parsing |
| `tests/performance_metrics_tests.rs` | `llmeter::performance::metrics` |
| `tests/quality_cli_tests.rs` | `llmeter::quality::*` planning surfaces |
| `tests/result_schema_tests.rs` | backward-compatible run serialization |

## Expectations for new benchmark surfaces

- Native performance math must include direct tests for percentile behavior, rates, and aggregate counts.
- CLI changes must include parse coverage for new subcommands and key validation paths.
- Quality adapters must remain dry-run by default and test command preview generation without installing tools or downloading datasets.
- Result schema changes must preserve old JSON readability when fields are absent.

Last updated: 2026-07-01
