# 005. Docs Reader UI, Auth Gate & Admin Access Control

Date: 2026-11-19
Status: Accepted (in active implementation)

## Context

ExoNovel's product documentation (`docs/` and `AGENTS.md`) is **private**—not a public part of the website and completely separate from `exosystems.net`. The standalone **ExoNovel** application (`https://exonovel.exosystems.net`) includes its own built-in **Leptos** Markdown documentation reader protected behind Neon Managed Better Auth and an admin-controlled access list.

## High-Level Story

1. **ExoNovel-Only Scope & Leptos Implementation**: The Docs Reader lives inside the independent **ExoNovel** Leptos application (never inside `exosystems.net`), rendered in Rust/Leptos (`comrak` + `ammonia` sanitization or `leptos_md`) and backed by Axum endpoints that verify Neon Better Auth tokens (`NEON_AUTH_JWKS_URL`).
2. **Immediate Post-Login Visibility & Gate**: Unauthenticated visitors cannot access docs routes or Markdown payloads. As soon as `exonomyapp@gmail.com` (or an email allowlisted in `docs_access_allowlist`) signs in to ExoNovel, the **Docs Reader** navigation option (and **Settings** for `exonomyapp@gmail.com`) is immediately visible in the ExoNovel UI.
3. **Reader & Explorer UX**: Authorized users get a full GitHub Flavored Markdown reader with a **statefully collapsible and resizable** left-hand navigation tree, search filter, right-rail heading outline, and light/dark themes.

## Sub-ADRs

- [001-reader](005-docs-ui/001-reader.md) — Left-nav explorer (collapsible/resizable) & Markdown reader UI
- [002-gate](005-docs-ui/002-gate.md) — Better Auth sign-in gate & Admin access settings

