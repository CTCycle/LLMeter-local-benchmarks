# LLMeter validation campaign

Last updated: 2026-09-30

## Purpose and authority

This document is the compact, repeatable validation plan for the local single-user CLI. It defines campaign order, evidence boundaries, and claim limits; it is not a dated run ledger.

- The [project status ledger](../project_status_ledger.md) is the canonical current-state summary.
- Architecture and user documents define intended behavior.
- Tests are executable evidence. Hosted CI and public-release links are the evidence for native platform and distribution claims.
- Validation output is transient. Do not create a second tracked documentation tree of dated ledgers, generated reports, provider logs, or copied audit narratives.

## Campaign rules

- Validate the current source revision and record its exact boundary. A local result does not certify a different hosted or released revision.
- Use the official launcher where launcher behavior is in scope, isolated LLMETER_HOME, configuration, output, and Cargo target roots, and serialized tests because configuration tests mutate process environment variables.
- For live performance checks, use the documented 120-second timeout, --load-measurement off when comparing runs, explicit --max-requests, raw and report outputs, and response previews disabled.
- The default performance ceiling is 500 warmup-plus-measured requests. Verify the adjacent above-limit refusal before using a larger matrix.
- Treat provider failures and unsupported capabilities as recorded outcomes. Do not substitute a different provider, model, compiler, or platform for the requested evidence.
- Treat swap pressure, host contention, provider availability, and model stalls as evidence boundaries. They may invalidate timing or coverage claims without proving an application defect.
- Keep fixture, local live-provider, hosted, and public-distribution evidence separate.
- llmeter quality remains dry-run planning; no external evaluator is installed or executed by this campaign.

## Ordered tier gates

| Tier | Scope | Current disposition | Durable evidence anchor | Claim boundary |
|---|---|---|---|---|
| T0 | Repository identity, local quality gates, startup, configuration, non-TTY behavior, and the official Windows launcher. | PASS at the recorded local boundary. | [CLI contract tests](../../../tests/cli_contract_tests.rs), [launcher E2E](../../../tests/launcher_powershell_e2e.rs), and the current ledger. | Provider-backed behavior still needs an externally running provider. |
| T1 | Interactive menu, catalogs, provider transport, result persistence, reports, and local lifecycle operations. | PASS at the recorded Windows and fixture boundaries. | [PTY tests](../../../tests/pty_menu_e2e.rs), [mock-provider E2E](../../../tests/mock_provider_e2e.rs), [result tests](../../../tests/test_results.rs), [report tests](../../../tests/test_reporting.rs). | Windows ConPTY and deterministic provider evidence do not certify every terminal or live provider. |
| T2 | Standard llm and embeddings workflows, including multi-model orchestration. | PASS at the recorded Ollama boundary. | [benchmark execution](../architecture/benchmark_execution.md), [benchmarks](../user/benchmarks.md), and the ledger's live boundary. | Other providers, models, and unsupported endpoint combinations remain outside the live claim. |
| T3 | Performance planning, request accounting, profile execution, metrics, persistence, reports, privacy, and representative live runs. | PARTIAL: the ceiling, fixture matrices, current Ollama profiles, matched functional matrix, reload, privacy, and trace identity pass; comparative timing is not qualified. | [performance CLI tests](../../../tests/performance_cli_tests.rs), [performance metrics tests](../../../tests/performance_metrics_tests.rs), [resilience E2E](../../../tests/resilience_e2e.rs), and the ledger. | Swap pressure prevents numeric timing or model-ranking claims; current LiteLLM smoke is unavailable. |
| T4 | Provider preset contracts, representative provider interoperability, quality planning, and public distribution. | PASS at the exact recorded provider and GitHub release boundaries; crates.io remains separate. | [mock-provider E2E](../../../tests/mock_provider_e2e.rs), [supported platforms](../../../SUPPORTED_PLATFORMS.md), [release checklist](release_checklist.md), and hosted release run [34574075684](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/34574075684). | Fixture coverage is not universal live-provider certification. External quality scores and registry installation are not claimed. |
| T5 | Controlled failure, interruption, completed-result durability, restart/recovery, request ceiling, and native PTY behavior. | PASS at the finite release boundary for the exact hosted candidate; the current uncommitted ConPTY repair is locally revalidated but needs hosted rerun. | [resilience E2E](../../../tests/resilience_e2e.rs), [Windows PTY tests](../../../tests/pty_menu_e2e.rs), [Unix PTY tests](../../../tests/pty_menu_unix_e2e.rs), and hosted CI run [36722865383](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36722865383). | This does not claim resumable in-progress runs, production-scale stress beyond the configured ceiling, or every unsupported terminal. |

## Current release boundary

The current checkout is 6ef50aa on develop and contains an uncommitted Windows ConPTY interrupt monitor plus performance-run cancellation checks. Local formatting, locked check, Clippy, rustdoc, release build, audit/tree inspection, 156 serialized tests, Windows ConPTY 10/10, and a one-request Ollama smoke pass. Hosted CI run 36722865383 covers the earlier exact candidate 1fb3dbd1f3211e25280481f5bbdab439a850a2f9, not this uncommitted change.

The live performance boundary is functional rather than comparative: current Ollama qwen3.5:2b completed default latency, throughput, and sweep profiles with 15, 16, and 81 measured requests, and the matched qwen3.5:2b/qwen3.5:9b matrix completed 80 measured requests. Swap warnings disqualify timing comparisons. LiteLLM was not available because the Docker engine was unreachable.

The public v0.4.0 GitHub release is verified with four archives, checksums, packaged tests, and provenance. The first crates.io publication and clean registry install were not attempted.

## Evidence map

| Concept | Executable anchors | What may be claimed |
|---|---|---|
| Startup and configuration | cli_contract_tests.rs, config_validation.rs, launcher E2E | Invalid configuration fails closed, non-TTY behavior is explicit, and provider/base-URL precedence follows the documented contract. |
| Provider protocol and presets | provider tests, mock_provider_e2e.rs | The registered preset matrix and bounded OpenAI-compatible fixture contract pass; live capability claims remain scoped. |
| Standard benchmarks | benchmark tests, mock-provider E2E, recorded Ollama run boundary | Standard and embeddings suites preserve per-record errors and complete at the exercised provider/model boundary. |
| Performance planning | performance_cli_tests.rs, performance_metrics_tests.rs, mock-provider E2E | Profile defaults, accounting, 500-request safety, metrics, and persistence are deterministic at the fixture boundary. |
| Persistence and reports | test_results.rs, result_schema_tests.rs, test_reporting.rs | Schema 3.0, atomic output, redaction, preview policy, reload, and report generation are covered. |
| Interruption and recovery | resilience_e2e.rs, platform PTY suites | Completed results survive abrupt termination and interruption without convincing partial output, with fresh-process recovery at the finite boundary. |
| Cross-platform distribution | .github/workflows/ci.yml, release workflow, hosted run links | Native build and release claims apply only to the exact hosted candidate and published artifacts. |
| Quality planning | quality_cli_tests.rs | External framework catalog and dry-run plans are validated; external execution is out of scope. |

## Required local gates

Run from the repository root. Use an isolated target directory when the shared target tree is locked.

```
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features -- --test-threads=1
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --locked --no-deps --all-features
cargo build --locked --release --all-features
```

For a release candidate, also run the audit, dependency-tree, package, extracted-binary, and hosted workflow checks in the [release checklist](release_checklist.md).

## Change-to-validation map

| Changed area | Minimum follow-up |
|---|---|
| Providers, presets, catalog cache, auth, or probes | Provider unit/probe tests, mock-provider E2E, and an available live status/model smoke. |
| CLI parsing, startup, launcher, menus, or prompts | CLI contracts, Windows PTY E2E, and launcher smoke; hosted Unix PTY when the candidate changes terminal behavior. |
| Benchmark registry, request construction, runner, or semantics | Affected benchmark tests, provider fixture E2E, persistence, and report smoke. |
| Performance plans, telemetry, metrics, concurrency, or cancellation | Performance CLI/metrics tests, request-ceiling/refusal E2E, resilience E2E, PTY suites, and a representative live scenario. |
| Result schema, persistence, redaction, or reporting | Schema, result-store, reporting, mock saved-run, reload, and privacy checks. |
| CI, packaging, version, release workflow, or support claims | Local gates, exact hosted matrix, artifact checksums/provenance, and the release checklist. |

## Claim exclusions

This campaign does not certify:

- universal provider compatibility or live support for unexercised presets;
- model quality, clinical correctness, or external benchmark scores;
- hardware-independent performance rankings when host pressure is reported;
- checkpoint-based resumption or stress beyond the configured request ceiling;
- secure remote self-update, crates.io availability, or a clean registry install before those gates are explicitly completed.
