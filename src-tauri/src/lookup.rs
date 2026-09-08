//! 点词 / 划词查释义：用户在翻译结果里点了某个词或划了一段短语，
//! 结合所在句子给出它在这句话里的中文释义。结果不落库，前端按需缓存。

use serde::{Deserialize, Serialize};

use crate::db::models::Profile;
use crate::llm::{ChatMessage, ChatRequest};
use crate::translate::{scene_label, strip_code_fence};

#[derive(Debug, Clone, Serialize)]
pub struct LookupResult {
    pub text: String,
    /// 结合上下文的中文释义
    pub meaning: String,
    /// 词性缩写，如 n. / v. / adj. / phr.；解析失败时为空
    pub pos: String,
}

pub fn build_request(profile: &Profile, scene: &str, text: &str, context: &str) -> ChatRequest {
    let industry = if profile.industry.trim().is_empty() {
        String::new()
    } else {
        format!("用户所在行业：{}。\n", profile.industry.trim())
    };

    let system = format!(
        "你是一名英语母语者，同时是资深{occupation}，技术栈为 {stack}，了解中国程序员的工作和生活语境。\n\
用户是一名中文{occupation}，正在阅读一句英文，点了其中一个词或短语想知道意思。\n\
{industry}\
当前场景：{scene}。\n\
\n\
请结合所在句子解释这个词或短语。要求：\n\
1. 先给它在这句话里的意思；如果它还有更常见的其他含义，用分号再补一个，总共不超过 30 个字。\n\
2. 技术词给程序员圈里通行的中文说法（如 endpoint → 接口 / 端点）。\n\
3. 不要解释整句，不要举例，不要寒暄。\n\
\n\
只输出一个 JSON 对象，不要输出其他文字，格式：\n\
{{ \"meaning\": \"<中文释义>\", \"pos\": \"<词性缩写：n. / v. / adj. / adv. / prep. / phr. 等，短语填 phr.>\" }}",
        occupation = profile.occupation.trim(),
        stack = profile.tech_stack.trim(),
        industry = industry,
        scene = scene_label(scene),
    );

    let user = format!("句子：{}\n查询：{}", context.trim(), text.trim());

    ChatRequest {
        system,
        messages: vec![ChatMessage::user(&user)],
        temperature: 0.2,
        json_mode: true,
    }
}

#[derive(Deserialize)]
struct RawOutput {
    meaning: String,
    #[serde(default)]
    pos: String,
}

/// 解析失败就把原文当释义，不报错——查个词不值得打断用户
pub fn parse_output(text: &str, content: &str) -> LookupResult {
    let cleaned = strip_code_fence(content.trim());
    match serde_json::from_str::<RawOutput>(cleaned) {
        Ok(raw) if !raw.meaning.trim().is_empty() => LookupResult {
            text: text.to_string(),
            meaning: raw.meaning.trim().to_string(),
            pos: raw.pos.trim().to_string(),
        },
        _ => LookupResult {
            text: text.to_string(),
            meaning: cleaned.to_string(),
            pos: String::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json() {
        let r = parse_output("endpoint", r#"{"meaning":"接口；端点","pos":"n."}"#);
        assert_eq!(r.meaning, "接口；端点");
        assert_eq!(r.pos, "n.");
    }

    #[test]
    fn falls_back_to_raw() {
        let r = parse_output("endpoint", "接口");
        assert_eq!(r.meaning, "接口");
        assert!(r.pos.is_empty());
    }
}
