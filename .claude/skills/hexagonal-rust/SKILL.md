---
name: hexagonal-rust
description: Guidelines and constraints for developing Rust crates (apps/api and tools/scraper) using Hexagonal Architecture.
---

# Hexagonal Architecture Rules (Rust)

When creating or modifying code in `apps/api` or `tools/scraper`:

1. **Domain Isolation (`src/domain/`)**:
   - Only define pure data structures and business models (`Garment`, `SearchResultItem`, `ScrapedProduct`).
   - Never import `axum`, `reqwest`, `redis`, or database drivers in domain files.
   - Only `serde` annotations are allowed for serialization.

2. **Port Contracts (`src/ports/`)**:
   - Define all side effects and I/O behind traits decorated with `#[async_trait::async_trait]`.
   - Ensure every trait is marked `Send + Sync`.
   - Keep traits decoupled from concrete protocol formats.

3. **Application Layer (`src/application/`)**:
   - Coordinate domain services and use cases (`SearchService`).
   - Depend strictly on port traits via `Arc<dyn Port>` or generics. Never instantiate concrete adapters here.

4. **Adapters & Handlers (`src/adapters/`)**:
   - Implement port traits for specific technologies (Axum, Redis, Qdrant, Bedrock, HTML scraping).
   - Keep tech-specific transformation logic (e.g. converting Qdrant payloads to domain structs) within the adapter.

5. **Quality Checks**:
   - Always verify your work with `cargo check` and `cargo clippy --workspace -- -D warnings`.
   - Ensure all identifiers, logs, comments, and commit messages are in English.
