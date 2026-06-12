# Getting started

## Requirements

- Python 3.14+
- Ollama installed and available on `PATH`
- At least one local Ollama model

## Installation

```bash
git clone <repo-url>
cd llmeter
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -e '.[dev]'
```

On Windows PowerShell:

```powershell
py -3.14 -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
python -m pip install -e ".[dev]"
```

## First run

```bash
llmeter
```

This opens the interactive main menu. Choose option 1 to start the Ollama server, then explore the available options.

Or run a quick benchmark non-interactively:

```bash
llmeter bench run --models all --benchmarks all --start-server
```

## Verify it works

```bash
llmeter status
```

Should show Ollama installed, server running, and API version.

Last updated: 2026-06-12
