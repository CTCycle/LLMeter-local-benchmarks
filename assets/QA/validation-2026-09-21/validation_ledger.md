# LLMeter validation ledger — 2026-09-21

Last updated: 2026-09-21

## Campaign metadata

| Field | Value |
|---|---|
| Repository | `CTCycle/LLMeter-local-benchmarks` |
| Branch | `develop` |
| Revision | `9bbaacb09823383edf0b0ea4ba9fb8ee5f5ba2cc` |
| Package | `llmeter 0.4.0` |
| Environment | Windows x86-64, PowerShell, stable Rust toolchain |
| Provider/model | No live provider required for Tier 0; mock-provider E2E used for release-binary coverage |
| Campaign boundary | Tier 0: environment, evidence, and startup |
| Canonical summary | [`project_status_ledger.md`](../../docs/project_status_ledger.md) |
| Campaign plan | [`validation_campaign.md`](../../docs/runtime/validation_campaign.md) |

## Slice ledger

| Slice | Capability | Exists | Exercised | Status | Scenarios executed | Evidence | Remaining gap | Last validated | Commit/revision |
|---|---|---:|---:|---|---|---|---|---|---|
| `T0-01` | Repository, CI, release, QA, and documentation reconciliation | YES | YES | `PASS` | Compared local revision and remote refs; queried current develop CI, v0.4.0 release workflow, and public release assets; reconciled stale release/QA wording. | [Tier 0 evidence](tier0-evidence.md), [CI run](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35526614502), [release run](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/34574075684), [public release](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.4.0) | crates.io publication/install remains unverified. | 2026-09-21 | `9bbaacb09823383edf0b0ea4ba9fb8ee5f5ba2cc` |
| `T0-02` | Current Rust quality baseline | YES | YES | `PASS` | `cargo fmt`; locked all-target/all-feature check; warning-denied Clippy; serialized all-target/all-feature tests; warning-denied rustdoc; release build. | [Tier 0 evidence](tier0-evidence.md) | Hosted CI is corroborating evidence, not a substitute for this local run. | 2026-09-21 | `9bbaacb09823383edf0b0ea4ba9fb8ee5f5ba2cc` |
| `T0-03` | Release-binary startup, help, non-TTY behavior, and exit codes | YES | YES | `PASS` | Release `--version`/`--help`; piped no-command and `menu`; invalid timeout; release-binary mock-provider E2E. | [Tier 0 evidence](tier0-evidence.md), [mock-provider test](../../../tests/mock_provider_e2e.rs) | Provider-success/live benchmark behavior is outside this slice. | 2026-09-21 | `9bbaacb09823383edf0b0ea4ba9fb8ee5f5ba2cc` |
| `T0-04` | Configuration precedence and persisted provider state | YES | YES | `PASS` at exercised local boundary | Default provider; persisted `lmstudio`; environment `llama.cpp`; CLI `openai-compatible`; base URL `/v1` normalization; embedded-credential rejection. | [Tier 0 evidence](tier0-evidence.md), [configuration tests](../../../src/config.rs) | Malformed persisted JSON is covered by the full regression suite rather than a new manual fixture. | 2026-09-21 | `9bbaacb09823383edf0b0ea4ba9fb8ee5f5ba2cc` |
| `T0-05` | PowerShell launcher contract | YES | YES | `PARTIAL` | Release-binary reuse; `--version`; forwarded provider/base URL/status; noninteractive `Clean -WhatIf` refusal without file mutation. | [Tier 0 evidence](tier0-evidence.md), [`run_llmeter.ps1`](../../../run_llmeter.ps1) | Fallback-target selection and interactive protected-path mutation check were not completed; destructive confirmation was intentionally not accepted. | 2026-09-21 | `9bbaacb09823383edf0b0ea4ba9fb8ee5f5ba2cc` |

## Tier result

`PARTIAL`: the quality, startup, configuration, and release-binary boundaries passed. The only open Tier 0 item is the named launcher boundary in `T0-05`; no functional failure was observed.

## Next slices

Execute `T1-01` through `T1-07` in order. Preserve fixture/live/hosted distinctions, retain stdout and file evidence, and stop before performance sweeps if `T3-02` JSONL request accounting is reproduced.
