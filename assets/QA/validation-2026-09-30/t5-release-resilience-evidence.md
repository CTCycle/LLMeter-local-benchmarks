# Tier 5 release-resilience evidence

Last updated: 2026-09-30

## Final revision and focused results

- Final candidate revision: `1fb3dbd1f3211e25280481f5bbdab439a850a2f9` on `develop`.
- Package: `llmeter 0.4.0`
- Host: Windows 11 Pro `10.0.26200`, x86-64; Rust `1.98.0`, Cargo `1.98.0`.

Final focused commands passed:

- `cargo test --locked --test resilience_e2e -- --test-threads=1`: 1/1.
- `cargo test --locked --test pty_menu_e2e -- --test-threads=1`: 10/10.
- `cargo test --locked --test mock_provider_e2e -- --test-threads=1`: 23/23, including the configured ceiling and above-ceiling refusal.
- `cargo test --locked --all-targets --all-features -- --test-threads=1`: 156/156.

## Restart and interruption boundary

`tests/resilience_e2e.rs` used an isolated temporary home/output root and a
loopback mock provider. It completed a first performance result, started a
later delayed request, terminated the child with `Child::kill()`, and proved:

- the first JSON/report snapshot was byte-for-byte unchanged;
- no second canonical JSON, malformed report, or atomic-write temporary file appeared;
- a fresh process successfully executed `report list` and `report show`;
- a subsequent fresh process completed another benchmark;
- both completed schema `3.0` performance results remained independently loadable with distinct run IDs.

The Windows ConPTY interruption suite seeded the same completed-result
boundary, interrupted a delayed second run, checked preservation and report
reload, and completed a second valid result. The separate confirmation
interruption coverage also passed.

## Native terminal boundary

Hosted [CI run `36722865383`](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36722865383)
passed all four platform jobs for the exact final revision. The native Unix
PTY suite passed 4/4 on each of Ubuntu x86-64, macOS Apple silicon, and macOS
Intel. The Windows ConPTY suite passed 10/10 in the same all-target workflow.
The Unix cases cover menu interrupt, nested cancel/back navigation,
performance-confirmation interrupt, and delayed-request interruption with
fresh-process recovery.

## Scope boundary

This evidence closes the finite T5-02 release contract: controlled failures,
durable completed results, safe interruption without convincing partial
artifacts, fresh-process recovery, the configured 500-request ceiling and
above-limit refusal, Windows ConPTY, and native Unix PTY behavior on the three
hosted targets. Checkpoint-based resumption and stress beyond the configured
500-request ceiling are not product requirements and are not claimed.

## Disposition

`T5-02: PASS` at the finite release boundary above. This is not a claim of
resumable in-progress benchmarks, production-scale stress certification, or
native terminal behavior on every unsupported or unhosted environment.
