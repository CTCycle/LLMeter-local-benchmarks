# CLI flow

## Main menu

1. List providers.
2. List models.
3. Benchmark workspace.
4. Reports.
5. Help.
6. Exit.

## Benchmark workspace

1. View available benchmark tests.
2. Run a guided benchmark that first resolves the provider for that run, then selects the benchmark suite (`llm` or `embeddings`), and then shows only matching models and benchmarks.
3. View latest result as terminal report.
4. Generate report from saved result.
5. Back.

## Reports workspace

1. List saved result and report files.
2. View latest result as terminal report.
3. Generate Markdown or HTML report.
4. Back.

## Scriptable entry point

`src/main.rs` parses arguments with `clap`, builds `AppConfig`, resolves the effective provider for the active command, creates a `ProviderClient`, and dispatches to provider, model, benchmark, report, menu, or built-in help commands.

`src/runner.rs` owns benchmark orchestration, work-unit planning, and shared progress reporting so interactive and non-interactive benchmark runs use the same terminal progress lifecycle.

Last updated: 2026-06-15
