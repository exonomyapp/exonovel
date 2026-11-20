# 005. Docs Reader UI, Auth Gate & Admin Access Control

Date: 2026-11-19
Status: Accepted (in active implementation)

## Context

ExoNovel's product documentation (`docs/` and `AGENTS.md`) is **private**—not a public part of the website. The project needs an in-app Markdown reader so the owner and authorized reviewers can browse and read docs live as they evolve, protected behind Neon Better Auth and an admin-controlled access list.

## High-Level Story

1. **Private Auth Gate**: Unauthenticated visitors see a dedicated **Sign-In** screen (powered by **Neon Better Auth** with Google, GitHub, and email/password), never the docs tree or Markdown payloads. Both frontend routes and Express `/api/docs*` endpoints enforce authentication and authorization.
2. **Admin Account & Settings Page**: The designated admin account (`exonomyapp@gmail.com`) can open a **Settings** view inside the app to allow or revoke specific users/emails who are permitted to view the documentation.
3. **Reader & Explorer UX**: Authenticated authorized users get a full GitHub Flavored Markdown reader with a **statefully collapsible and resizable** left-hand navigation tree, search filter, right-rail heading outline, and light/dark themes.

## Sub-ADRs

- [001-reader](005-docs-ui/001-reader.md) — Left-nav explorer (collapsible/resizable) & Markdown reader UI
- [002-gate](005-docs-ui/002-gate.md) — Better Auth sign-in gate & Admin access settings
