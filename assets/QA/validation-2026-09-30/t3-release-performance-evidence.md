# Tier 3 release-performance evidence

Last updated: 2026-09-30

## Revision and environment

- Repository: `CTCycle/LLMeter-local-benchmarks`
- Base checkout revision: `922da56ebf96834d905d918400f2dee0ea3c3491`
- Candidate revision: not assigned; this evidence was collected from the uncommitted working tree based on that revision.
- Package: `llmeter 0.4.0`
- Host: Windows, Rust `1.98.0`, Cargo `1.98.0`
- Provider fixture: ephemeral loopback OpenAI-compatible mock provider in `tests/mock_provider_e2e.rs`
- Live provider prerequisite: official launcher status/models checks could not reach `http://localhost:11434/v1`; no live model or provider-path claim is added here.

## Deterministic release ceiling

Focused command:

```text
cargo test --locked --test mock_provider_e2e performance_reaches_configured_request_ceiling_with_complete_accounting -- --test-threads=1
```

Result: PASS.

- Matrix: 5 prompt sizes × 1 output size × concurrency `1,2,4,8,16` = 25 scenarios.
- Warmup: 1 per scenario = 25 requests.
- Measured: 19 per scenario = 475 requests.
- Total planned and observed mock chat requests: exactly 500.
- Every scenario persisted 19 successful measured traces with unique request IDs/run indices.
- Result schema: `3.0`; run kind: `performance`; persisted scenario count: 25.
- JSON reload through `report show`: PASS.
- Default privacy flags and absent response previews: PASS.
- Atomic-write temporary-file cleanup: PASS.

Adjacent refusal command used the same matrix with 20 measured runs (525
total requests), without `--allow-large-matrix`.

Result: PASS. The command failed before any performance chat request, reported
`requests 525 exceed --max-requests 500`, and wrote no result artifact.

The complete focused mock-provider file then passed 23/23, including the
existing JSONL/accounting, default-matrix, bounded-concurrency, provider
contract, launcher, and the two release-ceiling cases.

## Local quality evidence

The following passed on the working tree:

- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets --all-features`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `cargo test --locked --all-targets --all-features -- --test-threads=1`
- warning-denied `cargo doc --locked --no-deps --all-features`
- `cargo build --locked --release --all-features`
- `cargo audit` after updating transitive `rustls` from `0.23.44` to patched `0.23.45`
- `cargo tree --duplicates`

## Live and hosted boundary

The required checks:

```text
.\run_llmeter.ps1 --provider ollama status
.\run_llmeter.ps1 --provider ollama models --json
```

now pass with Ollama `0.34.0` and five exposed models. The full default
`latency`, `throughput`, and `sweep` profiles for `qwen3.5:2b` pass, and the
matched non-streaming `qwen3.5:2b`/`qwen3.5:9b` matrix passes with 20 measured
samples per scenario. See the [live Ollama revalidation evidence](t3-live-ollama-revalidation-evidence.md)
for run IDs, accounting, report reload, and privacy checks.

The optional LiteLLM-over-Ollama path remains unavailable at
`http://localhost:4000/v1`, and the high swap pressure observed during
telemetry disqualifies comparative timing interpretation. Current
four-platform CI for this uncommitted candidate was also not run.

## Disposition

`T3-05: PARTIAL`. The configured fixture ceiling, above-limit refusal,
accounting, current Ollama profiles, matched live functional matrix,
persistence, reload, and privacy boundaries pass locally. The optional
LiteLLM path, timing interpretation under swap pressure, and exact-candidate
hosted CI remain open. No universal performance or cross-host numeric claim is
made.
