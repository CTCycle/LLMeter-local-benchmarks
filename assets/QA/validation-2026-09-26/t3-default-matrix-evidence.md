# T3-03 default performance matrix evidence

Last updated: 2026-09-26

## Scope and boundary

This slice executes every built-in performance profile through the real CLI at its profile-owned default matrix against the deterministic OpenAI-compatible fixture. It closes the deterministic default-matrix execution boundary only; it does not claim live-provider timing, provider-side concurrency, production-sized workload behavior, or statistical interpretation.

The implementation under test is source revision `2af310a5b9e395485dfc8a578dd19afc57fe7406` (`test: execute default performance profile matrices`). The product implementation is unchanged from the preceding T3-03 validation; the revision adds a focused regression for the previously unexecuted default matrices.

Environment: Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0.

## Default-matrix execution

The regression `performance_profiles_execute_default_matrices_through_fixture` in [`tests/mock_provider_e2e.rs`](../../../tests/mock_provider_e2e.rs) ran four independent CLI processes against the fixture. It checked the persisted plan dimensions, warmup and measured counts, scenario records, successful traces, and total fixture chat requests.

| Profile | Prompt sizes | Output sizes | Concurrency | Warmup / runs | Scenarios | Measured / total requests |
|---|---|---|---|---:|---:|---:|
| `smoke` | `[128, 512]` | `[128]` | `[1]` | `1 / 3` | 2 | `6 / 8` |
| `latency` | `[128, 512, 2048]` | `[128]` | `[1]` | `1 / 5` | 3 | `15 / 18` |
| `throughput` | `[512]` | `[256]` | `[1, 2, 4, 8]` | `1 / 4` | 4 | `16 / 20` |
| `sweep` | `[128, 512, 2048]` | `[64, 128, 256]` | `[1, 2, 4]` | `1 / 3` | 27 | `81 / 108` |
| **Total** | — | — | — | — | **36** | **118 / 154** |

All measured fixture traces succeeded. The fixture saw exactly 154 chat-completion POST requests, including 36 warmup requests and 118 measured requests. Persisted scenario traces contain the measured requests; warmup requests are intentionally not persisted as measured records.

## Commands and results

| Command | Result |
|---|---|
| `cargo test --locked --test mock_provider_e2e performance_profiles_execute_default_matrices_through_fixture -- --exact` | PASS — 1 test |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --locked --all-targets --all-features` | PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | PASS |
| `cargo test --locked --all-targets --all-features -- --test-threads=1` | PASS — 147 tests |
| `$env:RUSTDOCFLAGS='-D warnings'; cargo doc --locked --no-deps --all-features` | PASS |
| `cargo build --locked --bin llmeter` | PASS |

The live-provider recheck was attempted with an isolated `LLMETER_HOME` using `cargo run --locked -- --provider ollama --timeout 2 status`. It returned exit code 1 with `API reachable: no`, `Models exposed: 0`, and a failed request to `http://localhost:11434/v1`. No local Ollama model cache was available. This is an environment limitation for the live continuation, not a fixture or application failure.

## Final status and next slices

- T3-03 deterministic default-matrix execution: `PASS` at the fixture boundary.
- `benchmark.performance` and Tier 3: remain `PARTIAL`. The bounded live Ollama progression from 2026-09-24 remains valid at its recorded scope, but full default-size live profiles, repeated samples, and broader provider/model/host variance are still open.
- The next actionable live slice is a named-budget repeated `latency`/`throughput` run when a generation-capable provider and model are available. The current Ollama-unavailable condition is recorded as a blocker for that live slice.
- Tier 4 best-effort-provider coverage, external quality execution, and crates.io publication/install were not in scope for this fixture slice and remain explicitly incomplete or owner-gated.
- Tier 5 resilience, interruption, filesystem/restart, scale, and non-Windows terminal validation remains `UNRUN` at its dedicated campaign boundary.

No product defect was uncovered by this slice. The new regression protects the default profile matrices and their warmup/measured accounting without changing runtime behavior.
