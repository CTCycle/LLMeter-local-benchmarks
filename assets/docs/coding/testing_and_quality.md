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

## Check and documentation

Compile all targets and features without producing a binary:

```bash
cargo check --locked --all-targets --all-features
```

Build documentation with rustdoc warnings denied:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
```

## Formatting

`cargo fmt` ensures consistent code style.

Check formatting:

```bash
cargo fmt --all -- --check
```

## CI

GitHub Actions runs on push and pull request on native Ubuntu, Windows, and
both macOS architectures. Each matrix job uses the locked dependency graph
and runs all-target/all-feature check, Clippy with warnings denied, the
serialized test suite, and documentation with rustdoc warnings denied.
Formatting is checked once on Ubuntu because it is platform-independent.
`tests/pty_menu_e2e.rs` supplies the Windows ConPTY boundary, while
`tests/pty_menu_unix_e2e.rs` must execute natively on Ubuntu, macOS Intel, and
macOS Apple silicon.

## Test coverage

No coverage threshold is enforced. Integration tests are grouped by area:

| Test file | Coverage |
|---|---|
| `tests/cli_contract_tests.rs` | Non-TTY dispatch, exit codes, help/catalog output, and CLI option parsing. |
| `tests/launcher_powershell_e2e.rs` | Windows launcher build reuse, argument forwarding, confirmation, protected-home refusal, and cleanup. |
| `tests/lifecycle_cli_e2e.rs` | Install, update, uninstall, and purge lifecycle behavior. |
| `tests/mock_provider_e2e.rs` | Real CLI execution against a local mock OpenAI-compatible `/v1` provider, including the preset baseline contract. |
| `tests/report_cli_e2e.rs` | Report list, show, and generate flows against saved runs. |
| `tests/result_schema_tests.rs` | Run serialization and schema identity. |
| `tests/test_results.rs` | Result persistence, atomic writes, CSV safety, and privacy policy. |
| `tests/test_reporting.rs` | Markdown/HTML aggregation and output escaping. |
| `tests/test_metrics.rs` | Benchmark metric extraction and pairwise similarity. |
| `tests/test_registry.rs` | Benchmark registry selection and suite invariants. |
| `tests/performance_cli_tests.rs` | Performance plan construction, matrix estimation, safety guards, and CLI parsing. |
| `tests/performance_metrics_tests.rs` | Performance metric math: percentiles, rates, and aggregate counts. |
| `tests/provider_probe_tests.rs` | Provider capability probing and model metadata parsing. |
| `tests/resource_snapshot_tests.rs` | Environment and resource snapshot capture. |
| `tests/resilience_e2e.rs` | Abrupt termination, durable-result preservation, restart report loading, and recovery. |
| `tests/quality_cli_tests.rs` | Quality catalog and dry-run plan generation. |
| `tests/pty_menu_e2e.rs` | Windows ConPTY launch, interruption, and child-process cleanup. |
| `tests/pty_menu_unix_e2e.rs` | Native Unix PTY menu cancellation, performance interruption, cleanup, and recovery. |

Unit coverage in `src/ui.rs` also locks down menu key normalization so an Enter release cannot leak from a prompt into the next menu selection.

`tests/mock_provider_e2e.rs` runs the shared baseline contract across all registered provider presets. The fixture result is contract evidence only; live-provider certification requires an explicitly supplied server.

## Expectations for new benchmark surfaces

- Native performance math must include direct tests for percentile behavior, rates, and aggregate counts.
- CLI changes must include parse coverage for new subcommands and key validation paths.
- Quality adapters must remain dry-run by default and test command preview generation without installing tools or downloading datasets.
- Result schema changes must keep saved `3.0` results readable when newly added optional fields are absent; files with any other `schema_version` are rejected on load.
- The configured `DEFAULT_MAX_PERFORMANCE_REQUESTS = 500` boundary must be
  exercised end to end against the deterministic fixture, with an adjacent
  above-limit refusal that proves no performance chat request is issued.
- Resilience coverage must include both `Child::kill()` abrupt termination and
  clean terminal interruption, preserving completed results and leaving no
  partial canonical or temporary output.
- Native interactive validation expects Windows ConPTY plus the Unix PTY
  integration test on each hosted Linux and macOS architecture; local Windows
  success does not substitute for those hosted executions.

Release-time gates beyond the standard suite (dependency audit, package dry-run,
release build, and extracted-binary smoke checks) are listed in the
[release checklist](../runtime/release_checklist.md). The current evidence
boundary for these gates is recorded in the
[project status ledger](../project_status_ledger.md).

Last updated: 2026-10-18
