# Interactive usage

## Main menu

```bash
llmeter
```

The default command opens an interactive menu with these options:

1. **Provider setup** - shows current provider status, provider presets and compatibility tiers, capability probing, and default provider persistence.
2. **Model inventory** - lists or refreshes exposed models, shows raw model metadata, and optionally estimates local cache footprint after a path is provided.
3. **Benchmark workspace** - opens benchmark selection and execution.
4. **Reports and comparisons** - opens saved result/report workflows.
5. **Help and examples** - prints built-in help topics.
6. **Exit** - returns to shell.

The provider is selected before launch with `--provider` and optional `--base-url`.

The model inventory keeps a client-local catalog cache for ordinary listing and metadata navigation. Use **Refresh exposed models** after loading or unloading a provider model; benchmark validation and measured/status paths independently refresh the provider catalog.

## Benchmark workspace

Opened via `llmeter` then selecting option 3, or directly with `llmeter bench menu`.

Options:

1. **Quick benchmark** - runs a conservative smoke-style performance plan with low run counts and Markdown reporting by default.
2. **Performance benchmark** - guides provider, probe depth, models, profile, warmups, measured runs, streaming, and report choices, then shows a plan preview before execution.
3. **Standard LLM benchmark** - runs the existing guided LLM benchmark suite.
4. **Embeddings benchmark** - runs the existing guided embeddings suite.
5. **Quality benchmark plan** - shows the quality catalog and points to scriptable dry-run planning.
6. **View benchmark catalog** - prints the benchmark catalog.
7. **View latest result as terminal report** - renders a saved JSON result.
8. **Generate report from saved result** - outputs Markdown and/or HTML.
9. **Back** - returns to main menu.

During execution, LLMeter shows a terminal progress bar for these phases:
- validation
- run planning
- benchmark step execution
- raw result saving
- report generation

Capability probing under provider setup and performance benchmarking reports each endpoint check during validation, so basic and full probes stay visible while `/v1/models`, chat, streaming, embeddings, or responses checks run.

The model inventory cache-footprint flow also reports metadata fetches and optional cache scans, and interactive report generation reuses the same terminal progress renderer as scriptable output generation.

Performance safety limits are shown during plan construction, before timed requests begin. The quality workspace is intentionally a dry-run planning surface for external evaluators.

## Reports workspace

Opened via `llmeter` then selecting option 4.

Options:

1. **List saved result and report files**.
2. **View latest result as terminal report**.
3. **Generate Markdown or HTML report**.
4. **Back**.

Last updated: 2026-08-02
