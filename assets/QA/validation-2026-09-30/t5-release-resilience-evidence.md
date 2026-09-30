# Tier 5 release-resilience evidence

Last updated: 2026-09-30

## Revision and focused results

- Base checkout revision: `922da56ebf96834d905d918400f2dee0ea3c3491`
- Candidate revision: not assigned; evidence was collected from the uncommitted working tree.
- Package: `llmeter 0.4.0`
- Host: Windows, Rust `1.98.0`, Cargo `1.98.0`

Focused commands and results:

- `cargo test --locked --test resilience_e2e -- --test-threads=1`: PASS, 1/1.
- `cargo test --locked --test pty_menu_e2e -- --test-threads=1`: PASS, 10/10.
- `cargo test --locked --test mock_provider_e2e -- --test-threads=1`: PASS, 23/23, including configured-ceiling and above-ceiling refusal.
- `cargo test --locked --all-targets --all-features -- --test-threads=1`: PASS on the Windows checkout.

## Restart and interruption boundary

`tests/resilience_e2e.rs` uses an isolated temporary home/output root and a
loopback mock provider. It completed a first performance result, started a
later delayed request, terminated the child with `Child::kill()`, and proved:

- the first JSON/report snapshot was byte-for-byte unchanged;
- no second canonical JSON, malformed report, or atomic-write temporary file appeared;
- a fresh process successfully executed `report list` and `report show`;
- a subsequent fresh process completed another benchmark;
- both completed schema `3.0` performance results remained independently
  loadable with distinct run IDs.

The Windows ConPTY interruption test now seeds the same completed-result
boundary, interrupts a delayed second run, checks preservation and report
reload, and completes a second valid result. The existing confirmation
interruption test remains separate.

## Native terminal boundary

`tests/pty_menu_unix_e2e.rs` adds four native Unix `expectrl` cases for menu
interrupt, nested cancel/back navigation, performance-confirmation interrupt,
and delayed-request interruption with fresh-process recovery. The file
type-checks under an explicit Unix-cfg sanity build, but this Windows-only
toolchain cannot execute a native Unix PTY. Ubuntu x86-64, macOS Intel, and
macOS Apple-silicon execution remain hosted-CI gates.

## Disposition

`T5-02: PARTIAL`. Platform-neutral abrupt termination, durable-state recovery,
configured ceiling/refusal, and Windows ConPTY pass locally. Native Unix PTY
execution and exact-candidate hosted CI remain unverified.
