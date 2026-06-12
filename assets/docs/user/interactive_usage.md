# Interactive usage

## Main menu

```
llmeter
```

The default command opens an interactive menu driven by `inquire` with these options:

1. **Start Ollama server** — starts `ollama serve` if not running.
2. **Stop Ollama server** — stops the tracked server process.
3. **List installed models** — shows local models with size, params, quantization.
4. **Benchmark workspace** — opens the benchmark submenu.
5. **Reports** — opens the report submenu.
6. **Exit** — returns to shell.

## Benchmark workspace

Opened via `llmeter` then selecting option 4, or directly with `llmeter bench menu`.

Options:
1. **View available benchmark tests** — prints the benchmark catalog.
2. **Run a guided benchmark** — walks through model selection, benchmark selection, configuration, execution, and output saving.
3. **View latest result as terminal report** — renders the most recent JSON result via `comrak`.
4. **Generate report from saved result** — select a saved result and output Markdown/HTML.
5. **Back** — returns to main menu.

## Reports workspace

Opened via `llmeter` then selecting option 5, or directly with `llmeter report`.

Options:
1. **List saved result and report files** — shows recent JSON results and generated reports.
2. **View latest result as terminal report** — renders as formatted terminal output.
3. **Generate Markdown or HTML report** — select a result file and output format.
4. **Back** — returns to main menu.

Last updated: 2026-06-12
