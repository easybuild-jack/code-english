//! 模型接入层。业务代码只依赖 `LlmProvider` trait，
//! 加新厂商只需要新增一个实现并在 `build_provider` 里注册。

pub mod openai_compatible;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::db::models::ProviderConfig;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// user | assistant
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub system: String,
    /// 不含 system，按时间顺序；翻译只有一条 user，Ask 模式带多轮
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    /// 要求模型输出 JSON（OpenAI 兼容接口的 response_format）
    pub json_mode: bool,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    // id / display_name 二期做模型下拉框时使用
    #[allow(dead_code)]
    fn id(&self) -> &str;
    #[allow(dead_code)]
    fn display_name(&self) -> &str;
    async fn chat(&self, req: ChatRequest) -> AppResult<ChatResponse>;
    /// 设置页「测试连接」
    async fn test_connection(&self) -> AppResult<u128>;
}

/// 根据配置和 Key 构造具体实现。一期只有 openai_compatible。
pub fn build_provider(cfg: &ProviderConfig, api_key: String) -> AppResult<Box<dyn LlmProvider>> {
    match cfg.kind.as_str() {
        "openai_compatible" => Ok(Box::new(openai_compatible::OpenAiCompatibleProvider::new(
            cfg.clone(),
            api_key,
        ))),
        other => Err(AppError::Other(format!("暂不支持的模型类型: {other}"))),
    }
}
