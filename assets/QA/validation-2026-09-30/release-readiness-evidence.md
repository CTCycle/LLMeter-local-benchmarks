# Release-readiness evidence

Last updated: 2026-09-30

## Current working-tree recheck

The checkout is based on `6ef50aa` and contains an uncommitted Windows ConPTY
performance-interruption repair. Local formatting, check, Clippy, serialized
156-test, rustdoc, release-build, binary smoke, audit, Windows PTY 10/10, and
one-request live Ollama smoke all pass. The exact hosted CI result below
predates this source change; hosted revalidation is pending. See the [current-
tree validation recheck](current-tree-validation-recheck.md).

## Release impact of the Tier 3 limitation

The Tier 3 `PARTIAL` is a non-blocking host/provider evidence limitation for
the scoped release boundary. Swap pressure prevents a defensible numeric timing
or model-ranking claim on this host, and the optional LiteLLM route is currently
unavailable; neither finding is a product defect. The functional, safety,
quality, and exact-candidate hosted-CI gates remain the release requirements.
The limitation keeps the comprehensive-validation claim closed until a suitable
host or optional provider route is available.

## Candidate identity

- Final candidate SHA: `1fb3dbd1f3211e25280481f5bbdab439a850a2f9` on `develop`.
- Package version: `0.4.0`.
- Release binary `--version`: PASS (`llmeter 0.4.0`).
- Release binary `--help`: PASS.

## Local gates

PASS:

- `cargo fmt --all -- --check`;
- `cargo check --locked --all-targets --all-features`;
- `cargo clippy --locked --all-targets --all-features -- -D warnings`;
- `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features`;
- `cargo build --locked --release --all-features`;
- isolated `cargo audit` and `cargo tree --locked --duplicates` inspection;
- focused performance, mock-provider, resilience, and Windows ConPTY suites;
- complete serialized `cargo test --locked --all-targets --all-features --
  --test-threads=1` with 156 tests passed and zero failures;
- live Ollama default profiles and matched functional matrix, with the timing
  limitation recorded in the [T3 release-performance evidence](t3-release-performance-evidence.md).

The Unix PTY target is not executable on the Windows checkout; exact native
Unix execution is covered by hosted CI. Non-publishing package dry-run remains
deferred until an owner-authorized release operation.

## Tier summary

| Tier | Disposition | Boundary |
|---|---|---|
| Tier 0 | PASS | Existing recorded startup, evidence, and local quality boundary |
| Tier 1 | PASS | Existing recorded application-foundation boundary |
| Tier 2 | PASS | Existing recorded Ollama standard-workflow boundary |
| Tier 3 | PARTIAL (non-blocking) | Deterministic accounting, 500-request ceiling/refusal, full Ollama profiles, matched two-model functional matrix, persistence, reporting, privacy, and exact-candidate hosted CI pass; swap pressure invalidates comparative timing and current LiteLLM smoke is unavailable |
| Tier 4 | PASS | Existing representative provider/quality/GitHub distribution boundary; crates.io remains separate |
| Tier 5 | PASS | Finite T5-02 boundary: interruption safety, completed-result durability, fresh-process recovery, configured ceiling/refusal, Windows ConPTY, and hosted Unix PTY pass |
| Public distribution | PARTIAL | GitHub `v0.4.0` verified; crates.io publication/install remains owner-gated and unperformed |

## Hosted CI qualification

Hosted [CI run `36722865383`](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36722865383)
completed successfully for the exact candidate across Ubuntu x86-64, Windows
x86-64, macOS Intel, and macOS Apple silicon. The run passed the workflow test
steps; native Unix PTY was 4/4 on each Unix target and Windows ConPTY was 10/10.

## Outstanding release actions

1. Obtain a host without swap-pressure warnings only if comparative timing or model ranking is required; this is not a prerequisite for the scoped release, but it is required to close the comprehensive Tier 3 claim.
2. Complete the representative LiteLLM-over-Ollama path only if its disposable proxy becomes available; this optional provider check is non-blocking for the scoped release and no live smoke is currently claimed.
3. Keep crates.io publication and clean registry installation as a separate owner-authorized step; it was not attempted.
4. Rerun exact-candidate hosted CI for the uncommitted ConPTY repair before shipping that source change; this is separate from the non-blocking Tier 3 host/provider limitations.

## Comprehensive-validation gate

The candidate does not satisfy the comprehensive-validation gate because Tier 3
comparative timing remains unvalidated and the current optional LiteLLM smoke
was unavailable. Tier 5 is closed at the finite T5-02 release boundary. No
universal performance, hardware-independent ranking, or unsupported-provider
claim is made.
