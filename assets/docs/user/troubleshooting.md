# Troubleshooting

## Common issues

### Provider is not reachable

Check the selected provider and base URL:

```bash
llmeter providers list
llmeter --provider lmstudio status
llmeter --provider openai-compatible --base-url http://localhost:9000/v1 status
```

Start the provider server externally and confirm its `/v1/models` endpoint is available.

### No models found

Verify that the provider has a loaded or exposed model:

```bash
llmeter models
```

For LM Studio, load a model and start the local server. For llama.cpp, start `llama-server` with a model file. For Ollama, pull or create a model before benchmarking.

### Benchmark records show endpoint errors

Not every provider/model supports every OpenAI-compatible capability. `responses-generation`, `structured-output`, `tool-calling`, and `embeddings` may fail independently. These failures are saved as error records in JSON/CSV and shown in reports.

### Reports are empty or missing data

Check that the JSON result file exists in the output directory and is valid JSON. The output directory defaults to `benchmark_results/`.

## File locations

| Item | Default path |
|---|---|
| Raw JSON results | `benchmark_results/<run-id>.json` |
| CSV exports | `benchmark_results/<run-id>.csv` |
| Markdown reports | `benchmark_results/<run-id>.report.md` |
| HTML reports | `benchmark_results/<run-id>.report.html` |

## Known limitations

- LLMeter does not start or stop provider servers.
- Provider OpenAI compatibility varies by server version and model.
- No concurrent benchmark execution. Each benchmark runs serially.
- Results are machine-specific. Compare runs from the same host under similar load.

Last updated: 2026-06-12
