# 002-003. Decision Report: Global Real-Time Co-Editing Engines (CRDT vs. OT)

Date: 2026-11-21
Status: Accepted (Option A — `loro 1.18.0` selected by owner; see [005-loro](005-loro.md))
Parent: [002-data](../002-data.md)

## Purpose of This Report

ExoNovel needs Google Docs-style simultaneous multi-author editing across Web (WASM/Leptos), Desktop (Linux, macOS, Windows), and Mobile (Android, iOS), **plus** offline mobile editing that merges cleanly later and aligns with Rust + **Willow Protocol (`willow25`)**. This report surveys the leading open-source real-time collaboration engines from around the world so you can choose a foundation with clear visibility into performance, features, maturity, and risks.

---

## Evaluated Open-Source Contenders (Global Market)

### 1. Loro (`loro` — China-Founded Global Open Source, Rust + WASM)
- **Origin & Version**: Created and maintained by the Loro team (founded in China, MIT-licensed open source globally). Current verified Rust crate: **`1.18.0`**.
- **How It Works**: Rust-native core compiled to WebAssembly for browsers and native code for Desktop/Mobile. Uses the **Fugue** rich-text CRDT algorithm plus a versioned Directed Acyclic Graph (DAG) history.
- **Unique Strengths for ExoNovel**:
  - **First-class Movable Trees**: Native CRDT support for moving chapters between Parts, reordering scenes, or reorganizing Story Bible folders concurrently without duplicating items—something pure-text CRDTs struggle with cleanly.
  - **Built-in Time Travel & Versioning**: Authors can inspect historical snapshots, undo per-author cleanly, and branch/merge drafts natively.
  - **Rich Text (Peritext-inspired)**: Handles bold/italics/comments/links across concurrent edits cleanly.
  - **Fast Binary Snapshots & Deltas**: Compact payloads suited to syncing inside **`iroh-blobs` / `willow25`** paths.
- **Trade-offs**: Larger WASM binary (~150–300 kB gzipped vs ~18 kB for Yjs) and a younger editor-plugin ecosystem than Yjs (though ProseMirror/CodeMirror bindings exist).

### 2. Yrs / Yjs (`yrs` — Germany-Founded Open Source, Rust + JS/WASM)
- **Origin & Version**: Created by Kevin Jahns (Germany, MIT license). `yrs` is the official Rust port of **Yjs**. Current verified Rust crate: **`0.28.1`**.
- **How It Works**: Uses the **YATA** sequence CRDT algorithm with aggressive tombstone garbage collection to keep long-lived memory small.
- **Unique Strengths for ExoNovel**:
  - **Most Battle-Tested Editor Ecosystem**: Ready-made integrations for **TipTap, ProseMirror, Lexical, CodeMirror, Quill, and Monaco** across thousands of production apps.
  - **Tiny Web Footprint**: Roughly ~18 kB (JS wrapper) for browsers, fast page loads on slow mobile connections.
  - **Official Rust Server (`yrs`)**: Allows ExoNovel's Rust/Axum server and native desktop/mobile apps to merge edits without running Node.js.
- **Trade-offs**: Does **not** have a native Movable Tree CRDT (moving chapters/folders concurrently requires extra application locking/IDs) and prunes full edit history by default unless snapshots are stored separately.

### 3. Automerge (`automerge` — UK / Ink & Switch, Rust + WASM)
- **Origin & Version**: Pioneered by Martin Kleppmann and Ink & Switch (UK/international research lab, MIT license). Current verified Rust crate: **`0.12.0`**.
- **How It Works**: Rust JSON-document CRDT recording a full, Git-like immutable change graph. Designed specifically alongside local-first research (`automerge-repo`).
- **Unique Strengths for ExoNovel**:
  - **Git-Like Branching & Audit Trails**: Strong fit if permissioned forking (`003-content/002-forking`) and full cryptographic provenance per edit are top priority.
  - **Clean Sync Protocol (`automerge-repo`)**: Built-in delta sync over WebSockets and peer channels.
- **Trade-offs**: Historically higher memory and slower per-keystroke speed than Loro or Diamond Types on massive single-file novels unless batched cleanly into scenes/chapters; no built-in movable-tree primitive.

### 4. Diamond Types (`diamond-types` — Australia, Rust)
- **Origin & Version**: Created by Joseph "Seph" Gentle (Australia, ISC license; co-creator of ShareDB).
- **How It Works**: Range-tree / modified B-tree Rust engine delivering extreme raw speed—replaying millions of keystrokes in milliseconds.
- **Unique Strengths for ExoNovel**: Fastest pure plain-text throughput in Rust.
- **Trade-offs**: Focused strictly on **plain-text** sequences—no rich-text formatting marks, maps, or movable story trees out of the box. Best reserved for plain Markdown/Fountain screenplay text if rich formatting were unnecessary.

### 5. Cola (`cola` — Open-Source Rust Text CRDT)
- **How It Works**: Minimalist Rust plain-text sequence CRDT optimized for raw insertion/deletion speed in code/text editors.
- **Trade-offs**: Plain text only; lacks maps, trees, and rich-text annotations needed for Novels and Story Bibles.

### 6. Centralized Operational Transform (`ShareDB` / Server-Sequenced OT)
- **How It Works**: Google Docs' classic model—edits always flow through a central coordinating server that transforms conflicting cursor positions.
- **Trade-offs**: Fails ExoNovel's requirement for offline-first mobile authoring and decentralized **Willow / `iroh`** synchronization cleanly, and standard implementations rely heavily on Node.js rather than Rust.

---

## Comparison Summary Against ExoNovel Requirements

- **Can compile natively in our Rust server + WASM Leptos frontend?**: Loro, Yrs, Automerge, Diamond Types, and Cola are all Rust-native.
- **Handles Novels (movable Parts/Chapters/Scenes tree + rich text + Story Bible objects)?**: **Loro** stands out with native movable trees + rich text + time travel; **Yrs** relies on external app logic for tree moves but dominates rich-text editor plugins; **Automerge** models nested JSON objects with Git history cleanly.
- **Fits `willow25` / `iroh` offline sync?**: All Rust CRDTs produce binary snapshot/delta payloads that can be addressed by SHA-256 digest in Willow paths and exchanged over `iroh`.

## Proposed Decision Options (Pending Your Selection)

- **Option A — Loro (`1.18.0`)**: Recommended if **chapter/scene tree reordering, rich-text writing, built-in draft time travel, and compact Rust+WASM execution** matter most (matches ExoNovel's complex `Parts -> Chapters -> Scenes` and Story Bible hierarchy directly).
- **Option B — Yrs (`0.28.1`)**: Recommended if **fastest web bundle size and off-the-shelf web rich-text editor plugins (ProseMirror / TipTap / CodeMirror)** matter most, with ExoNovel implementing chapter-order moves explicitly.
- **Option C — Automerge (`0.12.0`)**: Recommended if **full immutable Git-like commit history per keystroke for provenance/forking** outweighs raw large-document throughput.
- **Option D — Hybrid (`Loro` or `Yrs` for Manuscripts + `Fountain` plain-text for Screenplays)**: Keep manuscript trees/rich prose on `Loro` or `Yrs`, while plain-text formats remain portable Markdown/Fountain exports.
