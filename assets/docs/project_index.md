# Project index

This is the entry point for the LLMeter document ontology.

## Navigation

1. Start here.
2. Pick a topic branch below.
3. Open the narrowest leaf file that matches what you need.
4. Expand into additional leaf files only when the answer requires crossing branches.
5. Return here to jump to a different topic.

## File catalog

### `architecture/`

| File | Summary |
|---|---|
| `architecture/system_overview.md` | Design goals, Rust technology stack, and module map. |
| `architecture/cli_flow.md` | Main menu, benchmark workspace, and reports workspace flows. |
| `architecture/provider_integration.md` | Provider presets, OpenAI-compatible API endpoints, and lifecycle policy. |
| `architecture/result_storage.md` | BenchmarkRun model, result records, and JSON/CSV/HTML/MD file formats. |

### `coding/`

| File | Summary |
|---|---|
| `coding/rust.md` | Rust edition, crate conventions, `cargo fmt`, `clippy`, and module structure. |
| `coding/testing_and_quality.md` | `cargo test`, `cargo clippy`, and CI quality gates. |

### `runtime/`

| File | Summary |
|---|---|
| `runtime/configuration.md` | Environment variables (`LLMETER_*`), `OLLAMA_HOST`, and the `AppConfig` struct. |
| `runtime/deployment.md` | `cargo install`, prebuilt binaries, musl targets, and versioning. |

### `user/`

| File | Summary |
|---|---|
| `user/getting_started.md` | Requirements, installation steps, and first-run walkthrough. |
| `user/interactive_usage.md` | Main menu, guided benchmark runs, and interactive report flows. |
| `user/scriptable_usage.md` | Subcommand reference, automation patterns, and CI integration. |
| `user/benchmarks.md` | Available benchmark tests, the `Benchmark` trait, and how to add new ones. |
| `user/troubleshooting.md` | Common issues, FAQ, and file locations. |

## Reading discipline

- Read this index first.
- Read one leaf file at a time and expand only when crossing branches.
- Keep the index updated when files are added, renamed, moved, or deleted.

## Context rules

- Only read ontology docs when the task requires them.
- Prefer leaf files over this index once you know the branch.
- Keep ontology documents in sync with implementation changes.
- Every file carries a `Last updated` date — use it to gauge freshness.

## Environment

- Default development OS: Windows.
- Documentation covers both PowerShell and CMD variants where applicable.
- Align with the launcher scripts in the project root.

Last updated: 2026-06-12
