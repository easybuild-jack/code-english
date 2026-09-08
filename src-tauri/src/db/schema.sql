-- CodeEnglish 本地数据库
-- 对应《需求背景文档》第 6 章「本地数据」与《产品PRD》第 5、6、7 章

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- 我的档案：单行表，id 固定为 1
CREATE TABLE IF NOT EXISTS profile (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    occupation    TEXT NOT NULL DEFAULT '程序员',
    tech_stack    TEXT NOT NULL DEFAULT 'Java, Spring, 前端',
    industry      TEXT NOT NULL DEFAULT '',
    default_scene TEXT NOT NULL DEFAULT 'work',          -- work | workplace | daily | shopping | travel
    updated_at    TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
INSERT OR IGNORE INTO profile (id) VALUES (1);
-- 场景枚举调整后的迁移：旧值统一归到 work
UPDATE profile SET default_scene = 'work'
WHERE default_scene NOT IN ('work', 'workplace', 'daily', 'shopping', 'travel');

-- 模型配置。api_key 不在库里，存系统钥匙串，键为 provider id
CREATE TABLE IF NOT EXISTS llm_providers (
    id           TEXT PRIMARY KEY,                        -- deepseek / doubao / openai ...
    display_name TEXT NOT NULL,
    kind         TEXT NOT NULL DEFAULT 'openai_compatible', -- openai_compatible | anthropic | gemini
    base_url     TEXT NOT NULL,
    model        TEXT NOT NULL,
    enabled      INTEGER NOT NULL DEFAULT 1,
    is_default   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
-- 一期只预置 DeepSeek。翻译用不上推理，选最便宜的 flash
INSERT OR IGNORE INTO llm_providers (id, display_name, kind, base_url, model, enabled, is_default)
VALUES ('deepseek', 'DeepSeek', 'openai_compatible', 'https://api.deepseek.com/v1', 'deepseek-v4-flash', 1, 1);
-- deepseek-chat / deepseek-reasoner 已于 2026-07-24 下线，老库里的预置值迁到 flash
UPDATE llm_providers SET model = 'deepseek-v4-flash'
WHERE id = 'deepseek' AND model IN ('deepseek-chat', 'deepseek-reasoner');

-- 翻译历史，同时也是练习记录
CREATE TABLE IF NOT EXISTS translation_history (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    source_text   TEXT NOT NULL,
    translation   TEXT NOT NULL,
    keywords_json TEXT NOT NULL DEFAULT '[]',             -- [{word, note}]
    alternatives_json TEXT NOT NULL DEFAULT '[]',         -- 已弃用，保留列避免迁移
    scene         TEXT NOT NULL,
    provider_id   TEXT NOT NULL,
    model         TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_history_created ON translation_history (created_at DESC);
UPDATE translation_history SET scene = 'work'
WHERE scene NOT IN ('work', 'workplace', 'daily', 'shopping', 'travel');

-- 收藏：单词 / 短语 / 句子
CREATE TABLE IF NOT EXISTS favorites (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    kind         TEXT NOT NULL,                           -- word | phrase | sentence
    text         TEXT NOT NULL,
    text_lower   TEXT NOT NULL,                           -- 去重与统计用
    meaning      TEXT NOT NULL DEFAULT '',
    domain       TEXT NOT NULL DEFAULT '',
    example      TEXT NOT NULL DEFAULT '',
    seen_count   INTEGER NOT NULL DEFAULT 0,              -- 在翻译历史中出现的次数
    mastered     INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    UNIQUE (text_lower)
);
CREATE INDEX IF NOT EXISTS idx_fav_kind ON favorites (kind, mastered);
