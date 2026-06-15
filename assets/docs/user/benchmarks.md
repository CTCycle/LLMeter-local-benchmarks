# Benchmarks

## Available tests

LLMeter exposes two suites:

- `llm` - the standard benchmark suite for generation, responses, consistency, prompt sizes, structured output, and tool calling
- `embeddings` - the separate embeddings-only suite

| ID | Name | Description |
|---|---|---|
| `chat-generation` | Basic generation latency | Streams `/v1/chat/completions` and measures wall time, TTFT, usage, and throughput. |
| `responses-generation` | Responses API generation | Calls `/v1/responses` when supported. |
| `consistency` | Response consistency | Repeats a prompt and reports exact-match ratio and pairwise text similarity. |
| `prompt-sizes` | Performance across prompt sizes | Runs short, medium, and long prompts. |
| `structured-output` | Structured JSON output | Requests JSON schema output and validates required keys. |
| `tool-calling` | Function/tool calling | Requests a tool call and validates function name and arguments. |
| `embeddings` | Embeddings API | Calls `/v1/embeddings` and reports latency and vector dimensions. |

## Benchmark trait

Every benchmark implements the `Benchmark` trait:

```rust
pub trait Benchmark: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
    ) -> Vec<BenchmarkResultRecord>;
}
```

Key types:

- `ProviderClient` - OpenAI-compatible HTTP client.
- `BenchmarkContext` - holds runs, max tokens, temperature, timeout, and extra request parameters.
- `BenchmarkResultRecord` - benchmark ID/name, model, prompt/run metadata, metrics, response preview, error, and metadata.

## Adding a benchmark

1. Create or extend a file in `src/benchmarks/`.
2. Implement `Benchmark`.
3. Register it in `src/benchmarks/registry.rs` and assign it to the correct suite.

The benchmark appears in both interactive and scriptable flows.

Last updated: 2026-06-15
