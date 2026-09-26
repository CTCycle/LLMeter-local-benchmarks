# LLMeter validation ledger — 2026-09-26

Last updated: 2026-09-26

## Campaign metadata

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | `develop` |
| Tested source revision | `2af310a5b9e395485dfc8a578dd19afc57fe7406` |
| Package | `llmeter 0.4.0` |
| Environment | Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Selected slice | T3-03 deterministic default performance matrices |
| Fixture boundary | Real CLI against the deterministic OpenAI-compatible mock provider; no live timing or provider certification claimed. |
| Live provider recheck | Ollama status was unavailable on 2026-09-26: API reachable `no`, models exposed `0`, exit code `1`. |

## Current campaign boundary

| Tier / slice | Status | Evidence boundary and remaining work |
|---|---|---|
| Tier 0 — environment, startup, and launcher | `PASS` | See the [2026-09-22 ledger](../validation-2026-09-22/validation_ledger.md). |
| Tier 1 — application foundations | `PASS` | T1-01 through T1-07 pass at the boundaries recorded in the [2026-09-23 ledger](../validation-2026-09-23/validation_ledger.md). |
| Tier 2 — standard benchmarks | `PASS` | Real-CLI Ollama evidence covers the standard families at the recorded 2026-09-24 boundary; other providers and unselected model combinations remain outside the claim. |
| T3-02 — JSONL workload accounting | `PASS` | The three-prompt accounting regression and bounded live smoke remain passing at the recorded 2026-09-24 boundary. |
| T3-03 — deterministic profile defaults and execution | `PASS` (fixture boundary) | All four profile-owned default matrices executed through the real CLI fixture: 36 scenario records and 154 total fixture chat requests. See [T3-03 default-matrix evidence](t3-default-matrix-evidence.md). |
| T3-03 — bounded live profile progression | `PASS` (one Ollama model, historical boundary) | The 2026-09-24 `qwen3.5:2b` runs passed at small prompt/output sizes and two measured requests per concurrency level. The provider was unavailable for recheck on 2026-09-26. |
| Tier 3 — performance subsystem | `PARTIAL` | Deterministic default execution is now covered. Full default-size live profiles, statistically useful repetitions, broader provider/model/host coverage, and production-sized interpretation remain open. |
| Tier 4 — provider, quality, and distribution | `PARTIAL` | Ollama and fixture evidence pass at recorded boundaries; best-effort live providers, external quality execution, and crates.io publication/install remain incomplete or owner-gated. |
| Tier 5 — resilience and edge cases | `UNRUN` | The dedicated broader transport/filesystem/restart/interruption/scale and native non-Windows campaign remains open. |

## Outstanding incomplete gates

| Gate | Status | Remaining boundary |
|---|---|---|
| `provider.presets` | `PARTIAL` | Live evidence covers Ollama only; representative TGI, text-generation-webui, Jan, and MLX-LM runs remain absent. |
| `provider.best-effort.live` | `UNVALIDATED` | No live status, model discovery, capability probe, or benchmark evidence exists for the best-effort targets. |
| `provider.optional-capabilities` | Validation debt | Cross-provider and model-variation evidence remains needed beyond the recorded Ollama/mock capabilities. |
| `benchmark.performance` | `PARTIAL` | Default matrices now pass through the fixture. Full default-size live progression and statistically useful repeated samples remain open; the current provider-unavailable recheck is recorded in the evidence. |
| `cli.interactive.non-windows` | Validation debt | Native terminal and interruption behavior remains unverified outside Windows. |
| `release.public-distribution` / `ISSUE-001` | `PARTIAL` / open | GitHub `v0.4.0` is verified; crates.io publication and clean installation remain owner-gated. |
| `quality.external-plans` / `ISSUE-003` | `PARTIAL` / open | Catalog and dry-run planning pass; a live external framework runner is outside the approved current boundary. |
| Tier 5 | `UNRUN` | The broader resilience and edge-case campaign has not been executed. |

## Local quality-gate result

The complete Windows local gate set passed on source revision `2af310a5b9e395485dfc8a578dd19afc57fe7406`: formatting, locked all-target/all-feature check, warning-denied Clippy, serialized all-target/all-feature tests (147 passed), warning-denied rustdoc, and the locked `llmeter` binary build. Detailed T3-03 evidence is in [t3-default-matrix-evidence.md](t3-default-matrix-evidence.md).

## Detailed evidence

- [T3-03 default performance matrix evidence](t3-default-matrix-evidence.md)
- [Prior T3-03 profile progression evidence](../validation-2026-09-24/t3-profile-progression-evidence.md)
- [Prior T3-02 JSONL accounting evidence](../validation-2026-09-24/t3-02-jsonl-accounting-evidence.md)
- [Canonical project status ledger](../../docs/project_status_ledger.md)
- [Validation campaign](../../docs/runtime/validation_campaign.md)
