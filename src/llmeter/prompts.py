SHORT_PROMPT = "Reply with exactly one concise sentence explaining what a benchmark measures."

MEDIUM_PROMPT = """
Summarize the practical tradeoffs of running a small language model locally instead of calling a hosted API.
Keep the answer to one short paragraph and mention latency, privacy, and hardware constraints.
""".strip()

LONG_PROMPT = """
You are evaluating a local language model for developer productivity tasks.
Write a compact but useful analysis of the following scenario:

A developer wants to use a local model for code explanation, short refactors, and project documentation.
The machine is a consumer laptop with limited memory and no dedicated server GPU.
The developer values privacy, predictable cost, and offline availability, but does not want to maintain a complex inference stack.

Discuss expected strengths, expected limitations, and what should be measured before adopting the setup.
Keep the response under 250 words.
""".strip()

CONSISTENCY_PROMPT = """
Return a JSON object with exactly three keys: summary, risks, recommendation.
The topic is: using a local LLM for lightweight software engineering assistance.
Keep each value short.
""".strip()

PROMPTS_BY_SIZE = {
    "short": SHORT_PROMPT,
    "medium": MEDIUM_PROMPT,
    "long": LONG_PROMPT,
}
