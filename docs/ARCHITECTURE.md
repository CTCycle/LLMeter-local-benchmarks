# Architecture

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
src/ollama_bench/
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

## CLI flow

### Main menu

1. Start Ollama server.
2. Stop Ollama server.
3. List installed models.
4. Benchmark workspace.
5. Reports.
6. Exit.

### Benchmark workspace

1. View available benchmark tests.
2. Run a guided benchmark.
3. View latest result as terminal report.
4. Generate report from saved result.
5. Back.

### Reports workspace

1. List saved result and report files.
2. View latest result as terminal report.
3. Generate Markdown or HTML report.
4. Back.

## Ollama API interaction

`OllamaClient` talks to the local Ollama API using these endpoints:

| Endpoint | Use |
|---|---|
| `GET /api/version` | Health check and API version. |
| `GET /api/tags` | Installed local model list. |
| `POST /api/show` | Model metadata. |
| `POST /api/generate` | Text generation and benchmark measurements. |
| `GET /api/ps` | Running models, available for later UI expansion. |

The latency benchmark uses streaming generation to measure client-side time to first token. The final Ollama response contains the duration fields used for throughput and prompt processing metrics when available.

## Server lifecycle

`OllamaServerManager` checks whether `ollama` exists on `PATH`, checks the API health endpoint, and starts `ollama serve` when requested.

When this CLI starts the server, it stores the PID in:

```text
~/.ollama-bench/ollama-server.pid.json
```

By default, `ollama-bench server stop` stops only that tracked process. This is safer than killing a server started by another terminal or by the user’s operating system service manager. `--force` can be used for broader process termination.

## Benchmark registration

Benchmarks implement the small protocol in `benchmarks/base.py`:

```python
id: str
name: str
description: str
run(client, model, context) -> list[BenchmarkResultRecord]
```

The default registry lives in `benchmarks/registry.py`. Adding a benchmark requires one new module and one registry entry.

## Result storage

Each benchmark run becomes one `BenchmarkRun` object:

```text
run_id
created_at
models
benchmark_ids
config
results[]
```

Each result record contains:

```text
benchmark_id
benchmark_name
model
run_index
prompt_name
metrics
response_preview
error
metadata
```

Raw files are saved as JSON and CSV. Formatted reports are generated as Markdown and HTML from the same JSON-compatible data model.

## Error handling

The runner validates that requested models are installed locally before benchmark execution. Per-benchmark execution catches individual benchmark errors and stores them as result records so one failed model or prompt does not destroy the whole run.

Fatal errors are reserved for conditions that make execution impossible:

- Ollama executable not found.
- Ollama server unavailable and not started.
- No selected models.
- Selected model missing.
- Invalid CLI option format.
