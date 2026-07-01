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

The GitHub Actions release workflow builds and uploads:

- `llmeter-linux-x86_64.tar.gz`
- `llmeter-macos-x86_64.tar.gz`
- `llmeter-macos-aarch64.tar.gz`
- `llmeter-windows-x86_64.zip`
- one `.sha256` checksum file per archive

Artifacts contain the binary, `README.md`, and `LICENSE`.

## Trust model

Published archives are GitHub Actions build outputs from a release tag. Users should verify the `.sha256` file before installing a downloaded archive.

Last updated: 2026-07-01
