# Project index

Last updated: 2026-09-30

## Purpose

This file is the entry point for the maintained documentation tree. Read it first, then read the [project status ledger](project_status_ledger.md) when the task involves current behavior, validation, release readiness, known issues, or operational risk. Open the smallest topic document that matches the task.

## Current snapshot

- Package version: `0.4.0` on `develop`.
- Persisted result schema: `3.0`; schema identity and run kind are mandatory, and older result shapes are rejected rather than silently normalized.
- Runtime surfaces: standard `llm` and `embeddings` suites, native `bench perf` scenarios, and dry-run external quality plans.
- Provider catalogs may be cached for ordinary interactive navigation; status, model listing, benchmark validation, and measured probes use fresh reads.
- `install`, `update`, and `uninstall` are local convenience file operations, not remote update or signature-verification mechanisms.
- Authorized `v*` tags publish Windows x86-64, GNU/Linux x86-64, macOS Intel, and macOS Apple silicon archives with checksums and provenance attestations. The public `v0.4.0` GitHub release is verified; the first crates.io publication remains owner-gated.

The current working tree also contains an uncommitted Windows ConPTY interruption repair. Local validation is green, while hosted CI for that exact working-tree change is still pending. The ledger records this boundary; it must not be inferred from the public release status.

## Documentation ontology

### Root

- `project_index.md` — entry point and map of the maintained documentation tree.
- `project_status_ledger.md` — canonical current operational status, evidence boundaries, active issues, validation debt, and revalidation triggers.

### Architecture

- `architecture/system_overview.md` — product goals, crate stack, and maintained module structure.
- `architecture/cli_flow.md` — main menu, benchmark workspace, reports workspace, and top-level dispatch.
- `architecture/provider_integration.md` — provider presets, OpenAI-compatible endpoint usage, and provider lifecycle boundaries.
- `architecture/benchmark_execution.md` — benchmark selection, planning, progress phases, and serial orchestration.
- `architecture/result_storage.md` — `BenchmarkRun`, result schema, run IDs, and persisted file shapes.
- `architecture/report_generation.md` — Markdown/HTML aggregation, summary metrics, and saved report generation.

### Coding

- `coding/shared_rules.md` — cross-module scope, ownership, serialization, and CLI-facing rules.
- `coding/rust.md` — Rust edition, crate conventions, data-shape rules, and module layout.
- `coding/testing_and_quality.md` — local and hosted quality gates and test expectations.
- `coding/error_handling.md` — `LLMeterError`, `anyhow`, provider failures, and recoverable benchmark errors.

### Runtime

- `runtime/modes.md` — interactive, scriptable, and report-only workflows.
- `runtime/startup.md` — build, launch, provider prechecks, and startup sequence.
- `runtime/configuration.md` — environment variables, CLI overrides, URL normalization, and `AppConfig`.
- `runtime/deployment.md` — source builds, `cargo install`, prebuilt binaries, platforms, and versioning.
- `runtime/release_checklist.md` — release gates, artifact verification, and trust model.
- `runtime/audit_implementation_plan.md` — durable implementation roadmap and audit decisions.
- `runtime/validation_campaign.md` — ordered validation scope, tier gates, and claim boundaries.
- `runtime/troubleshooting.md` — startup, provider connectivity, empty catalogs, and output-path recovery.

### User

- `user/getting_started.md` — prerequisites, installation, first run, and initial verification.
- `user/provider_setup.md` — provider presets, local servers, base URLs, and model exposure.
- `user/interactive_usage.md` — guided menu behavior and interactive report flows.
- `user/scriptable_usage.md` — non-interactive commands and automation examples.
- `user/benchmarks.md` — benchmark families, metrics, and extension workflow.
- `user/reports_and_results.md` — saved formats, privacy policy, and interpretation guidance.
- `user/troubleshooting.md` — user-facing troubleshooting and saved-output review.

## Source-of-truth rules

- Architecture and user documents describe the intended runtime contract.
- The status ledger is the only current-state catalog. Do not copy a detailed run narrative into it.
- The audit plan records durable implementation decisions and remaining roadmap work.
- The validation campaign records what to exercise, in what order, and what each tier can claim.
- Tests are executable evidence. Hosted CI and public-release links are the evidence for cross-platform and distribution claims.
- Transient validation output is not part of the maintained documentation tree. Do not add dated ledgers, generated reports, provider logs, or copied audit narratives as a parallel documentation tree.

## Maintenance rules

- Read this index and the status ledger before substantial implementation or validation work.
- Update affected topic documents when behavior, architecture, runtime, or UX changes.
- Update the ledger after implementation changes, meaningful validation, regression discovery, issue remediation, or release-state changes.
- Never promote an entry to `VALIDATED` from source inspection or test existence alone; record meaningful execution evidence and its boundary.
- Preserve the distinction between local, hosted, live-provider, fixture, and public-distribution evidence.
- Keep durable documentation sparse. Retain a standalone artifact only when it is required to reproduce or verify a claim and is linked from the canonical document.

## Reading order

1. Read this index.
2. Read `project_status_ledger.md` for current operational state.
3. Open the narrowest architecture, coding, runtime, or user document covering the question.
4. Use `validation_campaign.md` and `release_checklist.md` only when the task concerns validation or distribution.
5. Return here when switching documentation branches.

## Environment rules

- Windows is the default development environment.
- Keep PowerShell and CMD guidance aligned where commands differ.
- Keep runtime guidance aligned with `cargo` workflows and `run_llmeter.ps1`.
