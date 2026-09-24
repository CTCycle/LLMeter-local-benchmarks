# Tier 2 live standard benchmark evidence

Last updated: 2026-09-24

## Boundary

| Field | Value |
|---|---|
| Tested source revision | `c7438db98812b70c20738ddcd4821d27dab837a3` |
| Environment | Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Provider | Ollama at `http://localhost:11434/v1` |
| Fresh catalog | `status` and `models --json` succeeded; 8 models were exposed. |
| Evidence type | Real LLMeter CLI processes against the live provider. Output directories were isolated under this QA folder. |

The selected workflows use `qwen3.5:2b` for generation and `nomic-embed-text:latest` for embeddings. They validate only the selected models and this Ollama endpoint.

The fresh discovery commands were `target\debug\llmeter.exe --provider ollama status` and `target\debug\llmeter.exe --provider ollama models --json`; both exited 0.

## LLM standard suite

Command:

```powershell
$env:LLMETER_OUTPUT_DIR='assets/QA/validation-2026-09-24/artifacts/ollama-llm'
target\debug\llmeter.exe --provider ollama bench run --suite llm --models qwen3.5:2b --benchmarks all --runs 1 --max-tokens 24 --temperature 0.2 --export json --report both
```

Exit code was 0. The CLI ran all six LLM benchmark IDs in order and persisted 8 records, all successful:

| Benchmark | Records | Result |
|---|---:|---|
| `chat-generation` | 1 | Passed streaming chat generation. |
| `responses-generation` | 1 | Passed `/v1/responses`. |
| `consistency` | 1 aggregate | Passed; its consistency scenario issued repeated requests. |
| `prompt-sizes` | 3 | Short, medium, and long prompt scenarios passed. |
| `structured-output` | 1 | Passed structured JSON output. |
| `tool-calling` | 1 | Passed tool-call request and validation. |

Evidence files: [saved JSON](artifacts/ollama-llm/2026-09-24T065438.759140Z-p33076-qwen3.5-2b.json), [Markdown report](artifacts/ollama-llm/2026-09-24T065438.759140Z-p33076-qwen3.5-2b.report.md), and [HTML report](artifacts/ollama-llm/2026-09-24T065438.759140Z-p33076-qwen3.5-2b.report.html).

## Embeddings suite

Command:

```powershell
$env:LLMETER_OUTPUT_DIR='assets/QA/validation-2026-09-24/artifacts/ollama-embeddings'
target\debug\llmeter.exe --provider ollama bench run --suite embeddings --models nomic-embed-text:latest --benchmarks embeddings --runs 1 --export json --report both
```

Exit code was 0. The single record succeeded on `/v1/embeddings` and returned one 768-dimensional vector. Evidence: [saved JSON](artifacts/ollama-embeddings/2026-09-24T065508.366164Z-p26740-nomic-embed-text-latest.json), [Markdown report](artifacts/ollama-embeddings/2026-09-24T065508.366164Z-p26740-nomic-embed-text-latest.report.md), and [HTML report](artifacts/ollama-embeddings/2026-09-24T065508.366164Z-p26740-nomic-embed-text-latest.report.html).

## Multi-model and multi-benchmark orchestration

The CLI ran `chat-generation,structured-output` for both `qwen3.5:2b` and `nomic-embed-text:latest`:

```powershell
$env:LLMETER_OUTPUT_DIR='assets/QA/validation-2026-09-24/artifacts/ollama-multi-model'
target\debug\llmeter.exe --provider ollama bench run --suite llm --models qwen3.5:2b,nomic-embed-text:latest --benchmarks chat-generation,structured-output --runs 1 --max-tokens 8 --temperature 0.2 --export json --report both
```

It exited 0 and persisted four records. Both Qwen records succeeded. The embeddings-only model returned two controlled HTTP 400 records stating that it does not support chat. The error rows remained visible in the saved report; this is the expected capability boundary for that model, not a successful capability claim.

Evidence: [saved JSON](artifacts/ollama-multi-model/2026-09-24T065701.701308Z-p7336-qwen3.5-2b-nomic-embed-text-latest.json), [Markdown report](artifacts/ollama-multi-model/2026-09-24T065701.701308Z-p7336-qwen3.5-2b-nomic-embed-text-latest.report.md), and [HTML report](artifacts/ollama-multi-model/2026-09-24T065701.701308Z-p7336-qwen3.5-2b-nomic-embed-text-latest.report.html).

The standard and multi-model runs omitted response previews by default and set the persisted redaction marker. The evidence files contain benchmark metadata, metrics, reports, and the two provider error records; they do not include response previews.

## Status and limits

Tier 2 is `PASS` for the standard benchmark paths and orchestration exercised against this Ollama endpoint. The live path supplied positive evidence for every standard benchmark family and retained controlled unsupported records for a model that does not support chat.

This does not certify every Ollama model, TGI, text-generation-webui, Jan, MLX-LM, other provider presets, cross-provider capability behavior, or non-Windows execution. Those remain separate Tier 4/provider gates.
