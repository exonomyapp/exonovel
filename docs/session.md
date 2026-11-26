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

- **`a39b9949` (2026-11-26)** — `fix(style): restore mobile navbar logo size to 30px`: Scoped mobile navbar logo sizing inside `@media (max-width: 768px)` back to `30px` while keeping desktop `.navbar-logo` at 2x (`clamp(64px, 7.2vw, 80px)`).
- **`bf21f6f0` (2026-11-26)** — `feat(home): refine homepage copy, remove nav links, and double navbar logo`: Removed `Overview` and `Formats` links from the unauthenticated navbar, doubled the navbar logo size, and synced initial EN/DE/PL homepage copy in `crates/app/src/lib.rs` and `crates/server/src/i18n.rs`.
- **`91ebb436` (2026-11-26)** — `fix(app): clone chip_name before view! move in user chip`: Resolved Leptos closure move ownership compile error in the authenticated user chip.
- **`b760907b` (2026-11-26)** — `docs: rename Settings to Preferences in ADRs`: Aligned all ADR terminology with the app-wide `Preferences` naming convention.
- **`e057269c` (2026-11-26)** — `ci: improve CI diagnostic logging and fix path handling` & **`1b35a5aa` (2026-11-26)** — Google Drive auth/session alignment in `.github/workflows/deploy.yml` and `docs/`.

---

## 3. Staged Work in Progress (Ready to Push to `exonomyapp/exonovel@main`)

Changes staged across `crates/app/src/lib.rs`, `crates/server/src/i18n.rs`, `style/main.css`, and `docs/session.md`:

1. **Top Hero Copy & 3-Row Display Alignment (`crates/app/src/lib.rs`, `style/main.css`, `crates/server/src/i18n.rs`)**:
   - Removed `"ExoNovel —"`, `"(https://exonovel.exosystems.net)"`, and `"text-based"` from the hero section.
   - Structured `<h1 class="exo-landing-title">` on three explicit rows (`Collaborative Creative Writing` / `Where Authors Own Their Work` at `2/3` scale, followed by `... by Design!` at full headline scale right-aligned flush with `"Work"` directly above it).
2. **4-Card Educational Panel Overhaul (`crates/app/src/lib.rs`, `style/main.css`, `crates/server/src/i18n.rs`)**:
   - Removed `"01"`–`"04"` numbering from tabs and card headers, removed `"2 / 4"` counter between arrows, placed looping `←` / `→` controls cleanly at the end of the tab strip on desktop, and replaced wrapping mobile tabs (`@media (max-width: 768px)`) with a single-row `<select>` card selector synced bidirectionally with the `←` / `→` controls.
   - Updated all 4 panels into approachable, explanatory two-paragraph educational copy covering:
     1. *Your Writing, Your Rights* (shared creative worlds, credited comments, private/group control, decentralized identity ownership).
     2. *One Producer, Many Personas* (content-proxy Personas, anonymous or open publishing/chatting, isolated Persona workspaces under one independent P2P identity).
     3. *Write, Wherever You Are* (free browser + free native desktop/mobile apps, on-demand manual sync vs. automatic sync for a small monthly or smaller annual fee).
     4. *Connect Your Way* (AOTA open connections without lock-in).
3. **New 3-Mode Section Below Formats (`crates/app/src/lib.rs`, `crates/server/src/i18n.rs`)**:
   - Added `#modes` (`Choose How You Write and Sync`) below `#formats` (`Pick Your Format. Start Creating.`) presenting the **Free** (`Web & Offline Device Apps`), **Monthly** (`$9 / month` automatic encrypted sync), and **Annual** (`$90 / year` — `$7.50/month` effective rate) options in local currency equivalent, aligned with ADRs `002-data.md` / `004-pay.md` and Slide 3.
4. **Mobile Navbar Logo Sizing (+50% on Mobile Only, `style/main.css`)**:
   - Increased `@media (max-width: 768px) .navbar-logo` height by 50% (`30px -> 45px`, mobile dropdown sheet `top: 68px`), leaving desktop `.navbar-logo` (`clamp(64px, 7.2vw, 80px)`) untouched.
5. **Multilingual Sync (`crates/server/src/i18n.rs`)**:
   - Synchronized English (`_`), Polish (`pl`), and German (`de`) Fluent catalogs for all added and modified landing keys.

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

