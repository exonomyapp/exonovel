# ExoNovel — Unfinished Work in Progress (`session.md`)

> **Rule for This File**: `session.md` tracks **strictly unfinished work in progress and open roadmap items**. Completed work is distilled into the relevant ADRs (`docs/adr/`) or `docs/adr/006-stack/001-releases.md` and pruned from here.

---

## 1. Active Architectural & Build Guardrails (Governing Ongoing Work)

- **Sovereign, Independent Product (`exonovel.exosystems.net`)**: Production runs solely as compiled Rust (`crates/server`) + browser WASM (`crates/frontend`) behind systemd (`127.0.0.1:3001`), separate from `exosystems.net`.
- **Author Sovereignty & Willow Coordinate Parity**: Keep author `did:peer` identities, personas, and paywalled earnings author-owned, and keep PostgreSQL schemas aligned with `willow_*` coordinates for lossless `willow25` + `iroh` P2P portability (`002-data/002-iroh.md`).
- **4 GB Preview Container Build Boundary**: Audit Rust changes statically locally (never bootstrap `rustc` locally) and verify full Rust/WASM compilation in GitHub Actions (`AGENTS.md` Rule 12).

---

## 2. Unfinished Work in Progress (Immediate Queue)

1. **GitHub Actions Full Rust/WASM Build & Production Deploy (`https://exonovel.exosystems.net`)**:
   - Push the workspace to `main` so GitHub Actions (`cargo check`, `cargo test`, `cargo leptos build --release`) builds and deploys the full Rust app (complete EN/DE/PL UI localization across `settings.rs`, `profile.rs`, `connected_apps_ui.rs`, `cms.rs`, `lib.rs`, `server/src/i18n.rs`, multilingual `AGENTS.md` in `de`/`pl`, single `docs/session.md`, hierarchical MADR 4.0.0 tree & split CMS views, `adr_translations`, public homepage carousel, `/privacy`, `/terms`, and mobile styles).
2. **Google OAuth Brand, Client & Redirect URI Verification**:
   - In Google Cloud Console -> Google Auth Platform -> Clients -> **`ExoNovel`** (`902812495809-jspg...`), verify Authorized redirect URI `https://exonovel.exosystems.net/api/oauth/google/callback`, confirm `GOOGLE_OAUTH_CLIENT_ID` / `GOOGLE_OAUTH_CLIENT_SECRET` in `/opt/exonovel/env`, upload `public/assets/exonovel-google-logo.png` in Branding, and re-submit Google verification.
3. **Owner Review of Split CMS, Multilingual Docs & Mobile UI**:
   - Pending owner live review on `https://exonovel.exosystems.net` across desktop and mobile viewports (`Tooling & Proposals` vs `AI Inference`, `"Draft ADR from Proposal"` workflow, German/Polish UI & docs switching including `AGENTS.md`, and responsive navigation/Settings/Profile/Docs breakpoints).
   - Confirm `opencode` CLI binary path and active `NVIDIA_NIM_API_KEY` (or `NVIDIA_API_KEY`) in production `/opt/exonovel/env` so Proposal Note generation answers using active remote inference.

---

## 3. Open Product & Schema Interviews (Next Up)

1. **Content Models & Top 5 Formats (`003-content` / `003-001` / `003-002`)**: Finalize author component ownership, reusable story elements (characters, settings, plot beats), and permissioned forking rules ahead of `loro 1.18.0` CRDT co-editing (`002-005`).
2. **Author Paywalls & AOTA Multi-Currency Payments (`004-pay` / `004-002` / `004-003`)**: Specify paywall gating rules and Tier 1 rollout (`dLocal`, `Stripe Connect`, `Reown AppKit`) inside the Profile commercial hub.
3. **Offline `willow25` + `iroh` Native Packaging (`002-002` / `006-stack`)**: Sequence desktop (`Linux`, `macOS`, `Windows`) and Android **F-Droid (`foss` UnifiedPush-only)** builds ahead of Google Play and Apple Store submissions.
