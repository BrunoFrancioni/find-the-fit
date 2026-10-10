---
name: semantic-search-eval
description: Procedure to evaluate and test hybrid search results against the local API.
---

# Semantic Search Evaluation Procedure

When testing or validating search behavior in `apps/api`:

1. **Ensure Infrastructure is Up**:
   - Verify Qdrant is running on `http://localhost:6334` (gRPC) and `http://localhost:6333` (REST).
   - Verify Redis is running on `localhost:6379`.

2. **Execute Evaluation Queries**:
   Run curl requests against `/api/search` testing regional synonyms:
   ```bash
   # Exact lexical check
   curl -s "http://localhost:8080/api/search?q=buzo" | jq .

   # Semantic synonym checks
   curl -s "http://localhost:8080/api/search?q=hoodie" | jq .
   curl -s "http://localhost:8080/api/search?q=sudadera" | jq .
   ```

3. **Verify Hybrid Invariants**:
   - Verify that exact title matches have `match_type: "exact"` and appear at the top of the results array.
   - Verify that semantically related items have `match_type: "semantic"` with valid scores.
   - On repeating the exact same query, verify that `"cached": true` is returned from Redis.
