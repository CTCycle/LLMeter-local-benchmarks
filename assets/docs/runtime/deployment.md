# Deployment

## Installation

Install from source:

```bash
python -m pip install -e .
```

Install with dev dependencies:

```bash
python -m pip install -e '.[dev]'
```

The `llmeter` console script is registered in `pyproject.toml`:

```toml
[project.scripts]
llmeter = "llmeter.cli:main"
```

After installation, `llmeter` is available on `PATH`.

## Dependencies

- Runtime: `rich >=13.9,<15`
- Dev: `pytest >=8`, `ruff >=0.8`

## Versioning

Current version: `0.2.0`. Follows semantic versioning. Defined in `src/llmeter/__init__.py` and `pyproject.toml`.

## Packaging

Built with setuptools. Package discovery:

```toml
[tool.setuptools.packages.find]
where = ["src"]
```

## Platforms

Cross-platform (Windows, macOS, Linux). Platform-specific behavior is handled in `ollama/server.py` for process management differences (taskkill on Windows, signals on POSIX).

No platform-specific native binaries are distributed. The project is pure Python.

Last updated: 2026-06-12
