# Configuration

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama server base URL. Not prefixed with `LLMETER` because it is an Ollama standard variable. |
| `LLMETER_TIMEOUT` | `120` | HTTP request timeout in seconds. |
| `LLMETER_OUTPUT_DIR` | `benchmark_results` | Directory for result and report files. |
| `LLMETER_STATE_DIR` | `~/.llmeter` | Directory for runtime state (PID file, server logs). |
| `LLMETER_RUNS` | `3` | Default number of repeated runs per benchmark. |
| `LLMETER_NUM_PREDICT` | `128` | Default `num_predict` option passed to Ollama. |
| `LLMETER_TEMPERATURE` | `0.2` | Default temperature option passed to Ollama. |

## AppConfig struct

The `AppConfig` struct in `config.rs` reads these environment variables at instantiation time using `std::env::var()` with defaults. CLI flag values (parsed by `clap`) override the environment variables.

Fields:

- `host` — `String`, Ollama host URL.
- `timeout` — `f64`, HTTP timeout in seconds.
- `output_dir` — `PathBuf`, output directory for results.
- `state_dir` — `PathBuf`, state directory for PID and logs.
- `default_runs` — `u32`, default repetitions.
- `default_num_predict` — `u32`, default token cap.
- `default_temperature` — `f64`, default temperature.
- `api_base_url` — derived `String`, ensures a trailing `/api` path.

## CLI overrides

Every config field can be overridden at runtime via CLI flags (`--host`, `--timeout`, `--output-dir`, `--runs`, `--num-predict`, `--temperature`). Flag values take precedence over environment variables.

Last updated: 2026-06-12
