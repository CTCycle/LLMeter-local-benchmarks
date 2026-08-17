# Supported platforms

Last updated: 2026-08-17

## Support tiers

### Tier 1

- Windows 11 x86-64 using the MSVC Rust target. Native CI and the release workflow exercise checks, Clippy, tests, documentation, and the Windows ConPTY menu test. The release archive is a `.zip`.

### Tier 2

- Current Ubuntu x86-64 using the GNU Rust target. Native CI and the release workflow run checks, Clippy, tests, and documentation. The GNU binary requires a compatible glibc runtime and is not fully static.
- Current macOS x86-64 (Intel) and arm64 (Apple silicon) native builds. Native CI and the release workflow exercise both targets and publish `.tar.gz` archives.
- Linux musl targets are portability candidates only; no musl artifact is currently promised or published.

The minimum supported Rust version is not declared. Builds use the current stable toolchain until an MSRV is intentionally selected and added to CI.

## Runtime prerequisites

LLMeter does not manage provider processes or model installation. The user must start an OpenAI-compatible provider server and expose at least one model. `nvidia-smi` is optional and is used only when available for GPU telemetry.

## Distribution status

The release workflow is tag-gated and publishes the four archives above plus `SHA256SUMS` and GitHub artifact provenance attestations. The first `0.3.0` crates.io publication is manual; subsequent releases can use crates.io trusted publishing. Source builds and `cargo install --path . --locked` remain supported.
