# CLAUDE.md

This file provides architectural context, system design guidelines, and commands for Claude Code when working in this repository.

## 1. Project Overview & Core Philosophy

**FIND THE FIT** is a multi-store apparel discovery engine targeting the Argentine e-commerce market (Tienda Nube, Shopify, etc.).
- **Problem it solves:** Vocabulary mismatch in garment searches (e.g., regional synonyms like *"buzo"*, *"hoodie"*, *"sudadera"*, and *"canguro"*) without maintaining manual synsets or hardcoded alias dictionaries.
- **AI Strategy:** We deliberately employ **Deep Learning Vector Embeddings (Semantic Search)** rather than Generative LLMs or Generative RAG:
  - **Latency:** Sub-20ms vector queries vs. multi-second LLM text generation.
  - **Cost efficiency:** Cent-level pricing per million input tokens, zero output tokens.
  - **Zero Hallucination:** Every search result is a strictly scraped, verified physical product.
- **Search Strategy (Hybrid Search):** Always prioritize exact lexical matches first, followed by nearest-neighbor semantic vector matches, deduplicating IDs across both stages.

---

## 2. Monorepo Architecture & Strict Layer Rules

The repository is organized as a Cargo Workspace (Rust) alongside an npm project (Next.js):

### A. Scraper CLI (`tools/scraper`)
- Standalone CLI crate. It is kept isolated so heavy crawling tasks do not bloat the API deployment binary.
- Scrapes products with `reqwest` and `scraper` (CSS selectors configured in `StoreSelectorConfig`).
- Sanitizes Argentine Peso prices (`$ 45.900,00` -> `45900.0`).
- Generates vector embeddings for `product.title` and ingests points into Qdrant.

### B. Backend API (`apps/api`) — Hexagonal Architecture (Ports & Adapters)
- Framework: Axum with Tokio runtime.
- **Layering rules:**
  - `src/domain/`: Pure business entities (`Garment`, `SearchResultItem`, `MatchType`, `SearchResponse`). **Zero framework or I/O dependencies.**
  - `src/ports/`: Asynchronous trait contracts (`EmbeddingPort`, `SearchRepositoryPort`, `CachePort`). All traits must be `Send + Sync`.
  - `src/application/`: Orchestrates hybrid search (`SearchService`). Checks Redis cache -> runs lexical query & semantic vector query -> merges & deduplicates -> populates cache.
  - `src/adapters/`: Concrete implementations:
    - HTTP handlers with Axum (`http_handler.rs`).
    - Vector & lexical search with Qdrant (`qdrant_search.rs`).
    - Semantic query caching with Redis (`redis_cache.rs`).
    - Dual embedding providers (`embedding_providers.rs`).
  - `src/main.rs`: Composition root handling dependency injection based on environment variables.

### C. Frontend (`apps/web`) — Clean Architecture
- Framework: Next.js (App Router), React 19, Tailwind CSS.
- **Layering rules:**
  - `src/domain/`: Enterprise business entities and repository interfaces (TypeScript types). No React hooks, no Next.js APIs, no external HTTP clients.
  - `src/application/`: Use cases orchestrating search workflows.
  - `src/infrastructure/`: Repository implementations consuming `apps/api` via fetch/HTTP clients.
  - `src/presentation/`: React components, custom hooks (`useSearch`), and Tailwind UI styling.

---

## 3. Dual Embedding Provider Strategy (Resilience Rule)

To prevent AWS Bedrock quotas (`ThrottlingException` / HTTP 429) from blocking local work, the codebase implements a dual-adapter pattern via `EMBEDDING_PROVIDER`:
1. `bedrock`: Invokes AWS Bedrock (`amazon.titan-embed-text-v2:0`, 1024 dimensions, Cosine distance) with exponential backoff retries.
2. `mock`: Default fallback. Uses `LocalMockEmbeddingAdapter`, a deterministic hash-based 1024-dimensional normalized vector generator. Runs 100% offline with zero cloud credentials required.

*Invariant:* Vector dimensions must remain **1024** across Bedrock, the mock generator, and the Qdrant schema (`garments` collection).

---

## 4. Universal Code Conventions

- **Language:** Every variable name, function, struct, comment, docstring, log statement, and Git commit **must be strictly written in English**.
- **Error Handling:** Use `anyhow::Result` across application workflows and adapters. Domain-level errors use idiomatic `thiserror` enums.
- **Formatting & Linting:** Code must pass `cargo clippy --workspace -- -D warnings` and `cargo fmt` without warnings before completing tasks.

---

## 5. Development & Verification Commands

```bash
# 1. Local infrastructure (Qdrant & Redis)
docker compose up -d

# 2. Rust workspace checks
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace

# 3. Running tools & services
cargo run --bin scraper_cli        # Crawl e-commerce catalogs and index embeddings
cargo run --bin api                # Start Axum HTTP backend on port 8080

# 4. Frontend (from apps/web)
npm run dev
npm run build
npm run lint