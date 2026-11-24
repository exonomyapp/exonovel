# 005. Docs Reader UI, Auth Gate & Admin Access Control

Date: 2026-11-19
Status: Accepted (in active implementation)

## Context

ExoNovel's product documentation (`docs/` and `AGENTS.md`) is **private**—not a public part of the website and completely separate from `exosystems.net`. The standalone **ExoNovel** application (`https://exonovel.exosystems.net`) includes its own built-in **Leptos** Markdown documentation reader protected behind Neon Managed Better Auth and an admin-controlled access list.

## High-Level Story

1. **ExoNovel-Only Scope & Leptos Implementation**: The Docs Reader lives inside the independent **ExoNovel** Leptos application (never inside `exosystems.net`), rendered in Rust/Leptos (`comrak` + `ammonia` sanitization) and backed by Axum endpoints that verify Neon Better Auth tokens (`NEON_AUTH_JWKS_URL`).
2. **Welcoming Home, Universal Settings & Private Docs Gate**: Every signed-in user lands on a welcoming ExoNovel workspace home page (with no docs mention unless allowlisted) and has access to **Settings** (personal preferences, avatar uploads, profile-backed return navigation in `user_profiles`, and **Connected External Apps** for linking or selectively revoking optional integrations such as Google Drive without terminating ExoNovel login). As soon as `exonomyapp@gmail.com` or an enabled `did:peer` entry in `docs_access_allowlist` signs in, **Docs Reader** and **Google Drive** become visible in navigation; site-admin panels inside Settings remain restricted to `exonomyapp@gmail.com`.
3. **Reader, Explorer, Google Drive Archive & Maturity CMS Evolution**: Authorized users get a full GitHub Flavored Markdown reader with a **statefully collapsible and resizable** left-hand navigation tree, search filter, right-rail heading outline, light/dark themes, and an integrated **Google Drive Workspace** (1-click SSO-assisted launch, full docs archive backup, chat transcript Markdown export, and activity/trash management). Under [003-cms](005-docs-ui/003-cms.md), this reader evolves into a **Neon PostgreSQL MADR 4.0.0 & Tooling Maturity CMS** so ExoNovel tracks `on-hold` requirements, links ADRs to required tool versions, and captures release-driven feature proposals directly in-app.

## Sub-ADRs

- [001-reader](005-docs-ui/001-reader.md) — Left-nav explorer (collapsible/resizable) & Markdown reader UI
- [002-gate](005-docs-ui/002-gate.md) — Better Auth sign-in gate & Admin access settings
- [003-cms](005-docs-ui/003-cms.md) — Data-Driven ADR & Tooling Maturity CMS (MADR 4.0.0 + Neon PostgreSQL)
- [004-inference](005-docs-ui/004-inference.md) — Headless `OpenCode` Inference Providers, Live `/v1/models` Discovery & Cross-Provider Failover

