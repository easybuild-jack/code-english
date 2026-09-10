//! API Key 存系统钥匙串。Windows 走凭据管理器，macOS 走 Keychain，同一份代码。

use crate::error::AppResult;

const SERVICE: &str = "com.jackliu.codeenglish";
const FAVORITE_SYNC_TOKEN_ID: &str = "favorite-sync-token";

fn entry(provider_id: &str) -> AppResult<keyring::Entry> {
    Ok(keyring::Entry::new(SERVICE, provider_id)?)
}

pub fn set_api_key(provider_id: &str, key: &str) -> AppResult<()> {
    let e = entry(provider_id)?;
    if key.trim().is_empty() {
        // 清空即删除；不存在时忽略
        match e.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err.into()),
        }
    } else {
        e.set_password(key.trim())?;
        Ok(())
    }
}

pub fn get_api_key(provider_id: &str) -> AppResult<Option<String>> {
    match entry(provider_id)?.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn has_api_key(provider_id: &str) -> bool {
    matches!(get_api_key(provider_id), Ok(Some(k)) if !k.is_empty())
}

pub fn set_sync_token(token: &str) -> AppResult<()> {
    set_api_key(FAVORITE_SYNC_TOKEN_ID, token)
}

pub fn get_sync_token() -> AppResult<Option<String>> {
    get_api_key(FAVORITE_SYNC_TOKEN_ID)
}

pub fn has_sync_token() -> bool {
    has_api_key(FAVORITE_SYNC_TOKEN_ID)
}
