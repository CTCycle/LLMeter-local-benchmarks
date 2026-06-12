# Ollama Local Bench

A Python 3.14 interactive CLI for benchmarking locally installed Ollama models.

The project is intentionally small. It supports Ollama only, uses the local Ollama REST API, and keeps the benchmark system modular so new tests can be added without rewriting the CLI.

## Current MVP features

- Modern interactive terminal menu built with Rich.
- Scriptable subcommands for automation and CI usage.
- Ollama installation and server status checks.
- Start `ollama serve` when the server is not running.
- Stop only the server process started by this CLI, unless `--force` is used.
- List installed local Ollama models.
- Show model metadata from Ollama.
- Run one, many, or all benchmark tests against one or many models.
- Save raw benchmark results as JSON and CSV.
- Generate formatted Markdown and HTML reports.
- Render saved benchmark results as a readable terminal report.

## Requirements

- Python 3.14+
- Ollama installed and available on `PATH`
- At least one local Ollama model, for example:

```bash
ollama pull llama3.2
```

## Installation

```bash
git clone <your-new-repo-url>
cd ollama-local-bench
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -e '.[dev]'
```

On Windows PowerShell:

```powershell
py -3.14 -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
python -m pip install -e ".[dev]"
```

## Interactive usage

Open the main menu:

```bash
ollama-bench
```

Open the benchmark workspace directly:

```bash
ollama-bench bench menu
```

Open the report workspace directly:

```bash
ollama-bench report
```

## Scriptable usage

Check status:

```bash
ollama-bench status
```

Start Ollama if needed:

```bash
ollama-bench server start
```

List installed models:

```bash
ollama-bench models
```

Show model metadata:

```bash
ollama-bench show llama3.2
```

List benchmark tests:

```bash
ollama-bench bench list
```

Run all benchmarks against all installed models and generate raw plus formatted outputs:

```bash
ollama-bench bench run --models all --benchmarks all --start-server
```

Run selected benchmarks:

```bash
ollama-bench bench run \
  --models llama3.2,mistral \
  --benchmarks generation-latency,prompt-sizes \
  --runs 3 \
  --num-predict 128 \
  --temperature 0.2 \
  --export both \
  --report both
```

Generate a report from the latest saved JSON result:

```bash
ollama-bench report generate --format both
```

Show the latest saved result as a terminal report:

```bash
ollama-bench report show
```

## Benchmark tests included

| ID | Purpose |
|---|---|
| `generation-latency` | Measures wall time, time to first token, Ollama duration fields, and output token throughput. |
| `consistency` | Repeats the same prompt and reports exact-match and pairwise text similarity. |
| `prompt-sizes` | Runs short, medium, and long prompts to compare prompt processing and generation timing. |

## Output files

By default files are saved to:

```text
benchmark_results/
```

Generated outputs can include:

```text
<run-id>.json
<run-id>.csv
<run-id>.report.md
<run-id>.report.html
```

Change the output directory with:

```bash
ollama-bench --output-dir ./runs bench run --models all --benchmarks all
```

Or set:

```bash
export OLLAMA_BENCH_OUTPUT_DIR=./runs
```

## Configuration environment variables

| Variable | Default |
|---|---:|
| `OLLAMA_HOST` | `http://localhost:11434` |
| `OLLAMA_BENCH_TIMEOUT` | `120` |
| `OLLAMA_BENCH_OUTPUT_DIR` | `benchmark_results` |
| `OLLAMA_BENCH_STATE_DIR` | `~/.ollama-bench` |
| `OLLAMA_BENCH_RUNS` | `3` |
| `OLLAMA_BENCH_NUM_PREDICT` | `128` |
| `OLLAMA_BENCH_TEMPERATURE` | `0.2` |

## Architecture

```text
src/ollama_bench/
  cli.py                 argparse entrypoint and interactive menu orchestration
  ui.py                  Rich tables, panels, prompts, and terminal report rendering
  reporting.py           Markdown and HTML report generation
  results.py             JSON and CSV persistence
  runner.py              benchmark execution orchestration
  prompts.py             built-in benchmark prompts
  ollama/
    client.py            small Ollama REST client
    server.py            install detection plus server start and stop logic
  benchmarks/
    base.py              benchmark protocol and result dataclasses
    registry.py          benchmark registration
    generation.py        latency benchmark
    consistency.py       consistency benchmark
    prompt_sizes.py      short, medium, and long prompt benchmark
    metrics.py           metric helpers
```

## Adding a benchmark

Create a class with the benchmark interface:

```python
from ollama_bench.benchmarks.base import BenchmarkContext, BenchmarkResultRecord
from ollama_bench.ollama.client import OllamaClient

class MyBenchmark:
    id = "my-benchmark"
    name = "My benchmark"
    description = "What this benchmark measures."

    def run(self, client: OllamaClient, model: str, context: BenchmarkContext) -> list[BenchmarkResultRecord]:
        ...
```

Register it in `src/ollama_bench/benchmarks/registry.py`.

## Development

Run tests:

```bash
python -m pytest -q
```

Run the CLI without installing:

```bash
PYTHONPATH=src python -m ollama_bench
```

## Current limitations

- Ollama only.
- No async execution yet.
- No multi-host comparison yet.
- HTML reports are intentionally minimal and dependency-light.
- Real benchmarks are machine-specific and should be compared only on the same host under comparable load.
