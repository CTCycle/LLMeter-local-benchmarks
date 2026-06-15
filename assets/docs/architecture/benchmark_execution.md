# Benchmark execution

## Entry points

Benchmark runs originate from two surfaces:

- `llmeter bench run ...` in scriptable mode.
- `llmeter` or `llmeter bench menu` in interactive mode.

Both flows converge in `src/runner.rs` so selection, validation, progress, persistence, and report generation use the same execution path.

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

## Execution model

Execution is serial.

For each selected model:

1. For each selected benchmark:
2. Create a benchmark-scoped progress adapter.
3. Call `benchmark.run(...)`.
4. Append returned `BenchmarkResultRecord` values to the current `BenchmarkRun`.

There is no concurrent benchmark scheduling. This keeps timing simpler and makes local-machine comparisons more interpretable.

## Error behavior

Fatal failures stop the command when they happen before or outside benchmark execution, such as:

- provider unreachable
- unknown benchmark ID
- benchmark requested outside the active suite
- no selected models
- requested model not exposed by the provider

Benchmark-level capability failures do not abort the whole run. Instead, individual benchmarks return result records with `error` populated so the run can continue and reports still include the partial outcome.

Last updated: 2026-06-15
