# Scriptable usage

## Subcommand reference

| Command | Description |
|---|---|
| `llmeter status` | Show selected provider status. |
| `llmeter providers list` | List provider presets and default base URLs. |
| `llmeter providers set <provider>` | Persist the default provider for future runs. |
| `llmeter models` | List models exposed by the selected provider. |
| `llmeter show <model>` | Show model metadata from `/v1/models`. |
| `llmeter bench list [--suite <suite>]` | List available benchmarks, optionally filtered by suite. |
| `llmeter bench run [options]` | Run benchmarks non-interactively with live progress output. |
| `llmeter bench perf [options]` | Run native performance scenarios with warmups and concurrency sweeps. |
| `llmeter report list` | List saved result and report files. |
| `llmeter report show [result]` | Render a saved JSON result as a terminal report. |
| `llmeter report generate [result]` | Generate Markdown and/or HTML reports. |
| `llmeter quality list` | List quality benchmark catalog entries. |
| `llmeter quality plan [options]` | Print a dry-run external quality benchmark plan as JSON. |
| `llmeter help [topic]` | Show built-in help. |
| `llmeter /help [topic]` | Built-in help alias. |

## Automation patterns

Run everything against all models exposed by Ollama:

```bash
llmeter --provider ollama bench run --suite llm --models all --benchmarks all --export both --report both
```

`bench run` now reports validation, planning, current benchmark step, and completion percentage while the run is in progress.

Run selected capability benchmarks against LM Studio:

```bash
llmeter --provider lmstudio bench run \
  --suite llm \
  --models all \
  --benchmarks chat-generation,structured-output,tool-calling \
  --runs 5 \
  --max-tokens 256 \
  --temperature 0.2 \
  --export json \
  --report both
```

Use a custom llama.cpp URL:

```bash
llmeter --provider llama-cpp --base-url http://localhost:8081/v1 bench run \
  --suite embeddings \
  --models all \
  --benchmarks all
```

Custom output directory:

```bash
llmeter --output-dir ./ci-runs bench run --models all --benchmarks all
```

Run native performance smoke checks:

```bash
llmeter --provider ollama bench perf --models all --profile smoke --export both --report both
```

Run an explicit latency profile on Windows PowerShell:

```powershell
llmeter bench perf --models llama3.1 --profile latency --concurrency 1 --warmup 1 --runs 3
```

Run a full matrix sweep:

```bash
llmeter bench perf --models llama3.1 --profile sweep --prompt-tokens 128,512 --output-tokens 64,128 --concurrency 1,2
```

Preview external quality commands without installing tools automatically:

```bash
llmeter quality list
llmeter quality plan --framework lighteval --task "leaderboard|mmlu|5" --model llama3.1
llmeter quality plan --framework swe-bench --task swe-bench-lite --model llama3.1
```

## CI integration

Start the provider server before invoking LLMeter. LLMeter exits with a non-zero code on fatal errors.

Example:

```yaml
- name: Run local LLM benchmarks
  run: ./llmeter --provider ollama bench run --suite llm --models all --benchmarks all --export json --report md
```

Last updated: 2026-06-15
