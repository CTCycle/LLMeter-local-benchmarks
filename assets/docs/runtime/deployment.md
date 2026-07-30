# Deployment

## Installation

### From source

```bash
git clone <repo-url>
cd llmeter
cargo build --release
```

The binary is at `target/release/llmeter` (or `target/release/llmeter.exe` on Windows). Add it to your `PATH`, or copy it to a directory in your `PATH`:

```bash
cp target/release/llmeter ~/.local/bin/
```

On Windows PowerShell:

```powershell
copy target\release\llmeter.exe C:\Users\<you>\bin\
```

### Via cargo install

```bash
cargo install --path .
```

The crate is not currently published. Do not use `cargo install llmeter`; public package publication requires a separate owner approval.

To install into a user-owned folder, set `CARGO_INSTALL_ROOT` or pass `--root` directly:

```bash
cargo install --path . --root ~/.local
```

On Windows PowerShell:

```powershell
cargo install --path . --root "$env:USERPROFILE\\.local"
```

LLMeter runtime state is portable because it defaults to a single home folder:

- Windows: `%USERPROFILE%\\.llmeter`
- Unix: `~/.llmeter`
- Override: set `LLMETER_HOME` before running the CLI

That home folder contains persisted config under `config/` and default benchmark outputs under `benchmark_results/`.

### Managed CLI install

LLMeter can also install a managed copy of itself under `<LLMETER_HOME>/bin`. These commands are local convenience file-copy operations, not a secure remote updater or package manager:

```cmd
llmeter install
```

On Windows this writes:

- `llmeter.exe`
- `llmeter.cmd`
- `llmeter.ps1`

On Unix it installs an executable named `llmeter` and does not create Windows launcher files.

Use `--force` to overwrite an existing managed install:

```cmd
llmeter install --force
```

Update the managed install from a newer executable:

```cmd
llmeter update --source C:\path\to\new\llmeter.exe
```

Only use a replacement executable that you obtained and verified yourself. LLMeter does not download update metadata, verify signatures, select release channels, or roll back a failed remote update.

Or run the newer executable directly and let it refresh the managed install:

```cmd
C:\path\to\new\llmeter.exe update
```

Uninstall the managed copy:

```cmd
llmeter uninstall
```

Remove the managed copy plus `LLMETER_HOME` state:

```cmd
llmeter uninstall --purge-home
```

### Prebuilt binaries

Authorized `v*` tags publish validated GNU/Linux x86-64 and Windows x86-64 archives through the release workflow. Each archive contains the binary, README, LICENSE, and CHANGELOG, and the release includes `SHA256SUMS`. macOS remains a source-compatibility goal rather than a release artifact.

## Dependencies

- GNU/Linux binaries require a compatible glibc runtime; they are not fully static.
- Windows and macOS binaries use their normal operating-system runtime environment.
- Provider servers remain external runtime prerequisites.
- `nvidia-smi` is optional telemetry integration, not a hard dependency.
- Build-time: Rust toolchain (stable), listed in `Cargo.toml`.

## Versioning

Current version: `0.3.0`. Follows semantic versioning. Defined in `Cargo.toml`.

## Platforms

Windows x86-64 is Tier 1. Ubuntu GNU/Linux x86-64 is Tier 2 and requires compatible glibc. macOS and musl are source compatibility goals without current native release evidence. See `SUPPORTED_PLATFORMS.md`.

The binary uses `rustls`, so it does not require an OpenSSL runtime dependency. A musl-targeted Linux build is the portable Linux option; the released GNU/Linux artifact still has a glibc compatibility boundary.

Last updated: 2026-07-30
