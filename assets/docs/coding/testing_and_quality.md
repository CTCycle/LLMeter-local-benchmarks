# Testing and quality

## Test framework

Rust's built-in `#[test]` attribute is the test framework. Integration tests live in `tests/` and import the library crate via `use llmeter::...`.

Run all tests:

```bash
cargo test
```

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
4. Run `cargo test`.

## Test coverage

Currently no coverage threshold enforced. Each test file in `tests/` mirrors a source module:

| Test file | Module under test |
|---|---|
| `tests/test_metrics.rs` | `llmeter::benchmarks::metrics` |
| `tests/test_results.rs` | `llmeter::results` |
| `tests/test_registry.rs` | `llmeter::benchmarks::registry` |
| `tests/test_reporting.rs` | `llmeter::reporting` |

Last updated: 2026-06-12
