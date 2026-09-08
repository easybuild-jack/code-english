use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub occupation: String,
    pub tech_stack: String,
    pub industry: String,
    /// work | workplace | daily | shopping | travel
    pub default_scene: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub display_name: String,
    /// openai_compatible | anthropic | gemini
    pub kind: String,
    pub base_url: String,
    pub model: String,
    pub enabled: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: i64,
    pub source_text: String,
    pub translation: String,
    pub keywords_json: String,
    pub scene: String,
    pub provider_id: String,
    pub model: String,
    pub created_at: String,
}

pub struct NewHistory {
    pub source_text: String,
    pub translation: String,
    pub keywords_json: String,
    pub scene: String,
    pub provider_id: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub id: i64,
    /// word | phrase | sentence
    pub kind: String,
    pub text: String,
    pub meaning: String,
    pub domain: String,
    pub example: String,
    pub seen_count: i64,
    pub mastered: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewFavorite {
    pub kind: String,
    pub text: String,
    #[serde(default)]
    pub meaning: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub example: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FavoriteFilter {
    pub kind: Option<String>,
    pub mastered: Option<bool>,
    pub query: Option<String>,
    #[serde(default)]
    pub order_by_seen: bool,
}
