# T3 repeated live model validation evidence

Last updated: 2026-09-29

## Selected slice

This slice revisited the remaining `benchmark.performance` and `resilience.edge-cases` debt. It targeted a deliberately repeated live sample on the available Ollama provider, plus a second locally available generation model and a saved-output interruption check. It does not claim production-scale performance, cross-provider certification, or an apples-to-apples model comparison.

## Boundary

| Field | Value |
|---|---|
| Validation checkout | `5ae227c85f303dd2cdbf33c770f4a6d53aa13702` on `develop` |
| Package / binary | `llmeter 0.4.0`, invoked through the official `run_llmeter.ps1` launcher and its existing `target/release/llmeter.exe` |
| Environment | Microsoft Windows NT `10.0.26200.0`, x86-64, PowerShell, Rust `1.98.0`, Cargo `1.98.0` |
| Provider | Ollama `0.34.0`, `http://localhost:11434/v1` |
| Fresh catalog | 5 models, including `qwen3.5:2b`, `qwen3.5:9b`, and `nomic-embed-text:latest` |
| Isolation | Dedicated `LLMETER_HOME`/config scratch root; raw outputs retained only under the `assets/QA/validation-2026-09-29/artifacts/ollama-repeated-models/` evidence directory |

The initial status and `models --json` checks passed through the official launcher. No provider executable, model, container, PATH entry, or permanent configuration was added by this validation.

## Plan and execution

The combined dry-run passed with two models, one scenario per model, four warmups, 50 measured requests, 54 total requests, and a 128-request safety limit.

The live commands were then isolated per model so a slow model could not discard the other model's saved evidence:

```powershell
.\run_llmeter.ps1 --provider ollama --output-dir <qa>/qwen3.5-2b bench perf --models qwen3.5:2b --profile latency --prompt-tokens 512 --output-tokens 128 --concurrency 1 --warmup 2 --runs 25 --load-measurement off --telemetry standard --sample-interval-ms 1000 --export both --report both --detail detailed --max-requests 64
```

```powershell
.\run_llmeter.ps1 --provider ollama --timeout 180 --output-dir <qa>/qwen3.5-9b bench perf --models qwen3.5:9b --profile latency --prompt-tokens 128 --output-tokens 32 --concurrency 1 --warmup 1 --runs 5 --no-stream --load-measurement off --telemetry off --export both --report both --detail detailed --max-requests 16
```

## Results

| Model | Mode | Scenario | Warmup | Measured requests | Successful | Errors / timeouts | P50 wall ms | P95 wall ms | Mean output tok/s | Result |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| `qwen3.5:2b` | streaming | `512p-128o-c1` | 2 | 25 | 25 | 0 / 0 | 2603.679 | 2918.591 | 48.715 | PASS |
| `qwen3.5:9b` | non-streaming | `128p-32o-c1` | 1 | 5 | 5 | 0 / 0 | 6374.340 | omitted below 20 samples | 5.073 | PASS (smoke) |

The 2b run reported a mean wall time of `2626.760 ms`, maximum `3077.743 ms`, population standard deviation `162.944 ms`, request rate `0.3806/s`, and complete provider input/output token coverage. Detailed output retained 20 ordered request traces because the detailed trace policy caps retained traces at 20 while preserving the aggregate count of 25.

The 9b run reported a mean wall time of `6307.065 ms`, P90 `6997.452 ms`, maximum `6997.452 ms`, population standard deviation `502.254 ms`, request rate `0.1585/s`, and complete provider input/output token coverage. P95 and P99 were intentionally omitted because the sample count was five.

Both results are schema `3.0` performance runs with zero error records. `report list` and `report show` reloaded each saved JSON result successfully, and the generated Markdown/HTML reports were present.

## Interruption and limitation observations

An initial combined streamed invocation used the 25-run 512/128 scenario for both models. The 2b scenario completed, but the 9b streamed scenario remained at the running-scenario stage for approximately 13 minutes without reaching the persistence phase. It was interrupted with Ctrl+C. The output directory contained no partial result before the isolated reruns, so the interruption did not leave a convincing partial artifact. This is an observation at this provider/model boundary, not a pass for streamed 9b behavior.

The later 9b non-streaming smoke completed, but its environment snapshot showed approximately `0.780` memory-used ratio and `0.619` swap-used ratio. The 2b telemetry run recorded 65 samples, maximum memory-used ratio `0.569`, and zero swap use. These host conditions limit timing interpretation. The differing prompt/output sizes, stream modes, and sample counts also prevent a quantitative 2b-versus-9b model comparison.

Privacy checks over both JSON/CSV/Markdown/HTML result sets found no bearer token, authorization value, or API-key-shaped value. Both JSON results report `sensitive_values_redacted: true`, `response_previews_included: false`, and zero non-empty response previews.

After validation, the Ollama processes started during the run were stopped and port `11434` was confirmed not listening. The dedicated LLMeter scratch root was removed; the curated result/report artifacts remain under this QA directory.

## Adjacent quality gates

The current Windows tree passed all local gates after the live runs:

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --locked --all-targets --all-features` | PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | PASS |
| `cargo test --locked --all-targets --all-features -- --test-threads=1` | PASS, 153 passed |
| `RUSTDOCFLAGS=-D warnings cargo doc --locked --all-features --no-deps` | PASS |
| `cargo build --locked --release --all-features` | PASS |
| Release binary `--version` / `--help` | PASS |

## Final status boundary

This slice passes the repeated 2b live statistical subset, adds a second-model 9b non-streaming smoke, confirms saved-report reload/privacy behavior, and reinforces the no-partial-artifact interruption boundary. `benchmark.performance` and Tier 3 remain `PARTIAL`: broader repeated samples across models, live concurrency/provider/host variance, and production-sized interpretation remain open. Tier 5 remains `PARTIAL`: resumable in-progress state, production-scale ceilings, and native non-Windows terminal behavior remain open. Best-effort provider live certification, crates.io publication/install, and external evaluator execution remain separate owner or scope boundaries.

## Retained artifacts

- 2b [JSON](artifacts/ollama-repeated-models/qwen3.5-2b/2026-09-29T192940.328660Z-p9032-qwen3.5-2b.json), [CSV](artifacts/ollama-repeated-models/qwen3.5-2b/2026-09-29T192940.328660Z-p9032-qwen3.5-2b.csv), [Markdown report](artifacts/ollama-repeated-models/qwen3.5-2b/2026-09-29T192940.328660Z-p9032-qwen3.5-2b.report.md), and [HTML report](artifacts/ollama-repeated-models/qwen3.5-2b/2026-09-29T192940.328660Z-p9032-qwen3.5-2b.report.html)
- 9b [JSON](artifacts/ollama-repeated-models/qwen3.5-9b/2026-09-29T193152.819728Z-p35120-qwen3.5-9b.json), [CSV](artifacts/ollama-repeated-models/qwen3.5-9b/2026-09-29T193152.819728Z-p35120-qwen3.5-9b.csv), [Markdown report](artifacts/ollama-repeated-models/qwen3.5-9b/2026-09-29T193152.819728Z-p35120-qwen3.5-9b.report.md), and [HTML report](artifacts/ollama-repeated-models/qwen3.5-9b/2026-09-29T193152.819728Z-p35120-qwen3.5-9b.report.html)
