# 005-001. Markdown Reader & Statefully Collapsible/Resizable Left Nav

Date: 2026-11-19
Status: Accepted
Parent: [005-docs-ui](../005-docs-ui.md)

## Decision

1. **Left-Nav Docs Explorer**:
   - Displays `docs/` hierarchy (parent ADRs paired with sub-ADR folders) and `AGENTS.md`.
   - **Statefully Collapsible**: Desktop header toggle collapses/expands the sidebar and persists state in `localStorage`.
   - **Statefully Resizable**: Right-edge pointer drag handle (minimum `210px`, no maximum width constraint, keyboard `ArrowLeft`/`ArrowRight`, double-click reset) persists width in `localStorage`.
   - Folder expand/collapse states persist in `localStorage`, and live search filters by file name, title, and Markdown headings.
2. **Markdown Content Area**:
   - Full-width content area reaching the right edge of the viewport without artificial horizontal caps or empty right-side margins.
   - Full GitHub Flavored Markdown (tables, task checklists, blockquotes, fenced code blocks with copy, internal `.md` navigation, heading anchors).
   - Sticky right-rail **On this page** outline, light/dark theme toggle, and live polling against `/api/docs` and `/api/doc`.
3. **Google Docs & Drive Workspace**:
   - Inside **Preferences -> Connected Apps & Google Docs** (accessible to all signed-in accounts) and in the **Docs & ADRs** toolbar (for Docs-authorized users), ExoNovel separates optional Google Docs/Drive authorization (`drive.file` + `userinfo.email` with automatic `login_hint` from Neon Auth) from primary identity sign-in (`openid email profile`).
   - Users can connect or revoke Google tokens anytime (`/api/oauth/google/start`, `/api/oauth/google/callback`, `/api/oauth/google/revoke`), create editable native Google Docs (`application/vnd.google-apps.document`) converted from Markdown, browse/search/preview/rename/trash/restore/delete Drive files, save individual ADRs to Google Docs or `.md`, and run a full **`ExoNovel Docs`** folder archive backup preserving the `docs/` hierarchy.
   - All destructive or disconnect actions use inline confirmation UI controls (`window.alert` / `window.confirm` forbidden).
