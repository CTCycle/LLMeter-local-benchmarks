# Release checklist

## Before tagging

1. Confirm `Cargo.toml` version.
2. Update `CHANGELOG.md`.
3. Run:

```bash
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features -- --test-threads=1
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
cargo build --release --locked --all-features
cargo audit
cargo publish --locked --dry-run
```

On Windows, if the workspace `target/` tree is locked, rerun build-oriented commands with a temporary target directory:

```powershell
cargo build --release --target-dir "$env:TEMP\\llmeter-release-target"
```

## Release artifacts

The release workflow runs only from an authorized `v*` tag. It gates publication on the Cargo version, changelog heading, locked all-target/all-feature quality suite, dependency audit, package dry-run, native Linux/Windows all-feature builds, packaged mock-provider tests, archive extraction/content checks, and `--version`/`--help` smoke checks against the extracted binaries. It publishes only GNU/Linux x86-64 and Windows x86-64 archives with `SHA256SUMS`; macOS remains deferred.

## Trust model

Local builds inherit the trust of the checked-out source and Rust dependency resolution. A public archive trust model remains deferred until public binary distribution is approved.

Last updated: 2026-07-30
