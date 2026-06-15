# Troubleshooting

## Startup command fails immediately

Check:

- the binary path
- the selected provider flag
- the base URL format
- whether the provider server is already running

Use:

```bash
llmeter --provider ollama status
```

If the base URL is supplied without `/v1`, `AppConfig` normalizes it automatically. If the host or port is wrong, status will still fail.

## Provider health checks fail

Typical causes:

- provider server not started
- wrong port
- wrong provider preset
- firewall or local port conflict
- provider server exposes a non-OpenAI-compatible API shape

Use:

```bash
llmeter providers list
llmeter --provider openai-compatible --base-url http://localhost:8000/v1 status
```

## Model catalog is empty

Provider reachability alone is not enough. The provider must also expose at least one model through `/v1/models`.

Use:

```bash
llmeter models
```

If no models are listed:

- load a model in LM Studio
- start `llama-server` with a model for llama.cpp
- pull or create a model in Ollama
- confirm a custom server returns model entries in `/v1/models`

## Benchmark run rejects selected models

This happens when `bench run` names models that are not currently exposed by the provider. Re-run `llmeter models` and align the `--models` value with the live catalog.

## Result or report files are not created

Check:

- `--output-dir` or `LLMETER_OUTPUT_DIR`
- local filesystem permissions
- whether `--export none` or `--report none` was selected

Raw and formatted outputs are saved separately. A run can intentionally save only raw files, only reports, both, or neither.

## Saved JSON cannot be rendered as a report

`llmeter report show` and `llmeter report generate` require a valid `BenchmarkRun` JSON file. If the JSON file was edited manually or truncated, report loading will fail.

## Partial benchmark failures appear in reports

This is expected when a provider or model supports some OpenAI-compatible features but not others. Unsupported endpoints such as `/v1/responses` or `/v1/embeddings` are recorded as per-record errors instead of invalidating the entire run.

Last updated: 2026-06-15
