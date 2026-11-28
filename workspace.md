# workspace.md

Working conditions for this **AI Studio ExoNovel Workspace Container** (see `AGENTS.md` for behavioral rules).

## Conditions of Work

1. **Full ExoNovel Project Scope**:
   - Work prepared in this container for `exonomyapp/exonovel` covers the **entire ExoNovel project** (all Rust crates in `crates/`, stylesheets in `style/`, assets in `assets/`, CI/CD workflows in `.github/`, and all ADRs/docs in `docs/`), limited only by the project ADRs and the owner's instructions.

2. **Preview & Testing Model**:
   - **Never preview ExoNovel in AI Studio's app preview**: AI Studio's app preview hosts only the segregated **Sync Console** (`sync-console/`) served as the root workspace UI inside AI Studio and blocked on external public hosts.
   - **All ExoNovel previews happen on the live system**: Changes are pushed/merged to `exonomyapp/exonovel`, built and deployed automatically via GitHub Actions, and verified on the deployed live system (`https://exonovel.exosystems.net`, which runs live for pre-production testing and is not yet in public production).

3. **Segregated Sync Console & Local-Only Runtime Boundaries**:
   - Keep secrets (`.env`, `.env.local`), pending commit intent notes (`.local/pending-commit.json`), local build caches (`node_modules/`, `dist/`, `target/`), and local chat/runtime folders (`public/assets/aistudio/`, `.aistudio/`, `.neon/`, and `.agents/` internals outside `.agents/skills/`) strictly inside the local container. Sync `.agents/skills/` across workspaces so shared project skills stay aligned.
   - Maintain the segregated **Sync Console** codebase inside `sync-console/` alongside the root launcher/config files (`server.ts`, `index.html`, `package.json`, `vite.config.ts`, `tsconfig.json`, `metadata.json`, `.env.example`, `AGENTS.md`, `workspace.md`) in `exonomyapp/exonovel` cleanly separated from the ExoNovel Rust application (`crates/`, `style/`, `assets/`).
   - Protect remote-managed lockfiles (`Cargo.lock`) from accidental deletion or clutter, and keep all sync actions (pulling from `main`, pushing to `main`, creating PRs with Standard Merge, and deleting superseded remote files) optional and individually or batch-selectable.

4. **Direct Root GitHub Sync Console & Access Setup**:
   - Serve the **Sync Console** directly at the AI Studio preview root URL (`/`) while rejecting non-AI-Studio public hosts with HTTP `403`.
   - Keep `GITHUB_PAT` and `ADMIN_PASSCODE` strictly on the server (never exposed to browser bundles or logs).
   - Persist workspace passcode unlock via a 7-day `HttpOnly; Secure; SameSite=None; Partitioned` cookie and in-memory Bearer token with `scrypt` verification, lockout after 5 failed attempts, and `X-Requested-With` + Origin enforcement on mutations.

5. **Target Repository & Container Build Boundary**:
   - Target repository: `exonomyapp/exonovel` (`main` branch).
   - Container RAM limit (4 GB): never run `rustc` or `cargo` locally and ensure zero `target/` directories exist; verify TypeScript/Vite console changes locally (`npm run lint` and `npm run build`) and inspect Rust CI/deploy builds via `GET https://api.github.com/repos/exonomyapp/exonovel/commits/main/check-runs` and live site probes (`https://exonovel.exosystems.net/` + `/api/health`).
