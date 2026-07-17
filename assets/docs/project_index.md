# Project index
Last updated: 2026-07-01

## Purpose
This file is the root index for `assets/docs`. Read it first, then open the smallest topic file that matches the task.

## How To Navigate
1. Start with this file only.
2. Choose the topic branch that matches the task.
3. Open the narrowest leaf document that answers the question.
4. Expand to sibling files only when the task clearly crosses topic boundaries.
5. Keep documentation updates aligned with implementation changes.

## Naming Rules
- All files and folders under `assets/docs` use lower-case names.
- Topic folders group related leaf documents by subject.
- Root-level files are reserved for entry-point documents only.

## Documentation Ontology
### Root
- `project_index.md`
  - Entry point and master index for the full documentation tree.

### Architecture
- `architecture/system_overview.md`
  - Product goals, crate stack, and maintained module structure.
- `architecture/cli_flow.md`
  - Main menu, benchmark workspace, reports workspace, and top-level command dispatch.
- `architecture/provider_integration.md`
  - Provider presets, OpenAI-compatible endpoint usage, and provider lifecycle boundaries.
- `architecture/benchmark_execution.md`
  - Benchmark selection, execution planning, progress phases, and serial run orchestration.
- `architecture/result_storage.md`
  - `BenchmarkRun`, result-record schema, run IDs, and persisted raw/report file shapes.
- `architecture/report_generation.md`
  - Markdown and HTML report aggregation, summary metrics, and saved report generation flow.

### Coding
- `coding/shared_rules.md`
  - Cross-module rules for scope, ownership, serialization, and CLI-facing behavior.
- `coding/rust.md`
  - Rust edition, crate conventions, data-shape rules, and module layout.
- `coding/testing_and_quality.md`
  - Test expectations, `cargo fmt`, `clippy`, and CI quality gates.
- `coding/error_handling.md`
  - `LLMeterError`, `anyhow`, provider-failure handling, and recoverable benchmark error rules.

### Runtime
- `runtime/modes.md`
  - Interactive menu mode, scriptable command mode, and report-only workflows.
- `runtime/startup.md`
  - Build, launch, provider prechecks, and the expected startup sequence for local benchmarking.
- `runtime/configuration.md`
  - Environment variables, CLI overrides, base-URL normalization, and `AppConfig`.
- `runtime/deployment.md`
  - Source builds, `cargo install`, prebuilt binaries, platforms, and versioning.
- `runtime/release_checklist.md`
  - Maintainer checklist for tagging, validation, checksummed artifacts, and release trust model.
- `runtime/audit_implementation_plan.md`
  - Durable implementation roadmap for the Rust/CLI audit, scoped to the local single-user product.
- `runtime/troubleshooting.md`
  - Startup failures, provider connectivity, empty model catalogs, and output-path recovery.

### User
- `user/getting_started.md`
  - Prerequisites, installation, first-run flow, and initial verification steps.
- `user/provider_setup.md`
  - Provider preset selection, local server expectations, base URLs, and model exposure checks.
- `user/interactive_usage.md`
  - Main menu behavior, guided benchmark execution, and interactive report flows.
- `user/scriptable_usage.md`
  - Non-interactive subcommands, automation patterns, and CI-oriented execution.
- `user/benchmarks.md`
  - Available benchmarks, the benchmark trait model, and extension workflow.
- `user/reports_and_results.md`
  - Output files, report generation, terminal summaries, and how to inspect saved runs.
- `user/troubleshooting.md`
  - User-facing troubleshooting for providers, benchmark failures, and saved output review.

## Reading Order
1. Read this root index.
2. Open the smallest leaf file that covers the current question.
3. Expand to adjacent files only when the task crosses topic boundaries.
4. Return here when switching branches.

## Context Rules
- Read documentation files only when required by the active task.
- Defer reading until the task proves the file is needed.
- Keep all affected documents updated whenever behavior, architecture, runtime, or UX changes.
- Always include a `Last updated: YYYY-MM-DD` line when modifying a document.
- Pre-select files to read by folder structure and task intent before opening them.

## Environment Rules
- Windows is the default operating environment for this repository.
- Support both PowerShell and CMD guidance where commands differ.
- Keep runtime guidance aligned with `cargo` workflows and `run_llmeter.ps1`.
