# LLMeter validation campaign

Last updated: 2026-10-21

## Purpose and authority

This document is the compact, repeatable validation plan for the local single-user CLI. It defines campaign order, evidence boundaries, and claim limits; it is not a dated run ledger.

- The [project status ledger](../project_status_ledger.md) is the canonical current-state summary, including evidence boundaries, durable decisions, and validation debt.
- Architecture and user documents define intended behavior.
- Tests are executable evidence. Hosted CI and public-release links are the evidence for native platform and distribution claims.
- Validation output is transient. Do not create a second tracked documentation tree of dated ledgers, generated reports, provider logs, or copied audit narratives.

## Campaign rules

- Validate the current source revision and record its exact boundary. A local result does not certify a different hosted or released revision.
- Use the official launcher where launcher behavior is in scope, isolated `LLMETER_HOME`, configuration, output, and Cargo target roots, and serialized tests because configuration tests mutate process environment variables.
- For live performance checks, use the documented 120-second timeout, `--load-measurement off` when comparing runs, explicit `--max-requests`, raw and report outputs, and response previews disabled.
- The default performance ceiling is 500 warmup-plus-measured requests. Verify the adjacent above-limit refusal before using a larger matrix.
- Treat provider failures and unsupported capabilities as recorded outcomes. Do not substitute a different provider, model, compiler, or platform for the requested evidence.
- Treat swap pressure, host contention, provider availability, and model stalls as evidence boundaries. They may invalidate timing or coverage claims without proving an application defect.
- Keep fixture, local live-provider, hosted, and public-distribution evidence separate.
- `llmeter quality` remains dry-run planning; no external evaluator is installed or executed by this campaign.

## Ordered tier gates

| Tier | Scope | Current disposition | Durable evidence anchor | Claim boundary |
|---|---|---|---|---|
| T0 | Repository identity, local quality gates, startup, configuration, non-TTY behavior, and the official Windows launcher. | PASS in the four-platform hosted CI matrix at `c58a756` and the tag-gated release validation at `34f6b48` (format, check, Clippy, tests, rustdoc, audit, package dry run); local compile/test reruns after the TLS lock refresh were blocked by crate-download DNS failures. | [CI run at c58a756](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39658572919), [release workflow at 34f6b48](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39661462013), [CLI contract tests](../../../tests/cli_contract_tests.rs), [launcher E2E](../../../tests/launcher_powershell_e2e.rs), and the current ledger. | Provider-backed behavior still needs an externally running provider. |
| T1 | Interactive menu, catalogs, provider transport, result persistence, reports, and local lifecycle operations. | PASS at the recorded Windows, fixture, and hosted Unix PTY boundaries; the current `c58a756` four-platform CI run passes. | [CI run at c58a756](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39658572919), [PTY tests](../../../tests/pty_menu_e2e.rs), [mock-provider E2E](../../../tests/mock_provider_e2e.rs), [result tests](../../../tests/test_results.rs), [report tests](../../../tests/test_reporting.rs). | Windows ConPTY and deterministic provider evidence do not certify every terminal or live provider. |
| T2 | Standard `llm` and `embeddings` workflows, including multi-model orchestration. | PASS at the recorded Ollama boundary and the `d512be4` chat-generation and embeddings smokes. | [Benchmark execution](../architecture/benchmark_execution.md), [benchmarks](../user/benchmarks.md), and the ledger's live boundary. | Other providers, models, and unsupported endpoint combinations remain outside the live claim. |
| T3 | Performance planning, request accounting, profile execution, metrics, persistence, reports, privacy, and representative live runs. | PARTIAL: the ceiling, fixture matrices, recorded Ollama profiles, matched two-model matrix, reload, privacy, and trace identity pass; the recorded matrix reached a 0.971 maximum swap-used ratio, so comparative timing is not qualified. | [Performance CLI tests](../../../tests/performance_cli_tests.rs), [performance metrics tests](../../../tests/performance_metrics_tests.rs), [resilience E2E](../../../tests/resilience_e2e.rs), and the ledger. | Swap pressure prevents numeric timing or model-ranking claims. A disposable LiteLLM proxy passes a recorded live non-performance smoke. |
| T4 | Provider preset contracts, representative provider interoperability, quality planning, and public distribution. | PASS at the exact recorded provider and published `v0.5.1` GitHub release boundaries, with the LiteLLM proxy smoke and packaged mock-provider checks in all four native build jobs. | [Release workflow at 34f6b48](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39661462013), [Mock-provider E2E](../../../tests/mock_provider_e2e.rs), [supported platforms](../../../SUPPORTED_PLATFORMS.md), [release checklist](release_checklist.md), and the ledger's release boundary. | Fixture coverage is not universal live-provider certification. External quality scores and registry installation are not claimed. |
| T5 | Controlled failure, interruption, completed-result durability, restart/recovery, request ceiling, and native PTY behavior. | PASS at the exact `v0.5.1` release boundary and `c58a756` hosted boundary; both four-platform CI runs passed, including Unix PTY coverage. | [Release workflow at 34f6b48](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39661462013), [CI run at c58a756](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/39658572919), [Resilience E2E](../../../tests/resilience_e2e.rs), [Windows PTY tests](../../../tests/pty_menu_e2e.rs), [Unix PTY tests](../../../tests/pty_menu_unix_e2e.rs), and the CI workflow. | This does not claim resumable in-progress runs, production-scale stress beyond the configured ceiling, or every unsupported terminal. |

## Evidence rules and local gates

Run the local gates defined in [testing and quality](../coding/testing_and_quality.md) from the repository root, using an isolated target directory when the shared target tree is locked. For a release, also run the audit, dependency-tree, package, extracted-binary, and hosted workflow checks in the [release checklist](release_checklist.md).

## Claim exclusions

This campaign does not certify:

- universal provider compatibility or live support for unexercised presets;
- model quality, clinical correctness, or external benchmark scores;
- hardware-independent performance rankings when host pressure is reported;
- checkpoint-based resumption or stress beyond the configured request ceiling;
- secure remote self-update, crates.io availability, or a clean registry install before those gates are explicitly completed.
