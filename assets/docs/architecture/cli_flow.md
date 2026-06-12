# CLI flow

## Main menu

1. Start Ollama server.
2. Stop Ollama server.
3. List installed models.
4. Benchmark workspace.
5. Reports.
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

## Entry point

The `main()` function in `src/main.rs` parses arguments with `clap` and dispatches to the appropriate command handler or interactive menu via `llmeter::ui::main_menu()`.

Last updated: 2026-06-12
