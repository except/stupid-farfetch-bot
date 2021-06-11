use std::time::SystemTimeError;

use thiserror::Error;
use tokio::sync::broadcast::error::RecvError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("from_utf8={0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("invald_header_value={0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderName),

    #[error("tokio_ join={0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("url_parse={0}")]
    ParseError(#[from] url::ParseError),

    #[error("broadcast_recv={0}")]
    RecvError(#[from] RecvError),

    #[error("reqwest={0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("serde_json={0}")]
    SerdeJSON(#[from] serde_json::Error),

    #[error("system_time={0}")]
    SystemTimeError(#[from] SystemTimeError),

    #[error("timeout={0}")]
    TimeoutError(#[from] tokio::time::error::Elapsed),

    #[error("tokio_io={0}")]
    TokioIO(#[from] tokio::io::Error),

    #[error("unknown={0}")]
    Unknown(String),
}
