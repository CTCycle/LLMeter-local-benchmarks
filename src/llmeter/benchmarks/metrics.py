from __future__ import annotations

from typing import Any
from difflib import SequenceMatcher

from llmeter.ollama.client import GenerateResult
from llmeter.utils import ns_to_ms


def generation_metrics(result: GenerateResult) -> dict[str, Any]:
    return {
        "wall_time_ms": ns_to_ms(result.wall_time_ns),
        "time_to_first_token_ms": ns_to_ms(result.time_to_first_token_ns),
        "api_total_duration_ms": ns_to_ms(result.raw.get("total_duration")),
        "api_load_duration_ms": ns_to_ms(result.raw.get("load_duration")),
        "api_prompt_eval_duration_ms": ns_to_ms(result.raw.get("prompt_eval_duration")),
        "api_eval_duration_ms": ns_to_ms(result.raw.get("eval_duration")),
        "prompt_eval_count": result.raw.get("prompt_eval_count"),
        "eval_count": result.raw.get("eval_count"),
        "tokens_per_second": result.tokens_per_second,
        "response_chars": len(result.response),
        "done_reason": result.raw.get("done_reason"),
    }


def preview(text: str, limit: int = 180) -> str:
    compact = " ".join(text.split())
    if len(compact) <= limit:
        return compact
    return compact[: limit - 1] + "…"


def pairwise_similarity(values: list[str]) -> list[float]:
    scores: list[float] = []
    for left_index in range(len(values)):
        for right_index in range(left_index + 1, len(values)):
            scores.append(SequenceMatcher(None, values[left_index], values[right_index]).ratio())
    return scores
