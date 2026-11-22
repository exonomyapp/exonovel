# 002-005. Loro CRDT Real-Time Co-Editing & History Architecture

Date: 2026-11-21
Status: Accepted (Selected by owner; implementation staged alongside `003-content`)
Parent: [002-data](../002-data.md)

## Context

Following [002-003](003-crdt-report.md), the owner selected **Loro (`loro` `1.18.0`)** as ExoNovel's real-time simultaneous collaboration and offline-merging CRDT engine across Web (Rust → WASM in Leptos), Desktop (Linux, macOS, Windows), and Mobile (Android, iOS).

## Decision: Loro Features Used & Stack Integration

### 1. Core Loro CRDT Primitives Mapped to ExoNovel Objects
- **`LoroTree` (Movable Tree CRDT)**:
  - Represents each Book's structural hierarchy (**`Book → Parts → Chapters → Scenes`**) and **Story Bible** folder/category trees.
  - Multiple co-authors working online or offline can simultaneously drag, reorder, split, nest, or move chapters and scenes between Parts with guaranteed convergence and zero duplicate nodes or cyclic parent loops.
- **`LoroText` (Fugue + Peritext-Style Rich Text)**:
  - Represents chapter/scene manuscript prose, dialogue beats, and screenplay descriptions.
  - Supports concurrent character-level edits alongside rich-text marks (bold, italics, underline, links, inline comment anchors, and revision spans) with deterministic styling resolution.
- **`LoroMap`, `LoroList`, and `LoroMovableList`**:
  - Models structured entity fields (character traits, timeline coordinates, scene POV/location tags, language tags, inline comment threads, and contributor attributions).
  - Leverages `loro 1.18.0` safety checks (`ConcurrentOpsWithSamePeerID` prevention and `LoroMovableList` element tracking across snapshots/checkouts) so offline merges never silently fork history.
- **DAG Version History & Time Travel (`Frontiers` / `VersionVector` / `checkout`)**:
  - Records causal edit graphs natively so authors can browse draft history, inspect who wrote which passage, rewind to any historical frontier, or branch/fork a Story Architecture work cleanly.
- **`EphemeralStore` (Transient Presence)**:
  - Broadcasts live collaborator cursors, text selections, active scene focus, and typing indicators **without** bloating permanent CRDT document history or Willow storage.

### 2. Dual Persistence & Transport Across Cloud (Neon) and Native (`willow25` + `iroh`)
- **In Neon PostgreSQL (Centralized Web + Cloud Backup)**:
  - `crdt_updates`: Stores signed incremental Loro binary deltas keyed by `(doc_id, peer_did, frontiers_hash, willow_* coordinates)`.
  - `crdt_snapshots`: Stores periodic compacted Loro snapshots (`shallow` or full state snapshots) for fast initial document hydration in browsers.
- **Live Presence (`EphemeralStore`) & Delta Transport — Both `iroh-gossip` AND WebSockets**:
  - **Native Peers (Desktop/Mobile)**: Stream Loro incremental deltas and `EphemeralStore` cursor frames directly over **`iroh-gossip`** (`v0.102.0`, QUIC/UDP P2P and LAN).
  - **Web Browsers (WASM Leptos)**: Because browsers cannot open raw UDP sockets, web clients connect via **secure WebSockets (`wss://`)** through **ExoNovel's `iroh` WebSocket relay/bridge** (supporting both `iroh` WASM over WebSocket and an Axum server WebSocket room bridge), translating seamlessly into the exact same **`iroh-gossip`** topic so browser and native co-authors share one unified live room.
  - **Offline Sync & Relocatable Storage**: Compacted Loro snapshots and delta ranges map directly to `willow25` paths (`/works/{work_id}/crdt/snapshots/...` and `/deltas/...`) addressed by SHA-256 `payload_digest` and synchronized over **`iroh-docs` / `iroh-blobs`**.
  - **Bidirectional Relocation (Neon ↔ Willow)**: Because Loro state merges solely by causal DAG updates and deterministic snapshots, a user can flip cleanly to local-only `willow25` and later re-import their local Willow store back into Neon PostgreSQL without conflicts or duplicate revisions.

### 3. WASM Bundle & Server Alignment
- Both `crates/server` (Axum native) and `crates/frontend` (Leptos WASM) run the exact same `loro` Rust crate (`1.18.0`), guaranteeing identical binary encoding and merge semantics across every client and server.
