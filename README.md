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

- [x] Phase 1: Project Setup & Scraping CLI

    - [x] Monorepo workspace initialization with Rust and Next.js.

    - [x] Hexagonal architecture implementation for tools/scraper.

    - [x] Tienda Nube and Shopify CSS selector adapters configured.

    - [x] Local JSON export pipeline verified.

- [ ] Phase 2: Vectorization & Embeddings Pipeline

    - [ ] AWS Bedrock integration (aws-sdk-bedrockruntime).

    - [ ] Text-to-vector embedding conversion.

    - [ ] Vector database ingestion setup.

- [ ] Phase 3: Search Engine API

    - [ ] Rust API implementation with Hybrid Search (Lexical + Vector).

    - [ ] Redis caching layer for queries.

- [ ] Phase 4: Frontend Development

    - [ ] Clean Architecture React UI in Next.js.

    - [ ] Instant search filtering.

- [ ] Phase 5: Cloud Deployment

    - [ ] Serverless deployment to AWS Lambda via API Gateway.

    - [ ] Continuous Integration & Continuous Delivery (CI/CD).

## 🛠️ Getting Started (Local Development)

Prerequisites

- Rust (latest stable version)

- Node.js (v20+ recommended) & npm / pnpm

- Docker (for local databases)

### Running the Scraper

To extract products from configured e-commerce platforms:

```text
cargo run --bin scraper_cli
```

Extracted products will be saved to **scraped_products.json**.