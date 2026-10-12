use crate::application::search_service::SearchService;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn handle_search(
    State(service): State<Arc<SearchService>>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let query = match params.q {
        Some(q) if !q.trim().is_empty() => q,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Query parameter 'q' is required" })),
            )
                .into_response();
        }
    };

    match service.execute_hybrid_search(&query).await {
        Ok(results) => {
            (StatusCode::OK, Json(serde_json::to_value(results).unwrap())).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Search failed: {:?}", err) })),
        )
            .into_response(),
    }
}
