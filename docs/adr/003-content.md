# 003. Content Models & Story Architecture Sharing

Date: 2026-11-19
Status: Proposed (open for deep-dive interviews)

## Context

Writers organize their thoughts differently depending on the publication format, yet collaborative creative writing in ExoNovel also centers on modular story building blocks that authors can expose, share, co-edit, and reuse across projects without losing authorship.

## High-Level Story

1. **Format-Native Authoring**: Support the top five publication formats (**Novels**, **Articles & Creative Nonfiction**, **Short Stories & Flash Fiction**, **Screenplays & Scripts**, **Poetry**), distinguishing each format's **planning/authoring structure** from its **published output structure**.
2. **Modular Story Architecture**: Treat **Characters**, **Plots**, **Timelines / Threads**, and **Themes** as first-class, individually shareable entities owned by a `did:peer`.
3. **Permissioned Forking**: Any author with explicit permission from the owning `did:peer` can **fork** a shared character, plot, timeline/thread, or theme into their own project under a new `owner_did`, preserving provenance back to the original creator.

## Sub-ADRs

- [001-formats](003-content/001-formats.md) — Top 5 publication formats & planning vs. published structures
- [002-forking](003-content/002-forking.md) — Story architecture parts (characters, plots, timelines, themes) & permissioned forking
