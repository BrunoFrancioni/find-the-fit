use crate::domain::Garment;
use crate::ports::SearchRepositoryPort;
use anyhow::Result;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{Condition, Filter, SearchPointsBuilder};
use std::collections::HashMap;

pub struct QdrantSearchAdapter {
    client: Qdrant,
    collection_name: String,
}

impl QdrantSearchAdapter {
    pub async fn new(url: &str, collection_name: &str) -> Result<Self> {
        let client = Qdrant::from_url(url).build()?;
        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
        })
    }

    fn payload_to_garment(
        payload: &HashMap<String, qdrant_client::qdrant::Value>,
    ) -> Option<Garment> {
        Some(Garment {
            id: payload.get("product_id")?.as_str()?.to_string(),
            title: payload.get("title")?.as_str()?.to_string(),
            price: payload.get("price")?.as_double()?,
            product_url: payload.get("product_url")?.as_str()?.to_string(),
            image_url: payload.get("image_url")?.as_str()?.to_string(),
            store_name: payload.get("store_name")?.as_str()?.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl SearchRepositoryPort for QdrantSearchAdapter {
    async fn search_lexical(&self, query: &str, limit: u64) -> Result<Vec<Garment>> {
        // Query points filtered by title substring match in payload
        let search_request =
            SearchPointsBuilder::new(&self.collection_name, vec![0.0; 1024], limit)
                .filter(Filter::must([Condition::matches(
                    "title",
                    query.to_string(),
                )]))
                .with_payload(true);

        let response = self.client.search_points(search_request).await?;
        let items = response
            .result
            .into_iter()
            .filter_map(|point| Self::payload_to_garment(&point.payload))
            .collect();

        Ok(items)
    }

    async fn search_vector(&self, vector: &[f32], limit: u64) -> Result<Vec<(Garment, f32)>> {
        let search_request =
            SearchPointsBuilder::new(&self.collection_name, vector.to_vec(), limit)
                .with_payload(true);

        let response = self.client.search_points(search_request).await?;
        let items = response
            .result
            .into_iter()
            .filter_map(|point| {
                let score = point.score;
                let garment = Self::payload_to_garment(&point.payload)?;
                Some((garment, score))
            })
            .collect();

        Ok(items)
    }
}
