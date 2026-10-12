# FIND THE FIT

> Intelligent fashion discovery engine combining Lexical Search and Deep Learning Semantic Embeddings to unify garment catalogs across multiple e-commerce platforms.

---

## 📌 Project Overview

**FIND THE FIT** is a multi-store apparel search engine designed to solve the vocabulary mismatch problem in e-commerce (e.g., matching regional variations like *"buzo"*, *"hoodie"*, *"sudadera"*, and *"canguro"* without requiring explicit hardcoded mappings).

The system aggregates live products from e-commerce platforms across Argentina (Tienda Nube, Shopify, etc.), generates mathematical semantic representations (Vector Embeddings) for each product, and performs hybrid search (exact keyword match first, semantic AI match second).

---

## 🏛️ Architecture & System Design Decisions

### 1. AI Strategy: Semantic Embeddings vs. Generative LLMs
* **Decision:** We opted for **Deep Learning Embeddings (Vector Search)** over Fine-Tuning or Generative RAG pipelines.
* **Rationale:**
  * **Ultra-low latency:** Vector distance calculation operates in milliseconds, whereas text generation from LLMs incurs a 1-3 second delay.
  * **Cost efficiency:** Embedding generation runs at cents per million tokens with zero output token fees.
  * **Zero Hallucinations:** The engine returns strictly verified, scraped catalog items rather than synthesized responses.

### 2. Tech Stack & Architectural Patterns

            ┌────────────────────────────────────────┐
            │       Next.js (App Router / UI)        │
            │        Clean Architecture Layer        │
            └───────────────────┬────────────────────┘
                                │ HTTPS
             ┌──────────────────▼-────────────────────┐
             │       Rust Backend (Axum / Lambda)     │
             │         Hexagonal Architecture         │
             └──────┬──────────────────────────┬──────┘
                    │                          │
    ┌───────────────▼────────┐        ┌────────▼──────────────┐
    │  Amazon Bedrock (AI)   │        │   Vector Database     │
    │    Titan Embeddings    │        │  PGVector / Qdrant    │
    └────────────────────────┘        └───────────────────────┘

* **Frontend (`apps/web`):** Built with **Next.js (TypeScript & TailwindCSS)** adhering to **Clean Architecture** principles (separating Domain Entities, Application Use Cases, Infrastructure Data Providers, and Presentation UI).
* **Backend (`apps/api`):** Developed in **Rust** following **Hexagonal Architecture (Ports & Adapters)** to decouple business logic from web frameworks, vector databases, and cloud providers.
* **Scraper CLI (`tools/scraper`):** A standalone, highly concurrent **Rust CLI** using `reqwest` and `scraper` (CSS Selectors) designed to periodically crawl e-commerce platforms without inflating the production API binary.

### 3. Cost-Optimization & Rate Limiting Strategy
* **Cloudflare CDN/WAF:** First line of defense against bot attacks and DDoS.
* **Token Bucket Rate Limiting:** Enforced on API routes to avoid abusive API traffic.
* **Semantic Caching:** Common queries are cached to prevent redundant AWS Bedrock API calls.

---

## 📂 Monorepo Structure

The project is structured as a unified monorepo managed by **Cargo Workspaces**:

```text
find-the-fit/
├── Cargo.toml                    # Cargo Workspace root definition
├── Cargo.lock
├── docker-compose.yml            # Local infrastructure (Redis & Vector DB)
├── .gitignore                    # Global git ignore configurations
├── README.md                     # Project documentation
│
├── apps/
│   ├── web/                      # Frontend App (Next.js + Clean Architecture)
│   │   ├── src/
│   │   │   ├── domain/           # Core Entities & Value Objects
│   │   │   ├── application/      # Use cases & workflows
│   │   │   ├── infrastructure/   # API clients & external mappers
│   │   │   └── presentation/     # React Components & UI state
│   │   └── package.json
│   │
│   └── api/                      # Backend API (Rust + Hexagonal Architecture)
│       ├── src/
│       │   ├── domain/           # Business entities & search rules
│       │   ├── ports/            # Inbound & Outbound trait interfaces
│       │   ├── application/      # Service orchestration
│       │   ├── adapters/         # HTTP handlers, AWS Bedrock, Vector DB adapters
│       │   └── main.rs           # Composition root (Dependency Injection)
│       └── Cargo.toml
│
└── tools/
    └── scraper/                  # Rust Crawler / Scraping CLI
        ├── src/
        │   ├── domain.rs         # ScrapedProduct entity definition
        │   ├── ports.rs          # Scraper and Storage port definitions
        │   ├── adapters/         # Implementations (HTML scraper, JSON storage)
        │   │   ├── html_scraper.rs
        │   │   ├── json_storage.rs
        │   │   └── mod.rs
        │   └── main.rs           # Scraping execution entrypoint
        └── Cargo.toml
```

## 🚀 Current Status & Roadmap

- [x] **Phase 1: Project Setup & Scraping CLI**
  - [x] Monorepo workspace initialization with Rust and Next.js.
  - [x] Hexagonal architecture implementation for `tools/scraper`.
  - [x] Tienda Nube and Shopify CSS selector adapters configured.
  - [x] Local JSON export pipeline verified.
- [x] **Phase 2: Vectorization & Embeddings Pipeline**
  - [x] AWS Bedrock integration (`aws-sdk-bedrockruntime`) with exponential backoff retries.
  - [x] Dual-provider architecture: Pluggable AWS Bedrock (`Titan Text Embeddings V2`) and Local Deterministic Mock provider for decoupled local development.
  - [x] Qdrant Vector Database ingestion with Cosine distance indexing (via Docker).
- [x] **Phase 3: Search Engine API**
  - [x] Rust backend API implemented with Axum using Hexagonal Architecture (Ports & Adapters).
  - [x] Hybrid Search orchestrator prioritizing lexical matches before semantic vector neighbors.
  - [x] Redis caching layer for queries with TTL to optimize latency and cloud costs.
- [ ] **Phase 4: Frontend Development**
  - [ ] Clean Architecture React UI in Next.js (Domain, Application, Infrastructure, Presentation).
  - [ ] Instant search bar and responsive product grid.
- [ ] **Phase 5: Cloud Deployment**
  - [ ] Serverless deployment to AWS Lambda via API Gateway.
  - [ ] Continuous Integration & Continuous Delivery (CI/CD).

## 🛠️ Getting Started & Run Order

### Prerequisites
* **Rust** (stable toolchain via `rustup`)
* **Docker & Docker Compose** (for local datastores)
* **Node.js** (v20+ recommended) & **npm**

---

### Step-by-Step Execution Sequence

Follow this order when running the platform locally for the first time:

#### 1. Start Local Infrastructure

Launch the local **Qdrant Vector Database** and **Redis Cache** containers:

```bash
docker compose up -d
```

* Qdrant REST / Dashboard: http://localhost:6333/dashboard

* Qdrant gRPC: localhost:6334

* Redis: localhost:6379

#### 2. Configure Environment Variables

Ensure a .env file exists in the repository root:

```bash
EMBEDDING_PROVIDER=mock
QDRANT_URL=http://localhost:6334
QDRANT_COLLECTION_NAME=garments
REDIS_URL=redis://127.0.0.1:6379
PORT=8080
```

(Switch EMBEDDING_PROVIDER=bedrock along with AWS credentials when deploying to production).

#### 3. Run Scraping & Vector Indexing Pipeline

Crawl configured store catalogs, generate vector embeddings, and ingest them into Qdrant:

```bash
cargo run --bin scraper_cli
```

* Scraped items are exported locally to scraped_products.json.

* Embeddings are indexed into the garments collection.

#### 4. Start Backend API

Launch the Axum search server:

```bash
cargo run --bin api
```

The server will bind to http://localhost:8080.

#### 5. Start Frontend UI (Phase 4)

From the web application directory:

```bash
cd apps/web
npm install
npm run dev
```

Open http://localhost:3000 in your browser.

---

### 🧰 Useful Developer Commands

#### Workspace Health & Code Quality

```bash
# Check compilation across all workspace members
cargo check --workspace

# Strict lint verification (CI-equivalent, zero warnings)
cargo clippy --workspace -- -D warnings

# Format all Rust files
cargo fmt --all

# Run all test suites
cargo test --workspace
```

#### Search API Smoke Testing

Evaluate hybrid query responses directly from your terminal:

```bash
# Test lexical & semantic resolution
curl -s "http://localhost:8080/api/search?q=buzo" | jq .

# Test regional synonym mapping
curl -s "http://localhost:8080/api/search?q=hoodie" | jq .

# Verify Redis caching (run the same query twice and inspect the "cached" field)
curl -s "http://localhost:8080/api/search?q=buzo" | grep -o '"cached":true'
```

#### Infrastructure Management

```bash
# Check status of local containers
docker compose ps

# View real-time container logs
docker compose logs -f

# Reset local vector and cache storage
docker compose down -v
```

