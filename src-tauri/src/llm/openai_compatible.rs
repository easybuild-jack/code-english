//! OpenAI 兼容适配器：DeepSeek / 豆包 / 通义 / Ollama / OpenAI 都走这一份代码，
//! 只有 base_url、api_key、model 不同。

use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::{ChatMessage, ChatRequest, ChatResponse, LlmProvider};
use crate::db::models::ProviderConfig;
use crate::error::{AppError, AppResult};

const TIMEOUT: Duration = Duration::from_secs(15);

pub struct OpenAiCompatibleProvider {
    cfg: ProviderConfig,
    api_key: String,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(cfg: ProviderConfig, api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .expect("reqwest client");
        Self { cfg, api_key, client }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.cfg.base_url.trim_end_matches('/'), path.trim_start_matches('/'))
    }

    fn is_deepseek(&self) -> bool {
        self.cfg.base_url.contains("deepseek.com")
    }
}

#[derive(Deserialize)]
struct ChatCompletion {
    model: Option<String>,
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    error: Option<ApiError>,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &str {
        &self.cfg.id
    }

    fn display_name(&self) -> &str {
        &self.cfg.display_name
    }

    async fn chat(&self, req: ChatRequest) -> AppResult<ChatResponse> {
        if self.api_key.trim().is_empty() {
            return Err(AppError::MissingApiKey);
        }

        let mut messages = vec![json!({ "role": "system", "content": req.system })];
        messages.extend(req.messages.iter().map(|m| json!({ "role": m.role, "content": m.content })));
        let mut body = json!({
            "model": self.cfg.model,
            "temperature": req.temperature,
            "messages": messages,
        });
        if req.json_mode {
            body["response_format"] = json!({ "type": "json_object" });
        }
        // DeepSeek V4 默认开思考模式（effort=high），翻译用不上，只会慢且贵。
        // 这是 DeepSeek 私有参数，别的服务可能报「未知参数」，所以只对它下发。
        if self.is_deepseek() {
            body["thinking"] = json!({ "type": "disabled" });
        }

        let resp = self
            .client
            .post(self.url("chat/completions"))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            let msg = serde_json::from_str::<ApiErrorBody>(&text)
                .ok()
                .and_then(|b| b.error)
                .map(|e| e.message)
                .unwrap_or_else(|| text.clone());
            return Err(match status.as_u16() {
                401 => AppError::Other("API Key 无效".into()),
                402 => AppError::Other("余额不足".into()),
                429 => AppError::Other("请求过于频繁，稍后再试".into()),
                _ => AppError::Other(format!("模型服务返回 {}: {}", status.as_u16(), msg)),
            });
        }

        let parsed: ChatCompletion = serde_json::from_str(&text)
            .map_err(|e| AppError::BadModelOutput(format!("{e}: {text}")))?;
        let content = parsed
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| AppError::BadModelOutput("模型没有返回内容".into()))?;

        Ok(ChatResponse {
            content,
            model: parsed.model.unwrap_or_else(|| self.cfg.model.clone()),
        })
    }

    async fn test_connection(&self) -> AppResult<u128> {
        let started = Instant::now();
        self.chat(ChatRequest {
            system: "You are a ping service.".into(),
            messages: vec![ChatMessage::user("Reply with the single word: pong")],
            temperature: 0.0,
            json_mode: false,
        })
        .await?;
        Ok(started.elapsed().as_millis())
    }
}
