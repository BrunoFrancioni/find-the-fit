mod adapters;
mod application;
mod domain;
mod ports;

use adapters::embedding_providers::{BedrockEmbeddingAdapter, LocalMockEmbeddingAdapter};
use adapters::http_handler::{handle_search, health_check};
use adapters::qdrant_search::QdrantSearchAdapter;
use adapters::redis_cache::RedisCacheAdapter;
use application::search_service::SearchService;
use axum::{Router, routing::get};
use ports::EmbeddingPort;
use std::env;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    println!("🚀 Bootstrapping FIND THE FIT API...");

    // 1. Dependency: Embedding Provider
    let provider_type = env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "mock".to_string());
    let embedding_port: Arc<dyn EmbeddingPort> = if provider_type.to_lowercase() == "bedrock" {
        println!("🧠 Active Embedding Provider: AWS Bedrock (Titan V2)");
        Arc::new(BedrockEmbeddingAdapter::new().await)
    } else {
        println!("🛠️  Active Embedding Provider: Local Mock Deterministic");
        Arc::new(LocalMockEmbeddingAdapter)
    };

    // 2. Dependency: Qdrant Search
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let collection_name =
        env::var("QDRANT_COLLECTION_NAME").unwrap_or_else(|_| "garments".to_string());
    let search_repo = Arc::new(QdrantSearchAdapter::new(&qdrant_url, &collection_name).await?);

    // 3. Dependency: Redis Cache
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let cache_port = Arc::new(RedisCacheAdapter::new(&redis_url).await?);

    // 4. Application Service
    let search_service = Arc::new(SearchService::new(embedding_port, search_repo, cache_port));

    // 5. Axum HTTP Router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/search", get(handle_search))
        .layer(CorsLayer::permissive())
        .with_state(search_service);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("✅ Server running at http://localhost:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}
