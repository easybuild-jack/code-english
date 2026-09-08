use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("网络错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("钥匙串错误: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("还没配置 API Key")]
    MissingApiKey,

    #[error("模型返回格式异常: {0}")]
    BadModelOutput(String),

    #[error("{0}")]
    Other(String),
}

/// 前端只需要一段可读的错误文本，按 PRD 4.8 直接展示。
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
