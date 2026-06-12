from ollama_bench.benchmarks.metrics import pairwise_similarity
from ollama_bench.ollama.client import GenerateResult


def test_generate_result_tokens_per_second():
    result = GenerateResult(
        model="example",
        prompt="prompt",
        response="hello",
        raw={"eval_count": 20, "eval_duration": 2_000_000_000},
        wall_time_ns=3_000_000_000,
    )
    assert result.tokens_per_second == 10.0


def test_pairwise_similarity_identical_texts():
    scores = pairwise_similarity(["same", "same", "same"])
    assert scores == [1.0, 1.0, 1.0]
