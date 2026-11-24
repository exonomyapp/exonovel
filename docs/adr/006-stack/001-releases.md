# 006-001. Dependency Version Tracker & Upgrade Log

Date: 2026-11-21
Status: Accepted (Active Living Document)
Parent: [006-stack](../006-stack.md)

## Context

ExoNovel stays in active pre-launch development while several foundational open-source Rust and local-first tools mature toward production. This document tracks the versions currently declared in ExoNovel alongside the latest verified upstream releases, recording **why** each version bump matters and **which ExoNovel features** a release unlocks.

---

## Current Stack Inventory (Verified 2026-11-21)

### 1. Active Runtime Dependencies (`Cargo.toml` / CI)

| Technology | Role in ExoNovel | Currently Declared | Latest Verified Release | Upgrade Opportunity & Business Impact |
| :--- | :--- | :--- | :--- | :--- |
| **`leptos` / `leptos_meta` / `leptos_router` / `leptos_axum`** | Full-stack Rust SSR + WASM web UI | `0.8` (unpinned) | **`0.8.23`** *(and `0.9.0-rc1` prerelease)* | Automatically tracks `0.8.x` patches; evaluate `0.9` once stable to adopt compiler/hydration performance improvements |
| **`cargo-leptos`** | Build CLI for server + WASM bundles | `latest` via `taiki-e/install-action@v2` | `0.2.x` | Prebuilt binary in GitHub Actions keeps CI builds fast (~12m saved) and aligned with Leptos |
| **`axum`** | HTTP API & SSR router | `0.8` (unpinned) | **`0.8.9`** | Receives `0.8.x` routing, multipart/body-limit, and WebSocket fixes |
| **`sqlx`** | Neon PostgreSQL async driver & queries | `0.8` (postgres, rustls, json) | **`0.9.1`** *(requires Rust ≥ 1.94.0)* | Monitor `0.9.x` migration notes before bumping minor range once toolchain MSRV aligns |
| **`ed25519-dalek`** | `did:peer` keypair minting & verification | `2` (pkcs8, pem, rand_core) | **`3.0.0`** | Evaluate `3.0.0` API changes for signing W3C proofs and NextGraph wallet parity |
| **`image`** | Server-side 2 MB avatar processing (`64`/`256`/`1024` WebP/JPEG) | `0.25` | **`0.25.10`** | Pure-Rust decoding/encoding without native C libraries |
| **`comrak`** | GitHub Flavored Markdown docs parser | `0.35` | **`0.55.0`** | Significant version gap (`0.35 -> 0.55.0`); upgrading improves GFM compliance and extension support in Docs Reader |
| **`ammonia`** | HTML sanitization for rendered Markdown | `4` | **`4.x`** *(MSRV Rust ≥ 1.85)* | Tracks `4.x` security patches automatically |
| **`jsonwebtoken` / `reqwest` / `aes-gcm` / `tokio`** | Neon Auth JWKS verification & key sealing | `9` / `0.12` / `0.10` / `1` | Current `9.x` / `0.12.x` / `0.10.x` / `1.x` | Monitor major releases for HTTP/crypto hardening |
| **Neon PostgreSQL & Neon Managed Better Auth** | Cloud DB & OAuth/Email auth (`neon_auth`) | Managed SaaS | Live Managed Service | Tracks Neon Auth JWT, OAuth provider, and Postgres branching capabilities |

### 2. Planned Local-First, Collaboration & Payment Stack (Watching for Readiness)

| Technology | Target Capability Unlocked | Current Upstream State | Integration Trigger / Notes |
| :--- | :--- | :--- | :--- |
| **`willow25`** *(supersedes deprecated `willow-data-model` `0.8.0` & `meadowcap` `0.6.0`)* | Device-local encrypted data store & mobile sync | **`willow25 v0.7.11`** *(active pre-1.0)* | **Important upstream shift**: Generic `willow-data-model` and `meadowcap` crates were deprecated in favor of `willow25`. Adopt `willow25` when implementing the device export/mobile store |
| **`iroh` P2P (`iroh`, `iroh-docs`, `iroh-blobs`, `iroh-gossip`, `swarm-discovery`)** | Encrypted QUIC relays, mDNS LAN sync, live presence | `iroh` `0.35.x` / `1.x` modular crates (`iroh-dns 1.4.1`, `iroh-docs 0.101.0`, `iroh-gossip 0.102.0`) | Watch for stabilized `iroh` + `iroh-docs` sync alignment to embed in native desktop/mobile builds |
| **`nextgraph-rs` / `ng-wallet`** | Offline wallet, Pazzle unlock, QR/TextCode pairing | Active repo on `git.nextgraph.org` / GitHub mirror | Track crate extraction/compatibility for WebCrypto + OS keychain pairing |
| **`loro` CRDT Engine** | Simultaneous co-editing (`LoroTree` movable parts/chapters, `LoroText` prose, DAG time travel, `EphemeralStore` presence) | **`loro 1.18.0` (Accepted)** | Selected by owner (see [005-loro](../002-data/005-loro.md)); integrates into both Neon delta tables and `willow25` / `iroh-gossip` |
| **`leptos-fluent` + Mozilla Fluent (`.ftl`)** | 29-language Output UI & Input localization, Arabic/Hebrew RTL mirroring, Profile-pinned switchers | **`leptos-fluent 0.3.1` (Accepted)** | On-demand `.ftl` loading avoids unused-locale WASM bloat; covers `en, de, pl, fr, es, it, nl, pt, hu, ro, uk, ru, sr, el, ar, he, zh-Hans, zh-Hant, hi, bn, mr, te, ta, sw, ha, yo, ig, am, zu` (see [007-i18n](../007-i18n.md)) |
| **Push & Notifications (`web-push`, `a2`, `fcm_v1`, Tauri plugins, Brevo v3)** | 5-platform + Web + active Brevo multi-sender (`notes@exosystems.net` / `noreply@exosystems.net` / `support@exosystems.net`) | `web-push 0.11.0` / `a2 0.3.x` / `fcm_v1 0.3.0` / `tauri-plugin-notification 2.7.0` / `tauri-plugin-push-notifications 0.1.0` / **Brevo API v3 Active** | Powers [008-notify](../008-notify.md); **F-Droid (`foss` UnifiedPush-only)** first, then **Play (`play` FCM+UnifiedPush)** & Apple Store; GitHub-style Profile vs. Settings split |
| **Global Non-USD Payment Rails & Wallets** | **Tier 1**: dLocal, Stripe, Reown AppKit; **Tier 2**: Razorpay, Flutterwave/Paystack, Alipay+/WeChat Pay, TON, Lightning; **Tier 3**: Airwallex | **Accepted (AOTA Rollout Approved)** | Currency-neutral `(minor_units, currency_code)` ledger; all author wallet & commercial controls placed in **Profile** (see [004-pay](../004-pay.md)) |
| **Global Live UI & Motion (`PgListener` + SSE)** | Zero-reload live DOM patching (`<LiveRow>`, `<LiveField>`) backed by Neon `LISTEN/NOTIFY` over `DATABASE_URL_UNPOOLED` | **`sqlx::PgListener` + `axum` SSE (`0.8`)** | App-wide architecture defined in [009-live-ui](../009-live-ui.md) |
| **Maturity Scanner Headless CLI (Optional)** | Prose synthesis for unanticipated upstream capabilities | **`OpenCode` (`opencode serve`)** + local **Ollama / Qwen Code / Free API fallback** | Deterministic Rust poller first; pluggable headless CLI avoids single-provider rate-limit lock-in (see [005-003](../005-docs-ui/003-cms.md)) |

---

## Running Upgrade Log

| Date | Technology | Version Change | Justification & Unblocked ExoNovel Capabilities |
| :--- | :--- | :--- | :--- |
| **2026-11-24** | **Public SSR Home Page, Transparent Logo & Brand Alignment, `/privacy`, `/terms` & Google APIs Compliance (`crates/app/src/lib.rs`, `crates/server/src/main.rs`, `style/main.css`, `assets/`, `005-002`)** | *(Public OAuth Compliance)* | Replaced the unauthenticated login wall and SSR `"Loading ExoNovel session..."` blocker at `/` with an immediate SSR-rendered public **ExoNovel** landing page (`PublicLandingView`), dedicated `/signin` route, visible `"ExoNovel"` header wordmark paired with the transparent network symbol graphic (`/exonovel-symbol.png`, with the solid white logo backdrop removed across display modes) and `<head>` metadata matching Google OAuth branding, explicit app-purpose + Google Sign-In/Drive scope explanation + anti-AI-NCII statement above the fold, and expanded `/privacy` (Limited Use, no AI training, Connected Apps revocation, 30-day deletion) and `/terms` (Google APIs Terms of Service & zero-tolerance AI NCII clause). |
| **2026-11-23** | **CI Test Scoping, Recursion Limit, Warning Cleanup & Diagnostic Check Runs (`frontend/lib.rs`, `app/lib.rs`, `server/main.rs`, `live.rs`, `notify.rs`, `deploy.yml`, `AGENTS.md`)** | `actions@v4 -> v7/v8`, `recursion_limit = 512`, `dependabot lockfile-only` | **Root-cause fixes**: Resolved GitHub Actions Run #25 failure (`error: queries overflow the depth limit` in `exonovel_frontend` when `cargo test --workspace` unified `ssr` + `hydrate` features cleanly past Rust's default `128` limit) by adding `#![recursion_limit = "512"]` across all three crates, splitting CI validation cleanly (`cargo check -p exonovel-server --all-targets --features ssr`, `cargo check -p exonovel-frontend --target wasm32-unknown-unknown --features hydrate`, and `cargo test -p exonovel-server --features ssr`), clearing browser-vs-SSR compiler warnings in `crates/app/src/live.rs` and `crates/server/src/notify.rs`, publishing CI failure logs into public Check Runs (`checks: write`) so agents inspect exact failures without manual screenshots, and codifying `AGENTS.md` Rules 8 and 11–15 (`lockfile-only` Dependabot, `cleanup-dependabot-prs`, `.env.example` cleanup, evidence-only reporting). |
| **2026-11-21** | **Global Live UI (`009-live-ui`), Mode 2 Blind Relay, Public Willow i18n Namespace & `OpenCode` CLI Runner** | *(Architecture Finalized)* | Added app-wide zero-reload `PgListener` (`DATABASE_URL_UNPOOLED`) → SSE + CSS arrival animations (`009-live-ui`); mandated client-side E2EE blind storage for Mode 2 `iroh` relay buffering (`002-002`); selected public read-only `willow25`/`iroh-blobs` namespace for offline language packs (`007-i18n`); recommended headless `OpenCode` (`opencode serve`) with multi-provider/Ollama fallback (`005-003`). |
| **2026-11-21** | **Owner Decisions: `loro 1.18.0`, 29-Locale Output/Input i18n, Zero-Neon Startup, Brevo Active, GitHub-Style Profile/Settings** | *(Decisions Accepted)* | Selected **`loro 1.18.0`** (`002-005`); approved bidirectional **Neon ↔ `willow25`** import/export & zero-Neon native startup; expanded i18n to 29 locales (Ukrainian, Hungarian, Romanian, Serbian, Dutch, Greek, Hebrew RTL, Italian, Portuguese, Swahili, Hausa, Yoruba, Igbo, Amharic, Zulu + prior set) with Output vs Input language separation (`007`); approved AOTA Tier 1→3 payments (`004-003`); activated **Brevo** multi-sender on `exosystems.net` (`notes@` / `noreply@` / `support@`, `008`) and placed all author wallet/commercial controls in a GitHub-style Profile page. |
| **2026-11-21** | **Maturity CMS, `.github/dependabot.yml`, `007-i18n`, `008-notify`, Decision Reports** | *(ADR + Automation Baseline)* | Added daily Cargo + weekly GitHub Actions Dependabot monitoring (`.github/dependabot.yml`), designed the Neon PostgreSQL MADR 4.0.0 Tooling Maturity CMS (`005-003`), replaced redundant `localStorage` preferences (`002-004`), authored CRDT & Global Non-USD Payment decision reports (`002-003`, `004-003`), and specified 14-language RTL i18n (`007`) and 5-platform + optional Brevo notifications (`008`). |
| **2026-11-21** | **`image`** | *(New)* `-> 0.25` | Added pure-Rust `png`/`jpeg`/`webp`/`gif` processing so users can upload up to **2 MB** avatars and immediately serve lightweight **64×64** navbar thumbnails, **256×256** profile avatars, and **1024px** previews without native C dependencies. |
| **2026-11-21** | **Willow Schema Convention** | *(Design baseline)* | Recorded upstream deprecation of `willow-data-model` (`0.8.0`) & `meadowcap` (`0.6.0`) in favor of **`willow25` (`0.7.11`)**; aligned all centralized Neon PostgreSQL tables (`willow_namespace`, `willow_subspace`, `willow_path`, `willow_timestamp_us`, `payload_digest`, `byte_length`) for lossless relocation into `willow25`. |
| **2026-11-21** | **`cargo-leptos` & CI Cache** | `source build -> prebuilt + rust-cache@v2` | Replaced ~12-minute from-source installation with `taiki-e/install-action@v2` and added `Swatinem/rust-cache@v2` in `.github/workflows/deploy.yml`. |
| **2026-11-20** | **`leptos` / `axum` / `sqlx` / `comrak` / `ammonia`** | Node.js -> `0.8` / `0.8` / `0.8` / `0.35` / `4` | Migrated runtime 100% to compiled Rust + Leptos + Axum backed by Neon PostgreSQL & Neon Managed Better Auth. |
