---
name: clean-nextjs
description: Guidelines and architectural boundaries for the Next.js frontend (apps/web) using Clean Architecture.
---

# Clean Architecture Rules (Next.js & React)

When developing components, state, or services in `apps/web`:

1. **Directory Boundaries**:
   - `src/domain/`: Pure TypeScript entity definitions and repository interface contracts. No React hooks or fetch calls.
   - `src/application/`: Orchestrates search use cases and business flows.
   - `src/infrastructure/`: Concrete repository implementations that call `/api/search` using `fetch`.
   - `src/presentation/`: React components, custom presentation hooks (`useSearch`), and Tailwind CSS styling.

2. **Coupling Guardrails**:
   - Presentation components must never import files from `infrastructure/` directly. They interact with use cases or injected hooks.
   - Keep styling utility-based with Tailwind CSS. Avoid inline styles.
   - All code, comments, and commits must be in English.
