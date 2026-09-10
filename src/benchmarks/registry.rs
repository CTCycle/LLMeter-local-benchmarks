use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::benchmarks::api_calls::{
    EmbeddingsBenchmark, ResponsesGenerationBenchmark, StructuredOutputBenchmark,
    ToolCallingBenchmark,
};
use crate::benchmarks::base::Benchmark;
use crate::benchmarks::consistency::ResponseConsistencyBenchmark;
use crate::benchmarks::generation::BasicGenerationLatencyBenchmark;
use crate::benchmarks::prompt_sizes::PromptSizePerformanceBenchmark;
use crate::errors::LLMeterError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum BenchmarkSuite {
    Llm,
    Embeddings,
}

impl BenchmarkSuite {
    pub fn label(self) -> &'static str {
        match self {
            BenchmarkSuite::Llm => "llm",
            BenchmarkSuite::Embeddings => "embeddings",
        }
    }
}

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
        assert!(
            !self.benchmarks.iter().any(|b| b.id() == benchmark.id()),
            "Duplicate benchmark id '{}'. Benchmark ids must be globally unique.",
            benchmark.id()
        );
        self.benchmarks.push(benchmark);
    }

    pub fn all(&self) -> &[Box<dyn Benchmark>] {
        &self.benchmarks
    }

    pub fn all_in_suite(&self, suite: BenchmarkSuite) -> Vec<&dyn Benchmark> {
        self.benchmarks
            .iter()
            .filter(|benchmark| benchmark.suite() == suite)
            .map(|benchmark| benchmark.as_ref())
            .collect()
    }

    pub fn ids(&self) -> Vec<&str> {
        self.benchmarks.iter().map(|b| b.id()).collect()
    }

    pub fn ids_for_suite(&self, suite: BenchmarkSuite) -> Vec<&str> {
        self.benchmarks
            .iter()
            .filter(|benchmark| benchmark.suite() == suite)
            .map(|benchmark| benchmark.id())
            .collect()
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
        suite: BenchmarkSuite,
    ) -> Result<Vec<&dyn Benchmark>, LLMeterError> {
        let suite_benchmarks = self.all_in_suite(suite);
        if all_benchmarks || benchmark_ids.is_none() {
            return Ok(suite_benchmarks);
        }

        let ids = benchmark_ids.unwrap();
        let mut selected = Vec::new();
        for id in ids {
            match self.get(id) {
                Some(benchmark) if benchmark.suite() == suite => selected.push(benchmark),
                Some(_) => {
                    return Err(LLMeterError::Benchmark(format!(
                        "Benchmark '{id}' is not part of the {} suite.",
                        suite.label()
                    )));
                }
                None => {
                    let available = self.ids_for_suite(suite).join(", ");
                    return Err(LLMeterError::Benchmark(format!(
                        "Unknown benchmark '{id}' for the {} suite. Available: {available}",
                        suite.label()
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
