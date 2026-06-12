from __future__ import annotations

from ollama_bench.benchmarks.base import BenchmarkContext, BenchmarkResultRecord
from ollama_bench.benchmarks.metrics import generation_metrics, preview
from ollama_bench.ollama.client import OllamaClient
from ollama_bench.prompts import SHORT_PROMPT


class BasicGenerationLatencyBenchmark:
    id = "generation-latency"
    name = "Basic generation latency"
    description = "Measures wall time, time to first token, Ollama duration fields, and output token throughput."

    def run(self, client: OllamaClient, model: str, context: BenchmarkContext) -> list[BenchmarkResultRecord]:
        records: list[BenchmarkResultRecord] = []
        options = context.generation_options(temperature=0.0)
        for run_index in range(1, context.runs + 1):
            try:
                result = client.generate_stream(
                    model,
                    SHORT_PROMPT,
                    options=options,
                    keep_alive=context.keep_alive,
                    timeout=context.timeout,
                )
                records.append(
                    BenchmarkResultRecord(
                        benchmark_id=self.id,
                        benchmark_name=self.name,
                        model=model,
                        run_index=run_index,
                        prompt_name="short",
                        metrics=generation_metrics(result),
                        response_preview=preview(result.response),
                        metadata={"options": options},
                    )
                )
            except Exception as exc:  # noqa: BLE001, converted to result so multi-model runs continue
                records.append(
                    BenchmarkResultRecord(
                        benchmark_id=self.id,
                        benchmark_name=self.name,
                        model=model,
                        run_index=run_index,
                        prompt_name="short",
                        metrics={},
                        error=str(exc),
                        metadata={"options": options},
                    )
                )
        return records
