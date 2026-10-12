use crate::domain::VectorizedProduct;
use crate::ports::VectorDatabasePort;
use anyhow::Result;
use qdrant_client::Payload;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder,
    Vectors, vectors::VectorsOptions,
};
use serde_json::json;

pub struct QdrantVectorDbAdapter {
    client: Qdrant,
    collection_name: String,
}

impl QdrantVectorDbAdapter {
    pub async fn new(url: &str, collection_name: &str) -> Result<Self> {
        let client = Qdrant::from_url(url).build()?;
        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl VectorDatabasePort for QdrantVectorDbAdapter {
    async fn init_schema(&self) -> Result<()> {
        if !self.client.collection_exists(&self.collection_name).await? {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection_name)
                        .vectors_config(VectorParamsBuilder::new(1024, Distance::Cosine)),
                )
                .await?;
            println!("✅ Created Qdrant collection: '{}'", self.collection_name);
        }
        Ok(())
    }

    async fn upsert_products(&self, items: &[VectorizedProduct]) -> Result<()> {
        let mut points = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            if item.vector.len() != 1024 {
                eprintln!(
                    "⚠️ Skipping item '{}': expected vector length 1024, got {}",
                    item.product.title,
                    item.vector.len()
                );
                continue;
            }

            let payload = json!({
                "product_id": item.product.id,
                "title": item.product.title,
                "price": item.product.price,
                "product_url": item.product.product_url,
                "image_url": item.product.image_url,
                "store_name": item.product.store_name,
            });

            let qdrant_payload: Payload = Payload::try_from(payload)?;

            // Explicitly build dense vector to prevent empty vector serialization
            let vectors = Vectors {
                vectors_options: Some(VectorsOptions::Vector(item.vector.clone().into())),
            };

            points.push(PointStruct::new(idx as u64 + 1, vectors, qdrant_payload));
        }

        if points.is_empty() {
            return Err(anyhow::anyhow!("No valid vectors to upsert into Qdrant"));
        }

        let points_count = points.len();
        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, points).wait(true))
            .await?;

        println!(
            "✅ Successfully indexed {} vectors into Qdrant",
            points_count
        );
        Ok(())
    }
}
