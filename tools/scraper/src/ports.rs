use anyhow::Result;
use crate::domain::{ScrapedProduct, VectorizedProduct};

pub trait ProductStorage {
    fn save_all(&self, products: &[ScrapedProduct]) -> Result<()>;
    fn load_all(&self) -> Result<Vec<ScrapedProduct>>;
}

#[async_trait::async_trait]
pub trait EmbeddingGeneratorPort {
    /// Generates a vector embedding for a given text query/title
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait::async_trait]
pub trait VectorDatabasePort {
    /// Initializes collection schema if not exists
    async fn init_schema(&self) -> Result<()>;
    
    /// Persists vectorized products to the vector database
    async fn upsert_products(&self, items: &[VectorizedProduct]) -> Result<()>;
}