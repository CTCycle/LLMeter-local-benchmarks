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
cargo build --locked --release --all-features
cargo audit
cargo tree --duplicates
cargo publish --locked --dry-run
```

On Windows, if the workspace `target/` tree is locked, rerun build-oriented commands with a temporary target directory:

```powershell
cargo build --release --target-dir "$env:TEMP\\llmeter-release-target"
```

## Release artifacts

The release workflow runs only from an authorized `v*` tag. It gates publication on the Cargo version, changelog heading, locked all-target/all-feature quality suite, dependency audit, package dry-run, native Windows/Linux/macOS all-feature builds, packaged mock-provider tests, archive extraction/content checks, and `--version`/`--help` smoke checks against the extracted binaries. It publishes four archives with filename-only `SHA256SUMS` entries and GitHub artifact provenance attestations for every archive and the checksum file.

The local release-readiness validation for `0.3.0` is intended to pass the locked checks, package dry-run, release build, and temporary install/archive smoke gates. It does not create a tag or public release; hosted four-platform CI and release execution remain required gates.

## Trust model

Local builds inherit the trust of the checked-out source and Rust dependency resolution. Public archives are accompanied by checksums and GitHub artifact provenance attestations; users should verify both before running a downloaded binary.

## Publishing procedure

1. Review and commit the prepared changes, push `develop`, and require green four-platform CI.
2. Merge or fast-forward the verified commit to `main` and require CI there.
3. Confirm `Cargo.toml` and `CHANGELOG.md` identify `0.3.0`, the worktree is clean, and no `v0.3.0` tag or release exists.
4. Create and push an annotated `v0.3.0` tag from that `main` commit.
5. Verify all four archives, archive contents, checksums, extracted-binary smoke tests, and provenance attestations.
6. Independently download and verify the public assets, then manually run `cargo publish --locked` from the exact tagged source.
7. Verify `cargo install llmeter --version 0.3.0 --locked --root <clean-temp-root>` and the installed binary.
8. Configure crates.io trusted publishing for later releases. Stop publication if any hosted or registry verification fails.

Last updated: 2026-08-17
