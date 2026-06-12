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

## AppConfig dataclass

The `AppConfig` class in `config.py` reads these environment variables at instantiation time using `os.getenv()` with defaults. The dataclass uses `field(default_factory=...)` so that environment variables are evaluated once per instance, not once at import time.

Properties:

- `host` — Ollama host URL.
- `timeout` — float, HTTP timeout.
- `output_dir` — `Path`, output directory.
- `state_dir` — `Path`, state directory.
- `default_runs` — int, default repetitions.
- `default_num_predict` — int, default token cap.
- `default_temperature` — float, default temperature.
- `api_base_url` — derived property that ensures a trailing `/api` path.

## CLI overrides

Every config field can be overridden at runtime via CLI flags (`--host`, `--timeout`, `--output-dir`, `--runs`, `--num-predict`, `--temperature`). Flag values take precedence over environment variables.

Last updated: 2026-06-12
