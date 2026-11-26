# ExoNovel — Active Development & Open Work (`session.md`)

> **Scope & Purpose Notice (Coding Target: `exonomyapp/exonovel`)**:
> All implementation, styling, i18n, and architecture changes recorded in this file belong exclusively to the **`exonomyapp/exonovel`** Rust/Leptos codebase (`crates/`, `style/`, `assets/`, `.github/`, `docs/`). Staging container harness tooling (`server.ts`, `server/`, `src/`, `AGENTS.md`, `workspace.md`) is strictly excluded from `exonomyapp/exonovel` diffs and pushes.
> Keep `docs/session.md` focused on current active work, verified recent commits on `main`, and open unfinished roadmap items (distill finalized architecture into `docs/adr/`).

---

## 1. Active `exonomyapp/exonovel` Architectural & Build Boundaries

- **Target Application (`crates/server` + `crates/app` WASM)**: Production (`https://exonovel.exosystems.net`) runs solely as the compiled Rust + Leptos WebAssembly application (`006-stack.md`). Preview verification runs against the deployed live system rather than any prototype server.
- **Author Sovereignty & Willow Parity**: Author `did:peer` identities, multiple Personas, comments, and paywalled entitlements stay owned by their creator, with PostgreSQL `willow_*` coordinates aligned to `willow25` + `iroh` portability (`002-data/002-iroh.md`).
- **4 GB Preview Container Boundary**: Perform static Rust/Fluent/CSS audits locally without running `rustc` or `cargo` locally; full `cargo check`, `cargo test`, and `cargo leptos build --release` run in GitHub Actions after push (`AGENTS.md` Rule 12).

---

## 2. Recent Verified Commits Pushed to `exonomyapp/exonovel@main`

- **`b69ce5cf` (2026-11-26)** — `feat(home): refine hero layout, 4 educational cards, 3 sync modes & mobile logo`: Updated homepage copy, 4 educational panels, 3 sync modes (`Free` / `Monthly` / `Annual`), 3-row hero structure, and `45px` mobile logo height in CSS.
- **`a39b9949` (2026-11-26)** — `fix(style): restore mobile navbar logo size to 30px`: Scoped mobile navbar logo sizing inside `@media (max-width: 768px)` back to `30px` while keeping desktop `.navbar-logo` at 2x (`clamp(64px, 7.2vw, 80px)`).
- **`bf21f6f0` (2026-11-26)** — `feat(home): refine homepage copy, remove nav links, and double navbar logo`: Removed `Overview` and `Formats` links from the unauthenticated navbar, doubled the navbar logo size, and synced initial EN/DE/PL homepage copy in `crates/app/src/lib.rs` and `crates/server/src/i18n.rs`.
- **`91ebb436` (2026-11-26)** — `fix(app): clone chip_name before view! move in user chip`: Resolved Leptos closure move ownership compile error in the authenticated user chip.
- **`b760907b` (2026-11-26)** — `docs: rename Settings to Preferences in ADRs`: Aligned all ADR terminology with the app-wide `Preferences` naming convention.
- **`e057269c` (2026-11-26)** — `ci: improve CI diagnostic logging and fix path handling` & **`1b35a5aa` (2026-11-26)** — Google Drive auth/session alignment in `.github/workflows/deploy.yml` and `docs/`.

---

## 3. Staged Work in Progress (Ready to Push to `exonomyapp/exonovel@main`)

Changes staged across `Cargo.toml`, `.github/workflows/deploy.yml`, `crates/server/src/main.rs`, `crates/app/src/lib.rs`, `style/main.css`, and `docs/session.md`:

1. **Content-Hashed CSS/JS/WASM Delivery (`Cargo.toml`, `.github/workflows/deploy.yml`, `crates/server/src/main.rs`)**:
   - Enabled `hash-files = true` (`hash-file-name = "hash.txt"`), switched the server shell to `<HashedStylesheet id="leptos" options=options.clone() />`, packaged `hash.txt` with `LEPTOS_HASH_FILES=true` in `systemd`, and set `no-cache` on dynamic SSR HTML + immutable caching on `/pkg/*` so browsers never execute stale cached WASM or CSS against updated HTML (which previously broke hydration and button click handlers).
2. **Custom ExoNovel-Themed Mobile Card Dropdown (`crates/app/src/lib.rs`, `style/main.css`)**:
   - Replaced the native OS `<select>` picker with an in-page themed dropdown menu (`.exo-carousel-dropdown-trigger` + `.exo-carousel-dropdown-menu`) styled cleanly in ExoNovel's light/dark theme tokens, keeping `←` / `→` arrows bidirectionally synced while leaving desktop tabs untouched.
3. **Hero Title Font Proportions Across Viewports (`style/main.css`)**:
   - Adjusted mobile hero title clamping (`clamp(27px, 7.2vw, 38px)`) and hero padding (`22px 18px`) inside `@media (max-width: 768px)` alongside desktop (`clamp(38px, 4vw, 54px)`), ensuring rows 1 & 2 stay 1/3 smaller than `... by Design!` without dropping below body copy size.

---

## 4. Open Unfinished Work & Roadmap Interviews (Next Up)

1. **Google OAuth Brand, Client & Redirect URI Verification**:
   - Confirm production Google OAuth credentials, Authorized redirect URI `https://exonovel.exosystems.net/api/oauth/google/callback`, and branding submission under the `ExoNovel` OAuth client.
2. **Live Production Review (`https://exonovel.exosystems.net`)**:
   - Verify the updated hero typography alignment, 4-card educational panel, 3-mode section, mobile `45px` logo scaling, and EN/DE/PL switching once GitHub Actions deploys these changes.
3. **Content Models & Top 5 Formats (`003-content` / `003-001` / `003-002`)**:
   - Finalize author component ownership, reusable story elements (characters, plots, timelines, themes), and permissioned forking rules ahead of `loro 1.18.0` CRDT co-editing (`002-005`).
4. **Author Paywalls & AOTA Multi-Currency Payments (`004-pay` / `004-002` / `004-003`)**:
   - Specify paywall gating rules and Tier 1 checkout rollout (`dLocal`, `Stripe Connect`, `Reown AppKit`) inside the Profile commercial hub.
5. **Offline `willow25` + `iroh` Native Packaging (`002-data/002-iroh.md` / `006-stack.md`)**:
   - Sequence desktop (`Linux`, `macOS`, `Windows`) and Android **F-Droid (`foss` UnifiedPush-only)** builds ahead of Google Play and Apple Store submissions.

