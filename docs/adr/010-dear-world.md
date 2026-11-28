# 010. "Dear World" User Guides, PostgreSQL Seed Store & Public Onboarding

Date: 2026-11-27
Status: Accepted (PostgreSQL + `willow_*` coordinate store active now; `willow25` peer replication deferred until native device client rollout)

## Context

ExoNovel introduces new High School and University writers to a genuinely different way of writing together: story building blocks (`Characters`, `Plots`, `Timelines`, `Themes`), multiple `Personas`, permissioned remixing, and seamless collaboration between web browsers and offline native apps. Inspired by GitHub Docs' **"Hello World"** tutorial (`https://docs.github.com/en/get-started/using-github/hello-world`), ExoNovel needs a dedicated, welcoming **User Guides** series—distinct from internal architectural decision records—without requiring sign-in before someone can read how ExoNovel works.

At the same time, direct `willow25` (`v0.7.11`) peer replication is intentionally deferred on the hosted server until the native desktop and mobile client rollout begins, so guides need live PostgreSQL storage today while remaining immediately portable to `willow25` tomorrow.

## Decision

1. **"Dear World" Editorial Concept & Scope**:
   - Name the public user guide series **"Dear World"** and address readers in the warm, encouraging voice of a seasoned author mentoring budding writers.
   - Focus strictly on **how to use ExoNovel to write in a new way** (starting on the Desk, reusing story building blocks, inventing Personas, inviting classmates/friends across web and device apps, and choosing what stays public vs. behind a paywall)—never generic creative-writing craft lectures.
   - Structure the initial series as 4 focused chapters across English (`docs/guides/`), German (`docs/i18n/de/guides/`), and Polish (`docs/i18n/pl/guides/`):
     - `01-dear-world` — Your Writing Desk & First Hello
     - `02-first-story-and-blocks` — Your First Story & Building Blocks
     - `03-personas-groups-and-ownership` — Personas, Writing Circles & Keeping What You Make
     - `04-web-device-and-sharing` — Writing Anywhere, Collaborating & Sharing

2. **Public Access & Single Navigation Entry Point (Top Nav Bar Only)**:
   - Expose **Dear World** publicly at `/dear-world` (and `/dear-world/{slug}`) via public SSR + JSON APIs (`GET /api/guides` and `GET /api/guides/{slug}`), accessible **without signing in**.
   - Place **`Dear World`** in **one location only**: directly in the top navigation bar (and its mobile navigation menu) for both unauthenticated visitors and signed-in writers (paired with **`Desk`** once signed in; zero `"Home"` buttons anywhere in the navbar). Never duplicate `Dear World` launch links inside the signed-in **Desk**, page footers, or sub-pages rendered beneath the same top navigation bar.

3. **PostgreSQL Data Model with Deferred `willow25` Coordinates (`crates/server/src/guides.rs`)**:
   - Store canonical metadata and localized Markdown/HTML editions in PostgreSQL:
     - `dear_world_guides` (`slug PRIMARY KEY`, `chapter_order`, `audience`, `willow_namespace`, `willow_subspace`, `willow_path`, `willow_timestamp_us`, timestamps).
     - `dear_world_guide_translations` (`PRIMARY KEY (slug, locale)`, `title`, `summary`, `raw_markdown`, `html`, `headings_json`, `reading_minutes`, `word_count`, plus matching `willow_namespace = 'exonovel'`, `willow_subspace = 'public/dear-world'`, `willow_path = '/dear-world/{slug}'`, `willow_timestamp_us`).
   - Seed disk Markdown chapters idempotently on server startup (`seed_guides_from_disk`), sanitize rendered GFM HTML with `comrak` + `ammonia`, and fall back cleanly to English (`translationStatus: "en-fallback"`) for locales awaiting dedicated translation.
   - Because every guide record carries deterministic `willow_*` coordinates, the entire **"Dear World"** guide library will sync directly into ExoNovel's public read-only `willow25` namespace for offline device installs once `willow25` synchronization ships—without schema migration.

## Consequences

- Prospective High School and University writers can explore **Dear World** before creating an account and jump straight from reading into signing in or opening their **Desk**.
- Deferring `willow25` avoids blocking public documentation on pre-1.0 crate wiring while guaranteeing zero-friction future federation into ExoNovel's public Willow store.
