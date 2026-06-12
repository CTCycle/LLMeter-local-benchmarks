use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLMeterError {
    #[error("The 'ollama' executable was not found on PATH.")]
    OllamaNotInstalled,

    #[error("{0}")]
    OllamaServer(String),

    #[error("{0}")]
    ModelNotFound(String),

    #[error("{0}")]
    Benchmark(String),

    #[error("{0}")]
    InvalidOption(String),

    #[error("{0}")]
    Io(String),
}

impl From<std::io::Error> for LLMeterError {
    fn from(err: std::io::Error) -> Self {
        LLMeterError::Io(err.to_string())
    }
}
