use std::collections::HashMap;

pub const SHORT_PROMPT: &str =
    "Reply with exactly one concise sentence explaining what a benchmark measures.";

pub const MEDIUM_PROMPT: &str =
    "Summarize the practical tradeoffs of running a small language model locally instead of calling a hosted API.\n\
     Keep the answer to one short paragraph and mention latency, privacy, and hardware constraints.";

pub const LONG_PROMPT: &str =
    "You are evaluating a local language model for developer productivity tasks.\n\
     Write a compact but useful analysis of the following scenario:\n\
     \n\
     A developer wants to use a local model for code explanation, short refactors, and project documentation.\n\
     The machine is a consumer laptop with limited memory and no dedicated server GPU.\n\
     The developer values privacy, predictable cost, and offline availability, but does not want to maintain a complex inference stack.\n\
     \n\
     Discuss expected strengths, expected limitations, and what should be measured before adopting the setup.\n\
     Keep the response under 250 words.";

pub const CONSISTENCY_PROMPT: &str =
    "Return a JSON object with exactly three keys: summary, risks, recommendation.\n\
     The topic is: using a local LLM for lightweight software engineering assistance.\n\
     Keep each value short.";

pub const RESPONSES_PROMPT: &str =
    "In two concise bullet points, explain what a local LLM benchmark should measure.";

pub const STRUCTURED_OUTPUT_PROMPT: &str =
    "Analyze local LLM benchmarking. Return only a JSON object with keys summary, metrics, and recommendation. metrics must be an array of three short strings.";

pub const TOOL_CALL_PROMPT: &str =
    "Use the available tool to record a benchmark observation for model latency. The provider is local and the metric is time_to_first_token_ms with value 125.";

pub const EMBEDDINGS_INPUT: &str =
    "Local LLM benchmarking measures latency, throughput, reliability, and API capability support.";

pub fn prompts_by_size() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("short", SHORT_PROMPT);
    map.insert("medium", MEDIUM_PROMPT);
    map.insert("long", LONG_PROMPT);
    map
}
