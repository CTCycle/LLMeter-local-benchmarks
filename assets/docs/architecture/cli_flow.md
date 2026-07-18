# CLI flow

## Main menu

1. Provider setup.
2. Model inventory.
3. Benchmark workspace.
4. Reports and comparisons.
5. Help and examples.
6. Exit.

## Benchmark workspace

1. Quick benchmark.
2. Performance benchmark.
3. Standard LLM benchmark.
4. Embeddings benchmark.
5. Quality benchmark plan.
6. View benchmark catalog.
7. View latest result as terminal report.
8. Generate report from saved result.
9. Back.

## Reports workspace

1. List saved result and report files.
2. View latest result as terminal report.
3. Generate Markdown or HTML report.
4. Back.

## Scriptable entry point

`src/main.rs` parses arguments with `clap`, rejects interactive dispatch when stdin or stdout is not a terminal, builds `AppConfig`, resolves the effective provider for the active command, creates a `ProviderClient`, and dispatches to provider, model, benchmark, report, install/update/uninstall lifecycle, menu, or built-in help commands. CLI export/report formats are typed `ValueEnum` values and the displayed version is derived from Cargo package metadata.

`src/runner.rs` owns benchmark orchestration, work-unit planning, and shared progress reporting so interactive and non-interactive benchmark runs use the same terminal progress lifecycle.

Interactive menus use explicit `MenuAction` values rather than display labels or one-based numeric contracts. Only an Enter key press selects; Enter releases/repeats are ignored, Escape/Left returns from a workspace, and Ctrl+C restores the terminal before exiting.

Last updated: 2026-07-18
