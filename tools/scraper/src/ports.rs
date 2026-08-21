use anyhow::Result;
use crate::domain::ScrapedProduct;

pub trait ProductStorage {
    fn save_all(&self, products: &[ScrapedProduct]) -> Result<()>;
}

pub trait WebScraperPort {
    async fn scrape_catalog(&self, target_url: &str) -> Result<Vec<ScrapedProduct>>;
}