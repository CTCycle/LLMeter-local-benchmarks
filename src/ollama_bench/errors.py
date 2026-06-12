class OllamaBenchError(Exception):
    """Base exception for expected CLI errors."""


class OllamaNotInstalledError(OllamaBenchError):
    """Raised when the ollama executable cannot be found."""


class OllamaServerError(OllamaBenchError):
    """Raised when the Ollama API server is unavailable or returns an error."""


class ModelNotFoundError(OllamaBenchError):
    """Raised when a requested model is not installed locally."""


class BenchmarkError(OllamaBenchError):
    """Raised when a benchmark cannot be executed."""
