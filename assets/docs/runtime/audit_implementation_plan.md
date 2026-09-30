# LLMeter audit implementation plan

Last updated: 2026-09-30

## Purpose and product boundary

This is the durable roadmap for the Rust and CLI audit. It records implementation decisions and remaining work; it does not retain dated validation narratives.

LLMeter is a local, single-user benchmarking CLI. It talks to provider servers started by the user and does not require accounts, roles, multi-tenant isolation, a hosted control plane, a database server, or application-managed provider lifecycles.

The roadmap prioritizes:

- bounded, scriptable local execution;
- honest benchmark measurements and result interpretation;
- safe handling of local configuration, provider responses, and saved output;
- Windows-first development with practical native Linux and macOS coverage;
- reproducible public artifacts only at an authorized release boundary.

## Current baseline

- Package version is 0.5.0 release candidate on develop. The Windows ConPTY interruption repair and integration-fixture cleanup are committed; the full release gate and exact hosted candidate remain pending.
- The focused report, PowerShell launcher, lifecycle, and mock-provider regressions pass against the committed develop line. The complete local release gate and representative live smoke remain required before shipping.
- Hosted CI must certify the exact release-preparation commit across Ubuntu x86-64, Windows x86-64, macOS Intel, and macOS Apple silicon before main is synchronized.
- The public v0.4.0 GitHub release is verified by release run 34574075684. The first crates.io publication and clean registry installation remain owner-gated.
- Current validation is summarized in the [project status ledger](../project_status_ledger.md) and ordered in the [validation campaign](validation_campaign.md).

## Durable decisions

| Area | Decision |
|---|---|
| Product scope | Keep the application local and single-user. Do not add authentication, hosted orchestration, package-manager behavior, or provider process management without an explicit scope change. |
| Configuration | Validate CLI, environment, persisted configuration, URLs, numeric values, and provider identity at construction boundaries. Fail closed rather than silently selecting a fallback. |
| Provider discovery | Permit a client-local cache for ordinary interactive navigation, but require fresh model discovery for status, model listing, benchmark validation, and measured probes. |
| Result contract | Persist strict schema 3.0 with mandatory schema identity and run kind. Reject older, malformed, or unversioned files instead of silently normalizing them. |
| Output safety | Use atomic writes for JSON, CSV, Markdown, HTML, and configuration. Keep response previews opt-in and redact credential-shaped values and secret-named parameters. |
| Measurement semantics | Keep TTFT, inter-chunk timing, ITL, token-usage coverage, and the client-observed first-request estimate distinct. Never present provider-native load or unsupported statistical certainty. |
| Safety limits | Keep performance safety controls separate from provider parameters. The default request ceiling is 500, with explicit opt-ins for larger matrices or prompt sizes. |
| Interruption | Preserve completed results and clean terminal state on cancellation or abrupt termination. Do not claim resumable in-progress runs. |
| Distribution | Treat managed install/update/uninstall as local convenience file operations. Public archives require checksums, provenance, and exact hosted release evidence; crates.io remains a manual gate. |
| External quality | Keep quality integrations at catalog and dry-run plan generation. Do not install or execute external evaluators inside LLMeter. |

## Audit phases

| Phase | Current status | Maintained contract | Primary implementation and evidence |
|---|---|---|---|
| 0. Support and release contract | Complete | Windows Tier 1, GNU/Linux and macOS native release targets, glibc boundary, and explicit distribution limits. | [Supported platforms](../../../SUPPORTED_PLATFORMS.md), [deployment](deployment.md), [release checklist](release_checklist.md). |
| 1. Configuration and input validation | Complete | Fallible persisted configuration, URL/provider validation, numeric bounds, precedence, and diagnostic source context. | [Configuration](configuration.md), CLI/configuration tests, [error handling](../coding/error_handling.md). |
| 2. Provider protocol boundary | Complete | Bounded OpenAI-compatible requests, fresh versus cached catalogs, SSE handling, reserved fields, response limits, redirects, and ephemeral auth. | [Provider integration](../architecture/provider_integration.md), provider tests, [mock-provider E2E](../../../tests/mock_provider_e2e.rs). |
| 3. Output and privacy | Complete | Atomic output, strict result identity, CSV safety, preview policy, redaction, and failure cleanup. | [Result storage](../architecture/result_storage.md), [reports and results](../user/reports_and_results.md), result/report tests. |
| 4. CLI runtime contract | Complete | Non-TTY refusal, clean stdout/stderr separation, typed output choices, menu cancellation, and terminal restoration. | [Modes](modes.md), [CLI flow](../architecture/cli_flow.md), CLI contract and PTY tests. |
| 5. Statistically honest metrics | Complete | Sample counts, nearest-rank percentiles, population standard deviation, failure separation, warmups, and explicit telemetry/load modes. | [Benchmark execution](../architecture/benchmark_execution.md), performance metric tests, [reports and results](../user/reports_and_results.md). |
| 6. Telemetry and concurrency | Complete | Bounded telemetry sampling, interruptible sampler shutdown, bounded concurrency, shared immutable plans, and cancellation checks. | [Benchmark execution](../architecture/benchmark_execution.md), performance runner/configuration tests, resilience and PTY tests. |
| 7. CI and maintainer validation | Implemented; current source recheck pending hosted rerun | Four-target CI, serialized all-target/all-feature gates, release builds, native PTY suites, and exact-candidate release qualification. | [Testing and quality](../coding/testing_and_quality.md), [validation campaign](validation_campaign.md), hosted CI links in the ledger. |
| 8. Lifecycle and distribution | Local lifecycle complete; registry publication pending | Local file-copy lifecycle is explicit; GitHub binary distribution is verified; remote self-update and crates.io publication are not implied. | [Deployment](deployment.md), [release checklist](release_checklist.md), [supported platforms](../../../SUPPORTED_PLATFORMS.md). |

## Remaining work

| Work item | Status and boundary |
|---|---|
| Exact hosted revalidation of the current ConPTY repair | Required after the current source change is committed. The local tree is green; prior hosted CI remains valid only for its recorded candidate. |
| Comparative performance qualification | Open. Repeat the functional performance boundary on a host without material swap pressure before making timing or model-ranking claims. |
| Broader live provider coverage | Open. The registered preset fixture is validated; live certification for unexercised providers requires available, exact provider/model targets. |
| crates.io publication and clean install | Owner-gated and unperformed. Do not document registry installation as available until both publication and clean-root verification pass. |
| MSRV commitment | Deferred. Do not add an MSRV claim or CI lane until a support commitment is approved. |
| External evaluator execution | Out of scope. Expand only with an explicit adapter, process-ownership, dependency, and result contract. |

## Working convention

At the beginning of a continuation:

1. Read this plan, the [project index](../project_index.md), and the [status ledger](../project_status_ledger.md).
2. Run git status and preserve unrelated worktree changes.
3. Select one bounded item from the remaining-work table.
4. Add or update focused tests before broad validation.
5. Update the status ledger after meaningful evidence or a changed boundary.
6. Keep transient validation output outside tracked documentation; durable conclusions belong in the ledger, campaign, contract docs, tests, or release links.
7. Commit coherent implementation slices only when the user requests commit or push cadence.

## Definition of done for the local CLI

The audit is complete for the current local scope when:

- invalid input cannot panic or silently default;
- provider payload boundaries and capability limitations are explicit;
- scriptable execution is safe in pipes and CI;
- saved outputs are atomic, spreadsheet-safe, and privacy-documented;
- benchmark reports disclose sample limitations and telemetry conditions;
- Windows primary-path tests and hosted non-Windows PTY paths pass for the exact candidate;
- deployment documentation matches actual binaries and external provider prerequisites;
- release and registry claims remain separate and evidence-backed.
