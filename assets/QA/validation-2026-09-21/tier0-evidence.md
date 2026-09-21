# Tier 0 validation evidence — 2026-09-21

Last updated: 2026-09-21

All evidence below was collected from revision `9bbaacb09823383edf0b0ea4ba9fb8ee5f5ba2cc` on Windows x86-64. Temporary homes used by the CLI checks were removed after each probe. No provider credentials or arbitrary provider logs were retained.

## T0-01 — repository and external evidence reconciliation

- `git status --short --branch`: clean `develop` checkout before documentation changes.
- `git rev-parse HEAD`: `9bbaacb09823383edf0b0ea4ba9fb8ee5f5ba2cc`.
- `git ls-remote --heads origin`: `origin/develop` points to the same SHA; `origin/main` points to `5d5e41c2bb9d4c0c3181c0c9c5c21417b8bf91f6`.
- `git ls-remote --tags origin 'v0.4.0*'`: annotated tag `v0.4.0` resolves to release commit `5d5e41c2bb9d4c0c3181c0c9c5c21417b8bf91f6`.
- GitHub API confirmed baseline CI run `35526614502` completed successfully on the pre-campaign `develop` SHA, and post-push CI run `35619619837` completed successfully on the committed documentation/evidence revision across Windows, Ubuntu, macOS Intel, and macOS ARM64.
- GitHub API confirmed release run `34574075684` completed successfully, with validation, GNU/Linux, Windows, macOS Intel, macOS ARM64, and publication jobs all successful.
- GitHub API confirmed public release `v0.4.0` is non-draft/non-prerelease and publishes four archives plus `SHA256SUMS`.

The prior ledger wording that treated v0.4.0 hosted publication as blocked was stale and is corrected in this change. crates.io publication remains unverified.

## T0-02 — quality baseline

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --locked --all-targets --all-features` | PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | PASS |
| `cargo test --locked --all-targets --all-features -- --test-threads=1` | PASS — 132 passed, 0 failed |
| `$env:RUSTDOCFLAGS = '-D warnings'; cargo doc --locked --no-deps --all-features` | PASS |
| `cargo build --locked --release --all-features` | PASS |
| Release-binary `mock_provider_e2e` with `LLMETER_BIN=target/release/llmeter.exe` | PASS — 11 passed, 0 failed |

The serialized suite included 59 library tests, 6 CLI contract tests, 11 mock-provider tests, 6 performance-plan tests, 8 metrics tests, 2 probe tests, 9 PTY tests, 1 quality-plan test, 1 resource test, 3 schema tests, 4 metric tests, 9 registry tests, 7 reporting tests, and 6 result-store tests.

## T0-03 — release binary and CLI contract

| Probe | Observed result |
|---|---|
| `target/release/llmeter.exe --version` | Exit `0`; `llmeter 0.4.0` |
| `target/release/llmeter.exe --help` | Exit `0`; usage text present |
| Piped no-command invocation | Exit `2`; help printed; no TUI entered |
| Piped `menu` invocation | Exit `2`; help printed; no TUI entered |
| `target/release/llmeter.exe --timeout 0 status` | Exit `2`; finite positive timeout error |
| Release-binary mock-provider E2E | 11/11 passed, including cached/fresh model validation, provider catalog, unsupported endpoint handling, saved report flow, and all registered preset fixture coverage |

## T0-04 — configuration and persisted provider state

An isolated temporary home was used with `LLMETER_TIMEOUT=1` to bound unavailable-provider checks.

| Probe | Observed result |
|---|---|
| No persisted configuration | Status selected default `ollama`; unavailable endpoint returned controlled exit `1` |
| `providers set lmstudio` then restart | Exit `0`; persisted `config.json` contained only `{ "provider": "lmstudio" }`; subsequent status selected `lmstudio` |
| `LLMETER_PROVIDER=llama-cpp` | Environment selection overrode persisted provider |
| `--provider openai-compatible` | CLI selection overrode environment selection |
| `--base-url http://127.0.0.1:54321` | Displayed normalized `http://127.0.0.1:54321/v1` |
| `--base-url http://user:pass@127.0.0.1:54321/v1` | Exit `2`; embedded credentials rejected |

Malformed persisted JSON, unknown providers, numeric bounds, and provider-specific URL precedence also passed in the serialized regression suite.

## T0-05 — PowerShell launcher

Passed at the exercised boundary:

- `run_llmeter.ps1 --version` reused the existing release build and returned `llmeter 0.4.0`.
- `run_llmeter.ps1 --provider openai-compatible --base-url http://127.0.0.1:54321 status` forwarded arguments; the binary returned controlled provider-unavailable exit `1`.
- Noninteractive `run_llmeter.ps1 -Action Clean -WhatIf` failed closed with “requires an interactive console; no files were changed.”

Remaining boundary:

- Fallback target selection after a default-target build failure was not forced.
- The interactive protected-home mutation path was not confirmed because accepting the prompt would authorize a broad destructive operation. The waiting probe was cancelled safely.

## Overall boundary

Tier 0 is `PARTIAL`, not comprehensive validation. The partial status is evidence-limited to launcher coverage; the executed quality, startup, configuration, and release-binary checks passed. Tier 1 is next.
