# LLMeter

LLMeter is a self-contained Rust CLI for benchmarking local OpenAI-compatible LLM providers.

It works with provider servers that expose `/v1` APIs, including Ollama, LM Studio, llama.cpp, and custom OpenAI-compatible local endpoints.

## Highlights

- Provider presets for `ollama`, `lmstudio`, `llama-cpp`, and `openai-compatible`.
- Interactive terminal menu plus scriptable subcommands.
- Built-in benchmarks for chat generation, responses, prompt sizes, consistency, structured JSON output, tool/function calling, and embeddings.
- Raw JSON/CSV exports plus Markdown/HTML reports.
- Built-in `help` and `/help` command alias.
- No Python, Node.js, virtualenv, or runtime service dependency.

LLMeter does not start or stop provider servers. Start Ollama, LM Studio, llama.cpp, or your custom local server separately, then point LLMeter at its `/v1` base URL.

## Quick Start

```bash
llmeter providers list
llmeter --provider ollama status
llmeter --provider ollama models
llmeter --provider ollama bench run --models all --benchmarks all
```

Open the interactive menu:

```bash
llmeter
```

## Documentation

- [User Manual](USER_MANUAL.md) - installation, provider setup, command reference, benchmark usage, reports, and troubleshooting.
- [Project Docs](assets/docs/project_index.md) - architecture, runtime configuration, coding standards, and user docs used by maintainers.

## Development

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Last updated: 2026-06-12
