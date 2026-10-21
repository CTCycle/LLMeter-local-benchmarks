# Project status ledger

Last updated: 2026-10-21

This document is the canonical current operational status catalog for LLMeter. It records status at an explicit evidence boundary, summarizes the durable guarantees that the project commits to, and tracks remaining validation work. It does not replace architecture, user, test, or release documentation.

## Maintenance rules

- Read this ledger before substantial implementation or validation work.
- Update affected entries after implementation changes, meaningful validation, regression discovery, issue remediation, or release-state changes.
- Never mark a component `VALIDATED` from source inspection or test existence alone.
- Keep local, hosted, fixture, live-provider, and public-distribution evidence distinct.
- Keep the ledger sparse: link to the relevant contract, test, or release URL instead of copying a run narrative.
- Move fixed findings out of the active issue list; retain only durable decisions and open boundaries.

## Current validation boundary

- The latest GitHub binary release is [`v0.5.1`](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.5.1), published from the annotated tag at commit `34f6b4814d8cb537d9d065253913b95db318a3da` on `main`.
- The released `v0.5.0` candidate passed the full local gate (formatting, locked all-target/all-feature checks, warning-denied Clippy/rustdoc, dependency audit, release build, serialized tests, and native PTY coverage) and the four-platform hosted CI matrix before publication. `v0.5.1` passed its tag-gated hosted validation; a full local post-fix compile/test rerun was blocked by crate-download DNS failure.
- The `v0.5.1` release carries four native archives (Windows x86-64, GNU/Linux x86-64, macOS Intel, macOS Apple silicon), `SHA256SUMS`, and provenance attestations. The [tag-gated workflow](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39661462013) passed metadata validation, format, check, Clippy, tests, rustdoc, dependency audit, package dry run, all four native build/package jobs, packaged mock-provider checks, checksum generation, attestations, and GitHub release upload. GitHub release metadata lists all five uploaded assets; independent asset download and attestation verification has not been repeated for `v0.5.1`.
- The original `v0.5.1` candidate at `65d4c8e` failed its dependency audit on `RUSTSEC-2026-0382` and `RUSTSEC-2026-0381`, affecting `rustls-webpki 0.103.15` and `webpki-roots 1.0.9`. The lockfile was minimally updated to `0.103.16` and `1.0.10` at `c58a756`; four-platform hosted CI passed on both branches ([develop](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39658572919), [main](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39659362689)). With explicit authorization, the annotated `v0.5.1` tag was retargeted to main commit `34f6b48`; the release workflow then passed. Local formatting passes, but local compile/test reruns after the lockfile refresh could not download crates because this host could not resolve `static.crates.io`.
- The pre-fix `0.5.1` candidate passed the local serialized suite (147 passed, 0 failed) and four-platform CI, but those results do not apply to the corrected TLS dependency graph at `c58a756`.
- Recorded live Ollama evidence at `d512be4` covers `qwen3.5:2b`, `qwen3.5:9b`, and `nomic-embed-text:latest`. Standard `llm` chat-generation and embeddings smokes pass. A matched two-model `smoke` performance matrix completed 12 measured requests with zero errors, but telemetry recorded a maximum swap-used ratio of `0.971` and the swap-pressure warning, so comparative timing and model-ranking claims remain disqualified.
- A disposable `ghcr.io/berriai/litellm:main-latest` proxy (image digest `fc2cf93`) reached from Docker Desktop passed status, model discovery, and a live `chat-generation` run against Ollama through `qwen3.5:2b`; the container and Docker engine were stopped afterward.
- The first crates.io publication and clean registry installation have not been performed. The hosted `v0.5.1` package dry run passed; local package dry-run access was blocked by crates.io DNS resolution.
- The README badge row was rechecked after the CI repair: License, Release, CI, Rust 2021, and Platforms all resolve and render, and the CI badge reports `build: passing`. No logo asset exists in the repository or GitHub metadata, so the title remains text-only.
- `main` and `develop` were aligned at the `v0.5.1` release commit; GitHub reports only those two branches, with `main` protected.

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
| `application.startup` | `VALIDATED` | CLI contract coverage, the pre-fix `65d4c8e` release-mode `--version`/`--help` and official launcher smoke, provider resolution, non-TTY dispatch, and the four-platform CI at `c58a756` pass. See [startup](runtime/startup.md), [configuration](runtime/configuration.md), and [CLI contract tests](../../tests/cli_contract_tests.rs). | Live status and benchmarks still require a provider started by the user. |
| `cli.interactive` | `VALIDATED` | Windows ConPTY coverage and the interruption repair pass; the four-platform CI at `c58a756` passes the Unix PTY suite on Ubuntu, macOS Intel, and macOS Apple silicon. | Re-run native PTY coverage after future terminal or launcher changes. |
| `cli.scriptable` | `VALIDATED` | CLI contract tests, release-binary smoke, mock-provider execution, JSON cleanliness, documented exit codes, and report commands pass. See [scriptable usage](user/scriptable_usage.md) and [mock-provider E2E](../../tests/mock_provider_e2e.rs). | Provider-backed outcomes remain dependent on the external server and model. |
| `cli.help-catalogs` | `VALIDATED` | Provider, benchmark, quality, and built-in help catalogs are exercised through real CLI output and contract tests. | Re-run after changing catalog registration or help behavior. |
| `configuration.resolution` | `VALIDATED` | CLI, environment, and persisted precedence; URL normalization; numeric bounds; provider validation; and embedded-credential rejection are covered by unit and launcher tests. | Re-run configuration and error-contract tests after changing precedence or validation. |
| `provider.protocol` | `VALIDATED` | OpenAI-compatible requests, fresh versus cached catalog reads, SSE assembly, bounded responses, redirects, authorization, and optional capability handling pass the provider tests, the mock-provider E2E boundary, and the recorded live LiteLLM proxy smoke covering authenticated status, discovery, and chat completion. | Live capability support remains provider-, model-, version-, and endpoint-dependent. |
| `provider.presets` | `VALIDATED` | The catalog-derived fixture exercises every registered preset and its baseline contract through [mock-provider E2E](../../tests/mock_provider_e2e.rs); LiteLLM also has a recorded live proxy smoke. | Fixture evidence is not live certification; add live evidence only for an available representative target. |
| `provider.live.representative` | `VALIDATED` | Recorded live boundaries cover Ollama (`qwen3.5:2b`, `qwen3.5:9b`, `nomic-embed-text:latest`) and a disposable LiteLLM proxy, plus earlier llama.cpp interoperability checks. | Do not generalize to unexercised providers, models, versions, hosts, or optional endpoints. |
| `provider.best-effort.live` | `UNVALIDATED` | TGI, text-generation-webui, Jan, and MLX-LM have deterministic preset coverage only. | A live target and compatible model are required before claiming interoperability. |
| `benchmark.llm` | `VALIDATED` | The standard generation, responses, consistency, prompt-size, structured-output, and tool-calling paths passed at the recorded Ollama boundary; a `d512be4` live chat-generation smoke also passes. Unsupported capabilities remain explicit error records. See [benchmarks](user/benchmarks.md). | Broader provider/model combinations remain outside the live claim. |
| `benchmark.embeddings` | `VALIDATED` | Embeddings completed successfully for the recorded Ollama embedding model and a `d512be4` live `nomic-embed-text:latest` smoke, and remains reportable through the standard result path. | Embeddings support is not universal across providers or models. |
| `benchmark.performance` | `PARTIAL` | The 500-request ceiling and above-limit refusal, all profile-owned fixture matrices, the recorded Ollama default profiles, and the recorded matched two-model `smoke` timing matrix (4 warmups plus 12 measured requests, 0 errors, 4 records) pass; unique request IDs, persistence, reports, and privacy checks pass. | Host swap pressure reached a maximum used ratio of `0.971` and triggered the documented warning, so comparative timing and model-ranking claims remain disqualified. LiteLLM interoperability is covered separately by a recorded live non-performance smoke; broader host/provider variance remains open. See [benchmark execution](architecture/benchmark_execution.md). |
| `quality.external-plans` | `VALIDATED` | Quality catalog and dry-run plan generation pass [quality CLI tests](../../tests/quality_cli_tests.rs). | LLMeter does not install or execute external frameworks or publish their scores. |
| `results.persistence` | `VALIDATED` | Schema `3.0`, mandatory identity, atomic JSON/CSV writes, redaction, opt-in previews, distinct run IDs, and interrupted-run cleanup pass [result tests](../../tests/test_results.rs) and [schema tests](../../tests/result_schema_tests.rs). | Older, malformed, or unversioned results are intentionally rejected; in-progress runs are not resumable. |
| `reporting` | `VALIDATED` | Terminal list/show and Markdown/HTML generation reload standard, performance, error, and adversarial result shapes with escaping and privacy checks. A current performance result carrying the swap-pressure summary also reloads through terminal and Markdown report paths. See [report tests](../../tests/test_reporting.rs). | Unsupported or truncated result files fail closed. |
| `lifecycle.local-install` | `VALIDATED` | Install, overwrite refusal, rollback cleanup, update, uninstall, purge, and preservation of unrelated home data pass the lifecycle tests and documented local boundary. | These are local file operations, not a remote updater or package manager. |
| `launcher.powershell` | `VALIDATED` | Release-binary reuse, argument forwarding, fallback selection, noninteractive refusal, ConPTY confirmation, protected-home refusal, and owned cleanup pass [launcher E2E tests](../../tests/launcher_powershell_e2e.rs). | The primary launcher evidence is Windows-specific. |
| `test.local-quality-gates` | `PARTIAL` | At `c58a756`, hosted check, Clippy, serialized tests, and rustdoc pass on all four targets; the `v0.5.1` release workflow at `34f6b48` also passes format, check, Clippy, tests, rustdoc, dependency audit, and package dry run. Local formatting passes, but local compile/test reruns could not fetch the fixed TLS crates. The pre-fix `65d4c8e` local suite passed 147/147 and is not the current dependency boundary. See [testing and quality](coding/testing_and_quality.md). | Re-run the full local gate when crates.io DNS access is available. |
| `release.cross-platform` | `VALIDATED` | The [tag-gated `v0.5.1` workflow](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39661462013) passed all four native build/package jobs, packaged mock-provider checks, checksum generation, provenance attestations, and release upload. The [published release](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.5.1) lists the four platform archives and `SHA256SUMS`; independent download and attestation verification remains outstanding for this version. | Verify downloaded archives and attestations before treating them as independently checked. |
| `release.public-distribution` | `PARTIAL` | GitHub [`v0.5.1`](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.5.1) is published from `main` with four platform archives, `SHA256SUMS`, and provenance attestations. Registry publication and installation are not current claims. | Crates.io publication and a clean registry installation remain owner-gated. |
| `release.next-patch` | `VALIDATED` | The annotated `v0.5.1` tag resolves to main commit `34f6b48`. The initial candidate failed audit on two TLS advisories; after the minimal lockfile fix and explicitly authorized tag replacement, the complete tag-gated validation, four-platform packaging, checksum/attestation, and publication workflow passed. | Continue using the tag-gated release workflow from `main`; registry publication remains separate. |

## Open issues

| ID | Component | Severity | Current boundary | Required action |
|---|---|---|---|---|
| `ISSUE-001` | `release.public-distribution` | Medium | The GitHub `v0.5.1` release is published and the hosted package dry run passes; crates.io publication and installation are not current claims. | With owner approval and a valid registry token, publish from the exact release commit, then verify a clean registry install. |
| `ISSUE-003` | `quality.external-plans` | Low | External quality tools are intentionally planning-only. | Keep execution outside the runtime contract unless a separate adapter and ownership boundary are approved. |

The first `v0.5.1` tag workflow stopped on newly published TLS advisories. After updating the two affected locked crates, explicit authorization to retarget the tag, and a successful full release workflow, `v0.5.1` is published from `main`. The host still cannot resolve `static.crates.io`, so a local post-fix compile/test rerun remains partial; hosted gates pass. Comparative performance and crates.io distribution remain separate boundaries.

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
