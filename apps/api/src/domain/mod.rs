use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Garment {
    pub id: String,
    pub title: String,
    pub price: f64,
    pub product_url: String,
    pub image_url: String,
    pub store_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub garment: Garment,
    pub match_type: MatchType,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    Exact,
    Semantic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub total_results: usize,
    pub cached: bool,
    pub results: Vec<SearchResultItem>,
}
