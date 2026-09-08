//! 前端可调用的全部命令。命名与前端 `src/api/*.ts` 一一对应。

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::ask;
use crate::db::models::*;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::llm;
use crate::translate::{self, TranslationResult};
use crate::secrets;

// ---------- 翻译 ----------

#[derive(Serialize)]
pub struct TranslateResponse {
    pub history_id: i64,
    pub result: TranslationResult,
    pub provider_id: String,
    pub model: String,
    /// 本次翻译里出现过的、已收藏的词（小写），前端用来显示 ★ 和「第 N 次遇到」
    pub seen_favorites: Vec<String>,
}

#[tauri::command]
pub async fn translate(
    db: State<'_, Db>,
    source: String,
    scene: String,
    provider_id: Option<String>,
) -> AppResult<TranslateResponse> {
    let source = source.trim().to_string();
    if source.is_empty() {
        return Err(AppError::Other("输入为空".into()));
    }
    if source.chars().count() > 500 {
        return Err(AppError::Other("太长了，分几句翻".into()));
    }

    let (cfg, provider) = resolve_provider(&db, provider_id)?;

    let profile = db.get_profile()?;
    let req = translate::build_request(&profile, &scene, &source);
    let resp = provider.chat(req).await?;
    let result = translate::parse_output(&resp.content)?;

    let history_id = db.insert_history(&NewHistory {
        source_text: source,
        translation: result.translation.clone(),
        keywords_json: serde_json::to_string(&result.keywords)?,
        scene,
        provider_id: cfg.id.clone(),
        model: resp.model.clone(),
    })?;

    let seen_favorites = db.bump_seen_counts(&result.translation)?;

    Ok(TranslateResponse {
        history_id,
        result,
        provider_id: cfg.id,
        model: resp.model,
        seen_favorites,
    })
}

fn resolve_provider(
    db: &Db,
    provider_id: Option<String>,
) -> AppResult<(ProviderConfig, Box<dyn llm::LlmProvider>)> {
    let cfg = match provider_id {
        Some(id) => db.get_provider(&id)?,
        None => db.default_provider()?,
    }
    .ok_or_else(|| AppError::Other("还没配置模型".into()))?;
    let api_key = secrets::get_api_key(&cfg.id)?.ok_or(AppError::MissingApiKey)?;
    let provider = llm::build_provider(&cfg, api_key)?;
    Ok((cfg, provider))
}

// ---------- Ask ----------

#[derive(Serialize)]
pub struct AskResponse {
    pub answer: String,
    pub model: String,
    /// 模型判定为与英语学习无关而拒绝
    pub refused: bool,
}

/// 多轮对话在前端内存里维护，这里只负责一次问答，不落库
#[tauri::command]
pub async fn ask(
    db: State<'_, Db>,
    messages: Vec<llm::ChatMessage>,
    scene: String,
    provider_id: Option<String>,
) -> AppResult<AskResponse> {
    let last = messages
        .last()
        .filter(|m| m.role == "user" && !m.content.trim().is_empty())
        .ok_or_else(|| AppError::Other("输入为空".into()))?;
    if last.content.chars().count() > 1000 {
        return Err(AppError::Other("问题太长了，精简一下".into()));
    }

    let (_cfg, provider) = resolve_provider(&db, provider_id)?;
    let profile = db.get_profile()?;
    let req = ask::build_request(&profile, &scene, messages);
    let resp = provider.chat(req).await?;

    let answer = resp.content.trim().to_string();
    let refused = answer.contains(ask::REFUSAL);
    Ok(AskResponse {
        answer,
        model: resp.model,
        refused,
    })
}

// ---------- 档案 ----------

#[tauri::command]
pub fn get_profile(db: State<'_, Db>) -> AppResult<Profile> {
    db.get_profile()
}

#[tauri::command]
pub fn save_profile(db: State<'_, Db>, profile: Profile) -> AppResult<()> {
    db.save_profile(&profile)
}

// ---------- 模型 ----------

#[derive(Serialize)]
pub struct ProviderView {
    #[serde(flatten)]
    pub config: ProviderConfig,
    pub has_api_key: bool,
}

#[tauri::command]
pub fn list_providers(db: State<'_, Db>) -> AppResult<Vec<ProviderView>> {
    Ok(db
        .list_providers()?
        .into_iter()
        .map(|c| ProviderView {
            has_api_key: secrets::has_api_key(&c.id),
            config: c,
        })
        .collect())
}

#[tauri::command]
pub fn save_provider(db: State<'_, Db>, config: ProviderConfig, api_key: Option<String>) -> AppResult<()> {
    db.save_provider(&config)?;
    if let Some(k) = api_key {
        secrets::set_api_key(&config.id, &k)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn test_provider(db: State<'_, Db>, provider_id: String) -> AppResult<u128> {
    let cfg = db
        .get_provider(&provider_id)?
        .ok_or_else(|| AppError::Other("模型不存在".into()))?;
    let key = secrets::get_api_key(&cfg.id)?.ok_or(AppError::MissingApiKey)?;
    llm::build_provider(&cfg, key)?.test_connection().await
}

// ---------- 历史 ----------

#[tauri::command]
pub fn list_history(
    db: State<'_, Db>,
    query: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> AppResult<Vec<HistoryItem>> {
    db.list_history(query.as_deref(), limit.unwrap_or(200), offset.unwrap_or(0))
}

#[tauri::command]
pub fn delete_history(db: State<'_, Db>, id: i64) -> AppResult<()> {
    db.delete_history(id)
}

#[tauri::command]
pub fn clear_history(db: State<'_, Db>) -> AppResult<()> {
    db.clear_history()
}

// ---------- 收藏 ----------

#[derive(Serialize)]
pub struct AddFavoriteResponse {
    pub id: i64,
    pub created: bool,
}

#[tauri::command]
pub fn add_favorite(db: State<'_, Db>, favorite: NewFavorite) -> AppResult<AddFavoriteResponse> {
    let (id, created) = db.add_favorite(&favorite)?;
    Ok(AddFavoriteResponse { id, created })
}

#[tauri::command]
pub fn list_favorites(db: State<'_, Db>, filter: Option<FavoriteFilter>) -> AppResult<Vec<Favorite>> {
    db.list_favorites(&filter.unwrap_or_default())
}

#[tauri::command]
pub fn update_favorite(db: State<'_, Db>, favorite: Favorite) -> AppResult<()> {
    db.update_favorite(&favorite)
}

#[tauri::command]
pub fn delete_favorite(db: State<'_, Db>, id: i64) -> AppResult<()> {
    db.delete_favorite(id)
}

/// 前端翻译结果渲染时用：哪些词已收藏、遇到过几次
#[tauri::command]
pub fn favorite_lookup(db: State<'_, Db>) -> AppResult<Vec<(String, i64)>> {
    db.favorite_lookup()
}

// ---------- 数据 ----------

#[tauri::command]
pub fn data_dir(db: State<'_, Db>) -> String {
    db.path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[tauri::command]
pub fn export_data(app: AppHandle, db: State<'_, Db>) -> AppResult<String> {
    let json = db.export_json()?;
    let dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().home_dir())
        .map_err(|e| AppError::Other(e.to_string()))?;
    let file = dir.join(format!(
        "codeenglish-export-{}.json",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    ));
    std::fs::write(&file, serde_json::to_vec_pretty(&json)?)?;
    Ok(file.to_string_lossy().to_string())
}
