# LLMeter validation ledger — 2026-09-26

Last updated: 2026-09-26

## Campaign metadata

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | `develop` |
| Tested source revision | `e635459a402414ee880bde42be0efaff0a245acf` |
| Package | `llmeter 0.4.0` |
| Environment | Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Selected slice | T3-04 repeated live latency/throughput profiles plus T5-01 bounded resilience subset |
| Fixture boundary | Real CLI against the deterministic OpenAI-compatible mock provider; fixture results remain separate from live timing and provider certification. |
| Live provider recheck | Ollama fresh status returned API reachable `yes`, 8 exposed models, and exit code `0`; repeated runs used exactly `qwen3.5:2b`. |

## Current campaign boundary

| Tier / slice | Status | Evidence boundary and remaining work |
|---|---|---|
| Tier 0 — environment, startup, and launcher | `PASS` | See the [2026-09-22 ledger](../validation-2026-09-22/validation_ledger.md). |
| Tier 1 — application foundations | `PASS` | T1-01 through T1-07 pass at the boundaries recorded in the [2026-09-23 ledger](../validation-2026-09-23/validation_ledger.md). |
| Tier 2 — standard benchmarks | `PASS` | Real-CLI Ollama evidence covers the standard families at the recorded 2026-09-24 boundary; other providers and unselected model combinations remain outside the claim. |
| T3-02 — JSONL workload accounting | `PASS` | The three-prompt accounting regression and bounded live smoke remain passing at the recorded 2026-09-24 boundary. |
| T3-03 — deterministic profile defaults and execution | `PASS` (fixture boundary) | All four profile-owned default matrices executed through the real CLI fixture: 36 scenario records and 154 total fixture chat requests. See [T3-03 default-matrix evidence](t3-default-matrix-evidence.md). |
| T3-03 — bounded live profile progression | `PASS` (one Ollama model) | The 2026-09-24 `qwen3.5:2b` runs passed at small prompt/output sizes and two measured requests per concurrency level. The 2026-09-26 recheck later became available and enabled the repeated T3-04 slice. |
| T3-04 — repeated live latency/throughput | `PASS` (one Ollama model, custom bounded matrix) | `latency` and `throughput` each ran 4 scenarios with one warmup and three measured requests per scenario at prompt sizes 32/64, output 16, and concurrency 1/2. All 24 measured requests returned HTTP 200 with 0 errors; standard telemetry captured host swap-pressure warnings. Full default-size and broader statistical/provider/host coverage remains open. See [T3-04 evidence](t3-t5-followup-evidence.md). |
| Tier 3 — performance subsystem | `PARTIAL` | Deterministic default execution, bounded historical progression, and the new repeated live subset pass. Full default-size live profiles, higher concurrency, statistically useful repetitions, broader provider/model/host coverage, and production-sized interpretation remain open. |
| Tier 4 — provider, quality, and distribution | `PARTIAL` | Ollama and fixture evidence pass at recorded boundaries; best-effort live providers, external quality execution, and crates.io publication/install remain incomplete or owner-gated. |
| Tier 5 — resilience and edge cases | `PARTIAL` | T5-01 passed bounded provider/CLI failures, atomic output cleanup, performance safety ceilings, invalid output-path handling, and a Windows ConPTY confirmation interruption. Mid-run interruption, restart/state restoration, broader scale, and native non-Windows terminal behavior remain open. See [T5-01 evidence](t3-t5-followup-evidence.md). |

## Outstanding incomplete gates

| Gate | Status | Remaining boundary |
|---|---|---|
| `provider.presets` | `PARTIAL` | Live evidence covers Ollama only; representative TGI, text-generation-webui, Jan, and MLX-LM runs remain absent. |
| `provider.best-effort.live` | `UNVALIDATED` | No live status, model discovery, capability probe, or benchmark evidence exists for the best-effort targets. |
| `provider.optional-capabilities` | Validation debt | Cross-provider and model-variation evidence remains needed beyond the recorded Ollama/mock capabilities. |
| `benchmark.performance` | `PARTIAL` | Default matrices pass through the fixture, and the repeated live latency/throughput subset passes for `qwen3.5:2b`. Full default-size live progression, higher concurrency, and statistically useful repeated samples remain open; host swap-pressure warnings limit timing interpretation. |
| `cli.interactive.non-windows` | Validation debt | Native terminal and interruption behavior remains unverified outside Windows. |
| `release.public-distribution` / `ISSUE-001` | `PARTIAL` / open | GitHub `v0.4.0` is verified; crates.io publication and clean installation remain owner-gated. |
| `quality.external-plans` / `ISSUE-003` | `PARTIAL` / open | Catalog and dry-run planning pass; a live external framework runner is outside the approved current boundary. |
| Tier 5 | `PARTIAL` | The T5-01 bounded failure/atomic-output/safety/interruption subset passed. The broader restart, mid-run interruption, scale, and non-Windows campaign remains open. |

## Local quality-gate result

The complete Windows local gate set passed on source revision `e635459a402414ee880bde42be0efaff0a245acf`: formatting, locked all-target/all-feature check, warning-denied Clippy, serialized all-target/all-feature tests (146 passed), warning-denied rustdoc, and the locked `llmeter` binary build. Detailed T3-04/T5-01 evidence is in [t3-t5-followup-evidence.md](t3-t5-followup-evidence.md).

## Detailed evidence

- [T3-03 default performance matrix evidence](t3-default-matrix-evidence.md)
- [T3-04 repeated live profiles and T5-01 resilience evidence](t3-t5-followup-evidence.md)
- [Prior T3-03 profile progression evidence](../validation-2026-09-24/t3-profile-progression-evidence.md)
- [Prior T3-02 JSONL accounting evidence](../validation-2026-09-24/t3-02-jsonl-accounting-evidence.md)
- [Canonical project status ledger](../../docs/project_status_ledger.md)
- [Validation campaign](../../docs/runtime/validation_campaign.md)
