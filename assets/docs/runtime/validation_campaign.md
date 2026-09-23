# LLMeter validation campaign

Last updated: 2026-09-23

## Purpose and authority

This document is the durable, compact version of the comprehensive validation roadmap. It turns the ordered campaign into a repeatable operating plan without copying every scenario into the current-state ledger.

- [`project_status_ledger.md`](../project_status_ledger.md) is the canonical current-status summary.
- Dated detailed run ledgers and sanitized evidence live under [`assets/QA/`](../../QA/).
- This campaign records what must be exercised, which evidence boundary is acceptable, and what remains unvalidated.

The campaign is for the local, single-user Rust CLI. Provider servers and models are external dependencies; fixture, live, hosted-CI, and public-release evidence are separate gates.

## Status and evidence rules

Use the ledger taxonomy for current components. Within a campaign slice, use these operational labels:

| Label | Meaning |
|---|---|
| `PASS` | The slice was executed at its stated boundary and the expected contract held. |
| `PARTIAL` | Some scenarios passed, but a named boundary, scenario, or evidence class remains open. |
| `FAIL` | The expected contract did not hold and needs remediation before the tier can pass. |
| `BLOCKED` | An external provider, credential, platform, or owner decision prevents execution. |
| `UNRUN` | The slice is planned but has not been executed in the current campaign. |
| `UNVALIDATED` | Implementation evidence exists, but it is insufficient for the claimed boundary. |

Never promote a fixture result to live-provider certification, local execution to hosted evidence, or a dry-run quality plan to external quality-framework execution. Every detailed run records the exact revision, environment, provider/model where applicable, scenarios actually executed, evidence, and remaining gap.

## Operating loop

Each slice follows the same bounded loop:

```text
Inspect -> execute narrow scenario -> capture sanitized evidence
       -> reproduce/trace any failure -> add focused regression coverage
       -> apply the smallest fix -> rerun the failure and adjacent regression set
       -> update the canonical ledger -> proceed
```

Do not run sweep-scale performance workloads before the JSONL accounting slice is proven. Do not batch unrelated fixes, expose secrets in artifacts, or infer a provider/platform result from a different evidence lane.

## Ordered tiers

### Tier 0 — environment, evidence, and startup

This is the first campaign tier. It makes later results trustworthy.

| Slice | Gate | Current campaign state | Evidence / next action |
|---|---|---|---|
| `T0-01` | Reconcile revision, CI, release, QA references, and documentation truth. | `PASS` | Current SHA, remote branch, exact-commit four-target CI, tagged-release boundary, and QA links are recorded in the [2026-09-22 ledger](../../QA/validation-2026-09-22/validation_ledger.md). |
| `T0-02` | Formatting, locked check, Clippy, serialized all-target/all-feature tests, and rustdoc. | `PASS` | Windows local run passed 137 tests; command results are in [Tier 0 evidence](../../QA/validation-2026-09-22/tier0-evidence.md). |
| `T0-03` | Release-binary version/help/startup, non-TTY behavior, exit codes, and invalid configuration. | `PASS` | Release-binary probes returned the expected `0`/`2`/`1` boundaries, and 11 mock-provider cases passed; see the [evidence record](../../QA/validation-2026-09-22/tier0-evidence.md). |
| `T0-04` | Configuration precedence, persisted provider state, URL normalization, and fail-closed validation. | `PASS` at exercised local boundary | Persisted `lmstudio`, environment `llama-cpp`, CLI `openai-compatible`, normalized `/v1`, and embedded-credential rejection were exercised in an isolated home; see the [evidence record](../../QA/validation-2026-09-22/tier0-evidence.md). |
| `T0-05` | PowerShell launcher reuse/build selection, forwarding, destructive-operation protection, and owned cleanup. | `PASS` | Five Windows E2E scenarios passed, including fallback selection and ConPTY confirmation/refusal checks in disposable fixtures; see the [evidence record](../../QA/validation-2026-09-22/tier0-evidence.md). |

Tier 0 is `PASS` on revision e21dd0ec271a9a1e009bf257771715d948c6cf92. The former launcher evidence gap was covered by deterministic fallback and isolated ConPTY scenarios; the complete current-revision gate is recorded in the [2026-09-22 ledger](../../QA/validation-2026-09-22/validation_ledger.md).

### Tier 1 — application foundations

The next tranche exercises the provider-independent CLI, Windows terminal workspaces, provider catalog freshness, HTTP/SSE/authentication limits, persistence/privacy, reports, and local lifecycle semantics.

| Slice | Focus | Required boundary | Current campaign state |
|---|---|---|---|
| `T1-01` | Provider, benchmark, quality, and built-in help catalogs. | Real CLI stdout snapshots, including invalid topics. | `PASS` on tested tree revision `0568177`; see [2026-09-23 stdout evidence](../../QA/validation-2026-09-23/tier1-evidence.md). |
| `T1-02` | Main menu, nested workspaces, Back/Escape, EOF, Ctrl+C, and confirmation interruption. | Windows ConPTY plus one retained manual navigation record. | `PASS` on revision `46be1d2`; see [2026-09-23 T1-02 evidence](../../QA/validation-2026-09-23/t1-02-interactive-evidence.md). |
| `T1-03` | `/v1/models`, cached navigation, fresh operational validation, and missing models. | Mock request sequence and output/error evidence. | `PASS` on revision `eeb10fc`; see [2026-09-23 model catalog evidence](../../QA/validation-2026-09-23/t1-03-model-catalog-evidence.md). |
| `T1-04` | HTTP status, redirects, reserved fields, SSE assembly, response limits, auth, and URL safety. | Official Windows launcher, deterministic mock capture, unit-boundary reserved-field check, and persisted-output secret inspection. | `PASS` on revision `5e900ed`; Windows launcher evidence and the separate four-platform hosted CI run are recorded in the [T1-04 evidence note](../../QA/validation-2026-09-23/t1-04-provider-transport-evidence.md). |
| `T1-05` | Schema `3.0`, JSON/CSV, IDs, redaction, previews, formulas, and atomic replacement. | Generated files and failure-path directory inspection. | `PASS` on revision `0f2ea40`; see the [T1-05 result persistence evidence note](../../QA/validation-2026-09-23/t1-05-result-persistence-evidence.md). |
| `T1-06` | Report list/show/generate roundtrip for standard/performance/error/adversarial data. | Markdown, HTML, and terminal output. | `PASS` on Windows revision `d8e9c94`; the real CLI covered both run kinds, controlled errors, escaping, privacy, and listing the generated reports. See [T1-06 evidence](../../QA/validation-2026-09-23/t1-06-report-cli-evidence.md). |
| `T1-07` | Isolated install/update/uninstall/purge and rollback behavior. | Complete before/after tree in a dedicated temporary home. | `PASS` on Windows revision `d8e9c94`; real-CLI install, refusal, rollback, successful update, and purge passed. The failed-update helper cleanup defect was fixed and regression-tested. See [T1-07 evidence](../../QA/validation-2026-09-23/t1-07-lifecycle-evidence.md). |

The current automated suite already supplies meaningful evidence for much of Tier 1, but a tier claim requires the dedicated scenario/evidence boundary above. T1-01 through T1-07 now pass at their recorded boundaries. Tier 1 is complete. Continue with Tier 2's real-CLI standard benchmark workflows; keep T3-02 JSONL accounting as the stop gate before performance workloads.

### Tier 2 — core benchmark workflows

Exercise every built-in standard path through the real CLI: streaming chat generation, `/v1/responses`, consistency and prompt sizes, structured output and tool calling, embeddings, and multi-model/multi-benchmark orchestration. Positive behavior and controlled unsupported/error records must both be retained where applicable. Record progress, request order, persisted records, and reports.

Current campaign state: `UNRUN` at the dedicated Tier 2 boundary. Existing mock-provider and limited Ollama runs do not cover every standard path and orchestration case. Begin with a bounded real-CLI subset using the currently exposed Ollama models, then complete the remaining workflows in manageable groups.

### Tier 3 — performance subsystem

Validate profile planning and safety guards before measured runs. The critical stop gate is `T3-02`: compare actual JSONL prompt count with scenario count, request-budget validation, progress total, HTTP request count, and persisted records. If the suspected synthetic-count mismatch reproduces, stop the campaign and fix it before smoke, latency, throughput, or sweep workloads. Then validate statistics, bounded concurrency, capability/model inventory, telemetry/resource probes, and full profile progression.

Current campaign state: `PARTIAL`; planning, safety, metrics, probes, and one smoke scenario have prior evidence, but the dedicated profile progression is incomplete. `T3-02` remains `UNRUN` and is the stop gate before any broader workload.

### Tier 4 — providers, quality planning, and distribution

Run one independent slice per available live provider/model. Preserve exact provider and model identity; mark unavailable targets `BLOCKED` rather than inheriting fixture evidence. Keep quality-framework integrations at their current dry-run planning boundary unless execution is explicitly approved. Verify tagged release archives, checksums, provenance, public downloads, and—if approved—clean `cargo install` from crates.io as separate gates.

Current campaign state: `PARTIAL`. Ollama status and model discovery currently pass and earlier LLM, embeddings, and performance smokes are recorded. The best-effort provider matrix and cross-provider optional capabilities remain unvalidated; external quality execution and crates.io publication remain scope/owner gated.

### Tier 5 — resilience and edge cases

Exercise transport/protocol failures, filesystem and atomic-write failures, repeated operations and restart/state restoration, long-running interruption, scale/safety ceilings, and native non-Windows terminal behavior. The expected result is bounded, readable failure with no corrupted or convincing partial artifact.

Current campaign state: `UNRUN` at the dedicated Tier 5 boundary. Some focused transport, persistence, and lifecycle failure paths have coverage in earlier slices, but the broader resilience and native-terminal sweep remains open.

## Regression map

| Changed area | Minimum adjacent regression |
|---|---|
| CLI/startup/launcher | CLI contracts, one mock-provider E2E, PTY suite, launcher version/status/refusal. |
| Configuration | Configuration tests and invalid-config CLI contracts. |
| Provider/request construction | Provider unit/probe tests, mock E2E, and one provider smoke. |
| Benchmark/runner semantics | Registry, affected benchmark, mock run, saved-run/report E2E. |
| Performance planning/metrics | Performance CLI and metrics tests plus one smoke run. |
| JSONL workload handling | Focused workload/count tests and performance safety tests. |
| Persistence/reporting | Schema, result-store, reporting, and saved-run E2E. |
| Lifecycle | Isolated-home lifecycle tests. |
| CI/release workflow | Workflow syntax plus the relevant native/release gate. |

Run the full locked all-target/all-feature suite once at each tier boundary and after the final remediation; focused slices are preferred between boundaries.

## Comprehensive-validation gate

Do not call a revision comprehensively validated until Tier 0–3 are green except for intentional product limitations, the JSONL accounting risk is disproven or fixed/regression-tested, every standard benchmark has real-CLI evidence, at least one live provider/model path is retained, provider-specific availability is explicit, interruption/persistence are safe, Windows and current CI remain green, release documentation matches GitHub, stale QA references are repaired, and every `PARTIAL`, `BLOCKED`, `UNKNOWN`, or `UNRUN` entry has a named boundary.

The current campaign does not make that comprehensive claim. Tier 0 passed at its recorded boundary; Tier 1 is next.
