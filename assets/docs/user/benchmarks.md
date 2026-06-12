# Benchmarks

## Available tests

| ID | Name | Description |
|---|---|---|
| `generation-latency` | Basic generation latency | Measures wall time, time to first token, Ollama duration fields, and output token throughput for a short prompt. |
| `consistency` | Response consistency | Repeats the same prompt multiple times and reports exact-match ratio and pairwise text similarity. |
| `prompt-sizes` | Prompt size performance | Runs short, medium, and long prompts to compare prompt processing time vs generation time. |

## Benchmark protocol

Every benchmark is a class that follows this protocol:

```python
class MyBenchmark:
    id = "my-benchmark"          # unique string key
    name = "My benchmark"        # human-readable name
    description = "..."          # what it measures

    def run(
        self,
        client: OllamaClient,
        model: str,
        context: BenchmarkContext,
    ) -> list[BenchmarkResultRecord]:
        ...
```

Key types:

- `OllamaClient` — small REST client for the Ollama API.
- `BenchmarkContext` — holds `runs`, `num_predict`, `temperature`, and any extra options.
- `BenchmarkResultRecord` — dataclass with `benchmark_id`, `model`, `run_index`, `prompt_name`, `metrics` (dict), `response_preview`, and optional `error`.

## Adding a benchmark

1. Create a new file in `src/llmeter/benchmarks/`.
2. Implement the class following the protocol above.
3. Import and register it in `src/llmeter/benchmarks/registry.py`:

```python
from llmeter.benchmarks.my_benchmark import MyBenchmark

default_registry().register(MyBenchmark())
```

The benchmark will automatically appear in the catalog and be selectable in both interactive and scriptable modes.

Last updated: 2026-06-12
