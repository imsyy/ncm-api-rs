/// 错误类型定义
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NcmError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (code={code}): {msg}")]
    Api { code: i64, msg: String },

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, NcmError>;
