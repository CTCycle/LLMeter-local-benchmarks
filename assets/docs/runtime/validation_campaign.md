# LLMeter validation campaign

Last updated: 2026-09-30

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

The current automated suite already supplies meaningful evidence for much of Tier 1, but a tier claim requires the dedicated scenario/evidence boundary above. T1-01 through T1-07 pass at their recorded boundaries. Tier 1 is complete. Tier 2 now passes at the exercised live Ollama boundary; continue with Tier 3 after keeping the T3-02 JSONL accounting regression in place.

### Tier 2 — core benchmark workflows

Exercise every built-in standard path through the real CLI: streaming chat generation, `/v1/responses`, consistency and prompt sizes, structured output and tool calling, embeddings, and multi-model/multi-benchmark orchestration. Positive behavior and controlled unsupported/error records must both be retained where applicable. Record progress, request order, persisted records, and reports.

Current campaign state: `PASS` at the exercised Ollama boundary on revision `c7438db`. The live CLI covered streaming chat, `/v1/responses`, consistency, prompt sizes, structured output, tool calling, embeddings, and a two-model/two-benchmark orchestration run. The two unsupported chat calls from the embeddings-only model were retained as controlled error records. See the [2026-09-24 Tier 2 evidence](../../QA/validation-2026-09-24/t2-standard-workflows-evidence.md). Other providers and unselected model combinations remain outside this pass.

### Tier 3 — performance subsystem

The current recheck passed performance safety, all four profile-owned default
matrices, the bounded fixture ceiling through concurrency 16, the 500-request
and above-limit regressions, the mock-provider E2E boundary, and the complete
serialized local test suite with 156 tests passed. The fixed release binary
also passed the current full `latency` profile (15/15 measured requests),
`throughput` profile (16/16), and `sweep` profile (81/81) for `qwen3.5:2b`.
The matched non-streaming matrix for `qwen3.5:2b` and `qwen3.5:9b` passed 80/80
measured requests with 20 observations per scenario. The sweep request-identity
defect was fixed so output-size branches have distinct IDs. See the [current
T3 release-performance evidence](../../QA/validation-2026-09-30/t3-release-performance-evidence.md)
and [current live Ollama evidence](../../QA/validation-2026-09-30/t3-live-ollama-revalidation-evidence.md).

Validate profile planning and safety guards before measured runs. The critical stop gate `T3-02` passes on revision `c7438db`: a three-prompt JSONL workload matched scenario count, request-budget validation, progress total, mock HTTP request count, and persisted records. The mismatched synthetic-count risk was fixed before the one-scenario live Ollama smoke. T3-03 now also executes every profile-owned default matrix through the real CLI fixture: 36 scenario records and 154 total fixture chat requests passed on revision `2af310a`. The bounded live `qwen3.5:2b` progression for all four profile names at concurrency 1 and 2 remains valid at its recorded 2026-09-24 boundary. The live runs persisted successful request traces, telemetry, environment snapshots, inventory, and capability results. The first cold non-stream capability probe timed out and succeeded on a warmed repeat; Ollama returned HTTP 501 for embeddings on this generation model.

The bounded fixture continuation on revision `7d83c22` added two prompt sizes,
one output size, concurrency 1/2/4/8, 8 measured requests per scenario, and
72 total chat requests. The next continuation on revision `1b9d38b` extends
the same fixture boundary through concurrency 16, uses 16 measured requests
per scenario, and accounts for 10 scenarios and 170 total chat requests. It
strengthens fixture-scale scheduling and accounting evidence without
expanding the live-provider or statistical claim.

Current campaign state: `PARTIAL`; T3-02, deterministic T3-03 including
default-matrix execution, the bounded fixture ceiling, the full current Ollama
profiles, the matched two-model functional matrix, persistence/reporting/
privacy, and local quality gates pass at their stated boundaries. The optional
LiteLLM path is blocked because its endpoint and Docker engine are unavailable;
swap pressure disqualifies comparative timing interpretation, native Unix PTY
execution remains hosted-CI scoped, and exact-candidate four-platform CI is
open. Broader provider/model/host variance and universal numeric performance
remain validation debt.

#### T3-05 — matched live variance and release-scale qualification

This is the finite Tier 3 release slice for the local, single-user benchmark
CLI. It does not certify universal provider, model, host, or timing behavior.
Acceptance requires all of the following:

- T3-02 JSONL accounting and deterministic default-matrix fixture execution remain green.
- The real CLI reaches the configured `DEFAULT_MAX_PERFORMANCE_REQUESTS = 500` ceiling against the loopback fixture, with exact provider accounting, complete persisted measured traces, reloadable schema `3.0` output, privacy flags, and no temporary files.
- A matrix above 500 requests is rejected before any performance chat request and produces no result artifact unless an explicit override is supplied.
- Full live default `latency`, `throughput`, and `sweep` profiles remain green for the recorded baseline model.
- At least two locally available generation models execute an identical non-streaming matched matrix with prompt size, output size, concurrency including a value greater than one, warmup, measured count, telemetry, stream mode, and request cap held constant.
- The primary matched scenarios retain at least 20 measured observations so the current percentile implementation can represent P95; swap-pressure warnings disqualify comparative timing interpretation while leaving functional validation scoped as such.
- A second OpenAI-compatible provider path is exercised where already available, preferably the recorded LiteLLM-over-Ollama route, without adding provider lifecycle management or permanent configuration.
- JSON, CSV, Markdown, and HTML outputs reload successfully, response previews remain absent unless explicitly requested, and privacy scanning is clean.
- Hosted Ubuntu x86-64, Windows x86-64, macOS Intel x86-64, and macOS Apple-silicon CI pass on the exact candidate revision.

Current T3-05 disposition: `PARTIAL`. The deterministic ceiling and above-limit
fixture regressions, current Ollama default profiles, matched two-model
functional matrix, persistence/reload, and privacy checks pass. The optional
LiteLLM path is unavailable, swap pressure disqualifies comparative timing
interpretation, and exact-candidate hosted CI remains open. Broader cross-host
numeric performance remains validation debt. See the [current live Ollama
evidence](../../QA/validation-2026-09-30/t3-live-ollama-revalidation-evidence.md).

The current scoped candidate is `c5864bf4e303646fca74ea98707ca4081172c0b9` on
`develop`; exact hosted qualification remains pending for this revision.

### Tier 4 — representative provider compatibility, quality planning, and distribution

Tier 4 is evidence-based rather than installation-count-based. Provider
servers and models remain external QA dependencies, and LLMeter does not add
provider lifecycle management, model downloading, containers, or installation
logic. The four evidence lanes are:

| Evidence lane | Purpose | Provider installation required |
|---|---|---:|
| Preset contract matrix | Exercise every registered preset against the deterministic OpenAI-compatible client contract. | No |
| Representative live matrix | Demonstrate interoperability against selected architecturally distinct implementations with exact provider/version/model/host records. | Only temporary or already-available providers |
| External quality planning | Validate dry-run plan generation for every supported external evaluator. | No |
| Distribution | Validate the packaged LLMeter release, archive contents, checksums, provenance, and extracted-binary smoke behavior. | No |

The Tier 4 claim at this boundary is:

> LLMeter regression-tests all registered provider presets against its OpenAI-compatible contract. Live-provider validation is performed against a representative compatibility matrix and does not imply certification of every provider version, model, extension, or optional endpoint.

#### T4-01 — provider preset contract matrix

Scope every `ProviderKind` in `src/providers.rs`: catalog label, compatibility
tier, default URL, CLI selection, explicit base-URL override, `/v1/models`,
streaming and non-streaming `/v1/chat/completions`, request/response parsing,
and controlled failures for unsupported optional `/v1/responses` and
`/v1/embeddings`. The catalog-derived fixture must use an ephemeral loopback
port and must not require a real provider default port.

Acceptance requires every current preset to pass the deterministic fixture,
future presets to require corresponding catalog-derived coverage, and no live
claim to be inferred. The current focused boundary passes in
`mock_provider_e2e.rs`; transport authorization and persisted-secret checks
remain covered by the earlier T1-04 fixture evidence. See the [Tier 4 preset
contract evidence](../../QA/validation-2026-09-29/tier4-provider-contract-evidence.md).

#### T4-02 — representative live-provider interoperability

Use a small, exact matrix rather than installing every preset:

| Target | Required record | Current state |
|---|---|---|
| Ollama | Existing live evidence for `qwen3.5:2b` and `nomic-embed-text:latest`, plus current availability recheck. | `PASS` at the current Ollama boundary: version `0.34.0`, five exposed models, full default `latency`/`throughput` profiles for `qwen3.5:2b`, and capability-probe smoke. |
| llama.cpp | Official CPU `llama-server` image, one small GGUF model, status/fresh models, core non-streaming and streaming chat, capability probes, and bounded performance smoke. | `PASS` at the exact temporary boundary: `0.5.0-dev` build `11243`, image digest and `gemma-3-270m-it-Q8_0.gguf` hash recorded in the [live evidence](../../QA/validation-2026-09-29/tier4-live-provider-evidence.md). |
| LiteLLM | Ephemeral OpenAI-compatible proxy over the existing Ollama `qwen3.5:2b` backend, with the same compact interoperability boundary. | `PASS` at the exact temporary boundary: package `1.103.0`, image digest, alias, backend route, and process-only proxy key boundary recorded in the [live evidence](../../QA/validation-2026-09-29/tier4-live-provider-evidence.md). |

Record status, models, capability probes, one standard generation subset, and
one smoke performance run only when the provider is available. Unsupported
`responses` or `embeddings` behavior is a recorded capability result, not an
application failure. TGI, text-generation-webui, Jan, and MLX-LM remain
fixture-validated but not live-certified unless an already-available target can
be exercised with negligible temporary setup. See the [Tier 4 live-provider
evidence](../../QA/validation-2026-09-29/tier4-live-provider-evidence.md).

Current state: `PASS` at the explicitly recorded representative boundary:
Ollama, llama.cpp, and LiteLLM-over-existing-Ollama. The two disposable Docker
runtimes used the official images, exact resolved digests, a compact model or
existing backend, and one-scenario performance smokes; task-owned runtime
resources were removed after capture. This remains representative evidence,
not universal provider certification, and all other presets remain
fixture-validated only.

#### T4-03 — external quality planning

Validate LLMeter's planning surface for LightEval, Inspect AI, lm-eval-harness,
and SWE-bench: CLI parsing, catalog/task mapping, model propagation, command
preview, serialized plan shape, unknown framework handling, unknown-task
planning, and dataset/external-tool/code-execution flags. Do not install or
execute those frameworks, download datasets, or run SWE-bench.

The current `quality_cli_tests.rs` boundary passes for all four adapters and
the complete catalog. External evaluator execution is outside the currently
validated LLMeter runtime contract. See the [Tier 4 quality and distribution
evidence](../../QA/validation-2026-09-29/tier4-quality-distribution-evidence.md).

#### T4-04 — distribution

Keep distribution independent from provider compatibility. The existing
`v0.4.0` release evidence verifies four native GitHub archives, expected
contents, `SHA256SUMS`, checksum verification, hosted provenance, extracted
`--version`/`--help` smoke tests, and packaged mock-provider E2E. crates.io
publication and clean installation remain separate owner-gated evidence; do
not publish or retag as part of this campaign.

Current state: `PASS` at the GitHub release boundary and `UNVERIFIED` for
crates.io. See the [release report](../QA/release-0.4.0/release-report.md) and
the [Tier 4 quality and distribution evidence](../../QA/validation-2026-09-29/tier4-quality-distribution-evidence.md).

Tier 4 is `PASS` at the representative compatibility boundary: T4-01, T4-02,
and T4-03 pass; T4-04 passes for GitHub distribution with crates.io separate.
The T4-02 result certifies only the recorded Ollama, llama.cpp, and LiteLLM
implementations and is not a universal provider-certification claim. Tier 3,
Tier 5, best-effort live-provider coverage, and crates.io remain separate open
boundaries.

### Tier 5 — resilience and edge cases

The 2026-09-28 current-tree revalidation and 2026-09-29 focused rerun covered
the bounded provider/CLI failure boundary (21/21 mock-provider cases), atomic-write cleanup (2/2),
performance safety ceilings (7/7), report reload at the T1-06 boundary (1/1),
the bounded fixture matrix through concurrency 16, and the full Windows
ConPTY suite (10/10). A focused continuation also
validated two completed invocations in fresh processes sharing one output
directory, Ctrl+C during a delayed performance request with no partial
artifact, and a fresh-process recovery run in that same output directory.
Report reload and repeated completed runs do not prove resumable in-progress
state; restart/state restoration, production-scale ceilings, and native
non-Windows terminal behavior remain open.
See the [scale-16 evidence](../../QA/validation-2026-09-28/t3-t5-scale-16-continuation-evidence.md), [bounded scale continuation evidence](../../QA/validation-2026-09-28/t3-t5-scale-continuation-evidence.md), and [current-tree revalidation evidence](../../QA/validation-2026-09-28/t3-t5-revalidation-evidence.md).

Exercise transport/protocol failures, filesystem and atomic-write failures, repeated operations and restart/state restoration, long-running interruption, scale/safety ceilings, and native non-Windows terminal behavior. The expected result is bounded, readable failure with no corrupted or convincing partial artifact.

Current campaign state: `PARTIAL` at the dedicated Tier 5 boundary. The selected T5-01 subset and current-tree continuation pass bounded provider/CLI failures, output-path failure handling, atomic-write cleanup, performance safety ceilings, the bounded fixture matrix through concurrency 16, repeated completed operations, a delayed-request interruption with no partial artifact, the interrupted streamed-9b run with no partial saved result, fresh-process recovery after interruption, and Windows ConPTY confirmation interruption. Resumable restart/state restoration, production-scale ceilings, and native non-Windows terminal behavior remain open. See the [repeated live model evidence](../../QA/validation-2026-09-29/t3-repeated-model-evidence.md), [2026-09-28 scale-16 evidence](../../QA/validation-2026-09-28/t3-t5-scale-16-continuation-evidence.md), [2026-09-28 bounded scale evidence](../../QA/validation-2026-09-28/t3-t5-scale-continuation-evidence.md), [2026-09-28 current-tree evidence](../../QA/validation-2026-09-28/t3-t5-revalidation-evidence.md), and [T3-04/T5-01 evidence](../../QA/validation-2026-09-26/t3-t5-followup-evidence.md).

#### T5-02 — restart, configured ceiling, and Unix terminal resilience

This is the finite Tier 5 release slice for durable state and terminal
recovery. Acceptance requires:

- Controlled provider/protocol failures, invalid output paths, and atomic-write cleanup remain green.
- A completed result remains byte-for-byte intact across a later interrupted process; an interrupted in-progress benchmark leaves no canonical or convincing partial JSON, report, or temporary atomic-write artifact.
- Abrupt child termination is tested independently of clean Ctrl+C cancellation.
- A fresh process can list and show the completed result, then complete another benchmark in the same output directory with distinct, independently loadable run IDs.
- The configured 500-request fixture ceiling executes successfully and an above-ceiling plan is rejected before provider execution.
- Windows ConPTY coverage remains green.
- Native Unix PTY coverage passes on Linux, macOS Intel, and macOS Apple silicon through hosted CI, including menu interrupt, nested cancel/back navigation, performance-confirmation interrupt, active delayed-request interrupt, and fresh-process recovery.

Current T5-02 disposition: `PARTIAL`. The platform-neutral abrupt-termination
and restart E2E, configured ceiling, above-limit refusal, and Windows ConPTY
boundaries pass locally. Native Unix PTY execution and exact-candidate hosted
CI remain unverified on this Windows-only checkout.

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

The current campaign does not make that comprehensive claim. Tiers 0–2 pass at their recorded boundaries; Tier 3 remains partial pending larger repeated workloads and broader provider/host evidence. Tier 4 is `PASS` at the explicitly recorded representative compatibility boundary; Tier 5 remains partial, and crates.io plus external evaluator execution remain separate owner/scope boundaries.
