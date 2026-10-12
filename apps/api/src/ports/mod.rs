use crate::domain::Garment;
use anyhow::Result;

#[async_trait::async_trait]
pub trait EmbeddingPort: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait::async_trait]
pub trait SearchRepositoryPort: Send + Sync {
    /// Full-text / keyword search against stored garment metadata
    async fn search_lexical(&self, query: &str, limit: u64) -> Result<Vec<Garment>>;

    /// Vector cosine distance search against indexed embeddings
    async fn search_vector(&self, vector: &[f32], limit: u64) -> Result<Vec<(Garment, f32)>>;
}

#[async_trait::async_trait]
pub trait CachePort: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set_ex(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()>;
}
