# Configuration

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `LLMETER_HOME` | `%USERPROFILE%\\.llmeter` on Windows, `~/.llmeter` on Unix | Root directory for LLMeter state, including persisted config and default outputs. |
| `LLMETER_PROVIDER` | `ollama` | Provider preset: `ollama`, `lmstudio`, `llama-cpp`, or `openai-compatible`. |
| `LLMETER_BASE_URL` | provider default | OpenAI-compatible `/v1` base URL override. |
| `OLLAMA_HOST` | unset | Ollama host compatibility value, mapped to `<OLLAMA_HOST>/v1`. |
| `LMSTUDIO_BASE_URL` | unset | LM Studio base URL override. |
| `LLAMA_CPP_BASE_URL` | unset | llama.cpp base URL override. |
| `LLMETER_TIMEOUT` | `120` | HTTP request timeout in seconds. |
| `LLMETER_OUTPUT_DIR` | `<LLMETER_HOME>/benchmark_results` | Directory for result and report files. Relative or absolute values are used as provided. |
| `LLMETER_CONFIG_DIR` | `<LLMETER_HOME>/config` | Override the directory that stores `config.json`. |
| `LLMETER_RUNS` | `3` | Default repeated runs per benchmark. |
| `LLMETER_MAX_TOKENS` | `128` | Default generation output token cap. |
| `LLMETER_TEMPERATURE` | `0.2` | Default sampling temperature. |

## AppConfig struct

`AppConfig` in `src/config.rs` resolves the effective provider in this order:

1. command-specific CLI provider override such as `bench run --provider`
2. top-level `--provider`
3. `LLMETER_PROVIDER`
4. persisted user config from `<LLMETER_HOME>/config` unless `LLMETER_CONFIG_DIR` overrides it
5. built-in default `ollama`

Base URL resolution stays aligned to the resolved provider unless `--base-url` is explicitly set.

Fields:

- `provider` - selected provider preset.
- `base_url` - normalized OpenAI-compatible `/v1` base URL.
- `timeout` - HTTP timeout in seconds.
- `output_dir` - directory for results and reports, defaulting to `<LLMETER_HOME>/benchmark_results`.
- `default_runs` - default repetitions.
- `default_max_tokens` - default output token cap.
- `default_temperature` - default sampling temperature.

## CLI overrides

Use `--provider`, `--base-url`, `--timeout`, `--output-dir`, `--runs`, `--max-tokens`, and `--temperature`.

Persist a global default provider with:

```bash
llmeter providers set ollama
```

By default this writes `config.json` under `<LLMETER_HOME>/config/`.

Last updated: 2026-06-21
