# Python

## Version

Python 3.14 is required. This is enforced by `requires-python = ">=3.14"` in `pyproject.toml`, the `.python-version` file, and CI targeting Python 3.14.

## Conventions

- Use `from __future__ import annotations` in every file.
- Use `@dataclass(slots=True)` for data containers.
- Type-annotate all function signatures and return types.
- Prefer `Path` over `str` for filesystem paths.
- Use `enum` or literal `str` unions for fixed choices.
- Use `isinstance()` checks with `int | float` union for numeric dispatch.

## Dependencies

Only runtime dependency is `rich >=13.9,<15`. Standard library for everything else.

## Style

Ruff enforces the following rule sets:

- `E`, `F` — pycodestyle and pyflakes
- `I` — import sorting
- `UP` — pyupgrade (targeting Python 3.14)
- `B` — bugbear
- `SIM` — simplify

Line length: 120 characters.

Last updated: 2026-06-12
