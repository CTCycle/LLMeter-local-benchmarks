# T3/T4 live default performance and capability evidence

Last updated: 2026-09-29

## Boundary

| Field | Value |
|---|---|
| Validation source revision | `e65948cbc53d2acb9e507d24b3411193a74adcce` on `develop` |
| Environment | Windows x86-64, PowerShell, Rust `1.98.0`, Cargo `1.98.0` |
| Provider | Ollama `0.34.0`, `http://localhost:11434/v1` |
| Model | `qwen3.5:2b` |
| Provider setup | Pre-existing local Ollama process and model; no binary, model, container, PATH, or startup entry was added by this validation. |
| Evidence boundary | Full default `latency` and `throughput` profiles for one Ollama model, plus one explicit capability-probe smoke. This is not cross-provider or statistical certification. |

The final provider status recheck returned API reachable `yes` and five
exposed models. The selected model was present in the fresh catalog. The
profiles used `--max-requests 64`, `--load-measurement off`, standard
telemetry with a 1000 ms sample interval, both raw export formats, both report
formats, and detailed report output.

## Live default profiles

| Profile | Run ID | Scenarios | Warmup | Measured traces | Successful | Errors | Telemetry samples | Result |
|---|---|---:|---:|---:|---:|---:|---:|---|
| `latency` | `2026-09-29T120338.390264Z-p9896-qwen3.5-2b` | 3 (`128/512/2048` prompt, `128` output, concurrency `1`) | 3 | 15 | 15 | 0 | 104 | PASS |
| `throughput` | `2026-09-29T120654.678161Z-p18524-qwen3.5-2b` | 4 (`512` prompt, `256` output, concurrency `1/2/4/8`) | 4 | 16 | 16 | 0 | 127 | PASS |

Each saved result is schema `3.0`, records provider `ollama`, base URL
`http://localhost:11434/v1`, model `qwen3.5:2b`, complete request traces, and
the expected environment snapshot. No non-empty response preview, API key,
authorization token, or credential-like value was found in the generated
JSON, CSV, Markdown, or HTML artifacts.

Artifacts:

- `latency`: [JSON](artifacts/ollama-default-profiles/latency/2026-09-29T120338.390264Z-p9896-qwen3.5-2b.json), [CSV](artifacts/ollama-default-profiles/latency/2026-09-29T120338.390264Z-p9896-qwen3.5-2b.csv), [Markdown](artifacts/ollama-default-profiles/latency/2026-09-29T120338.390264Z-p9896-qwen3.5-2b.report.md), [HTML](artifacts/ollama-default-profiles/latency/2026-09-29T120338.390264Z-p9896-qwen3.5-2b.report.html)
- `throughput`: [JSON](artifacts/ollama-default-profiles/throughput/2026-09-29T120654.678161Z-p18524-qwen3.5-2b.json), [CSV](artifacts/ollama-default-profiles/throughput/2026-09-29T120654.678161Z-p18524-qwen3.5-2b.csv), [Markdown](artifacts/ollama-default-profiles/throughput/2026-09-29T120654.678161Z-p18524-qwen3.5-2b.report.md), [HTML](artifacts/ollama-default-profiles/throughput/2026-09-29T120654.678161Z-p18524-qwen3.5-2b.report.html)

Both telemetry runs recorded the warning `Swap used ratio exceeded 0.20
during telemetry sampling.` This is an environment limitation on timing
interpretation, not a request, persistence, or report failure.

## Capability-probe smoke

The explicit `--probe-capabilities --probe-all-endpoints` smoke used run
`2026-09-29T121124.382656Z-p28956-qwen3.5-2b` with one non-streaming
`32`-prompt/`16`-output request. The measured request returned HTTP 200 and
the capability results were:

| Endpoint | Result |
|---|---|
| `/v1/models` | Supported, HTTP 200 |
| `/v1/chat/completions` | Supported, HTTP 200 |
| Streaming chat completions | Supported, HTTP 200 |
| `/v1/responses` | Supported, HTTP 200 |
| `/v1/embeddings` | Unsupported, controlled HTTP 501; Ollama reported that the server was not started with embeddings enabled |

The capability smoke also returned the same five-model catalog and passed the
same credential/preview scan. Its [JSON](artifacts/ollama-default-profiles/capability-smoke/2026-09-29T121124.382656Z-p28956-qwen3.5-2b.json), [CSV](artifacts/ollama-default-profiles/capability-smoke/2026-09-29T121124.382656Z-p28956-qwen3.5-2b.csv), [Markdown](artifacts/ollama-default-profiles/capability-smoke/2026-09-29T121124.382656Z-p28956-qwen3.5-2b.report.md), and [HTML](artifacts/ollama-default-profiles/capability-smoke/2026-09-29T121124.382656Z-p28956-qwen3.5-2b.report.html) artifacts are retained.

## Adjacent regression boundary

The current-tree focused checks passed:

| Boundary | Result |
|---|---|
| Catalog-derived provider contract and launcher transport | `mock_provider_e2e`: 21/21 |
| Performance planning and safety | `performance_cli_tests`: 7/7 |
| External quality planning | `quality_cli_tests`: 3/3 |
| Saved report reload/generation | `report_cli_e2e`: 1/1 |
| Atomic replacement and failed-rename cleanup | `atomic_write`: 2/2 |
| Windows ConPTY navigation, interruption, and recovery | `pty_menu_e2e`: 10/10 |

No runtime defect was uncovered, so no source fix was required.

## Remaining limits

This evidence keeps the following boundaries explicit:

- `benchmark.performance` remains `PARTIAL`: the full default latency and
  throughput profiles now pass for one Ollama model, while the full sweep,
  broader live concurrency, repeated statistical sampling, provider/model
  variance, and host variance remain open.
- `provider.live.representative` remains `PARTIAL`: Ollama is current and
  validated at the boundary above; llama.cpp and a second known
  OpenAI-compatible implementation remain blocked by host availability and
  were not installed.
- Best-effort providers remain fixture-only and live-unvalidated.
- Tier 5 remains `PARTIAL`: the deterministic failure, atomic-write,
  interruption, recovery, and Windows ConPTY boundaries pass; resumable
  in-progress state, production-scale ceilings, and native non-Windows
  terminal behavior remain open.
- crates.io publication/install and external evaluator execution remain
  owner- or scope-gated.
