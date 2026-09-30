# Release-readiness evidence

Last updated: 2026-09-30

## Candidate identity

- Starting checkout SHA: `e0a4723d76c0836eff83b6190ebb575fc5562c2d` on `develop`
- Candidate SHA: `ac17e4d24d22d2b36761b969dad7d53e70a8f465`.
- Package version: `0.4.0`
- Release binary `--version`: PASS (`llmeter 0.4.0`)
- Release binary `--help`: PASS

## Local gates

PASS:

- `cargo fmt --all -- --check`;
- `cargo check --locked --all-targets --all-features`;
- `cargo clippy --locked --all-targets --all-features -- -D warnings`;
- `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features`;
- `cargo build --locked --release --all-features`;
- isolated `cargo audit` using the current advisory database;
- `cargo tree --locked --duplicates` inspection;
- focused serialized performance CLI/metrics, mock-provider, resilience, and
  Windows ConPTY suites;
- complete serialized `cargo test --locked --all-targets --all-features --
  --test-threads=1` with 156 tests passed and zero failures.

The Unix PTY target ran zero tests on Windows by its Unix cfg; native Unix
compilation and execution remain hosted-CI requirements. The non-publishing
`cargo publish --locked --dry-run` is deferred until the exact candidate is
committed; no dirty override is used.

## Tier summary

| Tier | Disposition | Boundary |
|---|---|---|
| Tier 0 | PASS | Existing recorded startup, evidence, and local quality boundary |
| Tier 1 | PASS | Existing recorded application-foundation boundary |
| Tier 2 | PASS | Existing recorded Ollama standard-workflow boundary |
| Tier 3 | PARTIAL | 500-request ceiling, deterministic regressions, current Ollama profiles, matched two-model functional matrix, persistence, reporting, and privacy pass; swap pressure invalidates comparative timing, LiteLLM is unavailable, and exact-candidate hosted CI remains open |
| Tier 4 | PASS | Existing representative provider/quality/GitHub distribution boundary; crates.io remains separate |
| Tier 5 | PARTIAL | Abrupt restart/recovery and Windows PTY pass; native Unix PTY and exact-candidate hosted CI remain open |
| Public distribution | PARTIAL | GitHub `v0.4.0` verified; crates.io publication/install remains owner-gated and unperformed |

## Outstanding release actions

1. Commit the reviewed candidate and run the exact four-platform hosted CI
   matrix, including native Unix PTY and resilience tests.
2. Complete the representative LiteLLM-over-Ollama path only if its disposable
   proxy becomes available; current Ollama profiles, matched matrix, and
   sanitized reload/privacy evidence are recorded in the [live Ollama
   revalidation evidence](t3-live-ollama-revalidation-evidence.md).
3. Run the non-publishing package dry-run from the exact committed candidate.
4. Keep crates.io publication and clean registry installation as a separate
   owner-authorized step; it was not attempted.

## Comprehensive-validation gate

The candidate does not satisfy the comprehensive-validation gate. Tier 3 and
Tier 5 remain explicitly partial, LiteLLM is unavailable, swap pressure limits
timing interpretation, and exact-candidate hosted CI is not yet qualified. No
release-ready or universal performance statement is made.
