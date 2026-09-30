# Project status ledger

Last updated: 2026-09-30

This document is the canonical current operational status catalog for LLMeter. It records status at an explicit evidence boundary; it does not replace architecture, user, test, or release documentation.

## Maintenance rules

- Read this ledger before substantial implementation or validation work.
- Update affected entries after implementation changes, meaningful validation, regression discovery, issue remediation, or release-state changes.
- Never mark a component `VALIDATED` from source inspection or test existence alone.
- Keep local, hosted, fixture, live-provider, and public-distribution evidence distinct.
- Keep the ledger sparse: link to the relevant contract, test, or release URL instead of copying a run narrative.
- Move fixed findings out of the active issue list and retain only durable decisions in the historical section.

## Current validation boundary

- Package version is `0.4.0` on `develop`. The current checkout is `6ef50aa` and contains an uncommitted Windows ConPTY interrupt monitor plus performance-run cancellation checks.
- The current tree is locally green: formatting, locked check, Clippy, rustdoc, release build, local audit/tree inspection, the serialized all-target/all-feature suite at 156/156, the Windows ConPTY suite at 10/10, and an isolated one-request Ollama smoke passed.
- Hosted CI run [36722865383](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36722865383) certifies the exact earlier candidate 1fb3dbd1f3211e25280481f5bbdab439a850a2f9, including the four native jobs and hosted Unix PTY cases. It does not certify the uncommitted current-tree repair; rerun it after that repair is committed.
- The public [`v0.4.0` GitHub release](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.4.0) is verified by hosted release run [34574075684](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/34574075684). The first crates.io publication and clean registry install remain unperformed.
- Current live Ollama evidence covers `qwen3.5:2b` and `qwen3.5:9b`. Default `latency`, `throughput`, and `sweep` profiles completed functionally, as did the matched two-model matrix, but swap pressure prevents comparative timing or model-ranking claims. LiteLLM is currently unavailable because no Docker engine is reachable.

## Status vocabulary

| Status | Meaning |
|---|---|
| `VALIDATED` | Implemented and confirmed by meaningful evidence at the stated scope. |
| `WORKING` | Believed to work from implementation or limited testing, but not fully validated. |
| `PARTIAL` | Implemented but incomplete, degraded, or valid for only part of the expected behavior. |
| `BLOCKED` | Validation is prevented by an external dependency, missing approval, unavailable service, or similar blocker. |
| `UNVALIDATED` | Implementation exists, but evidence is insufficient for a stronger claim. |
| `NOT_IMPLEMENTED` | The expected capability is absent. |

## Component status

| Component | Status | Evidence and current guarantee | Boundary or next check |
|---|---|---|---|
| `application.startup` | `VALIDATED` | CLI contract coverage, release-binary startup/help/status smoke, provider resolution, and non-TTY dispatch pass. See [startup](runtime/startup.md), [configuration](runtime/configuration.md), and [CLI contract tests](../../tests/cli_contract_tests.rs). | Live status and benchmarks still require a provider started by the user. |
| `cli.interactive` | `VALIDATED` | Current Windows ConPTY suite passes 10/10. The exact hosted candidate passed the native Unix PTY cases on Ubuntu x86-64 and both macOS targets in [CI run 36722865383](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36722865383). | The hosted result predates the uncommitted current-tree interruption repair; rerun hosted PTY coverage after commit. |
| `cli.scriptable` | `VALIDATED` | CLI contract tests, release-binary smoke, mock-provider execution, JSON cleanliness, documented exit codes, and report commands pass. See [scriptable usage](user/scriptable_usage.md) and [mock-provider E2E](../../tests/mock_provider_e2e.rs). | Provider-backed outcomes remain dependent on the external server and model. |
| `cli.help-catalogs` | `VALIDATED` | Provider, benchmark, quality, and built-in help catalogs are exercised through real CLI output and contract tests. | Re-run after changing catalog registration or help behavior. |
| `configuration.resolution` | `VALIDATED` | CLI, environment, and persisted precedence; URL normalization; numeric bounds; provider validation; and embedded-credential rejection are covered by tests and launcher smoke. | Re-run configuration and error-contract tests after changing precedence or validation. |
| `provider.protocol` | `VALIDATED` | OpenAI-compatible requests, fresh versus cached catalog reads, SSE assembly, bounded responses, redirects, authorization, and optional capability handling pass the provider tests and current 23/23 mock-provider E2E boundary. | Live capability support remains provider-, model-, version-, and endpoint-dependent. |
| `provider.presets` | `VALIDATED` | The catalog-derived fixture exercises every registered preset and its baseline contract through [mock-provider E2E](../../tests/mock_provider_e2e.rs). | Fixture evidence is not live certification; add live evidence only for an available representative target. |
| `provider.live.representative` | `VALIDATED` | Exact recorded live boundaries cover Ollama plus earlier disposable llama.cpp and LiteLLM interoperability checks. | Do not generalize to unexercised providers, models, versions, hosts, or optional endpoints. |
| `provider.best-effort.live` | `UNVALIDATED` | TGI, text-generation-webui, Jan, and MLX-LM have deterministic preset coverage only. | A live target and compatible model are required before claiming interoperability. |
| `benchmark.llm` | `VALIDATED` | The standard generation, responses, consistency, prompt-size, structured-output, and tool-calling paths passed at the recorded Ollama boundary; unsupported capabilities remain explicit error records. See [benchmarks](user/benchmarks.md). | Broader provider/model combinations remain outside the live claim. |
| `benchmark.embeddings` | `VALIDATED` | Embeddings completed successfully for the recorded Ollama embedding model and remains reportable through the standard result path. | Embeddings support is not universal across providers or models. |
| `benchmark.performance` | `PARTIAL` | The 500-request ceiling and above-limit refusal, all profile-owned fixture matrices, current Ollama default profiles, matched two-model functional matrix, unique request IDs, persistence, reports, and privacy checks pass. | Swap pressure disqualifies comparative timing; current LiteLLM smoke is unavailable; broader host/provider variance remains open. See [benchmark execution](architecture/benchmark_execution.md). |
| `quality.external-plans` | `VALIDATED` | Quality catalog and dry-run plan generation pass [quality CLI tests](../../tests/quality_cli_tests.rs). | LLMeter does not install or execute external frameworks or publish their scores. |
| `results.persistence` | `VALIDATED` | Schema `3.0`, mandatory identity, atomic JSON/CSV writes, redaction, opt-in previews, distinct run IDs, and interrupted-run cleanup pass [result tests](../../tests/test_results.rs) and [schema tests](../../tests/result_schema_tests.rs). | Older, malformed, or unversioned results are intentionally rejected; in-progress runs are not resumable. |
| `reporting` | `VALIDATED` | Terminal list/show and Markdown/HTML generation reload standard, performance, error, and adversarial result shapes with escaping and privacy checks. See [report tests](../../tests/test_reporting.rs). | Unsupported or truncated result files fail closed. |
| `lifecycle.local-install` | `VALIDATED` | Install, overwrite refusal, rollback cleanup, update, uninstall, purge, and preservation of unrelated home data pass the lifecycle tests and documented local boundary. | These are local file operations, not a remote updater or package manager. |
| `launcher.powershell` | `VALIDATED` | Release-binary reuse, argument forwarding, fallback selection, noninteractive refusal, ConPTY confirmation, protected-home refusal, and owned cleanup pass [launcher E2E tests](../../tests/launcher_powershell_e2e.rs). | The primary launcher evidence is Windows-specific. |
| `test.local-quality-gates` | `VALIDATED` | The current tree passes local formatting, locked checks, warning-denied Clippy/rustdoc, release build, audit/tree inspection, 156 serialized tests, and Windows PTY coverage. The exact hosted candidate passes the four-platform matrix in [CI run 36722865383](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36722865383). | Hosted CI must be rerun for the current uncommitted source change before it becomes a release candidate. |
| `release.cross-platform` | `VALIDATED` | The tag-gated workflow produced and published the four native `v0.4.0` archives with checksums, packaged tests, and provenance in [release run 34574075684](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/34574075684). | Repeat the hosted matrix and independent artifact verification for future tags. |
| `release.public-distribution` | `PARTIAL` | GitHub `v0.4.0` is public and verified. | The first crates.io publication and clean `cargo install` verification remain owner-gated. |

## Open issues

| ID | Component | Severity | Current boundary | Required action |
|---|---|---|---|---|
| `ISSUE-001` | `release.public-distribution` | Medium | GitHub archives are verified; registry installation is not a current claim. | Publish from an approved exact tag, then verify a clean registry install. |
| `ISSUE-003` | `quality.external-plans` | Low | External quality tools are intentionally planning-only. | Keep execution outside the runtime contract unless a separate adapter and ownership boundary are approved. |

The current-tree hosted revalidation is a release follow-up, not an application defect: commit the repair, rerun the exact four-platform workflow, and update the release boundary before shipping it.

## Validation debt

| Component | Confidence | Missing validation | Priority |
|---|---|---|---|
| `provider.live.representative` | Medium | Broader provider, model, version, host, and optional-endpoint variance beyond the recorded representatives. | P2 |
| `provider.best-effort.live` | Low | Live status, discovery, capability, and benchmark coverage for the remaining best-effort presets. | P2 |
| `benchmark.performance.comparative` | Medium | Comparative timing on a host without material swap pressure and an optional current LiteLLM smoke. | P2 |
| `release.crates-io` | Low | First publication and clean registry installation for the current package version. | P1 |

## Durable decisions

| Decision | Current contract |
|---|---|
| Local product boundary | LLMeter is a single-user local CLI. Provider processes and model installation remain external; no hosted control plane, account model, or application-managed provider lifecycle is required. |
| Catalog freshness | Interactive navigation may reuse a client-local cache; status, model listing, benchmark validation, and measured probes use fresh provider discovery. |
| Result safety | Result schema `3.0` is strict. Writes are atomic, response previews are opt-in, and credential-shaped values are redacted before persistence. |
| Performance semantics | TTFT, inter-chunk timing, ITL, token-usage coverage, and first-request load estimates remain distinct. Sample limits and host pressure are part of interpretation; no unsupported provider-native load claim is made. |
| Interruption and recovery | A cancelled or abruptly terminated run must not replace a completed result or leave a convincing partial artifact; fresh processes must still list, show, and create later results. |
| Distribution and lifecycle | Public archives are checksum/provenance-gated. Managed install/update/uninstall are local convenience operations and do not implement secure remote self-update. |
| External quality | Quality integrations stop at catalog and dry-run plan generation; external evaluators are not executed by LLMeter. |

## Revalidation triggers

| Changed area | Revalidate |
|---|---|
| Providers, presets, catalog cache, auth, or probes | `provider.protocol`, `provider.presets`, and the relevant live-provider boundary; run provider tests, mock E2E, and an available live smoke. |
| CLI parsing, startup, launcher, menus, or prompts | `application.startup`, `cli.interactive`, `cli.scriptable`, and `launcher.powershell`; run CLI contracts, PTY E2E, and launcher smoke. |
| Benchmark registry, requests, runner, or semantics | The affected `benchmark.*` entries plus provider protocol and reporting; run focused tests and mock E2E. |
| Performance plans, telemetry, metrics, concurrency, or cancellation | `benchmark.performance`, `test.local-quality-gates`, and the finite resilience boundary; run performance, resilience, and PTY tests plus a representative live scenario. |
| Result schema, persistence, redaction, or report templates | `results.persistence` and `reporting`; run schema, result-store, reporting, mock saved-run, and reload smoke. |
| CI, packaging, version, release workflow, or platform claims | `test.local-quality-gates`, `release.cross-platform`, `release.public-distribution`, and the [release checklist](runtime/release_checklist.md). |

## Source-of-truth relationship

The [project index](project_index.md) defines the documentation ontology. Architecture documents define intended behavior and module boundaries. The [audit implementation plan](runtime/audit_implementation_plan.md) records durable implementation decisions and roadmap work. The [validation campaign](runtime/validation_campaign.md) defines the ordered evidence strategy and claim boundaries. Tests, hosted CI, and public-release links are the durable evidence anchors; transient run output is not a parallel status system.
