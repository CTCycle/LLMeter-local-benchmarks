# T3/T5 bounded scale continuation evidence

Last updated: 2026-09-28

## Scope and revision

This record covers the next executable local slice after the prior
current-tree revalidation: a bounded higher-concurrency performance matrix,
the adjacent Tier 5 interruption/report/output checks, and a fresh provider
availability audit. It validates the current implementation on Windows
x86-64 at source revision `7d83c225856908fff31d496502e89775302f68a6`
(`develop`) with Rust `1.98.0` and Cargo `1.98.0`.

The source change is a focused regression test only. No runtime defect was
found and no runtime source fix was required.

## Bounded high-concurrency fixture slice

The new `performance_profiles_handle_bounded_high_concurrency_matrix` test
ran the real CLI against the deterministic OpenAI-compatible fixture with:

```text
profile: throughput
prompt sizes: 1, 2
output size: 1
concurrency: 1, 2, 4, 8
warmup requests: 1
measured runs per scenario: 8
streaming: disabled
load measurement: off
telemetry: off
```

The run produced 8 scenario records, 8 traces per scenario, 64 measured
requests, 8 warmup requests, and 72 captured chat requests. Every trace was
successful with the fixture's accepted HTTP 200/201 responses, and each
scenario retained the configured concurrency and request count. The test
passed `1/1` and is also included in the full mock-provider result below.

This is a bounded fixture-scale scheduling and accounting result. It does not
claim live-provider performance, production-scale capacity, statistical
significance, or host-variance coverage.

## Current-tree validation results

| Boundary | Command | Result |
|---|---|---|
| Performance planning and safety | `cargo test --locked --test performance_cli_tests -- --test-threads=1` | `PASS` — 7/7 |
| Default matrices plus bounded high-concurrency matrix and provider/error contracts | `cargo test --locked --test mock_provider_e2e -- --test-threads=1` | `PASS` — 20/20 |
| Report reload and generation | `cargo test --locked --test report_cli_e2e -- --test-threads=1` | `PASS` — 1/1 |
| Atomic replacement and failed-rename cleanup | `cargo test --locked --lib atomic_write -- --test-threads=1` | `PASS` — 2/2 |
| Windows ConPTY navigation and interruption/recovery | `cargo test --locked --test pty_menu_e2e -- --test-threads=1` | `PASS` — 10/10 |
| Formatting | `cargo fmt --all -- --check` | `PASS` |
| Locked all-target/all-feature check | `cargo check --locked --all-targets --all-features` | `PASS` |
| Warning-denied Clippy | `cargo clippy --locked --all-targets --all-features -- -D warnings` | `PASS` |
| Serialized all-target/all-feature tests | `cargo test --locked --all-targets --all-features -- --test-threads=1` | `PASS` — 150/150 |
| Warning-denied rustdoc | `$env:RUSTDOCFLAGS='-D warnings'; cargo doc --locked --no-deps --all-features` | `PASS` |
| Debug and release binaries | `cargo build --locked --bin llmeter` and `cargo build --locked --release --bin llmeter` | `PASS` |

No failure remained after the focused correction to accept both fixture
success statuses already covered by the provider contract.

## Provider availability and blocked live continuation

The live continuation was rechecked rather than inferred from the previous
ledger:

- The Ollama executable is installed, but no `ollama` process was running and
  TCP port `11434` was not listening.
- `cargo run --locked -- --provider ollama --timeout 2 status` returned exit
  code `1`, `API reachable: no`, and `Models exposed: 0`.
- The default local ports for LM Studio (`1234`), llama.cpp/LocalAI/TGI/MLX-LM
  (`8080`), vLLM (`8000`), SGLang (`30000`), text-generation-webui (`5000`),
  Jan (`1337`), and LiteLLM (`4000`) were also unreachable.

The full default-size live performance continuation is therefore `BLOCKED`
by provider availability on this host. No live-provider claim is added. The
best-effort provider matrix and cross-provider optional-capability slice
remain unvalidated, and the prior successful Ollama evidence remains bounded
to its recorded model and custom matrix.

## Final disposition

- `benchmark.performance` remains `PARTIAL`: deterministic default matrices,
  bounded concurrency through level 8, prior bounded live profiles, and all
  current safety/accounting checks pass; full default-size live workloads,
  broader provider/model/host samples, and statistical interpretation remain
  open.
- Tier 5 remains `PARTIAL`: the current suite still passes bounded failures,
  atomic cleanup, report reload, repeated completed runs, delayed-request
  interruption with no partial artifact, fresh-process recovery, and Windows
  ConPTY interruption. Resumable in-progress state, broader scale ceilings,
  and native non-Windows terminal behavior remain open.
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
