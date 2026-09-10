//! 收藏单词单向同步：本地优先，后台批量上传，失败静默保留待同步状态。

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::secrets;

const ENDPOINT_PATH: &str = "/api/words/collect";
const BATCH_SIZE: i64 = 50;
const MAX_BATCHES: usize = 3;
static RUNNING: AtomicBool = AtomicBool::new(false);

struct RunningGuard;

impl Drop for RunningGuard {
    fn drop(&mut self) {
        RUNNING.store(false, Ordering::Release);
    }
}

#[derive(Serialize)]
struct CollectRequest {
    words: Vec<String>,
}

#[derive(Deserialize)]
struct CollectResponse {
    code: String,
    data: CollectData,
}

#[derive(Deserialize)]
struct CollectData {
    words: Vec<String>,
}

pub async fn run(db: &Db, base_url: &str) -> AppResult<()> {
    if base_url.trim().is_empty() {
        return Ok(());
    }
    let Some(token) = secrets::get_sync_token()? else {
        return Ok(());
    };
    if RUNNING.swap(true, Ordering::AcqRel) {
        return Ok(());
    }
    let _guard = RunningGuard;
    let endpoint = endpoint_url(base_url)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    for _ in 0..MAX_BATCHES {
        let words = db.unsynced_words(BATCH_SIZE)?;
        if words.is_empty() {
            break;
        }

        let response = match client
            .post(endpoint.clone())
            .bearer_auth(&token)
            .json(&CollectRequest {
                words: words.clone(),
            })
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => response,
            _ => break,
        };
        let body = match response.json::<CollectResponse>().await {
            Ok(body) if body.code == "success" => body,
            _ => break,
        };

        let requested: HashSet<&str> = words.iter().map(String::as_str).collect();
        let accepted: Vec<String> = body
            .data
            .words
            .into_iter()
            .map(|word| word.trim().to_lowercase())
            .filter(|word| requested.contains(word.as_str()))
            .collect();
        if accepted.is_empty() {
            break;
        }
        db.mark_words_synced(&accepted)?;
    }

    Ok(())
}

fn endpoint_url(base_url: &str) -> AppResult<reqwest::Url> {
    let mut url = reqwest::Url::parse(base_url.trim())
        .map_err(|_| AppError::Other("同步服务地址无效".into()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(AppError::Other("同步服务地址必须是 http 或 https 地址".into()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::Other("同步服务地址不能包含账号或密码".into()));
    }
    url.set_path(ENDPOINT_PATH);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_fixed_collect_endpoint() {
        let url = endpoint_url("http://localhost:3000/ignored?x=1").unwrap();
        assert_eq!(url.as_str(), "http://localhost:3000/api/words/collect");
    }
}
