# Scriptable usage

## Subcommand reference

| Command | Description |
|---|---|
| `llmeter status` | Show Ollama installation and server status. |
| `llmeter server status` | Show server status (same as above). |
| `llmeter server start` | Start `ollama serve` if not running. |
| `llmeter server stop` | Stop tracked server process. Add `--force` for broad termination. |
| `llmeter models` | List installed local models. Add `--json` for raw output. |
| `llmeter show <model>` | Show model metadata from Ollama. |
| `llmeter bench list` | List available benchmarks. |
| `llmeter bench run [options]` | Run benchmarks non-interactively. |
| `llmeter report list` | List saved result and report files. |
| `llmeter report show [result]` | Render a saved JSON result as a terminal report. |
| `llmeter report generate [result]` | Generate Markdown and/or HTML reports. |

## Automation patterns

Run everything against all models:

```bash
llmeter bench run --models all --benchmarks all --start-server --export both --report both
```

Run specific benchmarks with custom settings:

```bash
llmeter bench run \
  --models llama3.2,mistral \
  --benchmarks generation-latency,prompt-sizes \
  --runs 5 \
  --num-predict 256 \
  --temperature 0.5 \
  --export json \
  --report both
```

Custom output directory:

```bash
llmeter --output-dir ./ci-runs bench run --models all --benchmarks all
```

## CI integration

Use `--start-server` to let LLMeter manage the Ollama server lifecycle in CI. The command exits with a non-zero code on fatal errors, making it suitable for CI pipelines.

Example GitHub Actions step (using prebuilt binary):

```yaml
- name: Download llmeter
  run: curl -Lo llmeter https://github.com/.../releases/latest/download/llmeter-linux-x86_64 && chmod +x llmeter
- name: Run benchmarks
  run: ./llmeter bench run --models all --benchmarks all --start-server --export json --report md
```

Last updated: 2026-06-12
