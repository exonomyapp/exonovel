# ExoNovel — Active Development & Open Work (`session.md`)

> **Shared Multi-Author Coordination Notice (`exonomyapp/exonovel`)**:
> - Multiple AI Studio accounts and author sessions sync with `exonomyapp/exonovel@main` (`crates/`, `style/`, `assets/`, `.github/`, `docs/`, and the shared `Sync Console`).
> - **Author Session Attribution**: Because `docs/session.md` is shared across collaborators, every session must track its active changes and notes inside a clearly attributed section identified by the account's email address (e.g., `thalperi@gmail.com`, `tamiroslaw@gmail.com`). Collaborators from other accounts should add or update their own `<email>` subsection rather than overwriting another session's work in progress.

---

## 1. Shared Architectural & Build Boundaries (`exonomyapp/exonovel`)

- **Target Application (`crates/server` + `crates/app` WASM)**: Production (`https://exonovel.exosystems.net`) runs solely as the compiled Rust + Leptos WebAssembly application (`006-stack.md`). Preview verification runs against the deployed live system rather than any prototype server.
- **Author Sovereignty & Willow Parity**: Author `did:peer` identities, multiple Personas, comments, and paywalled entitlements stay owned by their creator, with PostgreSQL `willow_*` coordinates aligned to `willow25` + `iroh` portability (`002-data/002-iroh.md`).
- **4 GB Preview Container Boundary**: Perform static Rust/Fluent/CSS audits locally without running `rustc` or `cargo` locally; full `cargo check`, `cargo test`, and `cargo leptos build --release` run in GitHub Actions after push (`AGENTS.md` Rule 12).

---

## 2. Recent Verified Commits on `exonomyapp/exonovel@main`

- **`60f3f167` (2026-11-27)** — `fix(deploy): package Leptos hash.txt and verify SSR root route`: Fixed `.github/workflows/deploy.yml` to locate `hash.txt` beside the compiled server binary, require non-empty `deploy/hash.txt`, and verify HTTP `200` on `/` SSR output after deploy.
- **`28ec5cf1` (2026-11-26)** — `fix(ui,build): enable asset file hashing, themed mobile card dropdown & hero scaling`: Enabled `hash-files = true`, `<HashedStylesheet>`, HTML `no-cache` headers, custom themed mobile card dropdown in `crates/app/src/lib.rs`, and hero font scaling in `style/main.css`.
- **`b69ce5cf` (2026-11-26)** — `feat(home): refine hero layout, 4 educational cards, 3 sync modes & mobile logo`: Updated homepage copy, 4 educational panels, 3 sync modes (`Free` / `Monthly` / `Annual`), 3-row hero structure, and `45px` mobile logo height.
- **`a39b9949` (2026-11-26)** — `fix(style): restore mobile navbar logo size to 30px`: Scoped mobile navbar logo sizing inside `@media (max-width: 768px)` back to `30px` while keeping desktop `.navbar-logo` at 2x (`clamp(64px, 7.2vw, 80px)`).
- **`bf21f6f0` (2026-11-26)** — `feat(home): refine homepage copy, remove nav links, and double navbar logo`: Removed `Overview` and `Formats` links from the unauthenticated navbar, doubled desktop navbar logo size, and synced EN/DE/PL copy.
- **`91ebb436` (2026-11-26)** — `fix(app): clone chip_name before view! move in user chip`: Resolved Leptos closure move ownership compile error in the authenticated user chip.
- **`b760907b` (2026-11-26)** — `docs: rename Settings to Preferences in ADRs`: Aligned ADR terminology with `Preferences`.
- **`e057269c` & `1b35a5aa` (2026-11-26)** — CI diagnostic logging and Google Drive auth/session alignment.

---

## 3. Active Work & Staged Changes by Author Session

### Author Session: `tamiroslaw@gmail.com` (AI Studio Workspace)
*Scope: Sync Console clean segregation into `sync-console/` and feature/security upgrades prepared in this container.*

1. **Clean Segregation into `sync-console/` (`sync-console/`, `server.ts`, `index.html`, `vite.config.ts`, `workspace.md`)**:
   - Segregated the Sync Console backend (`sync-console/server/`) and frontend (`sync-console/src/`) out of the repository root so ExoNovel's Rust codebase (`crates/`, `style/`, `assets/`) remains untouched and unmixed, while keeping root `server.ts` as a 1-line launcher into `sync-console/server/index.ts`.
   - Classified legacy root `server/` and `src/` files as `Superseded on main` so they can be cleanly reviewed and removed when ready.
2. **AI-Studio-Only Restriction, Passcode Hardening & 7-Day Persistence (`sync-console/server/workspaceAuth.ts`, `sync-console/src/App.tsx`)**:
   - Restricted Sync Console web and API access strictly to AI Studio preview and loopback hosts (`requireAiStudioEnvironment`), returning HTTP `403` on external public hosts.
   - Added `scrypt` verification for `ADMIN_PASSCODE`, 5-attempt lockout rate limiting, persistent dual-secret 7-day session signing keys (`HttpOnly; Secure; SameSite=None; Partitioned` cookie + Bearer token), and `X-Requested-With: exonovel-sync` + allowed Origin checks on mutations.
3. **Live Website Probe, Intent Commit Box, Inline CI Logs, SSE Auto-Refresh, Line Diffs & `Cargo.lock` Safety (`sync-console/server/githubSync.ts`, `sync-console/server/githubMonitor.ts`, `sync-console/src/components/`)**:
   - Added live website health probing (`https://exonovel.exosystems.net/` and `/api/health`) directly on the Sync Console dashboard.
   - Added automatic SSE live updates via ETag (`If-None-Match`) GitHub polling (`/api/admin/github/events`) so pushes from any session update the open UI automatically.
   - Added expandable inline GitHub Actions summary and full failure logs on the dashboard without leaving the page.
   - Replaced the single-line commit input with a multi-line Commit Message `<textarea>` (4000-char cap) pre-filled from `.local/pending-commit.json` and cleared after push, plus an optional Pull Request Description `<textarea>`, while keeping Standard Merge (`merge_method: 'merge'`).
   - Added `LineDiffViewer.tsx` defaulting to color-coded line-by-line diffs (opened directly by clicking any file name or its `Diff` button) with `Changes Only` / `Show Full File` and `Side-by-Side` toggles.
   - Protected `Cargo.lock` from deletion and hid it from the workspace diff list (`REMOTE_MANAGED_HIDDEN_FILES`).
4. **Communication & Label Clarity (`AGENTS.md`, `sync-console/src/components/GitHubSyncPanel.tsx`)**:
   - Refined Rule 9 concisely to keep answers balanced at a project-manager level—focused on outcomes and real consequences rather than code mechanics or oversimplified basics.
   - Replaced location-only diff statuses with concise reconciliation labels (`Add to GitHub`, `Edited here`, `Both changed · merge`, `New on GitHub`, `Removed here`, `Deleted on GitHub`, and `Unused local` for obsolete local-only files) and excluded obsolete local files from push/cleanup selections (`canPush: category !== 'superseded'`).
   - Added baseline-backed detection for files deleted on GitHub by other sessions (`deleted_remote`): blocks accidental re-upload on push, removes unmodified local copies on pull, and resolves modified/deleted conflicts in Merge Review (`Delete Local File` / `Keep Local File`).

### Author Session: `thalperi@gmail.com` (AI Studio Workspace)
*Scope: Both `exonomyapp/exonovel` product changes and shared `Sync Console` tooling authored from this account.*

1. **`exonomyapp/exonovel` App, CI, `AGENTS.md` & `docs/session.md` Reconciliation (`crates/`, `style/`, `assets/`, `.github/`, `docs/`, `AGENTS.md`)**:
   - Pulled `.github/workflows/deploy.yml` (`60f3f167`) into this workspace so local Rust/Leptos crates, styles, assets, and deployment workflow match `exonomyapp/exonovel@main` (`0` app diffs).
   - Live diff audit against `main` (`60f3f167`): `9` modified files (`.env.example`, `AGENTS.md`, `bun.lock`, `docs/session.md`, `index.html`, `metadata.json`, `package.json`, `server.ts`, `src/App.tsx`), `8` new locally, and `91` on `main` only (superseded prototype/Neon skill/asset files). Reconciled `AGENTS.md` (preserving Rules 2–19 from `main` alongside Rule 1 container/Sync Console scope, updated Rule 4 author-email attribution, and Rules 20–22) and `docs/session.md` (preserving all active `main` context and `28ec5cf1`/`60f3f167` history alongside the `thalperi@gmail.com` session block), and seeded baseline snapshots in `.aistudio/sync-base/` against `main` (`60f3f167`) so push/pull tracking starts synchronized.
   - Updated `AGENTS.md` Rule 4 and `docs/session.md` so multi-author session work is segregated by **author account email** (`thalperi@gmail.com`) across both ExoNovel product work and Sync Console updates, and so every completed turn ends by updating this `docs/session.md` section.
2. **Bidirectional 3-Way Merge (`node-diff3`), Selective Sync & Selection `Filter` (`server/syncMerge.ts`, `server/githubSync.ts`, `server.ts`, `src/components/GitHubSyncPanel.tsx`)**:
   - Added `.aistudio/sync-base/` SHA-256-keyed per-file baseline snapshot tracking (`seed`, `pull`, `push`, `merge`) and `node-diff3` `mergeDigIn` three-way text merging (`runThreeWayMergeText`) protecting **both Pull and Push**:
     - **Pull (`POST /api/admin/github/pull`)**: Preserves local-only edits when `baseSha === remoteSha`, auto-merges non-overlapping local + remote edits while keeping the remote blob as the next merge baseline, and returns HTTP `409` conflict records (`overlap`, `no_base`, `binary`) instead of overwriting local changes.
     - **Push (`POST /api/admin/github/sync`)**: Checks every staged upload for `baseSha !== remoteSha` (or unseen `knownRemoteShas` mismatches), returns HTTP `409` conflicts with pre-computed suggested merges before touching GitHub blobs/trees, and updates baselines on direct pushes (keeping PR pushes tied to `main` until merged).
     - **Merge Resolver (`POST /api/admin/github/resolve-merge`)**: Added inline **Merge Review Required** workflow in the Sync Console (`Use Auto-Merged`, `Use Local`, `Use Remote`, editable buffer, `Apply Merge Locally`, and binary resolution).
   - Renamed panel heading to **Sync Console**, added categorized diffs (`All Differences`, `ExoNovel App & CI`, `Sync Console`, `Superseded on main`, `Ignored (.gitignore)`), inline side-by-side diffs, selective deletions (`sha: null`), and the `Filter` checkbox right of `Clear` (enabled when `Select All` is unchecked).
3. **`.gitignore`-Driven Visibility & Live File Filter (`server/githubSync.ts`, `server.ts`, `src/components/GitHubSyncPanel.tsx`)**:
   - Evaluates active workspace diffs against `.gitignore` (`ignore` parser) while locking core secret/cache protections (`.env*`, `.git/`, `node_modules/`, `target/`).
   - Added per-file `Ignore` controls, an `Ignored (.gitignore)` management tab (add/remove rules, `Unignore`, optional `tracked on main` untracking), and a debounced live file filter (`180ms` after typing stops, theme-matched surface colors, right-aligned at 25% list width inline with quick-select actions on desktop and full-width on mobile).
4. **Header Toolbar, Render-Loop Fix & Persistent Unlock Session (`src/App.tsx`, `server/workspaceAuth.ts`, `workspace.md`)**:
   - Moved `Refresh Diff` to the sticky header next to the theme switcher (`Refresh Diff` desktop, `Refresh` + icon-only `Lock` mobile) and stabilized callback refs / in-flight guards to prevent render loops.
   - Replaced ephemeral per-boot HMAC signing keys in `server/workspaceAuth.ts` with a persistent signing key derived from `ADMIN_PASSCODE` (or `.aistudio/.ws-signing.key`) so server restarts during app updates preserve the 7-day `exonovel_ws_session` browser cookie.

---

## 4. Open Unfinished Work & Roadmap Interviews (Shared Next Up)

1. **Google OAuth Brand, Client & Redirect URI Verification**:
   - Confirm production Google OAuth credentials, Authorized redirect URI `https://exonovel.exosystems.net/api/oauth/google/callback`, and branding submission under the `ExoNovel` OAuth client.
2. **Live Production Review (`https://exonovel.exosystems.net`)**:
   - Verify the updated hero typography alignment, 4-card educational panel, 3-mode section, mobile logo scaling, and EN/DE/PL switching.
3. **Content Models & Top 5 Formats (`003-content` / `003-001` / `003-002`)**:
   - Finalize author component ownership, reusable story elements (characters, plots, timelines, themes), and permissioned forking rules ahead of `loro 1.18.0` CRDT co-editing (`002-005`).
4. **Author Paywalls & AOTA Multi-Currency Payments (`004-pay` / `004-002` / `004-003`)**:
   - Specify paywall gating rules and Tier 1 checkout rollout (`dLocal`, `Stripe Connect`, `Reown AppKit`) inside the Profile commercial hub.
5. **Offline `willow25` + `iroh` Native Packaging (`002-data/002-iroh.md` / `006-stack.md`)**:
   - Sequence desktop (`Linux`, `macOS`, `Windows`) and Android **F-Droid (`foss` UnifiedPush-only)** builds ahead of Google Play and Apple Store submissions.
