# 006. Application Stack: Rust & Leptos Product vs. AI Studio Docs Reader

Date: 2026-11-20
Status: Accepted (Product Stack Locked); Pending Owner Deployment Scope Decision for Temporary Docs Reader

## Context

The ExoNovel product is specified by the owner to be built in **Rust** with **Leptos** for the web frontend (`https://exonovel.exosystems.net`) and shared Rust crates across desktop and mobile targets.

During the architecture-interview and documentation phase inside **Google AI Studio**, the container environment provides only a **Node.js** runtime and does not include `rustc` or `cargo`. To satisfy [005-docs-ui](005-docs-ui.md) (a private, Neon-authenticated Markdown viewer inside AI Studio), a temporary **Node.js + Express + React/TypeScript** documentation reader was created inside the AI Studio repository.

This distinction created ambiguity regarding whether Node.js or TypeScript is part of the ExoNovel production runtime.

## Decision

### 1. Authoritative ExoNovel Product Stack (100% Rust, Zero Node.js)

The **ExoNovel** production application—web, desktop, and mobile—uses a pure **Rust** architecture with no Node.js runtime dependency:

- **Web Frontend & Server-Side Rendering (SSR)**: **Leptos** compiled to WebAssembly (`wasm32-unknown-unknown`) paired with **Axum** (`leptos_axum`) for HTTP routing and server functions.
- **PostgreSQL Database (`Neon PostgreSQL`)**: Connected natively from Rust via `sqlx` (or `tokio-postgres`) using TLS (`?sslmode=require`) and `DATABASE_URL` / `DATABASE_URL_UNPOOLED`.
- **Object Storage (`Neon Object Storage`)**: Connected natively from Rust via `aws-sdk-s3` (or `object_store`) using `AWS_ENDPOINT_URL_S3`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and presigned URLs for direct client uploads/downloads.
- **Authentication (`Neon Managed Better Auth`)**: Neon Auth runs as an external managed HTTP service (`https://ep-wispy-wave-bgqyejp5.neonauth.c-9.eu-central-1.aws.neon.tech/exonovel/auth`). The Leptos browser client calls Neon Auth HTTPS endpoints directly, and the Rust/Axum backend verifies session JSON Web Tokens (`JWT`) natively using `jsonwebtoken` + `reqwest` against `NEON_AUTH_JWKS_URL` (no Node.js Better Auth server SDK required).
- **Decentralized Identity & P2P Sync**: Native Rust crates (`ed25519-dalek` for `did:peer` identity/signing and `iroh` for peer-to-peer synchronization).

### 2. Temporary Google AI Studio Docs Reader (Node.js / TypeScript)

- The **Express (`server.ts`) + React/Vite** codebase in this workspace exists strictly as the **temporary Docs Reader and Auth Gate** inside Google AI Studio, because the AI Studio sandbox only builds and previews Node.js web applications.
- It shares the same external **Neon PostgreSQL**, **Neon Managed Better Auth**, and **Neon Object Storage** project (`curly-pine-66858617`), without imposing Node.js on the Rust/Leptos ExoNovel codebase.

## Consequences & Open Decision

- **No Node.js in ExoNovel Production**: Deploying the real ExoNovel product at `https://exonovel.exosystems.net` uses a compiled Rust/Leptos binary (`cargo-leptos`) alongside your existing ExoSystems Rust infrastructure.
- **Pending Owner Choice for the Docs Reader**:
  1. **Option A (Recommended — Zero Node.js on Server)**: Keep this temporary Node.js Docs Reader inside Google AI Studio solely for reading/writing documentation during interviews, and keep `https://exonovel.exosystems.net` 100% Rust + Leptos.
  2. **Option B (Port Docs Reader to Leptos)**: Port the Markdown documentation viewer and Neon Auth JWKS gate into your Rust + Leptos codebase so private docs live at a route on `https://exonovel.exosystems.net` without Node.js.
  3. **Option C (Deploy Temporary Node.js Reader)**: Run this Node.js Docs Reader temporarily behind Caddy/Nginx on `https://exonovel.exosystems.net` (or a subdomain such as `docs.exonovel.exosystems.net`) until the Leptos app replaces it.
