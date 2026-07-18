# Supported platforms

Last updated: 2026-07-18

## Support tiers

### Tier 1

- Windows 11 x86-64 using the MSVC Rust target. Native CI runs formatting-independent checks, Clippy, tests, documentation, and the Windows ConPTY menu test.

### Tier 2

- Current Ubuntu x86-64 using the GNU Rust target. Native CI runs checks, Clippy, tests, and documentation. The GNU binary requires a compatible glibc runtime and is not fully static.
- Current macOS x86-64 and arm64 source builds. These targets are compatibility goals but are not release-supported until native CI and an approved binary distribution workflow exist.
- Linux musl targets are portability candidates only; no musl artifact is currently promised or published.

The minimum supported Rust version is not declared. Builds use the current stable toolchain until an MSRV is intentionally selected and added to CI.

## Runtime prerequisites

LLMeter does not manage provider processes or model installation. The user must start an OpenAI-compatible provider server and expose at least one model. `nvidia-smi` is optional and is used only when available for GPU telemetry.

## Distribution status

Source builds and local `cargo install --path .` are supported. Public prebuilt binary distribution and package-manager publication are not currently approved; documentation and automation must not claim that release artifacts exist until that decision changes.
