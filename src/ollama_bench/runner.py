from __future__ import annotations

from pathlib import Path
from typing import Iterable

from ollama_bench.benchmarks.base import BenchmarkContext
from ollama_bench.benchmarks.registry import default_registry
from ollama_bench.config import AppConfig
from ollama_bench.errors import ModelNotFoundError, OllamaServerError
from ollama_bench.ollama.client import OllamaClient
from ollama_bench.results import BenchmarkRun, ResultStore
from ollama_bench.utils import utc_now_iso


def installed_model_names(client: OllamaClient) -> list[str]:
    return [str(model.get("name") or model.get("model")) for model in client.list_models() if model.get("name") or model.get("model")]


def validate_models(client: OllamaClient, selected_models: Iterable[str]) -> list[str]:
    requested = [model.strip() for model in selected_models if model.strip()]
    available = set(installed_model_names(client))
    missing = [model for model in requested if model not in available]
    if missing:
        raise ModelNotFoundError(f"Model(s) not installed locally: {', '.join(missing)}")
    if not requested:
        raise ModelNotFoundError("No models selected.")
    return requested


def run_benchmarks(
    *,
    client: OllamaClient,
    config: AppConfig,
    model_names: list[str],
    benchmark_ids: list[str] | None,
    all_benchmarks: bool,
    runs: int,
    num_predict: int,
    temperature: float,
    extra_options: dict | None = None,
) -> BenchmarkRun:
    if not client.is_running():
        raise OllamaServerError("Ollama server is not running.")

    models = validate_models(client, model_names)
    registry = default_registry()
    benchmarks = registry.select(benchmark_ids, all_benchmarks=all_benchmarks)
    context = BenchmarkContext(
        runs=runs,
        num_predict=num_predict,
        temperature=temperature,
        timeout=config.timeout,
        options=extra_options or {},
    )

    store = ResultStore(config.output_dir)
    run = BenchmarkRun(
        run_id=store.new_run_id(models),
        created_at=utc_now_iso(),
        models=models,
        benchmark_ids=[benchmark.id for benchmark in benchmarks],
        config={
            "api_base_url": config.api_base_url,
            "runs": runs,
            "num_predict": num_predict,
            "temperature": temperature,
            "timeout": config.timeout,
        },
    )

    for model in models:
        for benchmark in benchmarks:
            run.results.extend(benchmark.run(client, model, context))
    return run
