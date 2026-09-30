# Current-tree validation recheck

Last updated: 2026-09-30

## Scope and revision boundary

- Repository: `CTCycle/LLMeter-local-benchmarks`.
- Base checkout: `develop` at `6ef50aa`.
- The revalidated source change is currently in the working tree: a Windows
  ConPTY interrupt monitor plus cancellation checks in the performance runner.
  The Git index was not writable in this environment, so no new commit SHA is
  claimed for this recheck.
- Existing release evidence for candidate `1fb3dbd1f3211e25280481f5bbdab439a850a2f9`
  and hosted CI run `36722865383` remains valid for that earlier candidate but
  does not certify the working-tree fix.

## Defect found and repaired

The first current-tree serialized test run failed
`pty_interrupted_performance_run_leaves_no_partial_result_and_allows_recovery`:
the Windows ConPTY ETX input was not observed by the non-interactive
performance path, and the delayed child did not exit within the test's five
second bound. The repair adds a narrowly scoped ConPTY event monitor for
non-menu commands and checks the shared interrupt state between performance
requests. An interrupted run returns before `save_outputs`, preserving the
completed result and atomic-output boundary.

After the repair:

- `cargo test --locked --test pty_menu_e2e -- --test-threads=1`: 10/10 passed;
- `cargo test --locked --all-targets --all-features -- --test-threads=1`:
  156 passed, 0 failed;
- the focused delayed-request interruption test passed and verified recovery,
  unchanged completed output, and no temporary artifacts.

## Current local gates

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --locked --all-targets --all-features` | PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | PASS |
| `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features` | PASS |
| `cargo build --locked --release --all-features` | PASS |
| Release binary `--version` / `--help` | PASS (`llmeter 0.4.0`) |
| `cargo audit --no-fetch --stale --no-yanked` | PASS; 1,277 advisories loaded and 252 locked dependencies scanned |
| `cargo tree --locked --duplicates` | PASS; inspection completed |

The initial `cargo audit --locked` form is unsupported by the installed audit
CLI. A fetch-enabled retry was also blocked by the read-only shared Cargo
cache; the recorded audit uses the available local advisory database and
explicitly omits the yanked-crate registry check.

## Current live boundary

- Release-binary `ollama status`: PASS; endpoint reachable and five models
  exposed.
- Release-binary `ollama models --json`: PASS; `qwen3.5:2b`, `qwen3.5:9b`,
  two 27b models, and `nomic-embed-text:latest` were returned.
- Isolated one-request `qwen3.5:2b` non-streaming smoke: PASS; one successful
  schema `3.0` performance record, no response preview, and the scratch output
  root was removed after inspection. This is a provider smoke, not comparative
  timing evidence.
- Docker client `29.8.0` is installed, but no Docker engine is reachable;
  LiteLLM at `http://localhost:4000/v1` therefore remains unavailable.
- The Windows memory probe is permission-restricted in this execution
  environment. Existing application telemetry still records swap pressure on
  the matched live matrix, so comparative timing remains unqualified.

## Disposition

The current working tree is locally green and the Windows interruption defect
is repaired. Tier 3 remains `PARTIAL`: the full default Ollama profiles and
matched functional matrix remain valid, but comparative timing needs a host
without swap-pressure warnings and the optional current LiteLLM smoke needs an
available disposable proxy. Tier 4 remains PASS at its recorded representative
provider and GitHub distribution boundary; crates.io publication/install is
still owner-gated. Tier 5 remains PASS at the finite T5-02 boundary, with the
current Windows PTY interruption path revalidated locally. Hosted CI must be
rerun for the working-tree fix before it is treated as an exact-candidate
cross-platform release gate.
The environment-limited timing and optional provider gaps are non-blocking for
the scoped release boundary; they prevent only the comprehensive-validation and
numeric comparative-performance claims.

