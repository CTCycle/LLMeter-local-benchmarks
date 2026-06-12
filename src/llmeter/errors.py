class LLMeterError(Exception):
    """Base exception for expected CLI errors."""


class OllamaNotInstalledError(LLMeterError):
    """Raised when the ollama executable cannot be found."""


class OllamaServerError(LLMeterError):
    """Raised when the Ollama API server is unavailable or returns an error."""


class ModelNotFoundError(LLMeterError):
    """Raised when a requested model is not installed locally."""


class BenchmarkError(LLMeterError):
    """Raised when a benchmark cannot be executed."""
