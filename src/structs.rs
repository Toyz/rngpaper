use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub data: Vec<Data>,
    pub meta: Meta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub id: String,
    pub url: String,
    #[serde(rename = "short_url")]
    pub short_url: String,
    pub views: i64,
    pub favorites: i64,
    pub source: String,
    pub purity: String,
    pub category: String,
    #[serde(rename = "dimension_x")]
    pub dimension_x: i64,
    #[serde(rename = "dimension_y")]
    pub dimension_y: i64,
    pub resolution: String,
    pub ratio: String,
    #[serde(rename = "file_size")]
    pub file_size: i64,
    #[serde(rename = "file_type")]
    pub file_type: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub colors: Vec<String>,
    pub path: String,
    pub thumbs: Thumbs,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbs {
    pub large: String,
    pub original: String,
    pub small: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(rename = "current_page")]
    pub current_page: i64,
    #[serde(rename = "last_page")]
    pub last_page: i64,
    #[serde(rename = "per_page")]
    pub per_page: i64,
    pub total: i64,
    pub query: String,
    pub seed: ::serde_json::Value,
}
