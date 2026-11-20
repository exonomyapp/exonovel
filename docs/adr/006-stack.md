# 006. Application Stack: Standalone Rust & Leptos Product

Date: 2026-11-20
Status: Accepted

## Context

**ExoNovel** (`https://exonovel.exosystems.net`, repository `https://github.com/exonomyapp/exonovel`) is an independent product, completely separate from `exosystems.net`. Both applications have separate codebases, separate server deployments, and separate user-facing features.

The ExoNovel production application—including its built-in private **Docs Reader**—must run **100% in Rust and Leptos** without Node.js.

During documentation interviews inside **Google AI Studio**, the AI Studio preview sandbox provides only a Node.js runtime (`rustc` and `cargo` are not installed in the sandbox). To keep the docs readable inside AI Studio, a temporary Node.js + React/TypeScript viewer exists strictly inside the AI Studio preview environment.

## Decision

### 1. Independent ExoNovel Product & Built-In Leptos Docs Reader (Zero Node.js)

- **Independence from `exosystems.net`**: ExoNovel (`https://exonovel.exosystems.net`) is deployed and operated as its own standalone application, not merged into `exosystems.net`.
- **Full-Stack Rust & Leptos (`cargo-leptos` + Axum)**:
  - **Workspace Layout**: `app` (shared Leptos UI + server functions), `frontend` (WASM client hydration), and `backend` (Axum native server binary).
  - **Built-In Leptos Docs Reader**: Implemented natively inside ExoNovel using Leptos components and Rust Markdown rendering (`comrak` with GitHub Flavored Markdown extensions plus `ammonia` HTML sanitization, or `leptos_md`).
  - **Immediate Admin & Reader Visibility**: As soon as `exonomyapp@gmail.com` (or any email allowlisted in `docs_access_allowlist`) signs in to ExoNovel (`https://exonovel.exosystems.net`), the **Docs Reader** option is visible in ExoNovel's Leptos navigation, alongside **Settings** for the admin account.
- **Native Rust Connections to Neon Services**:
  - **Neon PostgreSQL**: Connected directly via `sqlx` (or `tokio-postgres`) with `?sslmode=require` using `DATABASE_URL` / `DATABASE_URL_UNPOOLED`.
  - **Neon Object Storage**: Connected directly via `aws-sdk-s3` (or `object_store`) using `AWS_ENDPOINT_URL_S3`, `AWS_ACCESS_KEY_ID`, and `AWS_SECRET_ACCESS_KEY`.
  - **Neon Managed Better Auth**: Leptos calls Neon Auth's hosted HTTPS endpoints (`https://ep-wispy-wave-bgqyejp5.neonauth.c-9.eu-central-1.aws.neon.tech/exonovel/auth`) directly from the browser; the Rust/Axum server verifies session tokens natively using `jsonwebtoken` + `reqwest` against `NEON_AUTH_JWKS_URL`.
  - **Decentralized Identity & P2P Sync**: Native Rust crates (`ed25519-dalek` for `did:peer` and `iroh` for P2P sync).

### 2. Google AI Studio Preview Reader Scope

- The Node.js files (`server.ts`, `src/`, `package.json`) exist solely so Google AI Studio can render a live preview inside its Node-only container while editing documentation and synced GitHub files (`https://github.com/exonomyapp/exonovel`).
- Production deployment at `https://exonovel.exosystems.net` runs the compiled **Rust + Leptos** ExoNovel application—not Node.js.

## Consequences

- `exosystems.net` stays completely untouched and decoupled from ExoNovel docs and ExoNovel releases.
- `https://exonovel.exosystems.net` owns its own pure-Leptos **Docs Reader** gated behind Neon Better Auth and `docs_access_allowlist`, immediately visible upon signing in as `exonomyapp@gmail.com`.
- Both this workspace and the Leptos production app read the same `docs/` directory, `AGENTS.md`, Neon PostgreSQL database, Neon Managed Better Auth instance, and `BREVO_SENDER_EMAIL` (`noreply@exosystems.net`) email setup.
