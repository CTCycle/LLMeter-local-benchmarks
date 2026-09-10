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
| `prompt-sizes` | Performance across prompt sizes | Runs short, medium, and long prompts and compares client-observed end-to-end timing; prompt-processing time is not measured independently. |
| `structured-output` | Structured JSON output | Requests JSON schema output and validates required keys. |
| `tool-calling` | Function/tool calling | Requests a tool call and validates function name and arguments. |
| `embeddings` | Embeddings API | Calls `/v1/embeddings` and reports latency and vector dimensions. |

Benchmark IDs are globally unique. Duplicate registration is treated as an invariant violation rather than silently ignoring one implementation.

## Performance profiles

`llmeter bench perf` adds performance profiles with one canonical set of defaults shared by scriptable and interactive execution:

- `smoke` - prompt sizes `128` and `512`, output size `128`, concurrency `1`, one warmup, and 3 measured runs
- `latency` - prompt sizes `128`, `512`, and `2048`, output size `128`, concurrency `1`, one warmup, and 5 measured runs
- `throughput` - prompt size `512`, output size `256`, concurrency `1`, `2`, `4`, and `8`, one warmup, and 4 measured runs
- `sweep` - prompt sizes `128`, `512`, and `2048`, output sizes `64`, `128`, and `256`, concurrency `1`, `2`, and `4`, one warmup, and 3 measured runs

Native performance runs record:

- request count, success count, error count, and error rate
- wall time min, mean, max, standard deviation, and p50, p90, p95, and p99
- TTFT min, mean, max, p50, p95, and p99 when streaming is enabled
- generation wall-time percentiles when TTFT exists
- inter-chunk latency percentiles from streamed response chunks
- ITL percentiles only when streaming TTFT and provider-reported output usage for at least two tokens are available
- requests per second, successful requests per second, and input/output token throughput
- output tokens per second including TTFT and excluding TTFT when generation timing exists
- performance scenario output throughput is aggregate across the scenario wall time and is shown in the dedicated Performance Summary, not the standard per-request summary column
- timeout, HTTP error, provider error, and empty-response counts
- per-request traces, capability probes, load estimates, model inventory, telemetry summaries, and environment snapshots in JSON output

Synthetic prompt sizes are estimates. Provider usage fields remain authoritative when available.

Load overhead is a client-observed first-request versus warm-request estimate. It does not measure provider restart, cache eviction, model loading, or native lifecycle telemetry. The default load mode is `first-request-estimate`; it can be disabled explicitly. Non-streaming runs do not report TTFT. Telemetry and capability probes are disabled by default.

Performance safety limits are enforced before requests are sent: prompt values above 32,768 estimated tokens or output values above 8,192 require `--allow-large-prompt`, and matrices above the default 500 warmup-plus-measured request budget require `--allow-large-matrix` after review. These controls are dedicated CLI flags, not provider parameters. New saved runs use strict result schema `3.0`.

## Quality preparation

`llmeter quality` is a planning surface, not a native evaluator.

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

The command emits dry-run adapter plans for `lighteval`, `inspect-ai`, `lm-eval-harness`, and `swe-bench`. These plans are not stored through a second result representation.

## Benchmark trait

Every standard benchmark implements the `Benchmark` trait:

```rust
pub trait Benchmark: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn suite(&self) -> BenchmarkSuite;
    fn planned_steps(&self, context: &BenchmarkContext) -> u32;
    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
        progress: &mut dyn BenchmarkProgressSink,
    ) -> Vec<BenchmarkResultRecord>;
}
```

Key types:

- `ProviderClient` - OpenAI-compatible HTTP client.
- `BenchmarkContext` - holds runs, max tokens, temperature, timeout, and extra request parameters.
- `BenchmarkProgressSink` - benchmark-scoped progress reporting boundary.
- `BenchmarkResultRecord` - benchmark ID/name, model, prompt/run metadata, metrics, response preview, error, and metadata.

## Adding a benchmark

1. Create or extend a file in `src/benchmarks/`.
2. Implement `Benchmark`.
3. Register it once in `src/benchmarks/registry.rs` and assign it to the correct suite.

The benchmark appears in both interactive and scriptable flows.

Last updated: 2026-09-10
