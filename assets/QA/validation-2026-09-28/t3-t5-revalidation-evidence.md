# T3/T5 current-tree revalidation and live-provider availability

Last updated: 2026-09-28

## Scope and revision

This record revisits the next actionable validation scope from the canonical
ledger: deterministic performance execution, the Windows-safe Tier 5 failure
and interruption boundaries, and an attempted continuation of the live
performance slice. It validates the current implementation rather than
carrying forward only the 2026-09-26 ledger text.

The tests ran on Windows x86-64 with Rust 1.98.0 and Cargo 1.98.0 on the
`develop` working tree after base revision `3573a80`. The runtime
implementation remained unchanged; the focused validation adds only the two
regression scenarios recorded below. The delayed-request interruption
scenario also verifies recovery by starting a fresh process in the same output
directory after the interrupted process exits.

## Current ledger audit

The 19 component entries in `assets/docs/project_status_ledger.md` currently
resolve to:

| Status | Components |
|---|---|
| `VALIDATED` (14) | `application.startup`, `cli.interactive`, `cli.scriptable`, `cli.help-catalogs`, `configuration.resolution`, `provider.protocol`, `benchmark.llm`, `benchmark.embeddings`, `results.persistence`, `reporting`, `lifecycle.local-install`, `launcher.powershell`, `test.local-quality-gates`, `release.cross-platform` |
| `PARTIAL` (4) | `provider.presets`, `benchmark.performance`, `quality.external-plans`, `release.public-distribution` |
| `UNVALIDATED` (1) | `provider.best-effort.live` |

No component is marked `BLOCKED`, `FAIL`, or `NOT_IMPLEMENTED`. The current
live continuation is nevertheless externally blocked by provider availability;
the existing `PARTIAL` and `UNVALIDATED` statuses remain the correct component
statuses. The separate open items are crates.io publication/install
(`ISSUE-001`) and external quality-framework execution (`ISSUE-003`).

## Current-tree validation results

| Boundary | Command | Result |
|---|---|---|
| Performance planning and safety | `cargo test --locked --test performance_cli_tests -- --test-threads=1` | `PASS` — 7/7 |
| Default performance matrices | `cargo test --locked --test mock_provider_e2e performance_profiles_execute_default_matrices_through_fixture -- --exact --test-threads=1` | `PASS` — 1/1 |
| Provider, CLI, JSONL, performance, launcher, and output failures | `cargo test --locked --test mock_provider_e2e -- --test-threads=1` | `PASS` — 19/19 |
| Report reload and generation boundary | `cargo test --locked --test report_cli_e2e -- --test-threads=1` | `PASS` — 1/1 |
| Atomic replacement and failed-rename cleanup | `cargo test --locked --lib atomic_write -- --test-threads=1` | `PASS` — 2/2 |
| Windows ConPTY navigation and interruption | `cargo test --locked --test pty_menu_e2e -- --test-threads=1` | `PASS` — 10/10 |

These results revalidate the deterministic T3 performance boundary and the
already bounded T5 failure, atomic-output, safety-ceiling, and Windows
confirmation-interruption subset. The report test confirms that saved result
files can be reopened and rendered by later CLI processes at the existing
T1-06 boundary.

## Focused Tier 5 continuation

Two current-tree regressions extend the Tier 5 boundary:

| Scenario | Result | Evidence boundary |
|---|---|---|
| Two completed `bench run` invocations in fresh processes reuse one output directory | `PASS` | `repeated_cli_runs_keep_distinct_results_and_reload_after_process_restart` produced two schema `3.0` JSON files with distinct run IDs; `report list` and `report show` reloaded both. |
| Ctrl+C during a delayed performance request, followed by a fresh-process recovery run | `PASS` | `pty_interrupted_performance_run_leaves_no_partial_result_and_allows_recovery` reached a loopback provider request, interrupted the ConPTY process while the request was delayed, found no result artifacts, then reused the same output directory from a fresh process and persisted one valid schema `3.0` performance result. |

This validates repeated completed operations, the narrow mid-run
interruption/no-partial-artifact contract, and fresh-process recovery after an
interrupted run. It does not imply resumable in-progress state: the runtime
still builds a run in memory and persists it only after completion.
Restart/state restoration for an interrupted run, broader scale ceilings, and
native non-Windows terminal behavior remain open.

No runtime defect was uncovered, so no source fix was required.

## Final local quality checks

The current tree also passed the relevant local quality checks after the
focused runs:

- `cargo fmt --all -- --check`
- `cargo test --locked --all-targets --all-features -- --test-threads=1` —
  149 passed, 0 failed
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `$env:RUSTDOCFLAGS='-D warnings'; cargo doc --locked --no-deps --all-features`

Hosted CI run [36392887683](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36392887683)
passed all four platform jobs for pushed commit `c966a49` (Ubuntu x86-64,
Windows x86-64, macOS Intel x86-64, and macOS Apple silicon aarch64). This
confirms the focused test additions and ledger update on the hosted matrix;
it does not create a new release claim.

## Live performance continuation

The next live slice remains full default-size performance profiles, or another
named-budget repeated profile slice when a generation-capable provider/model is
available. The current host could not execute it:

- `ollama.exe` is installed, but no `ollama` process or listener on port 11434
  was present.
- `ollama list` failed while waiting for the server to start and reported
  access-denied cleanup for the canonical Ollama app log path.
- The inspected `C:\Users\Thomas V\.ollama\cache` contained only
  `model-recommendations.json`; no model blobs were available there.
- `cargo run --locked -- --provider ollama --timeout 2 status` returned exit
  code 1 with `API reachable: no`, `Models exposed: 0`, and a failed request to
  `http://localhost:11434/v1`.

This is an external environment limitation, not an LLMeter failure. The
successful 2026-09-26 Ollama evidence remains valid at its recorded custom
matrix, while full default-size live timing, broader concurrency, model/
provider/host variance, and statistical interpretation remain open.

## Final disposition and next actions

- `benchmark.performance` remains `PARTIAL`: deterministic default matrices
  and the prior bounded live subset pass, but the live continuation is blocked
  until a generation-capable provider and model are available.
- Tier 5 remains `PARTIAL`: repeated completed operations, the narrow
  delayed-request interruption/no-partial-artifact boundary, and
  fresh-process recovery after interruption now pass, while resumable
  restart/state restoration, broader scale ceilings, and native non-Windows
  terminal behavior remain unvalidated.
- `provider.best-effort.live` remains `UNVALIDATED`: no TGI,
  text-generation-webui, Jan, or MLX-LM service was available for live status,
  discovery, capability, and benchmark evidence.
- `release.public-distribution` and `ISSUE-001` remain owner-gated on the
  first crates.io publication and clean install. `quality.external-plans` and
  `ISSUE-003` remain the approved dry-run/scope boundary.

The next executable slices are to restore a generation-capable local provider
for the full default-size live profiles and separately design a true
restart/state-restoration scenario for an interrupted run. Broader scale and
native non-Windows terminal evidence remain separate follow-ups. No status was
promoted from fixture evidence to live certification.
