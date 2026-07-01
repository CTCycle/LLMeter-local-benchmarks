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

Or directly from a published crate (when available):

```bash
cargo install llmeter
```

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

LLMeter can also install a managed copy of itself under `<LLMETER_HOME>/bin`:

```cmd
llmeter install
```

This writes:

- `llmeter.exe`
- `llmeter.cmd`
- `llmeter.ps1`

Use `--force` to overwrite an existing managed install:

```cmd
llmeter install --force
```

Update the managed install from a newer executable:

```cmd
llmeter update --source C:\path\to\new\llmeter.exe
```

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

Prebuilt binaries for Linux (x86_64), macOS (x86_64, arm64), and Windows (x86_64) are available from the releases page. These are built with `rustls` — no `openssl` dependency is required.

Release archives are produced by GitHub Actions from release tags:

| Platform | Artifact |
|---|---|
| Linux x86_64 | `llmeter-linux-x86_64.tar.gz` |
| macOS x86_64 | `llmeter-macos-x86_64.tar.gz` |
| macOS arm64 | `llmeter-macos-aarch64.tar.gz` |
| Windows x86_64 | `llmeter-windows-x86_64.zip` |

Each archive is published with a matching `.sha256` checksum. Verify the checksum before installing a downloaded binary.

## Dependencies

- Runtime: **None**. The binary is self-contained.
- Build-time: Rust toolchain (stable), listed in `Cargo.toml`.

## Versioning

Current version: `0.3.0`. Follows semantic versioning. Defined in `Cargo.toml`.

## Platforms

Cross-platform (Windows, macOS, Linux). Binary naming is `llmeter.exe` on Windows and `llmeter` on Unix.

The binary is compiled with `rustls` (no `openssl`), making it fully statically linkable for musl targets.

Last updated: 2026-07-01
