# Release checklist

## Before tagging

1. Confirm `Cargo.toml` version.
2. Update `CHANGELOG.md`.
3. Run:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -- --test-threads=1
cargo build --release
```

On Windows, if the workspace `target/` tree is locked, rerun build-oriented commands with a temporary target directory:

```powershell
cargo build --release --target-dir "$env:TEMP\\llmeter-release-target"
```

## Release artifacts

Public binary distribution is not currently approved, and no release workflow is represented as active. When an owner approves distribution, implement tag/version/changelog validation, native artifact smoke tests, checksums, and the trust controls appropriate to the hosting model before publishing.

## Trust model

Local builds inherit the trust of the checked-out source and Rust dependency resolution. A public archive trust model remains deferred until public binary distribution is approved.

Last updated: 2026-07-18
