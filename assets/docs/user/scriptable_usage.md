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
| `llmeter bench perf [options]` | Run native performance scenarios with warmups, load estimates, endpoint probe progress, telemetry, and concurrency sweeps. |
| `llmeter bench performance [options]` | Visible alias for `llmeter bench perf`. |
| `llmeter report list` | List saved result and report files. |
| `llmeter report show [result]` | Render a saved JSON result as a terminal report. |
| `llmeter report generate [result]` | Generate Markdown and/or HTML reports. |
| `llmeter quality list` | List quality benchmark catalog entries. |
| `llmeter quality plan [options]` | Print a dry-run external quality benchmark plan as JSON. |
| `llmeter install [--force]` | Install a managed CLI copy into `<LLMETER_HOME>/bin`. |
| `llmeter update [--source <exe>]` | Refresh the managed CLI copy from a newer executable. |
| `llmeter uninstall [--purge-home]` | Remove the managed CLI copy and optionally all LLMeter home data. |
| `llmeter help [topic]` | Show built-in help. |
| `llmeter /help [topic]` | Built-in help alias. |

## Automation patterns

Running `llmeter` or `llmeter menu` without a terminal is a usage error: the CLI prints help to standard output and exits with code `2`. Scripted callers should always select a subcommand. Benchmark progress is written to standard error so standard output remains separate from progress rendering.

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

Install the managed CLI copy for `cmd.exe` use:

```cmd
llmeter install
```

Update it from a newer downloaded binary:

```cmd
llmeter update --source C:\downloads\llmeter.exe
```

Remove it:

```cmd
llmeter uninstall
```

Run native performance smoke checks:

```bash
llmeter --provider ollama bench perf --models all --profile smoke --export both --report both
```

Disable streaming when TTFT is not needed or the provider streaming endpoint is unreliable:

```bash
llmeter --provider ollama bench perf --models all --profile smoke --runs 1 --no-stream
```

Add capability probing, client-side load overhead estimation, and detailed telemetry:

```bash
llmeter --provider ollama bench performance --models all --profile latency --runs 3 --warmup 1 --probe-capabilities --load-measurement cold-warm-estimate --telemetry full --report both --export both
```

When capability probing is enabled, LLMeter reports each validation step before timed scenarios begin so long provider checks remain visible in terminal output.

Load-estimate probes, model inventory metadata/cache scans, and `llmeter report generate ...` now also emit progress updates instead of staying silent until completion.

Load overhead is reported as an estimate unless provider-native telemetry is available. Model cache scanning is opt-in with `--scan-model-cache` and never downloads or mutates model files.

Run an explicit latency profile on Windows PowerShell:

```powershell
llmeter bench perf --models llama3.1 --profile latency --concurrency 1 --warmup 1 --runs 3
```

Run a full matrix sweep:

```bash
llmeter bench perf --models llama3.1 --profile sweep --prompt-tokens 128,512 --output-tokens 64,128 --concurrency 1,2
```

Preview a performance matrix before sending requests:

```bash
llmeter bench perf --models llama3.1 --profile sweep --prompt-tokens 128,512 --output-tokens 64,128 --concurrency 1,2 --dry-run
```

`bench perf` prints models, prompt sizes, output sizes, concurrency levels, scenario count, warmup requests, measured requests, total requests, and the active request limit before execution. The default `--max-requests` value is `500`; larger matrices must reduce the matrix, raise `--max-requests`, or pass `--param unsafe_large_matrix=true`.

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

Last updated: 2026-07-18
