# T1-02 Windows interactive menu evidence

Last updated: 2026-09-23

## Run metadata

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | `develop` |
| Tested revision | `46be1d2879fd5da6bd4b4f2f4e4615bd2e6c4f1f` |
| Source/test baseline | `e21dd0ec271a9a1e009bf257771715d948c6cf92` (source and tests unchanged at the tested revision) |
| Package | `llmeter 0.4.0` |
| Environment | Windows x64, PowerShell 7.6.6, Rust 1.98.0, Cargo 1.98.0 |
| Provider/model | No live provider or model. The isolated Ollama status used the intentionally unavailable loopback URL `http://127.0.0.1:9/v1`. |
| Disposable home | `LLMETER_HOME=assets/QA/validation-2026-09-23/.t1-02-manual-home-20260923-a`; the application did not create it. |

## Automated ConPTY scenarios

Command:

```powershell
cargo test --locked --test pty_menu_e2e -- --test-threads=1
```

Result: **PASS — 9 passed, 0 failed** on the tested revision. The cases covered Ctrl+C and EOF exit codes, nested selector cancellation, submenu Back, pause cancellation/interruption, numeric prompt cancellation, and interruption during the performance confirmation prompt.

## Manual navigation record

Launched the built CLI in a Windows ConPTY with `--timeout 0.1`, an isolated `LLMETER_HOME`, and the loopback URL above. The initial provider status showed Ollama, API unreachable, and zero exposed models; the main menu remained available. No provider operation, benchmark, report generation, or configuration change was selected.

Observed route:

1. Main menu → **Provider setup**. The status, preset list, capability probe, provider selection, and Back entries rendered. Pressed Escape; returned to the main menu.
2. Main menu → **Model inventory**. The exposed-model list, refresh, raw metadata, cache estimate, and Back entries rendered. Pressed Escape; returned to the main menu.
3. Main menu → **Benchmark workspace**. Quick, performance, standard LLM, embeddings, quality plan, catalog, report, and Back entries rendered. Pressed Escape; returned to the main menu.
4. Main menu → **Reports and comparisons**. List, view, generate, and Back entries rendered. Pressed Escape; returned to the main menu.
5. Main menu → **Help and examples**. Help text and the Enter-to-continue prompt rendered. Pressed Enter; returned to the main menu.
6. Selected **Exit**. The process exited with code `0`.

## Result boundary

T1-02 is **PASS** for the Windows ConPTY boundary: automated interruption/navigation cases passed and the manual workspace route completed cleanly. The provider-unavailable status was an intentional local precondition, not a live-provider test. Non-Windows terminal behavior remains unvalidated.
