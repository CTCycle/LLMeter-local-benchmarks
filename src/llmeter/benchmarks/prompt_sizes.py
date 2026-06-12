from __future__ import annotations

from llmeter.benchmarks.base import BenchmarkContext, BenchmarkResultRecord
from llmeter.benchmarks.metrics import generation_metrics, preview
from llmeter.ollama.client import OllamaClient
from llmeter.prompts import PROMPTS_BY_SIZE


class PromptSizePerformanceBenchmark:
    id = "prompt-sizes"
    name = "Performance across prompt sizes"
    description = "Runs short, medium, and long prompts to compare prompt processing and generation timing."

    def run(self, client: OllamaClient, model: str, context: BenchmarkContext) -> list[BenchmarkResultRecord]:
        records: list[BenchmarkResultRecord] = []
        options = context.generation_options(temperature=0.0)
        for prompt_name, prompt in PROMPTS_BY_SIZE.items():
            for run_index in range(1, context.runs + 1):
                try:
                    result = client.generate_stream(
                        model,
                        prompt,
                        options=options,
                        keep_alive=context.keep_alive,
                        timeout=context.timeout,
                    )
                    metrics = generation_metrics(result)
                    metrics["prompt_chars"] = len(prompt)
                    records.append(
                        BenchmarkResultRecord(
                            benchmark_id=self.id,
                            benchmark_name=self.name,
                            model=model,
                            run_index=run_index,
                            prompt_name=prompt_name,
                            metrics=metrics,
                            response_preview=preview(result.response),
                            metadata={"options": options},
                        )
                    )
                except Exception as exc:  # noqa: BLE001
                    records.append(
                        BenchmarkResultRecord(
                            benchmark_id=self.id,
                            benchmark_name=self.name,
                            model=model,
                            run_index=run_index,
                            prompt_name=prompt_name,
                            metrics={},
                            error=str(exc),
                            metadata={"options": options, "prompt_chars": len(prompt)},
                        )
                    )
        return records
