# Benchmarks

## Benchmark families

LLMeter currently exposes three benchmark families:

- `llm` - the standard serial benchmark suite for generation, responses, consistency, prompt sizes, structured output, and tool calling
- `embeddings` - the separate embeddings-only suite
- `performance` - native scenario-based latency, throughput, warmup, and concurrency benchmarking through `llmeter bench perf`

| ID | Name | Description |
|---|---|---|
| `chat-generation` | Basic generation latency | Streams `/v1/chat/completions` and measures wall time, TTFT, usage, and throughput. |
| `responses-generation` | Responses API generation | Calls `/v1/responses` when supported. |
| `consistency` | Response consistency | Repeats a prompt and reports exact-match ratio and pairwise text similarity. |
| `prompt-sizes` | Performance across prompt sizes | Runs short, medium, and long prompts. |
| `structured-output` | Structured JSON output | Requests JSON schema output and validates required keys. |
| `tool-calling` | Function/tool calling | Requests a tool call and validates function name and arguments. |
| `embeddings` | Embeddings API | Calls `/v1/embeddings` and reports latency and vector dimensions. |

## Performance profiles

`llmeter bench perf` adds production-oriented performance profiles:

- `smoke` - quick validation with conservative defaults
- `latency` - prompt-size-focused percentile benchmarking at concurrency `1`
- `throughput` - concurrency sweep with fixed prompt/output sizes
- `sweep` - matrix benchmarking across prompt sizes, output sizes, and concurrency levels

Native performance runs record:

- request count, success count, error count, and error rate
- wall time min, mean, max, standard deviation, and p50, p90, p95, and p99
- TTFT min, mean, max, p50, p95, and p99 when streaming is enabled
- generation wall-time percentiles when TTFT exists
- TPOT and ITL percentiles when token timing is available
- requests per second, successful requests per second, and input/output token throughput
- output tokens per second including TTFT and excluding TTFT when generation timing exists
- timeout, HTTP error, provider error, and empty-response counts
- per-request traces, capability probes, load estimates, model inventory, telemetry summaries, and environment snapshots in JSON output

Synthetic prompt sizes are estimates. Provider usage fields remain authoritative when available.

Load overhead is reported as an estimate unless provider-native telemetry exists. Do not interpret it as true model-load time. Non-streaming runs do not report TTFT unless a provider supplies native timing.

## Quality preparation

`llmeter quality` is a planning surface, not a native evaluator in this phase.

Built-in catalog coverage includes:

- `mmlu`
- `gsm8k`
- `arc-challenge`
- `hellaswag`
- `truthfulqa`
- `winogrande`
- `humaneval`
- `swe-bench-lite`
- `swe-bench-verified`
- `swe-bench-full`
- `swe-bench-multilingual`

The command emits dry-run adapter plans for `lighteval`, `inspect-ai`, `lm-eval-harness`, and `swe-bench`.

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

Last updated: 2026-06-18
