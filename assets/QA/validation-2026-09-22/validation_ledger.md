# LLMeter validation ledger — 2026-09-22

Last updated: 2026-09-22

## Campaign metadata

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | develop |
| Revision | e21dd0ec271a9a1e009bf257771715d948c6cf92 |
| Package | llmeter 0.4.0 |
| Environment | Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Provider/model | No live provider; mock-provider E2E and a deliberately unavailable loopback endpoint |
| Campaign boundary | Tier 0: environment, evidence, and startup |
| Canonical summary | [Project status ledger](../../docs/project_status_ledger.md) |
| Campaign plan | [Validation campaign](../../docs/runtime/validation_campaign.md) |

## Slice ledger

| Slice | Capability | Exists | Exercised | Status | Scenarios executed | Evidence | Remaining gap | Last validated | Commit/revision |
|---|---|---:|---:|---|---|---|---|---|---|
| T0-01 | Repository, CI, release, QA, and documentation reconciliation | YES | YES | PASS | Confirmed clean starting checkout on develop, local HEAD and origin/develop at the same SHA, and completed CI for that exact SHA. Reconciled the existing v0.4.0 release evidence as a separate tagged-release boundary. | [Tier 0 evidence](tier0-evidence.md), [current CI run](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35738024756), [v0.4.0 release report](../release-0.4.0/release-report.md) | First crates.io publication/install remains unverified and tracked separately. | 2026-09-22 | e21dd0ec271a9a1e009bf257771715d948c6cf92 |
| T0-02 | Current Rust quality baseline | YES | YES | PASS | Formatting, locked all-target/all-feature check, warning-denied Clippy, serialized all-target/all-feature tests, warning-denied rustdoc, and release build. The full suite passed 137 tests. | [Tier 0 evidence](tier0-evidence.md), [current CI run](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35738024756) | Local Windows evidence remains distinct from hosted platform evidence. | 2026-09-22 | e21dd0ec271a9a1e009bf257771715d948c6cf92 |
| T0-03 | Release-binary startup, help, non-TTY behavior, and exit codes | YES | YES | PASS | Release binary version/help; no-command and explicit menu without a terminal; invalid timeout; 11 release-binary mock-provider E2E cases. Native process exit codes were captured directly. | [Tier 0 evidence](tier0-evidence.md), [mock-provider test](../../../tests/mock_provider_e2e.rs) | Provider-success/live benchmark behavior is outside this slice. | 2026-09-22 | e21dd0ec271a9a1e009bf257771715d948c6cf92 |
| T0-04 | Configuration precedence and persisted provider state | YES | YES | PASS at exercised local boundary | In an isolated temporary LLMeter home, exercised default Ollama, persisted lmstudio, environment llama-cpp, CLI openai-compatible, base URL normalization to /v1, and rejection of embedded credentials without echoing the test sentinel. | [Tier 0 evidence](tier0-evidence.md), [configuration tests](../../../src/config.rs) | No live provider was required; malformed and unknown persisted configuration remain covered by regression tests. | 2026-09-22 | e21dd0ec271a9a1e009bf257771715d948c6cf92 |
| T0-05 | PowerShell launcher contract | YES | YES | PASS | Five Windows E2E cases covered release-binary reuse and forwarding, noninteractive refusal without mutation, fallback-target selection after a forced default-build failure, confirmed cleanup of owned paths in a disposable home, and protected-home refusal after confirmation. | [Tier 0 evidence](tier0-evidence.md), [launcher E2E tests](../../../tests/launcher_powershell_e2e.rs) | The evidence is specific to the Windows PowerShell and ConPTY boundary. | 2026-09-22 | e21dd0ec271a9a1e009bf257771715d948c6cf92 |

## Tier result

PASS at e21dd0ec271a9a1e009bf257771715d948c6cf92. All five Tier 0 slices passed at their stated boundaries. Local Windows checks, hosted CI, and the tagged v0.4.0 release evidence remain distinct evidence lanes.

## Next slices

Proceed with T1-01 through T1-07 in order. Preserve fixture/live/hosted distinctions, retain stdout and file evidence, and stop before performance sweeps if T3-02 JSONL request accounting is reproduced.
