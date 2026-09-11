use anyhow::{Context, Result};
use aws_sdk_bedrockruntime::Client as BedrockClient;
use aws_sdk_bedrockruntime::primitives::Blob;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use crate::ports::EmbeddingGeneratorPort;

#[derive(Serialize)]
struct TitanEmbeddingRequest<'a> {
    #[serde(rename = "inputText")]
    input_text: &'a str,
    dimensions: u32,
    normalize: bool,
}

#[derive(Deserialize)]
struct TitanEmbeddingResponse {
    embedding: Vec<f32>,
}

pub struct BedrockEmbeddingAdapter {
    client: BedrockClient,
    model_id: String,
}

impl BedrockEmbeddingAdapter {
    pub async fn new() -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = BedrockClient::new(&config);
        
        Self {
            client,
            model_id: "amazon.titan-embed-text-v2:0".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingGeneratorPort for BedrockEmbeddingAdapter {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let payload = TitanEmbeddingRequest {
            input_text: text,
            dimensions: 1024,
            normalize: true,
        };

        let request_body = serde_json::to_vec(&payload)?;
        let max_retries = 3;
        let mut backoff_ms = 1000;

        for attempt in 1..=max_retries {
            let result = self
                .client
                .invoke_model()
                .model_id(&self.model_id)
                .content_type("application/json")
                .accept("application/json")
                .body(Blob::new(request_body.clone()))
                .send()
                .await;

            match result {
                Ok(response) => {
                    let body_bytes = response.body().as_ref();
                    let parsed: TitanEmbeddingResponse = serde_json::from_slice(body_bytes)
                        .context("Failed to parse Bedrock response JSON")?;
                    return Ok(parsed.embedding);
                }
                Err(err) => {
                    if attempt < max_retries {
                        sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2;
                        continue;
                    }
                    return Err(anyhow::anyhow!("Bedrock invocation failed: {:?}", err));
                }
            }
        }

        Err(anyhow::anyhow!("Max retries exceeded for text: {}", text))
    }
}

/// Fallback local embedding adapter for development and testing
pub struct LocalMockEmbeddingAdapter;

#[async_trait::async_trait]
impl EmbeddingGeneratorPort for LocalMockEmbeddingAdapter {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Generates a deterministic 1024-dimensional normalized vector based on text hash
        let mut vector = vec![0.0f32; 1024];
        let mut hasher_state: u64 = 5381;
        
        for b in text.bytes() {
            hasher_state = hasher_state.wrapping_mul(33).wrapping_add(b as u64);
        }

        for i in 0..1024 {
            let val = ((hasher_state.wrapping_add(i as u64)).wrapping_mul(1103515245) % 1000) as f32 / 1000.0;
            vector[i] = val;
        }

        // Normalize vector (Cosine magnitude = 1.0)
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        }

        Ok(vector)
    }
}