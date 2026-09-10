# Testing and quality

## Test framework

Rust's built-in `#[test]` attribute is the test framework. Integration tests live in `tests/` and import the library crate via `use llmeter::...`.

Run all tests:

```bash
cargo test --locked --all-targets --all-features -- --test-threads=1
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
cargo clippy --locked --all-targets --all-features -- -D warnings
```

## Formatting

`cargo fmt` ensures consistent code style.

Check formatting:

```bash
cargo fmt --all -- --check
```

## CI

GitHub Actions runs on push and pull request on native Ubuntu and Windows runners. Both platforms use the locked dependency graph and run all-target/all-feature check, Clippy with warnings denied, the serialized test suite, and documentation with rustdoc warnings denied. Formatting is checked once on Ubuntu because it is platform-independent; the Windows job supplies native PTY and primary-platform evidence.

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
| `tests/pty_menu_e2e.rs` | Windows ConPTY launch, interruption, and child-process cleanup |

Unit coverage in `src/ui.rs` also locks down menu key normalization so an Enter release cannot leak from a prompt into the next menu selection.

`tests/mock_provider_e2e.rs` runs the shared baseline contract across all registered provider presets. The fixture result is contract evidence only; live-provider certification requires an explicitly supplied server.

## Expectations for new benchmark surfaces

- Native performance math must include direct tests for percentile behavior, rates, and aggregate counts.
- CLI changes must include parse coverage for new subcommands and key validation paths.
- Quality adapters must remain dry-run by default and test command preview generation without installing tools or downloading datasets.
- Result schema changes must preserve old JSON readability when fields are absent.
- Release validation must also run `cargo audit`, a locked package dry-run, a release build, and extracted-binary `--version`/`--help` plus mock-provider smoke checks.

The current local audit evidence covers 132 serialized all-target/all-feature tests on Windows. Native hosted Linux/Windows workflow execution is defined in CI and release workflows but must still be treated separately from local evidence when it has not been run for the current commit.

Last updated: 2026-09-10
