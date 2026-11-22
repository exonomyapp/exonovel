# 002-001. Neon PostgreSQL, Better Auth & Object Storage

Date: 2026-11-19
Status: Accepted
Parent: [002-data](../002-data.md)

## Context

ExoNovel standardizes its cloud persistence, authentication backend, and file storage on **Neon** (`org-still-credit-09220145`, project `curly-pine-66858617`, branch `production`).

## Decision

1. **Configuration (`neon.ts`)**:
   - Declare `auth: true` (Managed Better Auth) and `buckets: { assets: { access: "private" } }` (Object Storage).
   - Pull credentials (`DATABASE_URL`, `DATABASE_URL_UNPOOLED`, `NEON_AUTH_BASE_URL`, `NEON_AUTH_JWKS_URL`, `AWS_*` S3 vars) via `neon deploy`.
2. **Database Driver & Connections**:
   - Use **pooled** `DATABASE_URL` (`-pooler`) for runtime application queries in `server.ts` via `pg` (`node-postgres`) or `@neondatabase/serverless`.
   - Use **direct** `DATABASE_URL_UNPOOLED` for schema migrations and DDL.
3. **Object Storage**:
   - Use S3-compatible client with `forcePathStyle: true` (or `files-sdk/neon`) pointing at `AWS_ENDPOINT_URL_S3`, storing only object keys in Postgres and issuing short-lived presigned URLs on authorized reads.
4. **Managed Better Auth**:
   - Use `@neon/auth` client wrapper (`VITE_NEON_AUTH_URL`) for React UI and verify JWTs against `NEON_AUTH_JWKS_URL` on Express `/api/*` routes.
   - Register `https://exonovel.exosystems.net` as the production trusted origin (plus active preview origin when developing) and sync custom OAuth provider credentials (`GOOGLE_OAUTH_CLIENT_ID` / `GOOGLE_OAUTH_CLIENT_SECRET`, `GITHUB_OAUTH_CLIENT_ID` / `GITHUB_OAUTH_CLIENT_SECRET`) from server environment secrets to Neon Auth.
   - Configure Neon Auth email delivery using **Brevo** custom SMTP (`smtp-relay.brevo.com:587`) sending from **`noreply@exosystems.net`** (`"Exosystems"` — general/auth emails outside the notification domain), resolving relay credentials via `BREVO_API_KEY`, `BREVO_SMTP_KEY`, and `BREVO_SENDER_NOREPLY_*` environment variables.

