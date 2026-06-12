# Configuration

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `LLMETER_PROVIDER` | `ollama` | Provider preset: `ollama`, `lmstudio`, `llama-cpp`, or `openai-compatible`. |
| `LLMETER_BASE_URL` | provider default | OpenAI-compatible `/v1` base URL override. |
| `OLLAMA_HOST` | unset | Ollama host compatibility value, mapped to `<OLLAMA_HOST>/v1`. |
| `LMSTUDIO_BASE_URL` | unset | LM Studio base URL override. |
| `LLAMA_CPP_BASE_URL` | unset | llama.cpp base URL override. |
| `LLMETER_TIMEOUT` | `120` | HTTP request timeout in seconds. |
| `LLMETER_OUTPUT_DIR` | `benchmark_results` | Directory for result and report files. |
| `LLMETER_RUNS` | `3` | Default repeated runs per benchmark. |
| `LLMETER_MAX_TOKENS` | `128` | Default generation output token cap. |
| `LLMETER_TEMPERATURE` | `0.2` | Default sampling temperature. |

## AppConfig struct

`AppConfig` in `src/config.rs` reads environment variables at instantiation time. CLI flag values override environment variables.

Fields:

- `provider` - selected provider preset.
- `base_url` - normalized OpenAI-compatible `/v1` base URL.
- `timeout` - HTTP timeout in seconds.
- `output_dir` - directory for results and reports.
- `default_runs` - default repetitions.
- `default_max_tokens` - default output token cap.
- `default_temperature` - default sampling temperature.

## CLI overrides

Use `--provider`, `--base-url`, `--timeout`, `--output-dir`, `--runs`, `--max-tokens`, and `--temperature`.

Last updated: 2026-06-12
