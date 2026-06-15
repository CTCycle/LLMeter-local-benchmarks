use std::collections::HashMap;

use serde_json::Value;

use crate::benchmarks::base::{
    Benchmark, BenchmarkContext, BenchmarkProgressSink, BenchmarkResultRecord, BenchmarkStepStatus,
    BenchmarkStepUpdate,
};
use crate::benchmarks::metrics::{generation_metrics, preview};
use crate::benchmarks::registry::BenchmarkSuite;
use crate::prompts::{
    EMBEDDINGS_INPUT, RESPONSES_PROMPT, STRUCTURED_OUTPUT_PROMPT, TOOL_CALL_PROMPT,
};
use crate::providers::ProviderClient;

pub struct ResponsesGenerationBenchmark;
pub struct StructuredOutputBenchmark;
pub struct ToolCallingBenchmark;
pub struct EmbeddingsBenchmark;

impl Benchmark for ResponsesGenerationBenchmark {
    fn id(&self) -> &str {
        "responses-generation"
    }

    fn name(&self) -> &str {
        "Responses API generation"
    }

    fn description(&self) -> &str {
        "Measures generation through /v1/responses when the provider supports it."
    }

    fn suite(&self) -> BenchmarkSuite {
        BenchmarkSuite::Llm
    }

    fn planned_steps(&self, context: &BenchmarkContext) -> u32 {
        context.runs
    }

    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
        progress: &mut dyn BenchmarkProgressSink,
    ) -> Vec<BenchmarkResultRecord> {
        let options = serde_json::json!(context.options);
        let total_steps = self.planned_steps(context);
        (1..=context.runs)
            .map(|run_index| {
                progress.on_step(BenchmarkStepUpdate {
                    status: BenchmarkStepStatus::Started,
                    step_index: run_index,
                    total_steps,
                    run_index: Some(run_index),
                    prompt_name: Some("responses".to_string()),
                    message: "Issuing responses API request".to_string(),
                });
                match client.responses(
                    model,
                    Value::String(RESPONSES_PROMPT.to_string()),
                    context.max_tokens,
                    0.0,
                    Some(&options),
                ) {
                    Ok(result) => ok_record(
                        self,
                        model,
                        Some(run_index),
                        "responses",
                        generation_metrics(&result),
                        Some(preview(&result.response_text, 180)),
                        options.clone(),
                    ),
                    Err(error) => err_record(
                        self,
                        model,
                        Some(run_index),
                        "responses",
                        error.to_string(),
                        options.clone(),
                    ),
                }
                .tap(|_| {
                    progress.on_step(BenchmarkStepUpdate {
                        status: BenchmarkStepStatus::Completed,
                        step_index: run_index,
                        total_steps,
                        run_index: Some(run_index),
                        prompt_name: Some("responses".to_string()),
                        message: "Completed responses API request".to_string(),
                    });
                })
            })
            .collect()
    }
}

impl Benchmark for StructuredOutputBenchmark {
    fn id(&self) -> &str {
        "structured-output"
    }

    fn name(&self) -> &str {
        "Structured JSON output"
    }

    fn description(&self) -> &str {
        "Requests schema-constrained JSON and validates the returned object shape."
    }

    fn suite(&self) -> BenchmarkSuite {
        BenchmarkSuite::Llm
    }

    fn planned_steps(&self, context: &BenchmarkContext) -> u32 {
        context.runs
    }

    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
        progress: &mut dyn BenchmarkProgressSink,
    ) -> Vec<BenchmarkResultRecord> {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "summary": {"type": "string"},
                "metrics": {"type": "array", "items": {"type": "string"}, "minItems": 3, "maxItems": 3},
                "recommendation": {"type": "string"}
            },
            "required": ["summary", "metrics", "recommendation"],
            "additionalProperties": false
        });
        let options = serde_json::json!({
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "benchmark_analysis",
                    "schema": schema,
                    "strict": true
                }
            }
        });
        let messages = serde_json::json!([
            {"role": "user", "content": STRUCTURED_OUTPUT_PROMPT}
        ]);
        let total_steps = self.planned_steps(context);

        (1..=context.runs)
            .map(|run_index| {
                progress.on_step(BenchmarkStepUpdate {
                    status: BenchmarkStepStatus::Started,
                    step_index: run_index,
                    total_steps,
                    run_index: Some(run_index),
                    prompt_name: Some("structured-json".to_string()),
                    message: "Issuing structured output request".to_string(),
                });
                match client.chat_completion(
                    model,
                    messages.clone(),
                    context.max_tokens,
                    0.0,
                    false,
                    Some(&options),
                ) {
                    Ok(result) => {
                        let mut metrics = generation_metrics(&result);
                        let parsed = serde_json::from_str::<Value>(&result.response_text).ok();
                        let valid = parsed
                            .as_ref()
                            .map(is_valid_structured_output)
                            .unwrap_or(false);
                        metrics.insert("schema_valid".to_string(), Value::from(valid));
                        metrics.insert(
                            "json_parse_success".to_string(),
                            Value::from(parsed.is_some()),
                        );
                        ok_record(
                            self,
                            model,
                            Some(run_index),
                            "structured-json",
                            metrics,
                            Some(preview(&result.response_text, 180)),
                            options.clone(),
                        )
                    }
                    Err(error) => err_record(
                        self,
                        model,
                        Some(run_index),
                        "structured-json",
                        error.to_string(),
                        options.clone(),
                    ),
                }
                .tap(|_| {
                    progress.on_step(BenchmarkStepUpdate {
                        status: BenchmarkStepStatus::Completed,
                        step_index: run_index,
                        total_steps,
                        run_index: Some(run_index),
                        prompt_name: Some("structured-json".to_string()),
                        message: "Completed structured output request".to_string(),
                    });
                })
            })
            .collect()
    }
}

impl Benchmark for ToolCallingBenchmark {
    fn id(&self) -> &str {
        "tool-calling"
    }

    fn name(&self) -> &str {
        "Function/tool calling"
    }

    fn description(&self) -> &str {
        "Requests a tool call and validates the selected function and arguments."
    }

    fn suite(&self) -> BenchmarkSuite {
        BenchmarkSuite::Llm
    }

    fn planned_steps(&self, context: &BenchmarkContext) -> u32 {
        context.runs
    }

    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
        progress: &mut dyn BenchmarkProgressSink,
    ) -> Vec<BenchmarkResultRecord> {
        let options = serde_json::json!({
            "tools": [{
                "type": "function",
                "function": {
                    "name": "record_benchmark_observation",
                    "description": "Record one benchmark observation.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "provider": {"type": "string"},
                            "metric": {"type": "string"},
                            "value": {"type": "number"}
                        },
                        "required": ["provider", "metric", "value"],
                        "additionalProperties": false
                    }
                }
            }],
            "tool_choice": {
                "type": "function",
                "function": {"name": "record_benchmark_observation"}
            }
        });
        let messages = serde_json::json!([
            {"role": "user", "content": TOOL_CALL_PROMPT}
        ]);
        let total_steps = self.planned_steps(context);

        (1..=context.runs)
            .map(|run_index| {
                progress.on_step(BenchmarkStepUpdate {
                    status: BenchmarkStepStatus::Started,
                    step_index: run_index,
                    total_steps,
                    run_index: Some(run_index),
                    prompt_name: Some("tool-call".to_string()),
                    message: "Issuing tool-calling request".to_string(),
                });
                match client.chat_completion(
                    model,
                    messages.clone(),
                    context.max_tokens,
                    0.0,
                    false,
                    Some(&options),
                ) {
                    Ok(result) => {
                        let mut metrics = generation_metrics(&result);
                        let tool_call = result.raw.pointer("/choices/0/message/tool_calls/0");
                        let valid = tool_call.map(is_valid_tool_call).unwrap_or(false);
                        metrics.insert("tool_call_valid".to_string(), Value::from(valid));
                        metrics.insert(
                            "tool_call_returned".to_string(),
                            Value::from(tool_call.is_some()),
                        );
                        ok_record(
                            self,
                            model,
                            Some(run_index),
                            "tool-call",
                            metrics,
                            Some(preview(&result.raw.to_string(), 180)),
                            options.clone(),
                        )
                    }
                    Err(error) => err_record(
                        self,
                        model,
                        Some(run_index),
                        "tool-call",
                        error.to_string(),
                        options.clone(),
                    ),
                }
                .tap(|_| {
                    progress.on_step(BenchmarkStepUpdate {
                        status: BenchmarkStepStatus::Completed,
                        step_index: run_index,
                        total_steps,
                        run_index: Some(run_index),
                        prompt_name: Some("tool-call".to_string()),
                        message: "Completed tool-calling request".to_string(),
                    });
                })
            })
            .collect()
    }
}

impl Benchmark for EmbeddingsBenchmark {
    fn id(&self) -> &str {
        "embeddings"
    }

    fn name(&self) -> &str {
        "Embeddings API"
    }

    fn description(&self) -> &str {
        "Measures /v1/embeddings latency and returned vector dimensions."
    }

    fn suite(&self) -> BenchmarkSuite {
        BenchmarkSuite::Embeddings
    }

    fn planned_steps(&self, context: &BenchmarkContext) -> u32 {
        context.runs
    }

    fn run(
        &self,
        client: &ProviderClient,
        model: &str,
        context: &BenchmarkContext,
        progress: &mut dyn BenchmarkProgressSink,
    ) -> Vec<BenchmarkResultRecord> {
        let total_steps = self.planned_steps(context);
        (1..=context.runs)
            .map(|run_index| {
                progress.on_step(BenchmarkStepUpdate {
                    status: BenchmarkStepStatus::Started,
                    step_index: run_index,
                    total_steps,
                    run_index: Some(run_index),
                    prompt_name: Some("embedding".to_string()),
                    message: "Issuing embeddings request".to_string(),
                });
                match client.embeddings(model, Value::String(EMBEDDINGS_INPUT.to_string())) {
                    Ok(result) => {
                        let mut metrics = generation_metrics(&result);
                        let dimensions = result
                            .raw
                            .pointer("/data/0/embedding")
                            .and_then(|v| v.as_array())
                            .map(|v| v.len())
                            .unwrap_or(0);
                        metrics.insert(
                            "embedding_dimensions".to_string(),
                            Value::from(dimensions as u64),
                        );
                        metrics.insert(
                            "embedding_vectors".to_string(),
                            Value::from(
                                result
                                    .raw
                                    .get("data")
                                    .and_then(|v| v.as_array())
                                    .map(|v| v.len())
                                    .unwrap_or(0) as u64,
                            ),
                        );
                        ok_record(
                            self,
                            model,
                            Some(run_index),
                            "embedding",
                            metrics,
                            None,
                            serde_json::json!({"input_chars": EMBEDDINGS_INPUT.len()}),
                        )
                    }
                    Err(error) => err_record(
                        self,
                        model,
                        Some(run_index),
                        "embedding",
                        error.to_string(),
                        serde_json::json!({"input_chars": EMBEDDINGS_INPUT.len()}),
                    ),
                }
                .tap(|_| {
                    progress.on_step(BenchmarkStepUpdate {
                        status: BenchmarkStepStatus::Completed,
                        step_index: run_index,
                        total_steps,
                        run_index: Some(run_index),
                        prompt_name: Some("embedding".to_string()),
                        message: "Completed embeddings request".to_string(),
                    });
                })
            })
            .collect()
    }
}

trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }
}

impl<T> Tap for T {}

fn ok_record<B: Benchmark + ?Sized>(
    benchmark: &B,
    model: &str,
    run_index: Option<u32>,
    prompt_name: &str,
    metrics: HashMap<String, Value>,
    response_preview: Option<String>,
    options: Value,
) -> BenchmarkResultRecord {
    BenchmarkResultRecord {
        benchmark_id: benchmark.id().to_string(),
        benchmark_name: benchmark.name().to_string(),
        model: model.to_string(),
        run_index,
        prompt_name: Some(prompt_name.to_string()),
        metrics,
        response_preview,
        error: None,
        metadata: Some({
            let mut m = HashMap::new();
            m.insert("request".to_string(), options);
            m
        }),
    }
}

fn err_record<B: Benchmark + ?Sized>(
    benchmark: &B,
    model: &str,
    run_index: Option<u32>,
    prompt_name: &str,
    error: String,
    options: Value,
) -> BenchmarkResultRecord {
    BenchmarkResultRecord {
        benchmark_id: benchmark.id().to_string(),
        benchmark_name: benchmark.name().to_string(),
        model: model.to_string(),
        run_index,
        prompt_name: Some(prompt_name.to_string()),
        metrics: HashMap::new(),
        response_preview: None,
        error: Some(error),
        metadata: Some({
            let mut m = HashMap::new();
            m.insert("request".to_string(), options);
            m
        }),
    }
}

fn is_valid_structured_output(value: &Value) -> bool {
    value.get("summary").and_then(|v| v.as_str()).is_some()
        && value
            .get("metrics")
            .and_then(|v| v.as_array())
            .map(|items| items.len() == 3 && items.iter().all(|item| item.as_str().is_some()))
            .unwrap_or(false)
        && value
            .get("recommendation")
            .and_then(|v| v.as_str())
            .is_some()
}

fn is_valid_tool_call(value: &Value) -> bool {
    let name_ok = value.pointer("/function/name").and_then(|v| v.as_str())
        == Some("record_benchmark_observation");
    let args = value
        .pointer("/function/arguments")
        .and_then(|v| v.as_str())
        .and_then(|text| serde_json::from_str::<Value>(text).ok());
    let args_ok = args
        .as_ref()
        .map(|v| {
            v.get("provider").and_then(|p| p.as_str()).is_some()
                && v.get("metric").and_then(|m| m.as_str()) == Some("time_to_first_token_ms")
                && v.get("value").and_then(|n| n.as_f64()).is_some()
        })
        .unwrap_or(false);
    name_ok && args_ok
}
