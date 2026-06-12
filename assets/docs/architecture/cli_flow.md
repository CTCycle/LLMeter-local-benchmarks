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
2. Run a guided benchmark.
3. View latest result as terminal report.
4. Generate report from saved result.
5. Back.

## Reports workspace

1. List saved result and report files.
2. View latest result as terminal report.
3. Generate Markdown or HTML report.
4. Back.

## Scriptable entry point

`src/main.rs` parses arguments with `clap`, builds `AppConfig`, creates a `ProviderClient`, and dispatches to provider, model, benchmark, report, menu, or built-in help commands.

Last updated: 2026-06-12
