from __future__ import annotations

from collections.abc import Iterable

from ollama_bench.benchmarks.base import Benchmark
from ollama_bench.benchmarks.consistency import ResponseConsistencyBenchmark
from ollama_bench.benchmarks.generation import BasicGenerationLatencyBenchmark
from ollama_bench.benchmarks.prompt_sizes import PromptSizePerformanceBenchmark
from ollama_bench.errors import BenchmarkError


class BenchmarkRegistry:
    def __init__(self) -> None:
        self._benchmarks: dict[str, Benchmark] = {}

    def register(self, benchmark: Benchmark) -> None:
        if benchmark.id in self._benchmarks:
            raise BenchmarkError(f"Duplicate benchmark id: {benchmark.id}")
        self._benchmarks[benchmark.id] = benchmark

    def all(self) -> list[Benchmark]:
        return list(self._benchmarks.values())

    def ids(self) -> list[str]:
        return list(self._benchmarks.keys())

    def get(self, benchmark_id: str) -> Benchmark:
        try:
            return self._benchmarks[benchmark_id]
        except KeyError as exc:
            available = ", ".join(self.ids())
            raise BenchmarkError(f"Unknown benchmark '{benchmark_id}'. Available: {available}") from exc

    def select(self, benchmark_ids: Iterable[str] | None = None, *, all_benchmarks: bool = False) -> list[Benchmark]:
        if all_benchmarks or benchmark_ids is None:
            return self.all()
        return [self.get(benchmark_id) for benchmark_id in benchmark_ids]


def default_registry() -> BenchmarkRegistry:
    registry = BenchmarkRegistry()
    registry.register(BasicGenerationLatencyBenchmark())
    registry.register(ResponseConsistencyBenchmark())
    registry.register(PromptSizePerformanceBenchmark())
    return registry
