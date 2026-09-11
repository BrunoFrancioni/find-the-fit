use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScrapedProduct {
    pub id: String,
    pub title: String,
    pub price: f64,
    pub product_url: String,
    pub image_url: String,
    pub store_name: String,
}

/// Represents a product alongside its mathematical vector embedding
#[derive(Debug, Clone)]
pub struct VectorizedProduct {
    pub product: ScrapedProduct,
    pub vector: Vec<f32>,
}