# Tier 0 validation evidence — 2026-09-22

Last updated: 2026-09-22

Evidence was collected from revision e21dd0ec271a9a1e009bf257771715d948c6cf92 on Windows x86-64 with PowerShell 7.6.6, Cargo 1.98.0, and Rust 1.98.0. No live provider was used. The accepted configuration probes used a disposable LLMeter home under assets/QA and an intentionally unavailable loopback endpoint; the temporary home was removed after the probes. A discarded earlier shell attempt hit PowerShell's read-only HOME variable and is excluded from the evidence.

## T0-01 — repository and external evidence reconciliation

- The starting checkout was clean on develop. Local HEAD and origin/develop both resolved to e21dd0ec271a9a1e009bf257771715d948c6cf92.
- GitHub Actions CI run 35738024756 completed successfully for that exact SHA. Windows x86-64, Ubuntu x86-64, macOS Intel, and macOS ARM64 jobs all passed.
- The existing v0.4.0 GitHub release evidence remains tied to release commit 5d5e41c and release workflow 34574075684. That tagged-release boundary is separate from current-develop CI.
- QA and campaign links now resolve to the 2026-09-22 current-revision record. The first crates.io publication/install remains open and is not represented as verified.

## T0-02 — quality baseline

| Command | Result |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo check --locked --all-targets --all-features | PASS |
| cargo clippy --locked --all-targets --all-features -- -D warnings | PASS |
| cargo test --locked --all-targets --all-features -- --test-threads=1 | PASS — 137 passed, 0 failed |
| PowerShell set RUSTDOCFLAGS to -D warnings, then ran cargo doc --locked --no-deps --all-features | PASS |
| cargo build --locked --release --all-features | PASS |

The serialized suite comprised 59 library tests, 6 CLI contract tests, 5 launcher E2E tests, 11 mock-provider tests, 6 performance CLI tests, 8 performance metrics tests, 2 provider probe tests, 9 PTY tests, 1 quality CLI test, 1 resource snapshot test, 3 result-schema tests, 4 metric tests, 9 registry tests, 7 reporting tests, and 6 result-store tests.

## T0-03 — release binary and CLI contract

The release binary returned exit 0 for version and help, exit 2 with usage output for a no-command invocation and explicit menu without a terminal, and exit 2 with a validation error for an invalid zero timeout. These native process exit codes were captured with redirected .NET ProcessStartInfo handles rather than inferred from the shell wrapper. The release-binary mock-provider E2E suite passed 11 of 11 cases.

## T0-04 — configuration and persisted provider state

The accepted probes used an isolated temporary LLMeter home and a short timeout against 127.0.0.1:9, avoiding reliance on any provider service.

| Probe | Observed result |
|---|---|
| Default provider with unavailable loopback endpoint | Exit 1; status identified Ollama and reported the controlled connection failure. |
| Persist default provider as lmstudio, then restart | Set exited 0; config contained only the provider value; status identified LM Studio. |
| LLMETER_PROVIDER=llama-cpp | Environment selection overrode the persisted provider. |
| CLI --provider openai-compatible | CLI selection overrode environment and persisted values. |
| Base URL without /v1 | Output showed normalized http://127.0.0.1:9/v1. |
| Base URL with a test-only embedded credential | Exit 2; rejected, and the sentinel credential was absent from output. |

Malformed and unknown persisted configuration, numeric bounds, and provider validation also passed in the full serialized regression suite. The temporary home was removed after the probes.

## T0-05 — PowerShell launcher

The focused command cargo test --locked --test launcher_powershell_e2e -- --test-threads=1 passed all five tests:

- Existing release-binary reuse and argument forwarding passed.
- Noninteractive Clean -WhatIf refused before mutation; all fixture sentinels remained.
- A deterministic cargo shim forced the default build to fail; the fallback target built and was selected.
- ConPTY affirmative confirmation removed only launcher-owned paths under a disposable temporary home; keep markers, the script, and the fixture manifest remained.
- ConPTY affirmative confirmation against a protected temporary home was refused; all sentinels and fixture markers remained.

The full serialized suite also passed the launcher tests. No real user home or repository cleanup target was used.

## Overall boundary

Tier 0 is PASS at the stated Windows-local and hosted-CI boundaries on the recorded SHA. This does not certify a live provider, non-Windows PowerShell behavior, untested provider/model combinations, or crates.io installation.
