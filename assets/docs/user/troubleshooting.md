# Troubleshooting

## Common issues

### "ollama executable was not found on PATH"

Ensure Ollama is installed and `ollama` is available in your terminal. Test with:

```bash
ollama --version
```

### "Ollama did not become ready"

The server took too long to start. Check the server log:

```text
~/.llmeter/ollama-server.log
```

Common causes: another Ollama process is already running, port 11434 is in use, or the system is under high load.

### "Model not found"

Verify the model is installed:

```bash
llmeter models
```

If the model is not listed, pull it:

```bash
ollama pull <model-name>
```

### Reports are empty or missing data

Check that the JSON result file exists in the output directory and is valid JSON. The output directory defaults to `benchmark_results/` in the current working directory.

## File locations

| Item | Default path |
|---|---|
| Raw JSON results | `benchmark_results/<run-id>.json` |
| CSV exports | `benchmark_results/<run-id>.csv` |
| Markdown reports | `benchmark_results/<run-id>.report.md` |
| HTML reports | `benchmark_results/<run-id>.report.html` |
| Server PID file | `~/.llmeter/ollama-server.pid.json` |
| Server log | `~/.llmeter/ollama-server.log` |

## Known limitations

- Ollama only. Other providers are not supported.
- No concurrent benchmark execution. Each benchmark runs serially.
- Results are machine-specific. Compare runs from the same host under similar load.
- HTML reports are intentionally minimal and dependency-light.

Last updated: 2026-06-12
