# ExoNovel — Active Development & Open Work (`session.md`)

> **Shared Multi-Author Coordination Notice (`exonomyapp/exonovel`)**:
> - Multiple AI Studio accounts and author sessions sync with `exonomyapp/exonovel@main` (`crates/`, `style/`, `assets/`, `.github/`, `docs/`, and the shared `Sync Console`).
> - **Author Session Attribution**: Because `docs/session.md` is shared across collaborators, every session must track its active changes and notes inside a clearly attributed section identified by the account's email address (e.g., `exonomyapp@gmail.com`, `thalperi@gmail.com`, `tamiroslaw@gmail.com`). Collaborators from other accounts should add or update their own `<email>` subsection rather than overwriting another session's work in progress.
> - **Retention Policy (`AGENTS.md` Rule #4)**: Never delete or cycle out documented history from `docs/session.md` even after work is complete—only the owner may authorize removing history.

---

## 1. Shared Architectural & Build Boundaries (`exonomyapp/exonovel`)

- **Target Application (`crates/server` + `crates/app` WASM)**: Production (`https://exonovel.exosystems.net`) runs solely as the compiled Rust + Leptos WebAssembly application (`006-stack.md`). Preview verification runs against the deployed live system rather than any prototype server.
- **Author Sovereignty & Willow Parity**: Author `did:peer` identities, multiple Personas, comments, and paywalled entitlements stay owned by their creator, with PostgreSQL `willow_*` coordinates aligned to `willow25` + `iroh` portability (`002-data/002-iroh.md`).
- **4 GB Preview Container Boundary**: Perform static Rust/Fluent/CSS audits locally without running `rustc` or `cargo` locally; full `cargo check`, `cargo test`, and `cargo leptos build --release` run in GitHub Actions after push (`AGENTS.md` Rule 12).

---

## 2. Recent Verified Commits on `exonomyapp/exonovel@main`

- **`f4eb01a4` (2026-11-28)** — `feat(sync-console): segregate into sync-console/ with live SSE, health check, inline logs & line diffs`: Segregated the Sync Console into `sync-console/`, added live website health checks, ETag SSE updates, inline CI logs, scrypt passcode hardening, line-by-line diff viewer, `Cargo.lock` protection, multi-line commit box, and Standard Merge.
- **`7ce158d4` (2026-11-27)** — `chore: update 14 file(s) and remove 56 file(s)`: Removed the superseded root TypeScript docs prototype and 56 obsolete assets/files in favor of the Sync Console workspace.
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

### Author Session: `exonomyapp@gmail.com` (AI Studio Workspace)
*Scope: Core ExoNovel Rust/Leptos product work (`crates/`, `style/`, `docs/`, `AGENTS.md`) plus pulling and configuring the segregated `Sync Console` in this workspace.*

1. **"Dear World" Beginner User Guides (`010-dear-world`, `/dear-world`, `GET /api/guides`, `GET /api/guides/{slug}`)**:
   - Added 4 warm, seasoned-author starter chapters across English (`docs/guides/`), German (`docs/i18n/de/guides/`), and Polish (`docs/i18n/pl/guides/`) tailored for High School and University writers learning how to use ExoNovel.
   - Stored guides in PostgreSQL (`dear_world_guides` and `dear_world_guide_translations`) seeded automatically on server startup (`crates/server/src/guides.rs`) with `willow_*` coordinates ready for future `willow25` synchronization while `willow25` remains deferred.
   - Made **Dear World** publicly readable without sign-in and accessible from a single top-navbar entry point on desktop and mobile (removed duplicate links from Desk and footer).
2. **Navbar, Writer's "Desk", Fixed Footer & Language Switcher (`crates/app/src/lib.rs`, `style/main.css`)**:
   - Removed all `"Home"` buttons across unauthenticated and authenticated headers; signed-in writers see **`Desk`** and **`Dear World`**, moved **`Live`** to the right of the user chip, and placed admin **`Docs & ADRs`** inside the user chip dropdown (desktop and mobile).
   - Merged the signed-in workspace into one cohesive Writer's **`Desk`** (`WorkspaceHomeView`) with a top-right truncated `did:peer` + copy button, 5 format starters, 4 Story Building Blocks cards, and an interactive session-only Quick Idea Scratchpad (`InputLocaleBar`).
   - Pinned `PublicFooter` cleanly to the bottom across scrolling pages showing **`ExoNovel, from Exosystems`** (with stealth hover button to `https://exosystems.net`), legal links, and an upward 29-language switcher removed from the top bar.
3. **In-App Custom Pickers (`ExoSelect`, `ExoFileButton`) (`crates/app/src/ui.rs`, `crates/app/src/{preferences,profile,connected_apps_ui,cms}.rs`)**:
   - Replaced all native OS `<select>` menus across `crates/` with theme-matched in-app `ExoSelect` dropdowns and `ExoFileButton` image triggers so browser/OS native pickers never interrupt the ExoNovel UI.
4. **Preferences Cleanup, Sync Clarification & 3-Tab Profile Commercial Split (`crates/app/src/{preferences,profile}.rs`, `crates/server/src/i18n.rs`, `docs/adr/`)**:
   - Removed `"Theme Appearance"` and `"Enable Mode 3..."` / `"Current tier:"` controls from Preferences; eliminated `"mode"` / `"modes"` / `"option"` wording across UI, EN/DE/PL `.ftl` packs, and EN/DE/PL ADRs (`002-data`, `002-002`, `004-pay`, `008-notify`, `009-live-ui`, `006-001`), clarifying that Web authoring, Device authoring, and automatic collaboration sync between co-writers across Web ↔ Device are free everywhere while only personal Web ↔ Device sync carries a fee.
   - Split the combined Profile tab into distinct **`Wallet`**, **`Paywalls`**, and **`Plans`** sections.
5. **Unified Non-Floating Resizable Navigation Across Profile & Preferences (`crates/app/src/{ui,profile,preferences,connected_apps_ui,models}.rs`, `crates/server/src/{db,main,i18n}.rs`, `style/main.css`, `Cargo.toml`, `002-data/004-db-prefs.md`)**:
   - Replaced Profile's horizontal underline tabs and Preferences' floating left sidebar card with one shared flush, non-floating GitHub-Settings-style layout (`ExoSettingsPage`, `ExoSettingsSplit`, `SettingsResizeCtl`) featuring grouped navigation, 16×16 stroke icons, quiet group headings, hairline dividers, active accent bar, and hairline-underlined right-pane section headings.
   - Added an interactive WAI-ARIA `role="separator"` vertical splitter supporting pointer drag and keyboard resize (`200px–420px`, RTL-aware), persisted per page with zero `localStorage` and zero legacy shared column (`settings_nav_width` dropped): `user_profiles.profile_nav_width` (`profileNavWidth`) for **Profile** and `user_profiles.preferences_nav_width` (`preferencesNavWidth`) for **Preferences**, synchronized in ADR `002-004` (EN/DE/PL) and EN/DE/PL Fluent packs.
6. **Proposed Writer's Desk ADR (`011-desk.md` EN/DE/PL)**:
   - Documented the Writer's Desk demo scaffold (`WorkspaceHomeView`)—covering the 5-format starter shelf, 4 Story Building Blocks cards (`Characters`, `Plots`, `Timelines`, `Themes`), and session-only Quick Idea Scratchpad (`InputLocaleBar`)—alongside open design interview questions awaiting owner review before PostgreSQL persistence or editors are added.
7. **Workspace Sync Console Migration & Multi-Author Reconciliation (`sync-console/`, `server.ts`, `index.html`, `vite.config.ts`, `tsconfig.json`, `workspace.md`, `.env.example`, `package.json`, `AGENTS.md`, `docs/session.md`, `.local/pending-commit.json`)**:
   - Pulled the segregated `Sync Console` from `exonomyapp/exonovel@main` (`f4eb01a4`), installed `ignore` and `node-diff3`, removed obsolete prototype directories/files (`server/`, `src/`, `neon.ts`, `firebase-applet-config.json`, `public/assets/exonovel-google-logo.png`) while preserving `public/assets/aistudio/`, seeded 141 `.aistudio/sync-base/` baseline snapshots against `main` (including `Cargo.toml`), pre-filled `.local/pending-commit.json`, reconciled `AGENTS.md` (EN/DE/PL) preserving both GitHub's Rules 9/19/20–22 and the owner's Rule 4 retention policy, merged `docs/session.md` cleanly across all collaborator sessions, and verified build, typecheck, and live `/api/health` in `sync-console` mode with both GitHub and passcode secrets active.
   - **Owner Retention Decision Noted**: Historical commit summaries in Section 2 remain retained in `docs/session.md` per owner direction.
   - **Open Checkpoint Awaiting Owner Reply**: Review Proposed ADR `011-desk.md` and the Desk demo implementation to decide which placeholder blocks to keep, simplify, or redesign before PostgreSQL wiring.

### Author Session: `tamiroslaw@gmail.com` (AI Studio Workspace)
*Scope: Sync Console clean segregation into `sync-console/` and feature/security upgrades prepared in that container.*

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
*Scope: Both `exonomyapp/exonovel` product changes and shared `Sync Console` tooling authored from that account.*

1. **`exonomyapp/exonovel` App, CI, `AGENTS.md` & `docs/session.md` Reconciliation (`crates/`, `style/`, `assets/`, `.github/`, `docs/`, `AGENTS.md`)**:
   - Pulled `.github/workflows/deploy.yml` (`60f3f167`) into that workspace and seeded `.aistudio/sync-base/` baselines against `main`.
2. **Bidirectional 3-Way Merge (`node-diff3`), Selective Sync & Selection `Filter` (`server/syncMerge.ts`, `server/githubSync.ts`, `server.ts`, `src/components/GitHubSyncPanel.tsx`)**:
   - Added `.aistudio/sync-base/` SHA-256-keyed per-file baseline snapshot tracking and `node-diff3` `mergeDigIn` three-way text merging protecting both Pull and Push, plus inline Merge Review resolution.
3. **`.gitignore`-Driven Visibility & Live File Filter (`server/githubSync.ts`, `server.ts`, `src/components/GitHubSyncPanel.tsx`)**:
   - Evaluates active workspace diffs against `.gitignore` (`ignore` parser) with locked safety rules, per-file `Ignore` / `Unignore`, and debounced live file filtering.
4. **Header Toolbar, Render-Loop Fix & Persistent Unlock Session (`src/App.tsx`, `server/workspaceAuth.ts`, `workspace.md`)**:
   - Moved `Refresh Diff` to the sticky header and persisted the 7-day `exonovel_ws_session` unlock cookie across server restarts.

---

## 4. Open Unfinished Work & Roadmap Interviews (Shared Next Up)

1. **Writer's Desk Proposal Review (`011-desk.md`)**:
   - Confirm which placeholder blocks on the signed-in **Desk** (*5-format starter shelf*, *4 Story Building Blocks cards*, *Quick Idea Scratchpad*) to keep, simplify, or redesign before wiring PostgreSQL tables and editors.
2. **Google OAuth Brand, Client & Redirect URI Verification**:
   - Confirm production Google OAuth credentials, Authorized redirect URI `https://exonovel.exosystems.net/api/oauth/google/callback`, and branding submission under the `ExoNovel` OAuth client.
3. **Live Production Review (`https://exonovel.exosystems.net`)**:
   - Verify the updated **Dear World** guides, **Desk**, navbar/footer layout, in-app dropdowns, and unified **Profile** / **Preferences** split navigation once pushed and deployed via GitHub Actions.
4. **Content Models & Top 5 Formats (`003-content` / `003-001` / `003-002`)**:
   - Finalize author component ownership, reusable story elements (characters, plots, timelines, themes), and permissioned forking rules ahead of `loro 1.18.0` CRDT co-editing (`002-005`).
5. **Author Paywalls & AOTA Multi-Currency Payments (`004-pay` / `004-002` / `004-003`)**:
   - Specify paywall gating rules and Tier 1 checkout rollout (`dLocal`, `Stripe Connect`, `Reown AppKit`) inside the Profile commercial hub.
6. **Offline `willow25` + `iroh` Native Packaging (`002-data/002-iroh.md` / `006-stack.md`)**:
   - Sequence desktop (`Linux`, `macOS`, `Windows`) and Android **F-Droid (`foss` UnifiedPush-only)** builds ahead of Google Play and Apple Store submissions.
