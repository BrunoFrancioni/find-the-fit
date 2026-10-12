use crate::domain::{MatchType, SearchResponse, SearchResultItem};
use crate::ports::{CachePort, EmbeddingPort, SearchRepositoryPort};
use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;

pub struct SearchService {
    embedding_port: Arc<dyn EmbeddingPort>,
    search_repo: Arc<dyn SearchRepositoryPort>,
    cache_port: Arc<dyn CachePort>,
}

impl SearchService {
    pub fn new(
        embedding_port: Arc<dyn EmbeddingPort>,
        search_repo: Arc<dyn SearchRepositoryPort>,
        cache_port: Arc<dyn CachePort>,
    ) -> Self {
        Self {
            embedding_port,
            search_repo,
            cache_port,
        }
    }

    pub async fn execute_hybrid_search(&self, query: &str) -> Result<SearchResponse> {
        let normalized_query = query.trim().to_lowercase();
        let cache_key = format!("search:{}", normalized_query);

        // 1. Check Redis Cache (collapsed nested conditions)
        if let Ok(Some(cached_json)) = self.cache_port.get(&cache_key).await
            && let Ok(mut cached_response) = serde_json::from_str::<SearchResponse>(&cached_json)
        {
            cached_response.cached = true;
            return Ok(cached_response);
        }

        // 2. Exact keyword / Lexical Search
        let lexical_items = self
            .search_repo
            .search_lexical(&normalized_query, 10)
            .await
            .unwrap_or_default();

        // 3. Semantic Vector Search
        let query_vector = self
            .embedding_port
            .generate_embedding(&normalized_query)
            .await?;
        let vector_items = self
            .search_repo
            .search_vector(&query_vector, 15)
            .await
            .unwrap_or_default();

        // 4. Merge results (Exact matches first, semantic matches second, deduplicated)
        let mut final_results: Vec<SearchResultItem> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        for item in lexical_items {
            seen_ids.insert(item.id.clone());
            final_results.push(SearchResultItem {
                garment: item,
                match_type: MatchType::Exact,
                score: 1.0,
            });
        }

        for (garment, score) in vector_items {
            if !seen_ids.contains(&garment.id) {
                seen_ids.insert(garment.id.clone());
                final_results.push(SearchResultItem {
                    garment,
                    match_type: MatchType::Semantic,
                    score,
                });
            }
        }

        let response = SearchResponse {
            query: normalized_query,
            total_results: final_results.len(),
            cached: false,
            results: final_results,
        };

        // 5. Store in Redis Cache (TTL: 24 hours = 86400 seconds)
        if let Ok(serialized) = serde_json::to_string(&response) {
            let _ = self.cache_port.set_ex(&cache_key, &serialized, 86400).await;
        }

        Ok(response)
    }
}
