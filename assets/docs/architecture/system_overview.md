# System overview

## Design goals

The MVP is a focused Ollama-only CLI. It does not introduce a provider abstraction because the first useful version should be easy to read, easy to run, and easy to extend.

Primary goals:

1. Python 3.14 first.
2. Interactive by default, scriptable when needed.
3. Minimal runtime dependency set.
4. Clear result persistence.
5. Modular benchmark registration.
6. Good local error handling.
7. Useful Markdown and HTML reports.

## Stack

| Area | Choice | Reason |
|---|---|---|
| Language | Python 3.14 | Current target requested for the project. |
| CLI parser | `argparse` | Standard library, stable, enough for scriptable commands. |
| Interactive UI | `rich` | Polished terminal tables, panels, prompts, and Markdown rendering with one dependency. |
| HTTP client | `urllib.request` | Standard library, sufficient for local Ollama HTTP calls. |
| Persistence | JSON and CSV | Portable and easy to inspect. |
| Reports | Markdown and HTML | Good first reporting formats for GitHub and browsers. |
| Tests | `pytest` | Small, common, pragmatic. |

## Module map

```text
src/llmeter/
  cli.py                 Main entrypoint. Owns argparse commands and interactive flows.
  ui.py                  Rich output, menus, prompts, tables, terminal report rendering.
  reporting.py           Aggregated summaries plus Markdown and HTML report generation.
  results.py             BenchmarkRun model and JSON or CSV persistence.
  runner.py              Validates selected models and executes selected benchmark classes.
  config.py              Environment-backed defaults.
  errors.py              Project-specific exceptions.
  prompts.py             Built-in prompts used by benchmark tests.
  ollama/
    client.py            Ollama REST API client.
    server.py            Detects Ollama install, checks server, starts and stops tracked server.
  benchmarks/
    base.py              BenchmarkContext, BenchmarkResultRecord, Benchmark protocol.
    registry.py          Default benchmark registry.
    generation.py        Basic generation latency benchmark.
    consistency.py       Response consistency benchmark.
    prompt_sizes.py      Prompt size benchmark.
    metrics.py           Metrics and text similarity helpers.
```

Last updated: 2026-06-12
