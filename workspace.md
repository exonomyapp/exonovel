# workspace.md

Working conditions for this **AI Studio ExoNovel Workspace Container** (see `AGENTS.md` for behavioral rules).

## Conditions of Work

1. **Full ExoNovel Project Scope**:
   - Work prepared in this container for `exonomyapp/exonovel` covers the **entire ExoNovel project** (all Rust crates in `crates/`, stylesheets in `style/`, assets in `assets/`, CI/CD workflows in `.github/`, and all ADRs/docs in `docs/`), limited only by the project ADRs and the owner's instructions.

2. **Preview & Testing Model**:
   - **Never preview ExoNovel in AI Studio's app preview**: AI Studio's app preview hosts only the **Update `exonomyapp/exonovel`** sync console.
   - **All ExoNovel previews happen on the live system**: Changes are pushed/merged to `exonomyapp/exonovel`, built and deployed automatically via GitHub Actions, and verified on the deployed live system (`exonovel.exosystems.net`, which runs live for pre-production testing and is not yet in public production).

3. **Shared Workspace Console & Local-Only Runtime Boundaries**:
   - Keep secrets (`.env`, `.env.local`), local build caches (`node_modules/`, `dist/`, `target/`), and local chat media uploads (`public/assets/aistudio/`, `.aistudio/`, `.agents/`, `.neon/`) strictly inside the local container.
   - Maintain the shared **Update `exonomyapp/exonovel`** console (`server.ts`, `server/`, `src/`, `index.html`, `package.json`, `vite.config.ts`, `tsconfig.json`, `metadata.json`, `.env.example`, `AGENTS.md`, `workspace.md`) in `exonomyapp/exonovel` alongside the Rust codebase so collaborators working on `main` can import the repository into AI Studio and sync cleanly without interfering with Rust builds or production deployments.
   - Keep all sync actions (pulling from `main`, pushing to `main`, and deleting superseded remote files) optional and individually or batch-selectable in both directions.

4. **Direct Root GitHub Sync Console & Access Setup**:
   - Serve the **Update `exonomyapp/exonovel`** sync workspace directly at the preview root URL (`/`) without any ExoNovel product sign-in gate.
   - Keep `GITHUB_PAT` and `ADMIN_PASSCODE` strictly on the server (never exposed to browser bundles or logs).
   - Persist workspace passcode unlock via a 7-day `HttpOnly; Secure; SameSite=None; Partitioned` cookie and in-memory Bearer token so repeated unlocks are not required across reloads.

5. **Target Repository & Container Build Boundary**:
   - Target repository: `exonomyapp/exonovel` (`main` branch).
   - Container RAM limit (4 GB): never run `rustc` or `cargo` locally and ensure zero `target/` directories exist; verify TypeScript/Vite console changes locally (`npm run lint` and `npm run build`) and inspect Rust CI/deploy builds via `GET https://api.github.com/repos/exonomyapp/exonovel/commits/main/check-runs`.
