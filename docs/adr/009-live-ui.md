# 009. Global App-Wide Live UI Updates & Arrival Motion

Date: 2026-11-21
Status: Accepted (Global architectural rule across all ExoNovel features and pages)

## Context

Whenever new content is written to **Neon PostgreSQL** OR arrives in a local **`willow25` store** (including platform installs on **Linux, macOS, Windows, Android, and iOS** operating locally on the user's device without cloud DB access, when P2P updates sync in from other peers over `iroh`) that belongs in the view a user currently has open, the user must **never have to reload the view or restart the app**. Furthermore, newly arriving rows and updated fields must show **subtle, localized arrival motion** right where the change happens—making it visually clear to the eye that live content just landed. Because this applies identically across web and local device installs and every feature (Docs/ADR CMS, Maturity Proposals, Profile Wallet/Paywalls/Plans/Groups/Personas, Allowlist, Comments, Notifications, Stories), it is engineered once at the **global application shell level**, never per-feature.

## Decision

### 1. Storage-Agnostic Live Change Bus (Neon `PgListener` + Local `willow25` Store Watcher)
Both centralized Web mode and zero-Neon native installs emit the exact same typed **`LivePatchEvent`** schema (`entity_kind`, `entity_id`, `scope_key`, `op: insert | update | delete`, `changed_fields`, `payload`, `seq_ms`) into one global client signal bus:

- **Source A — Neon PostgreSQL `LISTEN/NOTIFY` via Unpooled Direct Connection (`DATABASE_URL_UNPOOLED`)**:
  - `crates/server` runs a dedicated resilient `sqlx::postgres::PgListener` worker using **`DATABASE_URL_UNPOOLED`** (required because PgBouncer connection pooling suppresses `LISTEN/NOTIFY` sessions).
  - Database triggers and server mutations emit `LivePatchEvent` notifications fanned out through a `tokio::sync::broadcast` channel to the single authorized **Axum SSE Stream (`/api/events/stream`)**, filtered by the signed-in user's `did:peer`, active persona, groups, and role.
- **Source B — Native `willow25` / `iroh` Store Watcher (Zero-Neon Parity)**:
  - On desktop and mobile platform installs running locally on device or connected via personal sync, an embedded Rust store observer watches every `willow25` namespace/subspace write—whether from another local process or synced silently in the background via **`iroh-docs` / `iroh-blobs`** from collaborators—maps the updated Willow path/entry into the exact same **`LivePatchEvent`**, and dispatches it over Tauri IPC / internal Rust broadcast channel straight into the UI without polling or manual refresh.

### 2. Global Keyed Reconciliation in Leptos (`LiveSyncContext`)
- Mounted once at the root `<App/>` shell so **every page and component** consumes the same live event store.
- Keyed reactive collections (`use_live_list` / `use_live_record`) match incoming events by primary key (`entity_id`) and apply **surgical, in-place signal updates**:
  - **`insert`**: Prepends or inserts the new row at its deterministic sort position without re-fetching or resetting scroll position.
  - **`update`**: Patches only the affected record and records which exact `changed_fields` changed in the `last_patched_at` signal map.
  - **`delete`**: Smoothly collapses and removes only the deleted row.

### 3. Global Visual Arrival Motion System (Tailwind CSS Primitives)
All tables, lists, cards, and detail fields render through shared global primitives (`<LiveRow>` and `<LiveField>` backed by global utility classes in `style/main.css`):
- **Row Insertion (`.exo-live-insert`)**: Modest vertical slide-and-fade entrance (`translateY(-6px) -> 0`, `opacity 0 -> 1`, ~320ms `cubic-bezier(0.16, 1, 0.3, 1)`) paired with a brief left/border accent glow that gently fades out over ~1.4 seconds.
- **Field Update (`.exo-live-update`)**: Subtle directional highlight sweep + gentle scale/tint cue (~900ms) localized strictly on the updated cell or badge so existing surrounding text never jumps.
- **Accessibility**: Respects `@media (prefers-reduced-motion: reduce)` by replacing spatial movement with a calm, non-moving opacity/border highlight.

## Consequences

- Zero manual page refreshes across the entire ExoNovel web and native UI.
- Every future feature built in Phases 2+ inherits live reactivity and standardized arrival animations automatically by using `<LiveRow>` and `<LiveField>` wired to `LiveSyncContext`.
