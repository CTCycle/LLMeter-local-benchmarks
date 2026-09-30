# Tier 3 release-performance evidence

Last updated: 2026-09-30

## Revision and environment

- Repository: `CTCycle/LLMeter-local-benchmarks`
- Starting revision: `e0a4723d76c0836eff83b6190ebb575fc5562c2d` on `develop`
- Candidate revision: `c5864bf4e303646fca74ea98707ca4081172c0b9`.
- Package: `llmeter 0.4.0`
- Host: Windows 11 Pro `10.0.26200`, x86-64; Rust `1.98.0`, Cargo `1.98.0`
- Provider fixture: ephemeral loopback OpenAI-compatible mock provider in `tests/mock_provider_e2e.rs`
- Installed Ollama client: `0.34.0`; endpoint `http://localhost:11434/v1`

The task-owned source changes are a bounded Unix PTY API correction and a
performance trace-identity correction. `tests/pty_menu_unix_e2e.rs` now uses
the locked `expectrl 0.9.0` / `ptyprocess 0.5.0` status API with bounded
polling and typed `WaitStatus` exit assertions. `src/performance/runner.rs`
now includes output-token size in each request ID. The former sweep IDs
collided across the three output sizes even though all 81 requests executed;
the default-matrix fixture now asserts 81 unique sweep IDs.

All authoritative current live runs used isolated `LLMETER_HOME`,
`LLMETER_CONFIG_DIR`, and output roots; `--timeout 120`,
`--load-measurement off`, standard telemetry, a 1000 ms sample interval,
`--max-requests 500`, JSON and CSV exports, Markdown and HTML reports, full
request traces, and no response previews. The disposable roots were removed
after the artifact audit.

## Deterministic release ceiling

Focused command:

```text
cargo test --locked --test mock_provider_e2e performance_reaches_configured_request_ceiling_with_complete_accounting -- --test-threads=1
```

Result: PASS.

- Matrix: 5 prompt sizes × 1 output size × concurrency `1,2,4,8,16` = 25 scenarios.
- Warmup: 1 per scenario = 25 requests.
- Measured: 19 per scenario = 475 requests.
- Total planned and observed mock chat requests: exactly 500.
- Every scenario persisted 19 successful measured traces with unique request IDs/run indices.
- Result schema: `3.0`; run kind: `performance`; persisted scenario count: 25.
- JSON reload through `report show`: PASS.
- Default privacy flags and absent response previews: PASS.
- Atomic-write temporary-file cleanup: PASS.

The adjacent refusal command used the same matrix with 20 measured runs,
which would require 525 requests, without `--allow-large-matrix`.

Result: PASS. The command failed before any performance chat request, reported
`requests 525 exceed --max-requests 500`, and wrote no result artifact.

The complete focused mock-provider file passed 23/23, including JSONL
accounting, all default matrices, bounded concurrency, provider contracts,
launcher behavior, and both release-ceiling cases. The sweep assertion
confirmed 81 unique request IDs after the output-size identity fix.

## CI-failure remediation

The starting hosted run [`36698403795`](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36698403795)
failed to compile `tests/pty_menu_unix_e2e.rs` on Ubuntu and both macOS
architectures because `ptyprocess 0.5.0` exposes `wait()` without a timeout
and returns `WaitStatus`, not an integer. Windows passed the run. The scoped
fix keeps the five-second bound by polling the non-blocking `status()` API,
force-killing only after the bound, and retaining explicit exit-code matching.
The follow-up hosted failure showed that `expectrl::spawn` does not interpret
shell quoting or `env` assignments; the tests now use
`expectrl::Session::spawn(std::process::Command)` with explicit arguments and
environment variables. Native Unix compilation and execution remain hosted-CI
requirements on this Windows checkout.

Hosted run [`36719598621`](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36719598621)
qualified the prior documentation follow-up revision
`548431bd28ff51d7adfe5d0a36e9ba4f5668c90e` as Windows PASS, but Ubuntu,
macOS Intel, and macOS Apple silicon failed the Unix PTY test step. The
failures were exit status `127` and a delayed-request setup timeout because
the quoted command was passed literally; the direct-command correction is the
new source candidate `c5864bf4e303646fca74ea98707ca4081172c0b9`.

## Local quality evidence

The following passed after the scoped source changes:

- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets --all-features`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features`
- `cargo build --locked --release --all-features`
- isolated `cargo audit` using the current advisory database
- `cargo tree --locked --duplicates` inspection
- `cargo test --locked --all-targets --all-features -- --test-threads=1`

The full serialized test command passed 156 tests with zero failures. The
Windows `pty_menu_unix_e2e` target ran zero tests by its Unix cfg; hosted CI is
required for the native Unix cases. The warning-denied rustdoc check and
release binary `--version` / `--help` smoke also passed.

## Current live Ollama profiles

The official launcher checks passed with Ollama `0.34.0`; the model inventory
contained `qwen3.5:2b` and `qwen3.5:9b` (five models total). The fixed current
profiles were:

| Profile / model | Run ID | Scenarios | Warmup | Measured | Total | Errors | Result |
|---|---|---:|---:|---:|---:|---:|---|
| `latency` / `qwen3.5:2b` | `2026-09-30T123739.621475Z-p9424-qwen3.5-2b` | 3 | 3 | 15 | 18 | 0 | PASS |
| `throughput` / `qwen3.5:2b` | `2026-09-30T123850.947857Z-p31096-qwen3.5-2b` | 4 | 4 | 16 | 20 | 0 | PASS |
| `sweep` / `qwen3.5:2b` | `2026-09-30T124048.622209Z-p10960-qwen3.5-2b` | 27 | 27 | 81 | 108 | 0 | PASS |

The fixed sweep retained 81 successful measured traces and 81 unique request
IDs. Each run saved a schema `3.0`, `performance` result with the expected
scenario and measured-trace counts. Fresh release-binary `report list` and
`report show` processes returned exit code `0` for every run.

## Matched two-model functional matrix

Run ID: `2026-09-30T124647.076747Z-p17212-qwen3.5-2b-qwen3.5-9b`.

The same non-streaming matrix ran for `qwen3.5:2b` and `qwen3.5:9b`: prompt
tokens `32`, output tokens `16`, concurrency `1,2`, one warmup and 20 measured
requests per scenario, standard telemetry, load measurement off, and the
500-request cap. The plan contained 4 scenarios, 4 warmups, 80 measured
requests, and 84 total requests. All 4 records and all 80 measured traces
succeeded; every scenario has enough samples for P95 representation, and the
80 request IDs were unique.

The saved result reported these host observations:

| Model | Concurrency | Average wall time |
|---|---:|---:|
| `qwen3.5:2b` | 1 | 319.02 ms |
| `qwen3.5:2b` | 2 | 718.78 ms |
| `qwen3.5:9b` | 1 | 3254.10 ms |
| `qwen3.5:9b` | 2 | 7626.93 ms |

Functional completion and persistence pass. The run reached approximately
0.95 maximum swap-used ratio and approximately 0.81 maximum memory ratio, so
comparative timing interpretation is not validated and no model ranking is
claimed.

## Persistence and privacy

Fresh release-binary processes ran `report list` and `report show` for each of
the four current saved run directories. The audit found:

- schema `3.0`, run kind `performance`, and zero result errors;
- expected scenario and measured-trace counts, with successful HTTP 200 traces;
- unique request IDs and complete, non-truncated request traces;
- JSON, CSV, Markdown, and HTML output for each run;
- `response_previews_included: false` and zero non-null response previews;
- no credential-shaped values in retained formats;
- zero `.tmp` or `.partial` artifacts after completion.

## Optional second-provider boundary

LiteLLM at `http://localhost:4000/v1` was unavailable. Docker was installed
but its Linux engine was not reachable, so no temporary proxy was started and
no provider, model, container, PATH, or permanent configuration was changed.
The previous Tier 4 LiteLLM-over-Ollama evidence remains historical
representative coverage and is not promoted to this current smoke.

## Disposition

`T3-05: PARTIAL`. The 500-request ceiling, above-limit refusal, deterministic
regressions, full current Ollama profiles, matched two-model functional matrix,
persistence, reload, trace identity, reporting, and privacy boundaries pass.
Comparative timing remains unvalidated under swap pressure, LiteLLM is
unavailable, and exact-candidate four-platform CI is pending. No universal or
cross-host numeric performance claim is made.
