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
