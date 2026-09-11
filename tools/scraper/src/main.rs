mod adapters;
mod domain;
mod ports;

use adapters::bedrock_embedding::{BedrockEmbeddingAdapter, LocalMockEmbeddingAdapter};
use adapters::html_scraper::{ECommerceScraper, StoreSelectorConfig};
use adapters::json_storage::JsonFileStorage;
use adapters::qdrant_vector_db::QdrantVectorDbAdapter;
use domain::{ScrapedProduct, VectorizedProduct};
use ports::{EmbeddingGeneratorPort, ProductStorage, VectorDatabasePort};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    println!("🚀 Starting garment extraction & vectorization pipeline...\n");

    let storage = JsonFileStorage::new("scraped_products.json");
    let mut products: Vec<ScrapedProduct> = Vec::new();

    // 1. Scraping execution
    let tienda_nube_scraper = ECommerceScraper::new("TiendaNube Brand", StoreSelectorConfig::tienda_nube());
    let url_tienda_nube = "https://www.taverniti.com.ar/hombre/buzos";

    println!("🔎 Scraping Tienda Nube store: {}...", url_tienda_nube);
    if let Ok(items) = tienda_nube_scraper.scrape(url_tienda_nube).await {
        println!("   └─ ✅ Found {} products.", items.len());
        products.extend(items);
    }

    let shopify_scraper = ECommerceScraper::new("Shopify Brand", StoreSelectorConfig::shopify());
    let url_shopify = "https://www.bowen.com.ar/collections/sweatshirts-hoodies";

    println!("🔎 Scraping Shopify store: {}...", url_shopify);
    if let Ok(items) = shopify_scraper.scrape(url_shopify).await {
        println!("   └─ ✅ Found {} products.", items.len());
        products.extend(items);
    }

    // Persist raw scraped data locally
    storage.save_all(&products)?;

    // 2. Setup Embedding Provider (Bedrock or Local Mock)
    let provider_type = env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "mock".to_string());
    let embedding_adapter: Arc<dyn EmbeddingGeneratorPort> = if provider_type.to_lowercase() == "bedrock" {
        println!("🧠 Using AWS Bedrock (Titan V2) for vector embeddings...");
        Arc::new(BedrockEmbeddingAdapter::new().await)
    } else {
        println!("🛠️  Using Local Mock Embedding Provider (deterministic)...");
        Arc::new(LocalMockEmbeddingAdapter)
    };

    // 3. Vector Database setup
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let collection_name = env::var("QDRANT_COLLECTION_NAME").unwrap_or_else(|_| "garments".to_string());

    let vector_db = QdrantVectorDbAdapter::new(&qdrant_url, &collection_name).await?;
    vector_db.init_schema().await?;

    let mut vectorized_items = Vec::new();

    for product in &products {
        print!("  -> Embedding: \"{}\" ... ", product.title);
        match embedding_adapter.generate_embedding(&product.title).await {
            Ok(vector) => {
                println!("Done ({} dimensions)", vector.len());
                vectorized_items.push(VectorizedProduct {
                    product: product.clone(),
                    vector,
                });
            }
            Err(e) => {
                println!("Failed: {:?}", e);
            }
        }
        // Small delay between requests to stay within AWS rate limits
        sleep(Duration::from_millis(50)).await;
    }

    // 4. Ingest into Vector Database
    if !vectorized_items.is_empty() {
        println!("\n📦 Ingesting vectorized items into Qdrant...");
        vector_db.upsert_products(&vectorized_items).await?;
        println!("\n🎉 Pipeline completed! Successfully processed and indexed {} garments.", vectorized_items.len());
    } else {
        println!("\n⚠️ No items were vectorized. Skipping database ingestion.");
    }

    Ok(())
}