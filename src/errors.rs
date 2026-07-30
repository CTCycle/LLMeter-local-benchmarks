use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LLMeterError {
    #[error("{0}")]
    Provider(String),

    #[error("{0}")]
    ModelNotFound(String),

    #[error("{0}")]
    Benchmark(String),

    #[error("{0}")]
    InvalidOption(String),

    #[error("{0}")]
    Configuration(String),

    #[error("{0}")]
    Io(String),

    #[error("Operation interrupted by the user.")]
    Interrupted,

    #[error("Operation canceled by the user.")]
    Canceled,
}

impl From<std::io::Error> for LLMeterError {
    fn from(err: std::io::Error) -> Self {
        LLMeterError::Io(err.to_string())
    }
}
