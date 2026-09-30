# Release-readiness evidence

Last updated: 2026-09-30

## Candidate identity

- Base checkout SHA: `922da56ebf96834d905d918400f2dee0ea3c3491`
- Candidate SHA: not assigned; the working tree is intentionally uncommitted while live and hosted release gates remain open.
- Package version: `0.4.0`
- Release binary `--version`: PASS (`llmeter 0.4.0`)
- Release binary `--help`: PASS

## Local gates

PASS:

- formatting check;
- locked all-target/all-feature check;
- warning-denied Clippy;
- serialized locked all-target/all-feature tests;
- warning-denied rustdoc;
- all-feature release build;
- `cargo audit` after the minimal locked `rustls 0.23.45` security update;
- `cargo tree --duplicates` inspection;
- focused performance CLI/metrics, mock-provider, resilience, and Windows
  ConPTY suites.

The non-publishing `cargo publish --locked --dry-run` was not completed because
Cargo refuses to package an uncommitted working tree without
`--allow-dirty`. No dirty override was used.

## Tier summary

| Tier | Disposition | Boundary |
|---|---|---|
| Tier 0 | PASS | Existing recorded startup, evidence, and local quality boundary |
| Tier 1 | PASS | Existing recorded application-foundation boundary |
| Tier 2 | PASS | Existing recorded Ollama standard-workflow boundary |
| Tier 3 | PARTIAL | 500-request ceiling, full current Ollama profiles, and matched two-model functional matrix pass; LiteLLM, timing interpretation under swap pressure, and current-candidate hosted CI remain open |
| Tier 4 | PASS | Existing representative provider/quality/GitHub distribution boundary; crates.io remains separate |
| Tier 5 | PARTIAL | Abrupt restart/recovery and Windows PTY pass; native Unix PTY and hosted CI remain open |
| Public distribution | PARTIAL | GitHub `v0.4.0` verified; crates.io publication/install remains owner-gated and unperformed |

## Outstanding release actions

1. Commit the reviewed candidate and run the exact four-platform hosted CI
   matrix, including native Unix PTY and resilience tests.
2. Complete the representative LiteLLM-over-Ollama path when its disposable
   proxy is available; current Ollama status, profiles, matched matrix, and
   sanitized reload/privacy/environment evidence are recorded in the [live
   Ollama revalidation evidence](t3-live-ollama-revalidation-evidence.md).
3. Re-run the package dry-run from the exact committed candidate.
4. Keep crates.io publication and clean registry installation as a separate
   owner-authorized step; it was not attempted.

## Comprehensive-validation gate

The candidate does not satisfy the comprehensive-validation gate. Tiers 3 and
5 remain explicitly partial, exact-candidate hosted CI is absent, the
LiteLLM provider path is unavailable, and swap pressure limits timing
interpretation. No release-ready or universal performance statement is made.
