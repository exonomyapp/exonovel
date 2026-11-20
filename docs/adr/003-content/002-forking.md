# 003-002. Story Architecture Parts & Permissioned Forking

Date: 2026-11-19
Status: Proposed (open for interview)
Parent: [003-content](../003-content.md)

## Context

Sharing in ExoNovel centers on discrete dimensions of a story's architecture so other authors can discover them—for example, taking interest in a character's biography—and include or adapt them in their own works.

## Decision

1. **First-Class Shareable Parts**:
   - **Characters**: Biographies, traits, arcs, relationship graphs.
   - **Plots**: Beat sequences, plotlines, structural outlines.
   - **Timelines / Threads**: Chronological events, narrative threads, continuity timelines.
   - **Themes**: Core motifs, thematic statements, symbolic arcs.
2. **Google Docs-Style Sharing & Live Co-Editing**:
   - Standard sharing controls (invite collaborators, link access, role-based view/comment/edit) plus live simultaneous co-editing.
3. **Permissioned Forking to a New Owner**:
   - Each shareable part carries an explicit `allow_fork` / fork-permission policy set by its `owner_did`.
   - When permitted, another author may **fork** the part into their own project:
     - A new independent copy is created with `owner_did` set to the forking author's `did:peer`.
     - `forked_from_id` and `origin_owner_did` remain recorded for provenance/attribution.

## Open Interview Questions

- What exact permission tiers should exist (Viewer, Commenter, Editor, Fork-Allowed)?
- Should forked parts stay decoupled from upstream edits, or support optional "pull upstream updates" notifications?
- How should attribution / royalties (if paywalled) flow when a forked character appears in a commercial novel?
