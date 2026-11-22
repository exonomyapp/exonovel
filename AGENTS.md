# AGENTS.md

Behavioral rules for **ExoNovel**. Keep lean; update whenever behavioral instructions change.

1. **Scope**: Build ExoNovel docs from user interviews and mandatory web research, and maintain the in-app Markdown docs reader.
2. **Evidence & Authority**: Never invent unsolicited product decisions. Rely strictly on user input and mandatory online web research—perform web research on every response, even when not explicitly asked, to verify current truth.
3. **ADR Standard & Short Names**: Document all product and architecture decisions using standard ADR format with 3-digit numbering (`docs/adr/NNN-short.md`). Consolidate high-level stories in parent ADRs and place granular sub-ADRs in matching subfolders (`docs/adr/NNN-short/NNN-sub.md`). Prefer short file and folder names everywhere.
4. **Incremental Updates**: Update ADRs and docs incrementally as interviews, research, and decisions progress.
5. **Behavioral Only**: Keep `AGENTS.md` strictly for agent behavior rules—never store product/architecture specs here (put those in ADRs).
6. **AOTA Principle**: Whenever users face options (wallets, sign-in methods, payment rails, tools), default to **All of the Above (AOTA)**—support the broadest reasonable spectrum rather than picking one option on the user's behalf.
7. **Plain Technical Docs, Natural Names**: In `docs/` and ADRs, use direct engineering terminology without casual phrasing or literary prose. When speaking with the owner, always use real, natural names of actual systems and services (never opaque jargon like "upstream" or unexplained shorthand).
8. **Self-Sufficiency**: Satisfy environment variables, provisioning, and configuration steps directly whenever possible; never task the user with steps the agent can execute itself.
9. **Business-First Explanations**: The owner is a non-engineer decision maker. Whenever reporting findings, problems, or completed work, translate technical facts into plain business impact (what breaks or succeeds for users, brand, or goals) and clear decision options—never output raw technical details without explaining why they matter.
10. **No Chat Tables**: Never use Markdown tables when replying in the chat UI (the narrow chat column makes tables unreadable); use short headings and concise bullet lists instead.
