use crate::benchmarks::api_calls::{
    EmbeddingsBenchmark, ResponsesGenerationBenchmark, StructuredOutputBenchmark,
    ToolCallingBenchmark,
};
use crate::benchmarks::base::Benchmark;
use crate::benchmarks::consistency::ResponseConsistencyBenchmark;
use crate::benchmarks::generation::BasicGenerationLatencyBenchmark;
use crate::benchmarks::prompt_sizes::PromptSizePerformanceBenchmark;
use crate::errors::LLMeterError;

pub struct BenchmarkRegistry {
    benchmarks: Vec<Box<dyn Benchmark>>,
}

impl BenchmarkRegistry {
    pub fn new() -> Self {
        BenchmarkRegistry {
            benchmarks: Vec::new(),
        }
    }

    pub fn register(&mut self, benchmark: Box<dyn Benchmark>) {
        if self.benchmarks.iter().any(|b| b.id() == benchmark.id()) {
            return;
        }
        self.benchmarks.push(benchmark);
    }

    pub fn all(&self) -> &[Box<dyn Benchmark>] {
        &self.benchmarks
    }

    pub fn ids(&self) -> Vec<&str> {
        self.benchmarks.iter().map(|b| b.id()).collect()
    }

    pub fn get(&self, benchmark_id: &str) -> Option<&dyn Benchmark> {
        self.benchmarks
            .iter()
            .find(|b| b.id() == benchmark_id)
            .map(|b| b.as_ref())
    }

    pub fn select(
        &self,
        benchmark_ids: Option<&[String]>,
        all_benchmarks: bool,
    ) -> Result<Vec<&dyn Benchmark>, LLMeterError> {
        if all_benchmarks || benchmark_ids.is_none() {
            return Ok(self.benchmarks.iter().map(|b| b.as_ref()).collect());
        }

        let ids = benchmark_ids.unwrap();
        let mut selected = Vec::new();
        for id in ids {
            match self.get(id) {
                Some(b) => selected.push(b),
                None => {
                    let available = self.ids().join(", ");
                    return Err(LLMeterError::Benchmark(format!(
                        "Unknown benchmark '{id}'. Available: {available}"
                    )));
                }
            }
        }
        Ok(selected)
    }
}

impl Default for BenchmarkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_registry() -> BenchmarkRegistry {
    let mut registry = BenchmarkRegistry::new();
    registry.register(Box::new(BasicGenerationLatencyBenchmark));
    registry.register(Box::new(ResponsesGenerationBenchmark));
    registry.register(Box::new(ResponseConsistencyBenchmark));
    registry.register(Box::new(PromptSizePerformanceBenchmark));
    registry.register(Box::new(StructuredOutputBenchmark));
    registry.register(Box::new(ToolCallingBenchmark));
    registry.register(Box::new(EmbeddingsBenchmark));
    registry
}
