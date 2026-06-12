from __future__ import annotations

from statistics import mean
import time

from llmeter.benchmarks.base import BenchmarkContext, BenchmarkResultRecord
from llmeter.benchmarks.metrics import generation_metrics, pairwise_similarity, preview
from llmeter.ollama.client import OllamaClient
from llmeter.prompts import CONSISTENCY_PROMPT
from llmeter.utils import ns_to_ms


class ResponseConsistencyBenchmark:
    id = "consistency"
    name = "Response consistency"
    description = "Repeats the same prompt and reports exact-match and pairwise text similarity."

    def run(self, client: OllamaClient, model: str, context: BenchmarkContext) -> list[BenchmarkResultRecord]:
        runs = max(2, context.runs)
        options = context.generation_options(temperature=context.temperature)
        responses: list[str] = []
        per_run_metrics: list[dict] = []
        errors: list[str] = []
        started = time.perf_counter_ns()

        for _ in range(runs):
            try:
                result = client.generate(
                    model,
                    CONSISTENCY_PROMPT,
                    options=options,
                    keep_alive=context.keep_alive,
                    timeout=context.timeout,
                )
                responses.append(result.response.strip())
                per_run_metrics.append(generation_metrics(result))
            except Exception as exc:  # noqa: BLE001
                errors.append(str(exc))

        ended = time.perf_counter_ns()
        scores = pairwise_similarity(responses)
        unique_count = len(set(responses)) if responses else 0
        exact_match_ratio = 1.0 if len(responses) <= 1 else round(1.0 - ((unique_count - 1) / max(1, len(responses) - 1)), 3)
        output_token_rates = [m.get("tokens_per_second") for m in per_run_metrics if m.get("tokens_per_second") is not None]

        metrics = {
            "requested_runs": runs,
            "successful_runs": len(responses),
            "failed_runs": len(errors),
            "unique_responses": unique_count,
            "exact_match_ratio": exact_match_ratio,
            "mean_pairwise_similarity": round(mean(scores), 4) if scores else None,
            "min_pairwise_similarity": round(min(scores), 4) if scores else None,
            "max_pairwise_similarity": round(max(scores), 4) if scores else None,
            "mean_tokens_per_second": round(mean(output_token_rates), 3) if output_token_rates else None,
            "wall_time_ms": ns_to_ms(ended - started),
        }

        response_preview = " | ".join(preview(text, 80) for text in responses[:3]) if responses else None
        return [
            BenchmarkResultRecord(
                benchmark_id=self.id,
                benchmark_name=self.name,
                model=model,
                run_index=None,
                prompt_name="consistency",
                metrics=metrics,
                response_preview=response_preview,
                error="; ".join(errors) if errors else None,
                metadata={"options": options},
            )
        ]
