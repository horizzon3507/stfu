use thiserror::Error;

pub type StfuResult<T> = Result<T, StfuError>;

#[derive(Error, Debug)]
pub enum StfuError {
    #[error("Could not resolve WirePlumber configuration directory: {0}")]
    ConfigDirResolution(String),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write temporary configuration file: {0}")]
    TempFile(#[from] std::io::Error),

    #[error("Failed to persist configuration file atomically: {0}")]
    Persist(String),

    #[error("Failed to execute systemctl: {0}")]
    SystemctlExec(String),

    #[error("systemctl exited with non-zero status: {0}")]
    SystemctlStatus(String),

    #[error("Failed to parse device inspection output: {0}")]
    DeviceParse(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}
