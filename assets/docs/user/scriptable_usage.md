# Scriptable usage

## Subcommand reference

| Command | Description |
|---|---|
| `llmeter status` | Show selected provider status. |
| `llmeter providers list` | List provider presets and default base URLs. |
| `llmeter models` | List models exposed by the selected provider. |
| `llmeter show <model>` | Show model metadata from `/v1/models`. |
| `llmeter bench list` | List available benchmarks. |
| `llmeter bench run [options]` | Run benchmarks non-interactively. |
| `llmeter report list` | List saved result and report files. |
| `llmeter report show [result]` | Render a saved JSON result as a terminal report. |
| `llmeter report generate [result]` | Generate Markdown and/or HTML reports. |
| `llmeter help [topic]` | Show built-in help. |
| `llmeter /help [topic]` | Built-in help alias. |

## Automation patterns

Run everything against all models exposed by Ollama:

```bash
llmeter --provider ollama bench run --models all --benchmarks all --export both --report both
```

Run selected capability benchmarks against LM Studio:

```bash
llmeter --provider lmstudio bench run \
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
  --models all \
  --benchmarks chat-generation,prompt-sizes,embeddings
```

Custom output directory:

```bash
llmeter --output-dir ./ci-runs bench run --models all --benchmarks all
```

## CI integration

Start the provider server before invoking LLMeter. LLMeter exits with a non-zero code on fatal errors.

Example:

```yaml
- name: Run local LLM benchmarks
  run: ./llmeter --provider ollama bench run --models all --benchmarks all --export json --report md
```

Last updated: 2026-06-12
