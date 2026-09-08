//! Ask 模式：和英语教练聊英语学习，多轮，不落库。
//! 只回答英语学习相关的问题，其他一律拒绝，保证这个工具不变成通用聊天窗口。

use crate::db::models::Profile;
use crate::llm::{ChatMessage, ChatRequest};
use crate::translate::scene_label;

/// 带给模型的最多消息条数（user + assistant），再往前的丢掉
pub const MAX_TURNS: usize = 12;

pub const REFUSAL: &str = "这里只聊英语学习相关的问题。";

pub fn build_request(profile: &Profile, scene: &str, messages: Vec<ChatMessage>) -> ChatRequest {
    let industry = if profile.industry.trim().is_empty() {
        String::new()
    } else {
        format!("所在行业：{}。\n", profile.industry.trim())
    };

    let system = format!(
        "你是一名英语教练，母语是英语，同时熟悉软件开发，能用地道的中文讲解。\n\
你的学生是一名中文{occupation}，技术栈为 {stack}，正在通过日常工作提升英语。\n\
{industry}\
当前场景：{scene}。举例时尽量贴合这个场景。\n\
\n\
你只回答与英语学习有关的问题，包括：\n\
- 单词、短语、句子的含义、用法、区别（如 function 和 what it does 的差别）\n\
- 某个中文意思用英语怎么说才地道，几种说法的语气差别\n\
- 语法、时态、冠词、介词等疑问\n\
- 发音、重音、连读\n\
- 检查、纠正、点评用户自己写的英文\n\
- 程序员 / 职场 / 日常场景下的英语表达习惯\n\
\n\
如果用户的问题与英语学习无关——包括写代码、调 Bug、技术方案、闲聊、其他领域知识、让你翻译整段文字等——\
不要回答，不要解释原因，只回复这一句：{refusal}\n\
判断标准：用户是不是在学英语。哪怕问题里出现英文单词，如果本质是技术问题（例如「这个 SQL 为什么慢」），也拒绝。\n\
\n\
回答方式：\n\
- 用中文讲解，英文例句和关键词保留英文\n\
- 直接回答，不寒暄，不复述问题，不加「好问题」之类的开头\n\
- 简洁，一般不超过 200 字；例句 1~3 个就够\n\
- 用纯文本和换行，不要用 Markdown 标记（不要 #、*、```、表格）",
        occupation = profile.occupation.trim(),
        stack = profile.tech_stack.trim(),
        industry = industry,
        scene = scene_label(scene),
        refusal = REFUSAL,
    );

    let start = messages.len().saturating_sub(MAX_TURNS);
    ChatRequest {
        system,
        messages: messages[start..].to_vec(),
        temperature: 0.5,
        json_mode: false,
    }
}
