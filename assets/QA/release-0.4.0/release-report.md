# LLMeter 0.4.0 release validation

Date: 2026-09-10

Commit: `5d5e41c2bb9d4c0c3181c0c9c5c21417b8bf91f6`

Local tag: `v0.4.0`

## Validation

- `cargo fmt --all -- --check`: passed.
- `cargo check --locked --all-targets --all-features`: passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `cargo test --locked --all-targets --all-features -- --test-threads=1`: passed, 132 tests.
- `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features`: passed.
- `cargo audit`: passed with no findings after refreshing the yanked `chacha20` lock entry to `0.10.2`.
- `cargo tree --duplicates`: passed; remaining duplicate versions are transitive platform and compatibility dependencies.
- Clean-tree `cargo publish --locked --dry-run`: passed for `llmeter v0.4.0` (90 packaged files).
- `cargo build --locked --release --all-features`: passed.
- Release binary `--version` and `--help`: passed; version is `llmeter 0.4.0`.
- Release-mode mock-provider E2E: passed, 11 tests.
- Extracted Windows archive mock-provider E2E: passed, 11 tests.
- Live Ollama smoke: standard LLM suite 8/8 records, embeddings 1/1, performance smoke 1/1, reports listed/rendered successfully.
- PowerShell launcher `--version` and `--provider ollama status`: passed; noninteractive destructive cleanup failed closed without changing files.

## Local artifact

- Archive: `llmeter-v0.4.0-x86_64-pc-windows-msvc.zip`
- Contents: `CHANGELOG.md`, `LICENSE`, `README.md`, `llmeter.exe`
- SHA-256: `473b6458e199dffb58fedb19a5332287fa3749f63cebc4b0b99ea9c549055f70`
- Checksum manifest: `SHA256SUMS`

## Release boundary

The original report was captured before hosted publication. On 2026-09-21, remote verification confirmed that annotated tag `v0.4.0` resolves to the release commit, release workflow run `34574075684` passed validation, GNU/Linux, Windows, macOS Intel, macOS ARM64, and publication jobs, and the public GitHub release contains four archives plus `SHA256SUMS`. This supersedes the local-only release boundary above for GitHub distribution. crates.io publication and clean `cargo install` verification remain unperformed.

Remote verification:

- [Release workflow run 34574075684](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/34574075684)
- [Public v0.4.0 release](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.4.0)
