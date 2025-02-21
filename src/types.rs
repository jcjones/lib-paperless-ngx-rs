#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PaperlessError {
    #[error("Too many tasks supplied")]
    TooManyTasks(),

    #[error("The configuration was incomplete")]
    IncompleteConfig(),

    #[error("API interaction error: {0}")]
    APIError(#[from] reqwest::Error),

    #[error("I/O error")]
    Io(#[from] std::io::Error),
}
