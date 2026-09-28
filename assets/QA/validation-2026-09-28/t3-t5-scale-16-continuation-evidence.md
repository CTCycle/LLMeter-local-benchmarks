# T3/T5 bounded scale-16 continuation evidence

Last updated: 2026-09-28

## Scope and revision

This record covers the next executable provider-independent slice after the
level-8 fixture boundary: a bounded performance matrix that reaches
concurrency 16, the adjacent Tier 5 regression boundary, and a fresh live
provider-availability check. It validates the current implementation on
Windows x86-64 at source revision `1b9d38b4f2cf120461ac3cefc2a2f84ac6112fa8`
(`develop`) with Rust `1.98.0` and Cargo `1.98.0`.

The source change is a focused E2E regression-test expansion. No runtime
defect was found and no runtime source fix was required.

## Current ledger audit

The 19 component gates in `assets/docs/project_status_ledger.md` resolve to:

| Status | Components |
|---|---|
| `VALIDATED` (14) | `application.startup`, `cli.interactive`, `cli.scriptable`, `cli.help-catalogs`, `configuration.resolution`, `provider.protocol`, `benchmark.llm`, `benchmark.embeddings`, `results.persistence`, `reporting`, `lifecycle.local-install`, `launcher.powershell`, `test.local-quality-gates`, `release.cross-platform` |
| `PARTIAL` (4) | `provider.presets`, `benchmark.performance`, `quality.external-plans`, `release.public-distribution` |
| `UNVALIDATED` (1) | `provider.best-effort.live` |

No component is marked `BLOCKED`, `BROKEN`, or `NOT_IMPLEMENTED`. The live
performance continuation is still externally blocked by provider availability;
that condition is recorded separately from the component taxonomy. `ISSUE-001`
(crates.io publication/install) and `ISSUE-003` (external quality-framework
execution) remain owner- or scope-gated.

## Bounded fixture-scale continuation

`performance_profiles_handle_bounded_high_concurrency_matrix` now exercises
the real CLI fixture with:

```text
profile: throughput
prompt sizes: 1, 2
output size: 1
concurrency: 1, 2, 4, 8, 16
warmup requests: 1 per scenario
measured runs: 16 per scenario
streaming: disabled
load measurement: off
telemetry: off
```

The run produced 10 scenario records, 16 traces per scenario, 160 measured
requests, 10 warmup requests, and 170 captured chat requests. Every trace was
successful with the fixture's accepted HTTP 200/201 responses, and the
level-16 scenario reached 16 concurrently scheduled measured requests rather
than only recording a larger label. This is fixture-scale scheduling and
accounting evidence; it does not claim live-provider capacity, production
scale, statistical significance, or host-variance coverage.

## Current-tree validation results

| Boundary | Command | Result |
|---|---|---|
| Bounded scale and provider/error contracts | `cargo test --locked --test mock_provider_e2e -- --test-threads=1` | `PASS` — 20/20 |
| Performance planning and safety | `cargo test --locked --test performance_cli_tests -- --test-threads=1` | `PASS` — 7/7 |
| Report reload and generation | `cargo test --locked --test report_cli_e2e -- --test-threads=1` | `PASS` — 1/1 |
| Atomic replacement and failed-rename cleanup | `cargo test --locked --lib atomic_write -- --test-threads=1` | `PASS` — 2/2 |
| Windows ConPTY navigation and interruption/recovery | `cargo test --locked --test pty_menu_e2e -- --test-threads=1` | `PASS` — 10/10 |
| Formatting | `cargo fmt --all -- --check` | `PASS` after correcting one formatter finding in the new test |
| Locked all-target/all-feature check | `cargo check --locked --all-targets --all-features` | `PASS` |
| Warning-denied Clippy | `cargo clippy --locked --all-targets --all-features -- -D warnings` | `PASS` |
| Serialized all-target/all-feature tests | `cargo test --locked --all-targets --all-features -- --test-threads=1` | `PASS` — 150/150 |
| Warning-denied rustdoc | `$env:RUSTDOCFLAGS='-D warnings'; cargo doc --locked --no-deps --all-features` | `PASS` |
| Debug and release binaries | `cargo build --locked --bin llmeter` and `cargo build --locked --release --bin llmeter` | `PASS` |

The selected Tier 5 boundary remains green: repeated completed operations,
atomic output cleanup, delayed-request interruption with no partial result,
fresh-process recovery, and Windows ConPTY interruption all passed. The
implementation persists a run only after completion, so this does not prove
resumable in-progress state or restart/state restoration.

## Live-provider availability recheck

The next live profile slice was attempted as an availability check and could
not start:

- No listener was present on the default ports for Ollama, LM Studio,
  llama.cpp/LocalAI/TGI/MLX-LM, vLLM, SGLang, text-generation-webui, Jan, or
  LiteLLM.
- `ollama.exe` is installed at
  `C:\Users\Thomas V\AppData\Local\Programs\Ollama\ollama.exe`, but no
  Ollama process was running.
- `target\debug\llmeter.exe --provider ollama --timeout 2 status` returned
  exit code `1`, `API reachable: no`, and `Models exposed: 0` for
  `http://localhost:11434/v1`.
- The inspected `C:\Users\Thomas V\.ollama\cache` directory returned no
  model-cache entries.

This is an external host limitation, not an LLMeter failure. No live result
was synthesized from the fixture run. Full default-size live profiles,
broader live concurrency, provider/model/host variance, and production-sized
statistical interpretation remain open.

## Final disposition and next actions

- `benchmark.performance` remains `PARTIAL`: deterministic default matrices,
  fixture scale through concurrency 16, prior bounded live profiles, and the
  safety/accounting checks pass; full live profiles and broader live evidence
  still require an available generation-capable provider/model.
- Tier 5 remains `PARTIAL`: the bounded failure, atomic-output, interruption,
  recovery, and Windows terminal boundaries pass; resumable restart/state
  restoration, production-scale ceilings, and native non-Windows terminal
  behavior remain open.
- `provider.best-effort.live` remains `UNVALIDATED`; no TGI,
  text-generation-webui, Jan, or MLX-LM service was available for live
  status, discovery, capability, or benchmark evidence.
- `release.public-distribution`/`ISSUE-001` remains owner-gated on crates.io
  publication and clean installation. `quality.external-plans`/`ISSUE-003`
  remains at the approved dry-run and scope boundary.

The next executable live slice is full default-size performance validation
when a generation-capable provider and model are available. The resumable
restart/state-restoration question remains a separate product-scope decision;
the current implementation intentionally persists only completed runs.
