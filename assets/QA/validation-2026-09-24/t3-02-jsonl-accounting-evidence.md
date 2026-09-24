# T3-02 JSONL accounting and bounded performance smoke

Last updated: 2026-09-24

## Finding and correction

The existing performance plan counted the profile's synthetic prompt-size list, but execution sent one scenario for each JSONL prompt. A custom workload with a different number of lines could therefore exceed the request budget and disagree with progress totals and saved scenario counts.

On source revision `c7438db98812b70c20738ddcd4821d27dab837a3`, plan creation loads the JSONL workload before safety validation and derives prompt-size entries from its nonblank prompts. Empty workloads and simultaneous `--jsonl`/`--prompt-tokens` inputs fail clearly. The runner checks that the workload still matches its validated prompt-size plan before model requests, and the run and save phases share the same scenario-count calculation. User guidance now describes JSONL prompt and sizing behavior in [scriptable usage](../../docs/user/scriptable_usage.md).

## Deterministic real-CLI regression

The new `jsonl_performance_accounting_matches_planning_requests_progress_and_results` case in [`tests/mock_provider_e2e.rs`](../../../tests/mock_provider_e2e.rs) runs the actual CLI against a local OpenAI-compatible fixture. Its three-line JSONL workload differs from the smoke profile's two synthetic prompt sizes.

The test verified:

- With one measured request per prompt and `--max-requests 2`, planning rejects the three-request workload before any workload POST is sent.
- `--dry-run --max-requests 3` reports 3 scenarios, 3 measured requests, and 3 total requests without sending a workload POST.
- The accepted CLI run sends exactly three `/v1/chat/completions` workload POSTs, one for each input prompt.
- Progress advances through steps `2/5`, `3/5`, and `4/5`; the remaining units are model inventory and environment capture, and the saved-run progress finishes at 100%.
- The saved JSON contains three scenario rows and the three corresponding prompt IDs.

The preflight still performs fresh model-catalog reads. The assertion is specifically that rejected and dry-run plans send no workload POSTs.

## Live smoke after the stop gate

After T3-02 passed, one bounded real Ollama smoke ran through the live CLI:

```powershell
$env:LLMETER_OUTPUT_DIR='assets/QA/validation-2026-09-24/artifacts/ollama-performance'
target\debug\llmeter.exe --provider ollama bench perf --profile smoke --models qwen3.5:2b --prompt-tokens 1 --output-tokens 8 --concurrency 1 --warmup 0 --runs 1 --max-requests 1 --load-measurement off --telemetry off --export json --report both
```

The plan reported 1 scenario and 1 total measured request. The run completed with 1/1 successful record and one persisted request trace (`/v1/chat/completions`, HTTP 200). The measured wall time was 8601.783 ms for this single sample; no performance comparison is inferred. See the [saved JSON](artifacts/ollama-performance/2026-09-24T065948.963614Z-p36164-qwen3.5-2b.json), [Markdown report](artifacts/ollama-performance/2026-09-24T065948.963614Z-p36164-qwen3.5-2b.report.md), and [HTML report](artifacts/ollama-performance/2026-09-24T065948.963614Z-p36164-qwen3.5-2b.report.html).

## Local gates

The following passed on Windows for the tested source tree:

```text
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features -- --test-threads=1
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
cargo build --locked --bin llmeter
```

The focused `mock_provider_e2e` suite passed 16/16, including the new accounting regression and current Tier 2 mock cases. The all-target/all-feature suite also passed. Hosted [CI run 35968716973](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35968716973) passed all four platform jobs on revision `28717289c2b0044ae337e5f10419aa62628fa492`, which contains the validated source revision `c7438db98812b70c20738ddcd4821d27dab837a3`; details are in the [validation ledger](validation_ledger.md).

## Status and remaining work

`T3-02` is `PASS`. `benchmark.performance` and Tier 3 remain `PARTIAL`: one tiny live smoke does not cover profile progression, production-sized workloads, concurrency, telemetry, model inventory variations, or provider/host variance. Do not interpret the single timing as a performance claim. Broader performance work remains a separate bounded slice.
