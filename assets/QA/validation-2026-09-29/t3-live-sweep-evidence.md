# T3 live full-sweep evidence

Last updated: 2026-09-29

## Boundary

| Field | Value |
|---|---|
| Validation checkout | `528fe1cbebf9deba6969425a2d959faad230a17e` on `develop` |
| Package / binary | `llmeter 0.4.0`, `target/release/llmeter.exe` |
| Environment | Windows 11 Pro `10.0.26200`, x86-64, PowerShell, Rust `1.98.0`, Cargo `1.98.0` |
| Provider | Ollama `0.34.0`, `http://127.0.0.1:11434/v1` |
| Model | `qwen3.5:2b` |
| Isolation | Dedicated temporary `LLMETER_HOME` and `LLMETER_CONFIG_DIR`; retained output under `artifacts/ollama-sweep/` |
| Evidence boundary | Full default `sweep` profile for one Ollama model; not cross-provider, cross-host, or statistical certification |

The provider and model were confirmed through the fresh `/v1/models` catalog before execution. No provider executable, model, container, PATH entry, startup entry, or permanent configuration was added by this validation.

The executed command used the exact provider lane above, the default sweep matrix, `--load-measurement off`, standard telemetry at a 1000 ms sample interval, both raw exports, both reports, detailed report output, and `--max-requests 500`.

## Full sweep result

| Check | Result |
|---|---:|
| Process exit code | `0` |
| Run ID | `2026-09-29T154003.572145Z-p12508-qwen3.5-2b` |
| Profile | `sweep` |
| Prompt sizes | `128`, `512`, `2048` |
| Output sizes | `64`, `128`, `256` |
| Concurrency | `1`, `2`, `4` |
| Planned scenarios | `27` |
| Planned warmups | `27` |
| Planned measured requests | `81` |
| Planned total requests | `108` |
| Persisted result records | `27` |
| Persisted measured traces | `81` (`3` per scenario) |
| Successful records | `27` |
| Error records | `0` |
| Timeout records | `0` |
| Schema / run kind | `3.0` / `performance` |
| Response previews | Disabled |

The CLI completed every scenario and reported `27 total, 27 ok, 0 errors`. The saved JSON records the exact provider, base URL, model, streaming mode, sweep profile, complete measured traces, and environment snapshot.

Retained artifacts:

- [JSON result](artifacts/ollama-sweep/2026-09-29T154003.572145Z-p12508-qwen3.5-2b.json)
- [CSV result](artifacts/ollama-sweep/2026-09-29T154003.572145Z-p12508-qwen3.5-2b.csv)
- [Markdown report](artifacts/ollama-sweep/2026-09-29T154003.572145Z-p12508-qwen3.5-2b.report.md)
- [HTML report](artifacts/ollama-sweep/2026-09-29T154003.572145Z-p12508-qwen3.5-2b.report.html)

The saved result was reloadable through `report list` and `report show`, both with exit code `0`.

## Privacy and runtime observations

- Recursive scans of the four retained artifacts found no credential-shaped values, bearer tokens, authorization values, API-key values, or non-empty response-preview values.
- Telemetry collected `235` samples. The maximum observed memory-used ratio was approximately `0.638`; the maximum swap-used ratio was approximately `0.368`.
- The runtime emitted `Swap used ratio exceeded 0.20 during telemetry sampling.` This limits timing interpretation but did not cause request, persistence, or report failure.
- A post-run listener check found no Ollama process. The validation did not start or stop Ollama, so this is recorded as an external post-run availability state rather than an LLMeter failure.

## Regression and quality gates

| Boundary | Result |
|---|---:|
| `performance_cli_tests` | `7 passed` |
| `mock_provider_e2e` | `21 passed` |
| `report_cli_e2e` | `1 passed` |
| `pty_menu_e2e` | `10 passed` |
| Serialized all-target/all-feature suite | `153 passed, 0 failed` |
| `cargo fmt --all -- --check` | `PASS` |
| Locked all-target/all-feature check | `PASS` |
| Warning-denied Clippy | `PASS` |
| Warning-denied rustdoc | `PASS` |
| Fresh all-feature release build | `PASS` |
| Release binary `--version` / `--help` | `PASS` |

No runtime defect or source-level fix was required.

## Hosted CI

Hosted [CI run `36594571542`](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36594571542) passed all four platform jobs for pushed commit `bf9029e`: Ubuntu x86-64, Windows x86-64, macOS Intel x86-64, and macOS Apple silicon aarch64. The matrix passed formatting, locked all-target checks, warning-denied Clippy, serialized tests, and documentation checks. Runner migration and capacity annotations were informational only.

## Final status boundary

This closes the full live `sweep` at the exact Ollama `0.34.0` / `qwen3.5:2b` / Windows host boundary. `benchmark.performance` and Tier 3 remain `PARTIAL` because broader live provider/model/host variance, repeated statistical sampling, and production-sized interpretation remain unvalidated. Tier 5 remains `PARTIAL` because resumable in-progress state, production-scale ceilings, and native non-Windows terminal behavior remain open. Best-effort provider live certification, crates.io publication/install, and external evaluator execution remain separate owner or scope boundaries.
