# Testing and quality

## Test framework

pytest is the test runner. Configuration in `pyproject.toml`:

```toml
[tool.pytest.ini_options]
testpaths = ["tests"]
pythonpath = ["src"]
```

Run all tests:

```bash
python -m pytest -q
```

## Ruff

Ruff runs as a linter only (no formatter). The selected rules are `E`, `F`, `I`, `UP`, `B`, `SIM` with `E501` ignored (line length handled by the 120-char limit setting).

Check lint:

```bash
ruff check .
```

## CI

GitHub Actions runs on push and pull request. Steps:
1. Setup Python 3.14.
2. Install project with `.[dev]`.
3. Run `ruff check .`
4. Run `python -m pytest -q`

## Test coverage

Currently no coverage threshold enforced. Tests live in `tests/` and mirror the source module structure. Each test file tests one source module (e.g. `test_results.py` tests `llmeter.results`).

Test files import from the installed package:

```python
from llmeter.benchmarks.base import BenchmarkResultRecord
from llmeter.results import BenchmarkRun, ResultStore
```

Last updated: 2026-06-12
