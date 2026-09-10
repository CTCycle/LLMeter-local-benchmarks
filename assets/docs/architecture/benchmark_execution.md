# Benchmark execution

## Entry points

Benchmark runs originate from two surfaces:

- `llmeter bench run ...` in scriptable mode.
- `llmeter bench perf ...` in scriptable mode for native performance scenarios.
- `llmeter` or `llmeter bench menu` in interactive mode.

Standard benchmark runs converge in `src/runner.rs`. Native performance scenarios use `src/performance/runner.rs`. These are separate current implementations for different execution responsibilities, not old/new compatibility paths. Both persist the same canonical `BenchmarkRun` shape and reuse the same typed save/report flow.

## Run request

`BenchmarkRunRequest` carries the normalized execution input:

- selected benchmark suite (`llm` or `embeddings`)
- selected model names
- selected benchmark IDs or the `all` flag
- repeated run count
- `max_tokens`
- `temperature`
- extra provider request parameters from repeated `--param key=value`

Defaults come from `AppConfig` when the CLI invocation omits explicit values.

## Validation and planning

`run_benchmarks()` performs these phases in order:

1. Check provider reachability through `/v1/models`.
2. Validate that each requested model is exposed by the selected provider.
3. Load the default benchmark registry.
4. Resolve the selected benchmark set within the requested suite.
5. Build a `BenchmarkContext`.
6. Compute a `BenchmarkExecutionPlan`.

The plan tracks:

- total models
- total benchmarks
- total planned benchmark steps

Planned steps come from each benchmark's `planned_steps()` implementation, which allows progress to reflect repeated runs and prompt variants before execution starts.

The command-level model inventory is refreshed from the provider before validation. Interactive model selection may reuse the client-local catalog cache, but an explicit refresh is available and benchmark/status paths use fresh `/v1/models` reads so measurements do not rely on stale selection data. Model identity is the `/v1/models` `id` field; alternate historical field names are not used as identity fallbacks.

`bench perf` first normalizes a `PerformancePlan` from CLI input:

- profile: `smoke`, `latency`, `throughput`, or `sweep`
- prompt token sizes
- output token sizes
- concurrency levels
- warmup request count
- measured run count
- optional JSONL workload path
- repeated provider request parameters
- load overhead measurement mode and probe count
- capability probe flags
- telemetry level and sample interval
- optional provider process hint and model cache scan path
- dry-run and request budget guard values

Profile defaults are defined by `PerformanceProfile` and consumed by both scriptable planning and the guided workflow. The plan rejects zero runs, zero concurrency, invalid load probe counts, telemetry sampling below 100 ms, oversized prompt/output token requests unless `--allow-large-prompt` is present, and oversized scenario matrices above `--max-requests` unless `--allow-large-matrix` is present. These safety controls are dedicated CLI flags and cannot be passed through provider `--param` values.

Scriptable performance runs print a plan estimate before timed requests begin. The estimate includes selected models, prompt sizes, output sizes, concurrency levels, scenario count, warmup requests, measured requests, total requests, and the active maximum request limit. `--dry-run` prints this estimate and exits before provider load probes, timed requests, saving, or report generation.

## Progress lifecycle

The terminal progress renderer receives a consistent lifecycle:

1. `Validating`
2. `Planning`
3. `Running`
4. `SavingResults`
5. `GeneratingReports`

While running, every benchmark step emits:

- current model
- current benchmark ID and name
- step index and total step count
- optional run index
- optional prompt name
- completed work units vs total work units

For performance runs, progress also covers pre/post scenario work around timed requests:

- capability endpoint probes when explicitly enabled
- per-model load estimate probes when enabled
- per-model metadata and optional cache scan inventory work
- final environment snapshot capture

## Execution model

Standard benchmark execution is serial.

For each selected model:

1. For each selected benchmark:
2. Create a benchmark-scoped progress adapter.
3. Call `benchmark.run(...)`.
4. Append returned `BenchmarkResultRecord` values to the current `BenchmarkRun`.

There is no concurrent scheduling inside the standard benchmark runner. This keeps timing simpler and makes local-machine comparisons more interpretable.

`bench perf` is the canonical concurrent performance runner and uses a bounded Tokio runtime for request concurrency. The runtime issues warmups first, then measured requests for each scenario matrix cell:

- model
- prompt workload or synthetic prompt size
- requested output token size
- concurrency level

Each scenario emits one summary record plus serialized request traces inside record metadata.

Before scenarios, performance runs can capture a provider capability matrix, load overhead estimate, and model inventory metadata/provider-cache-directory total. Each of those steps emits terminal progress. Load overhead is a client-side first-probe minus warm-probe estimate, not true model-load telemetry. Telemetry is disabled by default; standard and detailed telemetry modes sample system state during scenario execution and summarize the collected samples at run finalization.

## Performance scenarios

The built-in performance profiles are:

- `smoke` - conservative verification with concurrency `1`, prompt sizes `128` and `512`, `1` warmup, and `3` measured runs
- `latency` - concurrency `1`, prompt sizes `128`, `512`, and `2048`, `1` warmup, and `5` measured runs
- `throughput` - prompt size `512`, output size `256`, concurrency `1`, `2`, `4`, and `8`, `1` warmup, and `4` measured runs
- `sweep` - prompt sizes `128`, `512`, and `2048`, output sizes `64`, `128`, and `256`, concurrency `1`, `2`, and `4`, `1` warmup, and `3` measured runs

Prompt text generation is deterministic and provider-agnostic. The recorded `estimated_prompt_tokens` field is an approximation, while provider-reported usage remains the source of truth when available.

## Quality planning boundary

`llmeter quality ...` does not execute benchmark frameworks inside Rust. It emits catalog information and dry-run command previews for:

- `lighteval`
- `inspect-ai`
- `lm-eval-harness`
- `swe-bench`

Quality plans are separate planning output and are not persisted through an alternate `BenchmarkRun` representation.

## Error behavior

Fatal failures stop the command when they happen before or outside benchmark execution, such as:

- provider unreachable
- unknown benchmark ID
- benchmark requested outside the active suite
- no selected models
- requested model not exposed by the provider
- unsupported saved-result schema during report loading

Benchmark-level capability failures do not abort the whole run. Instead, individual benchmarks return result records with `error` populated so the run can continue and reports still include the partial outcome.

Last updated: 2026-09-10
