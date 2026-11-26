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
   - Push the workspace to `main` so GitHub Actions (`cargo check`, `cargo test`, `cargo leptos build --release`) builds and deploys the updated Rust app (app-wide `Preferences` rename across UI/code/`user_profiles`/docs, mobile single-row Avatar navbar + fixed viewport dropdown sheets, clean `ExoNovel` home header, active-persona filter + `Profile -> Personas` empty state, and scrubbed vendor-neutral UI/API/legal copy alongside existing EN/DE/PL UI & docs parity).
2. **Google OAuth Brand, Client & Redirect URI Verification**:
   - In Google Auth Platform -> Clients -> **`ExoNovel`**, verify Authorized redirect URI `https://exonovel.exosystems.net/api/oauth/google/callback`, confirm server Google OAuth credentials in the production environment, upload `public/assets/exonovel-google-logo.png` in Branding, and submit verification.
3. **Owner Review of Split CMS, Multilingual Docs, Preferences & Mobile UI**:
   - Pending owner live review on `https://exonovel.exosystems.net` across desktop and mobile viewports (mobile Avatar dropdown, language/notification viewport sheets, clean home header, active-persona filtering, `Tooling & Proposals` vs `AI Inference`, `"Draft ADR from Proposal"`, German/Polish UI & docs switching, and responsive Preferences/Profile/Docs breakpoints).
   - Confirm active headless CLI binary path and inference provider credentials in the production server environment so Proposal Note generation answers using active remote inference.

---

## 3. Open Product & Schema Interviews (Next Up)

1. **Content Models & Top 5 Formats (`003-content` / `003-001` / `003-002`)**: Finalize author component ownership, reusable story elements (characters, settings, plot beats), and permissioned forking rules ahead of `loro 1.18.0` CRDT co-editing (`002-005`).
2. **Author Paywalls & AOTA Multi-Currency Payments (`004-pay` / `004-002` / `004-003`)**: Specify paywall gating rules and Tier 1 rollout (`dLocal`, `Stripe Connect`, `Reown AppKit`) inside the Profile commercial hub.
3. **Offline `willow25` + `iroh` Native Packaging (`002-002` / `006-stack`)**: Sequence desktop (`Linux`, `macOS`, `Windows`) and Android **F-Droid (`foss` UnifiedPush-only)** builds ahead of Google Play and Apple Store submissions.
