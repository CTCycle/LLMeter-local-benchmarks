# T1-07 managed lifecycle validation

Last updated: 2026-09-23

## Validation boundary

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | `develop` |
| Package | llmeter 0.4.0 |
| Environment | Windows x86-64; Cargo 1.98.0; Rust 1.98.0 |
| Provider/model | No provider was called. |
| Scope | Windows real-CLI install, overwrite refusal, self-update rollback and success, uninstall, and home purge in a disposable root. |

## Local validation

| Command | Result |
|---|---|
| `cargo test --locked --all-features --test lifecycle_cli_e2e -- --test-threads=1` | 1 passed |
| `cargo test --locked --all-targets --all-features -- --test-threads=1` | Passed; includes this lifecycle scenario and the existing lifecycle unit and launcher suites. |

The test created its home and helper temp directory under a temporary `assets/QA/` fixture. The fixture was removed after the helper scripts completed.

## Scenarios and results

| Scenario | Evidence | Result |
|---|---|---|
| Install the running CLI into a managed home. | `windows_managed_lifecycle_cli_installs_updates_rolls_back_and_purges_in_isolation` | The managed executable matched the source binary, Windows CMD and PowerShell launchers were present, and the installed executable returned success for `--version`. |
| Refuse an overwrite without `--force`. | Same real-CLI scenario | The CLI returned usage error code 2, identified the existing install, and left its bytes unchanged. |
| Exercise the self-update rollback path. | Same real-CLI scenario; the test removes the staged replacement before the delayed helper moves it. | The helper restored the original executable, the installed binary remained runnable, and neither a backup nor the failed helper script remained. |
| Complete a successful self-update. | Same real-CLI scenario | The helper installed the replacement bytes; the updated executable remained runnable and no backup or helper file remained. |
| Uninstall and purge owned state. | Same real-CLI scenario | The managed executable and launchers, config, and benchmark results were removed. An unrelated sentinel in the home and the replacement fixture outside the home remained. |

The forced rollback exposed one cleanup defect: the failure branch restored the prior executable but left its temporary `.cmd` helper behind. The failure branch now deletes that helper, and the real-CLI rollback assertion verifies the cleanup.

## Remaining boundary

This is Windows-local evidence for managed file-copy lifecycle commands. It does not certify remote update or signature verification, a crates.io install, or native terminal behavior on non-Windows platforms. The lifecycle commands remain local convenience operations by design.
