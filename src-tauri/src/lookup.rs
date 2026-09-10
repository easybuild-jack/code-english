//! 点词 / 划词查释义：用户在翻译结果里点了某个词或划了一段短语，
//! 结合所在句子给出它在这句话里的中文释义。结果不落库，前端按需缓存。

use serde::{Deserialize, Serialize};

use crate::db::models::Profile;
use crate::llm::{ChatMessage, ChatRequest};
use crate::translate::{scene_label, strip_code_fence};

#[derive(Debug, Clone, Serialize)]
pub struct LookupResult {
    pub text: String,
    /// 当前例句和用户背景下的准确释义
    pub meaning: String,
    /// 不适用于当前例句、但常用的其他释义
    pub other_meanings: Vec<String>,
    /// 词性缩写，如 n. / v. / adj. / phr.；解析失败时为空
    pub pos: String,
    /// 按全局口音设置返回，不含两侧斜杠
    pub ipa: String,
}

pub fn build_request(
    profile: &Profile,
    scene: &str,
    text: &str,
    context: &str,
    source: &str,
    accent: &str,
) -> ChatRequest {
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
请结合用户职业、技术栈、场景、中文原文和完整英文例句解释查询内容。要求：\n\
1. meaning 只能给查询内容在当前完整例句中的准确含义。必须优先按当前技术语境消歧，例如代码语境中的 class 是“类”，不能是“班级”。\n\
2. other_meanings 列出不适用于当前例句、但这个词常用的其他中文释义；去重、简短、尽量完整，每项只写一个义项。短语没有其他常见义时返回空数组。\n\
3. 技术词使用程序员圈通行的中文说法（如 endpoint → 接口 / 端点）。\n\
4. ipa 返回查询内容的{accent_name} IPA 音标，不要带两侧 / /；短语给出整段连读音标。\n\
5. 不解释整句，不举例，不寒暄。\n\
\n\
只输出一个 JSON 对象，不要输出其他文字，格式：\n\
{{ \"meaning\": \"<当前语境释义>\", \"other_meanings\": [\"<其他常见释义>\"], \"pos\": \"<当前语境词性缩写：n. / v. / adj. / adv. / prep. / phr. 等，短语填 phr.>\", \"ipa\": \"<IPA>\" }}",
        occupation = profile.occupation.trim(),
        stack = profile.tech_stack.trim(),
        industry = industry,
        scene = scene_label(scene),
        accent_name = if accent == "uk" { "英式" } else { "美式" },
    );

    let user = format!(
        "中文原文：{}\n完整英文例句：{}\n查询：{}",
        source.trim(),
        context.trim(),
        text.trim()
    );

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
    other_meanings: Vec<String>,
    #[serde(default)]
    pos: String,
    #[serde(default)]
    ipa: String,
}

/// 解析失败就把原文当释义，不报错——查个词不值得打断用户
pub fn parse_output(text: &str, content: &str) -> LookupResult {
    let cleaned = strip_code_fence(content.trim());
    // 部分兼容接口会再包一层 {"type":"json_object","content":"{...}"}。
    // 先取出 content，避免把外层 JSON 原样显示给用户。
    let nested = serde_json::from_str::<serde_json::Value>(cleaned)
        .ok()
        .and_then(|v| v.get("content")?.as_str().map(str::to_string));
    let candidate = nested
        .as_deref()
        .map(|v| strip_code_fence(v.trim()))
        .unwrap_or(cleaned);
    match serde_json::from_str::<RawOutput>(candidate) {
        Ok(raw) if !raw.meaning.trim().is_empty() => LookupResult {
            text: text.to_string(),
            meaning: raw.meaning.trim().to_string(),
            other_meanings: raw
                .other_meanings
                .into_iter()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect(),
            pos: raw.pos.trim().to_string(),
            ipa: raw.ipa.trim().trim_matches('/').to_string(),
        },
        _ => LookupResult {
            text: text.to_string(),
            meaning: candidate.to_string(),
            other_meanings: vec![],
            pos: String::new(),
            ipa: String::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json() {
        let r = parse_output(
            "class",
            r#"{"meaning":"类","other_meanings":["班级","阶级","等级"],"pos":"n.","ipa":"klæs"}"#,
        );
        assert_eq!(r.meaning, "类");
        assert_eq!(r.other_meanings, ["班级", "阶级", "等级"]);
        assert_eq!(r.pos, "n.");
        assert_eq!(r.ipa, "klæs");
    }

    #[test]
    fn falls_back_to_raw() {
        let r = parse_output("endpoint", "接口");
        assert_eq!(r.meaning, "接口");
        assert!(r.pos.is_empty());
    }

    #[test]
    fn unwraps_json_object_envelope() {
        let r = parse_output(
            "push",
            r#"{"type":"json_object","content":"{\"meaning\":\"推送\",\"other_meanings\":[\"推动\"],\"pos\":\"v.\",\"ipa\":\"pʊʃ\"}"}"#,
        );
        assert_eq!(r.meaning, "推送");
        assert_eq!(r.other_meanings, ["推动"]);
    }

    #[test]
    fn request_contains_full_context_and_disambiguation_rule() {
        let profile = Profile {
            occupation: "程序员".into(),
            tech_stack: "Java 全栈".into(),
            industry: "软件开发".into(),
            default_scene: "work".into(),
        };
        let r = build_request(
            &profile,
            "work",
            "class",
            "Review all the methods in this class.",
            "检查这个类的所有方法",
            "us",
        );
        assert!(r.system.contains("class 是“类”"));
        assert!(r.system.contains("美式 IPA"));
        assert!(r.messages[0].content.contains("检查这个类的所有方法"));
        assert!(r.messages[0].content.contains("Review all the methods in this class."));
    }
}
