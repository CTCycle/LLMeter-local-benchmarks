# Interactive usage

## Main menu

```bash
llmeter
```

The default command opens an interactive menu with these options:

1. **List providers** - shows supported provider presets and default URLs.
2. **List models** - queries `/v1/models` from the selected provider.
3. **Benchmark workspace** - opens benchmark selection and execution.
4. **Reports** - opens saved result/report workflows.
5. **Help** - prints built-in help topics.
6. **Exit** - returns to shell.

The provider is selected before launch with `--provider` and optional `--base-url`.

## Benchmark workspace

Opened via `llmeter` then selecting option 3, or directly with `llmeter bench menu`.

Options:

1. **View available benchmark tests** - prints the benchmark catalog.
2. **Run a guided benchmark** - walks through model selection, benchmark selection, run count, token cap, temperature, and output saving, then shows live progress with current model, benchmark, run context, and completion percentage.
3. **View latest result as terminal report** - renders a saved JSON result.
4. **Generate report from saved result** - outputs Markdown and/or HTML.
5. **Back** - returns to main menu.

During execution, LLMeter shows a terminal progress bar for these phases:
- validation
- run planning
- benchmark step execution
- raw result saving
- report generation

## Reports workspace

Opened via `llmeter` then selecting option 4.

Options:

1. **List saved result and report files**.
2. **View latest result as terminal report**.
3. **Generate Markdown or HTML report**.
4. **Back**.

Last updated: 2026-06-15
