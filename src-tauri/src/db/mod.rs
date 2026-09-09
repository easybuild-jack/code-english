pub mod models;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppResult;
use models::*;

const SCHEMA: &str = include_str!("schema.sql");

/// 单连接 + Mutex 足够：个人工具，并发写入极少。
pub struct Db {
    conn: Mutex<Connection>,
    pub path: PathBuf,
}

impl Db {
    pub fn open(dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("codeenglish.db");
        let conn = Connection::open(&path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    fn with<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        f(&conn)
    }

    // ---------- profile ----------

    pub fn get_profile(&self) -> AppResult<Profile> {
        self.with(|c| {
            Ok(c.query_row(
                "SELECT occupation, tech_stack, industry, default_scene FROM profile WHERE id = 1",
                [],
                |r| {
                    Ok(Profile {
                        occupation: r.get(0)?,
                        tech_stack: r.get(1)?,
                        industry: r.get(2)?,
                        default_scene: r.get(3)?,
                    })
                },
            )?)
        })
    }

    pub fn save_profile(&self, p: &Profile) -> AppResult<()> {
        self.with(|c| {
            c.execute(
                "UPDATE profile SET occupation=?1, tech_stack=?2, industry=?3, default_scene=?4,
                 updated_at=datetime('now','localtime') WHERE id = 1",
                params![p.occupation, p.tech_stack, p.industry, p.default_scene],
            )?;
            Ok(())
        })
    }

    // ---------- providers ----------

    pub fn list_providers(&self) -> AppResult<Vec<ProviderConfig>> {
        self.with(|c| {
            let mut stmt = c.prepare(
                "SELECT id, display_name, kind, base_url, model, enabled, is_default
                 FROM llm_providers ORDER BY is_default DESC, created_at",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok(ProviderConfig {
                    id: r.get(0)?,
                    display_name: r.get(1)?,
                    kind: r.get(2)?,
                    base_url: r.get(3)?,
                    model: r.get(4)?,
                    enabled: r.get::<_, i64>(5)? != 0,
                    is_default: r.get::<_, i64>(6)? != 0,
                })
            })?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
    }

    pub fn get_provider(&self, id: &str) -> AppResult<Option<ProviderConfig>> {
        self.with(|c| {
            Ok(c.query_row(
                "SELECT id, display_name, kind, base_url, model, enabled, is_default
                 FROM llm_providers WHERE id = ?1",
                [id],
                |r| {
                    Ok(ProviderConfig {
                        id: r.get(0)?,
                        display_name: r.get(1)?,
                        kind: r.get(2)?,
                        base_url: r.get(3)?,
                        model: r.get(4)?,
                        enabled: r.get::<_, i64>(5)? != 0,
                        is_default: r.get::<_, i64>(6)? != 0,
                    })
                },
            )
            .optional()?)
        })
    }

    pub fn default_provider(&self) -> AppResult<Option<ProviderConfig>> {
        Ok(self
            .list_providers()?
            .into_iter()
            .find(|p| p.enabled && p.is_default))
    }

    pub fn save_provider(&self, p: &ProviderConfig) -> AppResult<()> {
        self.with(|c| {
            if p.is_default {
                c.execute("UPDATE llm_providers SET is_default = 0", [])?;
            }
            c.execute(
                "INSERT INTO llm_providers (id, display_name, kind, base_url, model, enabled, is_default)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET
                   display_name=excluded.display_name, kind=excluded.kind, base_url=excluded.base_url,
                   model=excluded.model, enabled=excluded.enabled, is_default=excluded.is_default",
                params![
                    p.id,
                    p.display_name,
                    p.kind,
                    p.base_url,
                    p.model,
                    p.enabled as i64,
                    p.is_default as i64
                ],
            )?;
            Ok(())
        })
    }

    // ---------- history ----------

    pub fn insert_history(&self, h: &NewHistory) -> AppResult<i64> {
        self.with(|c| {
            c.execute(
                "INSERT INTO translation_history
                 (source_text, translation, keywords_json, scene, provider_id, model)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    h.source_text,
                    h.translation,
                    h.keywords_json,
                    h.scene,
                    h.provider_id,
                    h.model
                ],
            )?;
            Ok(c.last_insert_rowid())
        })
    }

    pub fn list_history(&self, query: Option<&str>, limit: i64, offset: i64) -> AppResult<Vec<HistoryItem>> {
        self.with(|c| {
            let like = query
                .filter(|q| !q.trim().is_empty())
                .map(|q| format!("%{}%", q.trim()));
            let mut stmt = c.prepare(
                "SELECT id, source_text, translation, keywords_json, scene, provider_id, model, created_at
                 FROM translation_history
                 WHERE (?1 IS NULL OR source_text LIKE ?1 OR translation LIKE ?1)
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2 OFFSET ?3",
            )?;
            let rows = stmt.query_map(params![like, limit, offset], |r| {
                Ok(HistoryItem {
                    id: r.get(0)?,
                    source_text: r.get(1)?,
                    translation: r.get(2)?,
                    keywords_json: r.get(3)?,
                    scene: r.get(4)?,
                    provider_id: r.get(5)?,
                    model: r.get(6)?,
                    created_at: r.get(7)?,
                })
            })?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
    }

    pub fn delete_history(&self, id: i64) -> AppResult<()> {
        self.with(|c| {
            c.execute("DELETE FROM translation_history WHERE id = ?1", [id])?;
            Ok(())
        })
    }

    pub fn clear_history(&self) -> AppResult<()> {
        self.with(|c| {
            c.execute("DELETE FROM translation_history", [])?;
            Ok(())
        })
    }

    // ---------- favorites ----------

    /// 已存在则不新增，只把 seen_count +1，返回 (id, 是否新建)
    pub fn add_favorite(&self, f: &NewFavorite) -> AppResult<(i64, bool)> {
        self.with(|c| {
            let lower = f.text.trim().to_lowercase();
            let existing: Option<i64> = c
                .query_row("SELECT id FROM favorites WHERE text_lower = ?1", [&lower], |r| r.get(0))
                .optional()?;
            if let Some(id) = existing {
                c.execute(
                    "UPDATE favorites SET
                       seen_count = seen_count + 1,
                       meaning = CASE WHEN ?2 <> '' THEN ?2 ELSE meaning END,
                       example = CASE WHEN ?3 <> '' THEN ?3 ELSE example END
                     WHERE id = ?1",
                    params![id, f.meaning, f.example],
                )?;
                return Ok((id, false));
            }
            c.execute(
                "INSERT INTO favorites (kind, text, text_lower, meaning, domain, example, seen_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
                params![f.kind, f.text.trim(), lower, f.meaning, f.domain, f.example],
            )?;
            Ok((c.last_insert_rowid(), true))
        })
    }

    pub fn list_favorites(&self, filter: &FavoriteFilter) -> AppResult<Vec<Favorite>> {
        self.with(|c| {
            let like = filter
                .query
                .as_deref()
                .filter(|q| !q.trim().is_empty())
                .map(|q| format!("%{}%", q.trim()));
            let order = if filter.order_by_seen { "seen_count DESC, created_at DESC" } else { "created_at DESC" };
            let sql = format!(
                "SELECT id, kind, text, meaning, domain, example, seen_count, mastered, created_at
                 FROM favorites
                 WHERE (?1 IS NULL OR kind = ?1)
                   AND (?2 IS NULL OR mastered = ?2)
                   AND (?3 IS NULL OR text LIKE ?3 OR meaning LIKE ?3)
                 ORDER BY {order}"
            );
            let mut stmt = c.prepare(&sql)?;
            let mastered = filter.mastered.map(|m| m as i64);
            let rows = stmt.query_map(params![filter.kind, mastered, like], |r| {
                Ok(Favorite {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    text: r.get(2)?,
                    meaning: r.get(3)?,
                    domain: r.get(4)?,
                    example: r.get(5)?,
                    seen_count: r.get(6)?,
                    mastered: r.get::<_, i64>(7)? != 0,
                    created_at: r.get(8)?,
                })
            })?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
    }

    pub fn update_favorite(&self, f: &Favorite) -> AppResult<()> {
        self.with(|c| {
            c.execute(
                "UPDATE favorites SET kind=?1, meaning=?2, domain=?3, example=?4, mastered=?5 WHERE id=?6",
                params![f.kind, f.meaning, f.domain, f.example, f.mastered as i64, f.id],
            )?;
            Ok(())
        })
    }

    pub fn delete_favorite(&self, id: i64) -> AppResult<()> {
        self.with(|c| {
            c.execute("DELETE FROM favorites WHERE id = ?1", [id])?;
            Ok(())
        })
    }

    /// 翻译成功后调用：凡是收藏过的词出现在这次翻译里，seen_count +1。
    /// 大小写不敏感、整词匹配（PRD 第 9 章）。
    pub fn bump_seen_counts(&self, translation: &str) -> AppResult<Vec<String>> {
        let lower = translation.to_lowercase();
        self.with(|c| {
            let mut stmt = c.prepare("SELECT id, text_lower FROM favorites")?;
            let all: Vec<(i64, String)> = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<Result<_, _>>()?;
            let mut hit = Vec::new();
            for (id, t) in all {
                if contains_whole(&lower, &t) {
                    c.execute("UPDATE favorites SET seen_count = seen_count + 1 WHERE id = ?1", [id])?;
                    hit.push(t);
                }
            }
            Ok(hit)
        })
    }

    pub fn favorite_lookup(&self) -> AppResult<Vec<(String, i64)>> {
        self.with(|c| {
            let mut stmt = c.prepare("SELECT text_lower, seen_count FROM favorites")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
    }

    // ---------- export ----------

    pub fn export_json(&self) -> AppResult<serde_json::Value> {
        let history = self.list_history(None, i64::MAX, 0)?;
        let favorites = self.list_favorites(&FavoriteFilter::default())?;
        let profile = self.get_profile()?;
        Ok(serde_json::json!({
            "exported_at": chrono::Local::now().to_rfc3339(),
            "profile": profile,
            "history": history,
            "favorites": favorites,
        }))
    }
}

/// 整词匹配：needle 前后必须是非字母数字或字符串边界
fn contains_whole(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let i = start + pos;
        let j = i + needle.len();
        let before_ok = i == 0 || !haystack[..i].chars().last().map_or(false, |c| c.is_alphanumeric());
        let after_ok = j >= haystack.len() || !haystack[j..].chars().next().map_or(false, |c| c.is_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        start = i + 1;
        if start >= haystack.len() {
            break;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::contains_whole;

    #[test]
    fn whole_word_matching() {
        assert!(contains_whole("make this endpoint paginated.", "endpoint"));
        assert!(!contains_whole("these endpoints are fine", "endpoint"));
        assert!(contains_whole("add a global exception handler.", "global exception handler"));
    }
}
