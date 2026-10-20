# Project status ledger

Last updated: 2026-10-20

This document is the canonical current operational status catalog for LLMeter. It records status at an explicit evidence boundary, summarizes the durable guarantees that the project commits to, and tracks remaining validation work. It does not replace architecture, user, test, or release documentation.

## Maintenance rules

- Read this ledger before substantial implementation or validation work.
- Update affected entries after implementation changes, meaningful validation, regression discovery, issue remediation, or release-state changes.
- Never mark a component `VALIDATED` from source inspection or test existence alone.
- Keep local, hosted, fixture, live-provider, and public-distribution evidence distinct.
- Keep the ledger sparse: link to the relevant contract, test, or release URL instead of copying a run narrative.
- Move fixed findings out of the active issue list; retain only durable decisions and open boundaries.

## Current validation boundary

- Package version `0.5.0` is public. The annotated [`v0.5.0` tag](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.5.0) identifies the released source revision.
- The released candidate passed the full local gate (formatting, locked all-target/all-feature checks, warning-denied Clippy/rustdoc, dependency audit, release build, serialized tests, and native PTY coverage) and the four-platform hosted CI matrix before publication.
- The public `v0.5.0` GitHub release carries four native archives (Windows x86-64, GNU/Linux x86-64, macOS Intel, macOS Apple silicon) with `SHA256SUMS` and provenance attestations; the archives and attestations have been independently downloaded and verified.
- The current validated source revision is `d512be4`. Formatting, locked check, warning-denied Clippy/rustdoc, the serialized test suite, the dependency audit, the release build, and the package dry-run pass. The audit initially found seven current `rustls`/`quinn-proto` advisories; the locked graph was refreshed to `rustls 0.23.46` and `quinn-proto 0.11.18`, and the final audit is clean.
- Three consecutive hosted CI attempts at `d512be4` passed all four native jobs, including the Ubuntu, macOS Intel, and macOS Apple-silicon Unix PTY suites and the repaired performance-confirmation interruption test.
- Current live Ollama evidence covers `qwen3.5:2b`, `qwen3.5:9b`, and `nomic-embed-text:latest`. The standard `llm` chat-generation and embeddings smokes pass. A matched two-model `smoke` performance matrix completed 12 measured requests with zero errors, but telemetry recorded a maximum swap-used ratio of `0.971` and the swap-pressure warning, so comparative timing and model-ranking claims remain disqualified.
- A disposable `ghcr.io/berriai/litellm:main-latest` proxy (image digest `fc2cf93`) reached from Docker Desktop passed status, model discovery, and a live `chat-generation` run against Ollama through `qwen3.5:2b`; the container and Docker engine were stopped afterward.
- The first crates.io publication and clean registry installation have not been performed. The package dry-run passes at `d512be4`.
- The README badge row was rechecked after the CI repair: License, Release, CI, Rust 2021, and Platforms all resolve and render, and the CI badge reports `build: passing`. No logo asset exists in the repository or GitHub metadata, so the title remains text-only.
- Documentation-only commits after `d512be4` do not change the recorded runtime boundary; re-run the local gate when source, dependency, or workflow files change.

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
| `application.startup` | `VALIDATED` | CLI contract coverage, release-binary startup/help/status smoke, provider resolution, and non-TTY dispatch pass at the current `d512be4` local and hosted boundaries. See [startup](runtime/startup.md), [configuration](runtime/configuration.md), and [CLI contract tests](../../tests/cli_contract_tests.rs). | Live status and benchmarks still require a provider started by the user. |
| `cli.interactive` | `VALIDATED` | Windows ConPTY menu coverage and the committed interruption repair pass; the current `d512be4` gate ran the Windows PTY suite locally, and three consecutive hosted attempts passed the Unix PTY suite on Ubuntu, macOS Intel, and macOS Apple silicon. | Re-run native PTY coverage after future terminal or launcher changes. |
| `cli.scriptable` | `VALIDATED` | CLI contract tests, release-binary smoke, mock-provider execution, JSON cleanliness, documented exit codes, and report commands pass. See [scriptable usage](user/scriptable_usage.md) and [mock-provider E2E](../../tests/mock_provider_e2e.rs). | Provider-backed outcomes remain dependent on the external server and model. |
| `cli.help-catalogs` | `VALIDATED` | Provider, benchmark, quality, and built-in help catalogs are exercised through real CLI output and contract tests. | Re-run after changing catalog registration or help behavior. |
| `configuration.resolution` | `VALIDATED` | CLI, environment, and persisted precedence; URL normalization; numeric bounds; provider validation; and embedded-credential rejection are covered by unit and launcher tests. | Re-run configuration and error-contract tests after changing precedence or validation. |
| `provider.protocol` | `VALIDATED` | OpenAI-compatible requests, fresh versus cached catalog reads, SSE assembly, bounded responses, redirects, authorization, and optional capability handling pass the provider tests, the mock-provider E2E boundary, and a current live LiteLLM proxy smoke covering authenticated status, discovery, and chat completion. | Live capability support remains provider-, model-, version-, and endpoint-dependent. |
| `provider.presets` | `VALIDATED` | The catalog-derived fixture exercises every registered preset and its baseline contract through [mock-provider E2E](../../tests/mock_provider_e2e.rs); LiteLLM now also has a current live proxy smoke. | Fixture evidence is not live certification; add live evidence only for an available representative target. |
| `provider.live.representative` | `VALIDATED` | Recorded live boundaries cover Ollama (`qwen3.5:2b`, `qwen3.5:9b`, `nomic-embed-text:latest`) and a current disposable LiteLLM proxy, plus earlier llama.cpp interoperability checks. | Do not generalize to unexercised providers, models, versions, hosts, or optional endpoints. |
| `provider.best-effort.live` | `UNVALIDATED` | TGI, text-generation-webui, Jan, and MLX-LM have deterministic preset coverage only. | A live target and compatible model are required before claiming interoperability. |
| `benchmark.llm` | `VALIDATED` | The standard generation, responses, consistency, prompt-size, structured-output, and tool-calling paths passed at the recorded Ollama boundary; a current `d512be4` live chat-generation smoke also passes. Unsupported capabilities remain explicit error records. See [benchmarks](user/benchmarks.md). | Broader provider/model combinations remain outside the live claim. |
| `benchmark.embeddings` | `VALIDATED` | Embeddings completed successfully for the recorded Ollama embedding model and a current `d512be4` live `nomic-embed-text:latest` smoke, and remains reportable through the standard result path. | Embeddings support is not universal across providers or models. |
| `benchmark.performance` | `PARTIAL` | The 500-request ceiling and above-limit refusal, all profile-owned fixture matrices, the recorded Ollama default profiles, and a current matched two-model `smoke` timing matrix (4 warmups plus 12 measured requests, 0 errors, 4 records) pass; unique request IDs, persistence, reports, and privacy checks pass. | Host swap pressure reached a maximum used ratio of `0.971` and triggered the documented warning, so comparative timing and model-ranking claims remain disqualified. LiteLLM interoperability is covered separately by a live non-performance smoke; broader host/provider variance remains open. See [benchmark execution](architecture/benchmark_execution.md). |
| `quality.external-plans` | `VALIDATED` | Quality catalog and dry-run plan generation pass [quality CLI tests](../../tests/quality_cli_tests.rs). | LLMeter does not install or execute external frameworks or publish their scores. |
| `results.persistence` | `VALIDATED` | Schema `3.0`, mandatory identity, atomic JSON/CSV writes, redaction, opt-in previews, distinct run IDs, and interrupted-run cleanup pass [result tests](../../tests/test_results.rs) and [schema tests](../../tests/result_schema_tests.rs). | Older, malformed, or unversioned results are intentionally rejected; in-progress runs are not resumable. |
| `reporting` | `VALIDATED` | Terminal list/show and Markdown/HTML generation reload standard, performance, error, and adversarial result shapes with escaping and privacy checks. A current performance result carrying the swap-pressure summary also reloads through terminal and Markdown report paths. See [report tests](../../tests/test_reporting.rs). | Unsupported or truncated result files fail closed. |
| `lifecycle.local-install` | `VALIDATED` | Install, overwrite refusal, rollback cleanup, update, uninstall, purge, and preservation of unrelated home data pass the lifecycle tests and documented local boundary. | These are local file operations, not a remote updater or package manager. |
| `launcher.powershell` | `VALIDATED` | Release-binary reuse, argument forwarding, fallback selection, noninteractive refusal, ConPTY confirmation, protected-home refusal, and owned cleanup pass [launcher E2E tests](../../tests/launcher_powershell_e2e.rs). | The primary launcher evidence is Windows-specific. |
| `test.local-quality-gates` | `VALIDATED` | The current `d512be4` source passed formatting, locked all-target/all-feature check, warning-denied Clippy/rustdoc, the serialized test suite, the dependency audit, the release build, and the package dry-run. The audit-driven lock refresh to `rustls 0.23.46` and `quinn-proto 0.11.18` was revalidated by the full suite. See [testing and quality](coding/testing_and_quality.md). | Re-run the complete local gate after future source, dependency, or release-workflow changes. |
| `release.cross-platform` | `VALIDATED` | The tag-gated workflow produced and published four native `v0.5.0` archives with packaged tests, extracted-binary smoke checks, checksums, and provenance; independent downloads and attestation checks pass. | Repeat the hosted matrix and independent artifact verification for future tags. |
| `release.public-distribution` | `PARTIAL` | GitHub [`v0.5.0`](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.5.0) is public and verified across all four archives. The package dry-run passes at `d512be4`. Registry installation is not a current claim. | Owner-gated first crates.io publication followed by clean `cargo install` verification remains open. |

## Open issues

| ID | Component | Severity | Current boundary | Required action |
|---|---|---|---|---|
| `ISSUE-001` | `release.public-distribution` | Medium | GitHub `v0.5.0` archives are verified and the package dry-run passes at `d512be4`; registry publication and installation are not current claims. | With owner approval and a valid registry token, publish from the exact tag, then verify a clean registry install. |
| `ISSUE-003` | `quality.external-plans` | Low | External quality tools are intentionally planning-only. | Keep execution outside the runtime contract unless a separate adapter and ownership boundary are approved. |

The `v0.5.0` hosted revalidation and public artifact checks are complete release evidence, not an application defect. Comparative performance and registry distribution remain explicitly separate boundaries.

## Validation debt

| Component | Confidence | Missing validation | Priority |
|---|---|---|---|
| `provider.live.representative` | Medium | Broader provider, model, version, host, and optional-endpoint variance beyond Ollama and the disposable LiteLLM proxy. | P2 |
| `provider.best-effort.live` | Low | Live status, discovery, capability, and benchmark coverage for the remaining best-effort presets. | P2 |
| `benchmark.performance.comparative` | Medium | Comparative timing on a host whose telemetry stays below the documented 0.20 swap warning; the current matched matrix reached 0.971. | P2 |
| `release.crates-io` | Low | First publication and clean registry installation for the current package version. | P1 |

## Durable decisions

These are the contract-level decisions that validation and future work must preserve. The linked concept documents carry the maintained detail.

| Decision | Current contract |
|---|---|
| Product scope | LLMeter is a single-user local CLI. Provider processes and model installation remain external; no hosted control plane, account model, or application-managed provider lifecycle is required. See [system overview](architecture/system_overview.md) and [shared rules](coding/shared_rules.md). |
| Configuration | Invalid CLI, environment, persisted, URL, numeric, or provider values fail closed with named source context instead of silently defaulting. See [configuration](runtime/configuration.md) and [error handling](coding/error_handling.md). |
| Catalog freshness | Interactive navigation may reuse a client-local cache; status, model listing, benchmark validation, and measured probes use fresh provider discovery. See [provider integration](architecture/provider_integration.md). |
| Result contract | Result schema `3.0` is strict. Writes are atomic, response previews are opt-in, and credential-shaped values are redacted before persistence. See [result storage](architecture/result_storage.md). |
| Measurement semantics | TTFT, inter-chunk timing, ITL, token-usage coverage, and first-request load estimates remain distinct. Sample limits and host pressure are part of interpretation; no unsupported provider-native load claim is made. See [reports and results](user/reports_and_results.md). |
| Safety limits | Performance safety controls are separate from provider parameters: a default 500 warmup-plus-measured request ceiling, with explicit opt-ins for larger matrices or prompt/output sizes. See [benchmarks](user/benchmarks.md). |
| Interruption and recovery | A cancelled or abruptly terminated run must not replace a completed result or leave a convincing partial artifact; fresh processes must still list, show, and create later results. See [validation campaign](runtime/validation_campaign.md). |
| Distribution and lifecycle | Public archives are checksum/provenance-gated. Managed install/update/uninstall are local convenience operations and do not implement secure remote self-update. See [deployment](runtime/deployment.md). |
| External quality | Quality integrations stop at catalog and dry-run plan generation; external evaluators are not executed by LLMeter. See [benchmarks](user/benchmarks.md). |

## Revalidation triggers

| Changed area | Revalidate |
|---|---|
| Providers, presets, catalog cache, auth, or probes | `provider.protocol`, `provider.presets`, and the relevant live-provider boundary; run provider tests, mock E2E, and an available live smoke. |
| CLI parsing, startup, launcher, menus, or prompts | `application.startup`, `cli.interactive`, `cli.scriptable`, and `launcher.powershell`; run CLI contracts, PTY E2E, and launcher smoke. |
| Benchmark registry, requests, runner, or semantics | The affected `benchmark.*` entries plus provider protocol and reporting; run focused tests and mock E2E. |
| Performance plans, telemetry, metrics, concurrency, or cancellation | `benchmark.performance`, `test.local-quality-gates`, and the finite resilience boundary; run performance, resilience, and PTY tests plus a representative live scenario. |
| Result schema, persistence, redaction, or report templates | `results.persistence` and `reporting`; run schema, result-store, reporting, mock saved-run, and reload smoke. |
| CI, dependency lock/security refresh, packaging, version, release workflow, or platform claims | `test.local-quality-gates`, `release.cross-platform`, `release.public-distribution`, and the [release checklist](runtime/release_checklist.md). |

## Source-of-truth relationship

The [project index](project_index.md) defines the documentation ontology. Architecture and user documents define intended behavior. The [validation campaign](runtime/validation_campaign.md) defines the ordered evidence strategy and claim boundaries; the [release checklist](runtime/release_checklist.md) defines the release procedure. Tests, hosted CI, and public-release links are the durable evidence anchors; transient run output is not a parallel status system.
