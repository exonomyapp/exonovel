# 011. Writer's Desk Workspace Proposal (Demo Placeholders & Planned Modules)

Date: 2026-11-28
Status: Proposed (interactive demo scaffold in `crates/app/src/lib.rs` :: `WorkspaceHomeView`; awaiting owner review & interview)
Related: [003-content](003-content.md), [003-001](003-content/001-formats.md), [003-002](003-content/002-forking.md), [007-i18n](007-i18n.md), [010-dear-world](010-dear-world.md)

## Context

When signed in, ExoNovel replaces the generic `"Home"` label with **`Desk`** (`ActiveTab::Workspace`). Per owner direction:
1. Remove the old four shortcut buttons (`Manage Profile`, `Manage Preferences`, `Open Docs & ADRs`, `Open Dear World`) because those destinations already live in the navbar and user chip menu.
2. Remove the administrative top metadata rows (`ExoNovel Admin · Plan: free`, full name/pen-name line) from the Desk header.
3. Merge the former top welcome card and secondary workspace panels into **one unified Writer's Desk**, displaying only a truncated `did:peer` chip + copy button in the top-right corner.
4. Speak warmly and non-technically to **High School and University writers** about the stories and writing work being done rather than platform internals or sync tiers.
5. Present concrete **workspace placeholders** previewing planned ExoNovel features so the owner can review and shape the Desk before database tables and editors are built.

Research on collaborative fiction workspaces (Scrivener's Binder/Corkboard/Scratchpad paired with real-time CRDT co-authoring) highlights three core needs on a writer's home surface: starting a structured draft quickly, accessing modular story pieces (characters, plots, timelines, themes), and capturing unformed ideas in a low-friction scratchpad before deciding where they belong.

## Proposed Architecture & Current Demo Implementation

The current demo scaffold is implemented in `WorkspaceHomeView` (`crates/app/src/lib.rs`, lines 1125–1322) and styled via `.exo-desk-*` (`style/main.css`, lines 1286–1376). Today it runs purely in client memory (no PostgreSQL project/note tables yet) so the owner can see and test the proposed layout:

### 1. Desk Header & Identity Chip (`lib.rs` lines 1171–1193)
- **Layout**: Left column shows the `Desk` heading and a plain-language introduction written for High School and University writers. Top-right shows the truncated `did:peer` preview (`short_did_preview`) with a one-click copy icon and full `did:peer` hover tooltip.
- **Status**: Functional (reads the active persona's `did:peer` from `/api/me`).

### 2. Placeholder Block A — Format Starter Shelf ("Start a New Project", `lib.rs` lines 1196–1231)
- **What it is**: Five selectable format tiles matching [003-001](003-content/001-formats.md):
  - **Novels**
  - **Articles & Nonfiction**
  - **Short Stories**
  - **Screenplays & Scripts**
  - **Poetry**
- **What the demo does today**: Clicking any tile highlights it and updates a preview bar underneath showing the starter folders/sections that format will open with (e.g., *Story Outline · Character Cards · Chapter 1 Draft · World Timeline* for Novels).
- **Planned backend wiring**: Selecting a format and clicking a future **Create Project** action creates a project container in PostgreSQL seeded with that format's planning structure and `loro` CRDT document.

### 3. Placeholder Block B — Story Building Blocks Shelf ("Your Shelf of Story Pieces", `lib.rs` lines 1233–1254)
- **What it is**: Four cards corresponding directly to the modular, reusable, permissioned-forkable entities defined in [003-002](003-content/002-forking.md):
  - **Characters** (heroes, rivals, sidekicks, biographies, shared cast across friends' stories)
  - **Plots** (turning points, chapter beats, scene order)
  - **Timelines** (chronological events across multi-part stories or group projects)
  - **Themes** (core questions, motifs, and mood shared across co-writers)
- **What the demo does today**: Static 2x2 descriptive cards introducing the four building-block categories.
- **Planned backend wiring**: Live counters and direct launchers into each writer's personal and shared entity libraries (plus incoming fork/remix invitations).

### 4. Placeholder Block C — Quick Idea Scratchpad ("Jot Down a Quick Idea", `lib.rs` lines 1256–1315)
- **What it is**: An always-ready, zero-setup capture box modeled after Scrivener's Scratchpad:
  - Integrated **`InputLocaleBar`** ([007-i18n](007-i18n.md)) so writers can switch writing languages and RTL/LTR direction right at the pad.
  - **Working title or prompt** text input + **Opening lines or scene sketch** textarea.
  - **Pin Idea on Desk** and **Clear** buttons.
- **What the demo does today**: Updates in-memory Leptos signals (`scratch_title`, `scratch_body`, `scratch_saved`) and displays `"Idea pinned on your desk for this session"`. Notes disappear on page reload because PostgreSQL persistence is not wired yet.
- **Planned backend wiring**: Persist scratchpad notes in PostgreSQL (`desk_notes`) under the author's `did:peer`, with a future **"Promote to Project / Scene / Character"** action.

## Open Interview Questions for Owner Approval

Before turning these placeholders into PostgreSQL tables and active workflows, the owner must confirm:
1. **Keep, Trim, or Reorder Blocks**: Do all three sections (*5-Format Starter Shelf*, *4 Story Building Blocks*, *Quick Idea Scratchpad*) match your vision for the Desk, or should any block be removed, renamed, or replaced (for example, adding a *Recent Stories & Co-Writer Invites* list above the starters)?
2. **Format Starter Behavior**: When a writer clicks one of the 5 formats, should it immediately open a "New Project" title prompt, or should the Desk keep a compact list of existing drafts first and tuck "New Project" behind a button?
3. **Scratchpad Persistence**: Should **Pin Idea on Desk** save a list of sticky notes in PostgreSQL (`desk_notes`) across devices, or should the Desk instead open directly into a real mini-editor / recent draft?
4. **Building Blocks Visibility on Desk**: Should **Characters, Plots, Timelines, Themes** live as top-level cards on the Desk, or inside each individual story project's Binder?
