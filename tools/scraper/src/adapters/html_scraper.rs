use crate::domain::ScrapedProduct;
use anyhow::Result;
use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};

/// Selector configuration structure to adapt HTML parsing across different e-commerce layouts
pub struct StoreSelectorConfig {
    pub product_card: &'static str,
    pub title: &'static str,
    pub price: &'static str,
    pub image: &'static str,
    pub link: &'static str,
}

impl StoreSelectorConfig {
    /// Standard selectors for e-commerce platforms based on Tienda Nube
    pub fn tienda_nube() -> Self {
        Self {
            product_card: ".js-product-container, .item-product",
            title: ".js-item-name, .item-name",
            price: ".js-price-display, .item-price",
            image: ".js-item-image, .item-image img",
            link: "a.item-link, a.js-item-link",
        }
    }

    /// Standard selectors for e-commerce platforms based on Shopify
    pub fn shopify() -> Self {
        Self {
            product_card: ".product-card, .grid-view-item, .card-wrapper",
            title: ".product-card__title, .card__heading",
            price: ".price-item--sale, .price-item--regular",
            image: ".product-card__image, .card__media img",
            link: "a.product-card__link, a.full-unstyled-link",
        }
    }
}

pub struct ECommerceScraper {
    pub store_name: String,
    pub config: StoreSelectorConfig,
}

impl ECommerceScraper {
    pub fn new(store_name: &str, config: StoreSelectorConfig) -> Self {
        Self {
            store_name: store_name.to_string(),
            config,
        }
    }

    pub async fn scrape(&self, target_url: &str) -> Result<Vec<ScrapedProduct>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let response = client
            .get(target_url)
            // Spoof a modern browser User-Agent to prevent basic bot-blocking
            .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await?
            .text()
            .await?;

        let document = Html::parse_document(&response);

        // Compile CSS selectors
        let card_sel = Selector::parse(self.config.product_card).map_err(|e| anyhow::anyhow!("Selector error: {:?}", e))?;
        let title_sel = Selector::parse(self.config.title).map_err(|e| anyhow::anyhow!("Selector error: {:?}", e))?;
        let price_sel = Selector::parse(self.config.price).map_err(|e| anyhow::anyhow!("Selector error: {:?}", e))?;
        let img_sel = Selector::parse(self.config.image).map_err(|e| anyhow::anyhow!("Selector error: {:?}", e))?;
        let link_sel = Selector::parse(self.config.link).map_err(|e| anyhow::anyhow!("Selector error: {:?}", e))?;

        let mut products = Vec::new();

        for element in document.select(&card_sel) {
            // Extract title
            let title = element
                .select(&title_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            // Extract and sanitize raw price (e.g., "$ 45.900,00" -> 45900.0)
            let price_raw = element
                .select(&price_sel)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();

            let price = clean_price(&price_raw);

            // Extract image URL
            let image_url = element
                .select(&img_sel)
                .next()
                .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                .map(format_url)
                .unwrap_or_default();

            // Extract product URL
            let product_url = element
                .select(&link_sel)
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(format_url)
                .unwrap_or_else(|| target_url.to_string());

            if !title.is_empty() && price > 0.0 {
                products.push(ScrapedProduct {
                    id: format!("{}-{}", self.store_name.to_lowercase().replace(' ', "-"), products.len() + 1),
                    title,
                    price,
                    product_url,
                    image_url,
                    store_name: self.store_name.clone(),
                });
            }
        }

        Ok(products)
    }
}

/// Helper function to sanitize Argentine peso price strings
fn clean_price(raw: &str) -> f64 {
    let clean_str = raw
        .replace('.', "") // Remove thousands separator
        .replace(',', ".") // Convert decimal comma to dot
        .chars()
        .filter(|c| c.is_numeric() || *c == '.')
        .collect::<String>();

    clean_str.parse::<f64>().unwrap_or(0.0)
}

/// Helper function to normalize relative URLs (e.g., "//cdn..." or "/products/...")
fn format_url(url: &str) -> String {
    if url.starts_with("//") {
        format!("https:{}", url)
    } else if url.starts_with('/') {
        format!("https://{}", url)
    } else {
        url.to_string()
    }
}