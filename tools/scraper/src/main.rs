mod adapters;
mod domain;
mod ports;

use adapters::html_scraper::{ECommerceScraper, StoreSelectorConfig};
use adapters::json_storage::JsonFileStorage;
use domain::ScrapedProduct;
use ports::ProductStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Starting garment extraction for Argentine brands...\n");

    let mut all_products: Vec<ScrapedProduct> = Vec::new();

    // 1. Tienda Nube scraper configuration (e.g., Sweatshirts / Hoodies section)
    let tienda_nube_scraper = ECommerceScraper::new(
        "TiendaNube Brand", 
        StoreSelectorConfig::tienda_nube()
    );
    let url_tienda_nube = "https://www.taverniti.com.ar/hombre/buzos";

    println!("🔎 Scraping Tienda Nube store: {}...", url_tienda_nube);
    match tienda_nube_scraper.scrape(url_tienda_nube).await {
        Ok(products) => {
            println!("   └─ ✅ Found {} products.", products.len());
            all_products.extend(products);
        }
        Err(e) => eprintln!("   └─ ❌ Error scraping Tienda Nube: {:?}", e),
    }

    // 2. Shopify scraper configuration (e.g., Hoodies section)
    let shopify_scraper = ECommerceScraper::new(
        "Shopify Brand", 
        StoreSelectorConfig::shopify()
    );
    let url_shopify = "https://www.bowen.com.ar/collections/sweatshirts-hoodies";

    println!("🔎 Scraping Shopify store: {}...", url_shopify);
    match shopify_scraper.scrape(url_shopify).await {
        Ok(products) => {
            println!("   └─ ✅ Found {} products.", products.len());
            all_products.extend(products);
        }
        Err(e) => eprintln!("   └─ ❌ Error scraping Shopify: {:?}", e),
    }

    // 3. Persist unified results to local JSON
    println!("\n💾 Saving unified dataset...");
    let storage = JsonFileStorage::new("scraped_products.json");
    storage.save_all(&all_products)?;

    println!("\n🎉 Extraction completed! Total garments extracted: {}", all_products.len());

    Ok(())
}