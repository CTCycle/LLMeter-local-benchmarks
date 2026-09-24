# LLMeter validation ledger — 2026-09-24

Last updated: 2026-09-24

## Campaign metadata

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | `develop` |
| Tested source revision | `c7438db98812b70c20738ddcd4821d27dab837a3` |
| Package | `llmeter 0.4.0` |
| Environment | Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Live provider | Ollama at `http://localhost:11434/v1`; fresh status and model listing passed and returned 8 models. |
| Live models exercised | `qwen3.5:2b`, `nomic-embed-text:latest` |
| Local quality gates | Formatting, locked all-target/all-feature check, warning-denied Clippy, serialized all-target/all-feature tests, warning-denied rustdoc, and `cargo build --locked --bin llmeter` passed. |
| Hosted CI | [Run 35968716973](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35968716973) passed all four platform jobs on revision `28717289c2b0044ae337e5f10419aa62628fa492`, which contains the validated source revision `c7438db98812b70c20738ddcd4821d27dab837a3`: Ubuntu x86-64, Windows x86-64, macOS Intel x86-64, and macOS Apple silicon aarch64. |
| Prior tier evidence | [Tier 0 ledger](../validation-2026-09-22/validation_ledger.md); [Tier 1 ledger](../validation-2026-09-23/validation_ledger.md). |

## Current campaign boundary

| Tier / slice | Status | Evidence boundary and remaining work |
|---|---|---|
| Tier 0 — environment, startup, and launcher | `PASS` | See the [2026-09-22 ledger](../validation-2026-09-22/validation_ledger.md). |
| Tier 1 — application foundations | `PASS` | T1-01 through T1-07 pass at the boundaries recorded in the [2026-09-23 ledger](../validation-2026-09-23/validation_ledger.md). |
| Tier 2 — standard benchmarks | `PASS` | The real CLI passed all standard families on the current Ollama endpoint: six LLM benchmark IDs, embeddings, and a two-model/two-benchmark run retaining controlled unsupported-capability records. See [Tier 2 evidence](t2-standard-workflows-evidence.md). Other providers and unselected model combinations are not certified. |
| T3-02 — JSONL workload accounting | `PASS` | A three-prompt fixture matched plan count, request budget, progress, HTTP workload requests, and persisted records. A one-request live smoke passed after the fix. See [T3-02 evidence](t3-02-jsonl-accounting-evidence.md). |
| Tier 3 — performance subsystem | `PARTIAL` | The accounting stop gate and one small live smoke pass. Full profile progression, concurrent scenarios, telemetry/resource probes, and production-sized workload interpretation remain open. |
| Tier 4 — provider, quality, and distribution | `PARTIAL` | Ollama standard flows pass. The other live provider presets and cross-provider optional capabilities remain unvalidated. External quality execution remains at the dry-run boundary; crates.io publication/install remains owner-gated. |
| Tier 5 — resilience and edge cases | `UNRUN` | Broader transport/filesystem/restart/interruption/scale and native non-Windows terminal scenarios remain open. Earlier focused failure-path evidence does not close this tier. |

## Outstanding incomplete gates

| Gate | Status | Remaining boundary |
|---|---|---|
| `provider.presets` | `PARTIAL` | The registered-preset fixture contract passed, and live Ollama is covered. Representative TGI, text-generation-webui, Jan, and MLX-LM server/model runs remain absent. |
| `provider.best-effort.live` | `UNVALIDATED` | No live status, model discovery, capability probe, or benchmark evidence exists for those best-effort targets. |
| `provider.optional-capabilities` | Validation debt | Responses, structured output, tool calling, streaming, and embeddings have Ollama/mock evidence only; cross-provider evidence is still needed. |
| `benchmark.performance` | `PARTIAL` | The one-request smoke is a request-path check, not a production-sized profile or statistical comparison. |
| `cli.interactive.non-windows` | Validation debt | Native terminal and interruption behavior remains unverified outside Windows. |
| `release.public-distribution` / `ISSUE-001` | `PARTIAL` / open | GitHub `v0.4.0` is verified; crates.io publication and clean installation remain owner-gated. |
| `quality.external-plans` / `ISSUE-003` | `PARTIAL` / open | Catalog and dry-run planning pass; a live external framework runner is outside the approved current boundary. |
| Tier 5 | `UNRUN` | The broader resilience and edge-case campaign has not been executed. |

## Detailed evidence

- [Tier 2 live standard benchmark evidence](t2-standard-workflows-evidence.md)
- [T3-02 JSONL accounting and bounded live smoke evidence](t3-02-jsonl-accounting-evidence.md)
- [Canonical project status ledger](../../docs/project_status_ledger.md)
- [Validation campaign](../../docs/runtime/validation_campaign.md)
