# T3-04 repeated live profiles and T5-01 resilience evidence

Last updated: 2026-09-26

## Scope and source boundary

This follow-up covers the next actionable live performance slice after the
provider became available again, plus a bounded subset of the previously
unrun Tier 5 resilience campaign. It validates the current implementation on
Windows x86-64 at source revision
`e635459a402414ee880bde42be0efaff0a245acf` (`develop`). The product source
is unchanged from the previously validated performance implementation; this
revision contains the current ledger and QA updates.

The live provider was Ollama at `http://localhost:11434/v1`, with a fresh
status check returning `API reachable: yes` and 8 exposed models. The live
performance model was exactly `qwen3.5:2b`. No best-effort provider or
cross-provider claim is inferred.

## T3-04 repeated live latency/throughput slice

Both runs used the same explicit bounded matrix:

```text
--prompt-tokens 32,64
--output-tokens 16
--concurrency 1,2
--warmup 1
--runs 3
--max-requests 32
--load-measurement off
--telemetry standard --sample-interval-ms 1000
--export both --report both --detail detailed
```

Each profile planned 4 scenarios, 4 warmup requests, and 12 measured
requests, for 16 total requests under the 32-request cap. The real CLI
completed both profiles with 4/4 scenario records, 12/12 measured HTTP 200
traces, 0 errors, and 4 telemetry samples per run.

| Profile | Run ID | Scenarios | Warmup | Measured | HTTP traces | Result |
|---|---|---:|---:|---:|---:|---|
| `latency` | `2026-09-26T112723.031674Z-p35248-qwen3.5-2b` | 4 | 4 | 12 | 12 x 200 | PASS |
| `throughput` | `2026-09-26T130411.461162Z-p16376-qwen3.5-2b` | 4 | 4 | 12 | 12 x 200 | PASS |

The saved JSON records contain three measured request traces for every
scenario, the selected model/provider identity, environment and telemetry
summaries, and no response previews. The generated artifacts are:

- Latency: [JSON](artifacts/ollama-repeated-profiles/2026-09-26T112723.031674Z-p35248-qwen3.5-2b.json), [CSV](artifacts/ollama-repeated-profiles/2026-09-26T112723.031674Z-p35248-qwen3.5-2b.csv), [Markdown report](artifacts/ollama-repeated-profiles/2026-09-26T112723.031674Z-p35248-qwen3.5-2b.report.md), [HTML report](artifacts/ollama-repeated-profiles/2026-09-26T112723.031674Z-p35248-qwen3.5-2b.report.html).
- Throughput: [JSON](artifacts/ollama-repeated-profiles/2026-09-26T130411.461162Z-p16376-qwen3.5-2b.json), [CSV](artifacts/ollama-repeated-profiles/2026-09-26T130411.461162Z-p16376-qwen3.5-2b.csv), [Markdown report](artifacts/ollama-repeated-profiles/2026-09-26T130411.461162Z-p16376-qwen3.5-2b.report.md), [HTML report](artifacts/ollama-repeated-profiles/2026-09-26T130411.461162Z-p16376-qwen3.5-2b.report.html).

Standard telemetry reported host swap-pressure warnings during both runs.
That is an environment limitation on timing interpretation, not an LLMeter
request or persistence failure. These samples are useful repeated live
evidence, but they do not certify full default-size profiles, concurrency
levels above 2, statistical comparisons, another model/provider, or another
host.

## T5-01 bounded failure and interruption subset

The following current implementation paths passed on Windows:

| Boundary | Evidence | Status |
|---|---|---|
| Performance safety ceilings | `performance_cli_tests`: 7/7; oversized prompt, oversized matrix, telemetry interval, request estimates, and legacy safety-parameter behavior | PASS |
| Provider/CLI bounded failures | `mock_provider_e2e`: 18/18; controlled provider errors, redirects/unsafe URLs, oversized JSON/SSE, authorization, unsupported endpoints, JSONL budget refusal, and invalid output path | PASS at fixture/official-launcher boundary |
| Atomic output cleanup | `utils::tests` filtered by `atomic_write`: 2/2; replacement leaves complete content and rename failure removes the temporary file | PASS |
| Interactive interruption | `pty_performance_confirmation_interrupt_returns_130`: 1/1; native Windows ConPTY confirmation interruption returned 130 | PASS at confirmation boundary |

Commands executed:

```text
cargo test --locked --test performance_cli_tests -- --test-threads=1
cargo test --locked --test mock_provider_e2e -- --test-threads=1
cargo test --locked --lib atomic_write -- --test-threads=1
cargo test --locked --test pty_menu_e2e pty_performance_confirmation_interrupt_returns_130 -- --exact --test-threads=1
```

No product defect was uncovered in this subset, so no runtime source fix was
needed. The dedicated Tier 5 campaign remains `PARTIAL`: mid-run long-lived
interruption, restart/state restoration, broader scale ceilings, and native
non-Windows terminal behavior remain unvalidated. Earlier focused coverage
and this subset must not be promoted to a full Tier 5 PASS.

## Quality gates and disposition

The current Windows local quality gates passed after the live runs:

```text
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features -- --test-threads=1
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
cargo build --locked --bin llmeter
```

Tier 3 remains `PARTIAL` because the new live evidence uses a custom bounded
matrix and one Ollama model. Tier 4 remains `PARTIAL` for best-effort
providers and cross-provider capability variation. Owner-gated crates.io
publication/install and external quality-framework execution remain outside
this task.
