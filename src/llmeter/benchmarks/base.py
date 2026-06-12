from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Protocol

from llmeter.ollama.client import OllamaClient


@dataclass(slots=True)
class BenchmarkContext:
    runs: int = 3
    num_predict: int = 128
    temperature: float = 0.2
    timeout: float = 120.0
    keep_alive: str | int | None = "5m"
    options: dict[str, Any] = field(default_factory=dict)

    def generation_options(self, *, temperature: float | None = None, num_predict: int | None = None) -> dict[str, Any]:
        merged = dict(self.options)
        merged.setdefault("temperature", self.temperature if temperature is None else temperature)
        merged.setdefault("num_predict", self.num_predict if num_predict is None else num_predict)
        return merged


@dataclass(slots=True)
class BenchmarkResultRecord:
    benchmark_id: str
    benchmark_name: str
    model: str
    run_index: int | None
    prompt_name: str | None
    metrics: dict[str, Any]
    response_preview: str | None = None
    error: str | None = None
    metadata: dict[str, Any] = field(default_factory=dict)


class Benchmark(Protocol):
    id: str
    name: str
    description: str

    def run(self, client: OllamaClient, model: str, context: BenchmarkContext) -> list[BenchmarkResultRecord]:
        ...
