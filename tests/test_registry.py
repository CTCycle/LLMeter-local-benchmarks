from llmeter.benchmarks.registry import default_registry


def test_default_registry_contains_initial_benchmarks():
    registry = default_registry()
    assert registry.ids() == ["generation-latency", "consistency", "prompt-sizes"]


def test_select_specific_benchmark():
    registry = default_registry()
    selected = registry.select(["consistency"])
    assert len(selected) == 1
    assert selected[0].id == "consistency"
