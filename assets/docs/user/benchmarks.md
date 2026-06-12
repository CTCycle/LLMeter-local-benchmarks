# Benchmarks

## Available tests

| ID | Name | Description |
|---|---|---|
| `generation-latency` | Basic generation latency | Measures wall time, time to first token, Ollama duration fields, and output token throughput for a short prompt. |
| `consistency` | Response consistency | Repeats the same prompt multiple times and reports exact-match ratio and pairwise text similarity. |
| `prompt-sizes` | Performance across prompt sizes | Runs short, medium, and long prompts to compare prompt processing time vs generation time. |

## Benchmark trait

Every benchmark implements the `Benchmark` trait:

```rust
pub trait Benchmark: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(
        &self,
        client: &OllamaClient,
        model: &str,
        context: &BenchmarkContext,
    ) -> Vec<BenchmarkResultRecord>;
}
```

Key types:

- `OllamaClient` — reqwest-based HTTP client for the Ollama API.
- `BenchmarkContext` — holds `runs`, `num_predict`, `temperature`, and extra options.
- `BenchmarkResultRecord` — struct with `benchmark_id`, `model`, `run_index`, `prompt_name`, `metrics` (HashMap), `response_preview`, and optional `error`.

## Adding a benchmark

1. Create a new file in `src/benchmarks/`.
2. Implement the `Benchmark` trait for your struct.
3. Register it in `src/benchmarks/registry.rs`:

```rust
registry.register(Box::new(MyBenchmark));
```

The benchmark will automatically appear in the catalog and be selectable in both interactive and scriptable modes.

Last updated: 2026-06-12
