use crate::domain::ScrapedProduct;
use crate::ports::ProductStorage;
use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};

pub struct JsonFileStorage {
    file_path: String,
}

impl JsonFileStorage {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
}

impl ProductStorage for JsonFileStorage {
    fn save_all(&self, products: &[ScrapedProduct]) -> Result<()> {
        let json_data = serde_json::to_string_pretty(products)?;
        let mut file = File::create(&self.file_path)?;
        file.write_all(json_data.as_bytes())?;
        println!("✅ Successfully saved {} products to '{}'", products.len(), self.file_path);
        Ok(())
    }

    fn load_all(&self) -> Result<Vec<ScrapedProduct>> {
        let mut file = File::open(&self.file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let products: Vec<ScrapedProduct> = serde_json::from_str(&content)?;
        Ok(products)
    }
}