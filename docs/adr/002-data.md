# 002. Data, Persistence & Offline Sync Architecture

Date: 2026-11-19
Status: Accepted (Neon stack) / Proposed (iroh sync details)

## Context

ExoNovel spans a web application and native installations across **Linux, macOS, Windows, Android, and iOS**. Writers expect:
1. Durable cloud persistence, relational querying, and asset storage.
2. Google Docs-style live simultaneous co-editing.
3. Offline-capable local authoring and peer-to-peer sync on native desktop/mobile apps.

## High-Level Story

1. **Managed Cloud Persistence with Willow-Ready Portability (Neon)**:
   - **PostgreSQL**: Authoritative centralized relational store for `did:peer` identities, `user_personas`, `user_profiles`, `media_assets`, `groups`, `group_members`, `invites`, `referrals`, SSO links, project metadata, story architecture parts, and multi-level access control.
   - **Willow Protocol Coordinates**: Centralized tables store deterministic Willow coordinates (`willow_namespace`, `willow_subspace`, `willow_path`, `willow_timestamp_us`, plus SHA-256 `payload_digest` and `byte_length` on binary payloads) under each owner's `did:peer` subspace so users can relocate their complete data set into a local Rust Willow store for mobile/offline collaboration.
   - **Multi-Size Media Assets**: Image uploads up to 2 MB are decoded server-side into UI-appropriate sizes (`thumb` 64×64, `medium` 256×256, `large` <= 1024px WebP/JPEG) stored with content digests so navigation chips and lists never load 2 MB originals.
   - **Neon Better Auth**: Managed authentication tables in `neon_auth` branching with the database.
2. **Web, Device, Collaboration Sync vs. Personal Device Sync (`did:peer` Portability)**:
   - **On the Web (`Free`)**: Cloud-backed web workspace powered by PostgreSQL, accessible from any browser.
   - **On Your Device (`Free`)**: Native **Linux, macOS, Windows, Android, and iOS** builds boot locally with **zero cloud database requirement** using **`willow25`** (`0.7.11`) and **ExoNovel's free `iroh` (`v1.4.x`) discovery & relay zone**. **Blind Relay Mandate**: All content relayed or transiently buffered through ExoNovel's relay servers on its way to other peers is end-to-end encrypted on the sender's device so ExoNovel remains completely blind to any payload in transit.
   - **Free Automatic Collaboration Sync Across Web & Device (`Free for Co-Writers`)**: Whenever an author adds collaborators, synchronization between co-writers happens automatically so they can share and co-author work together—regardless of whether one writer uses a web browser and another works on a desktop or mobile device install.
   - **Personal Device Sync Plan (`Paid: local-currency equivalent of $9/month or $90/year`)**:
     - Covers **only** keeping an individual user's own offline device drafts continuously synced with that same user's own web profile.
     - Activated and billed via **Profile → Plans** (configured with **Profile → Wallet**, with ExoNovel acting as a peer using its own **`did:peer` ID** and AOTA wallets to accept local currencies).
     - Registers the user's `did:peer` IDs with **ExoNovel's managed `willow25` sync service** via finalized **Meadowcap capability delegation** inside `willow25`—backing up client-side AEAD-encrypted payloads (so ExoNovel stays **cryptographically blind** to private content) and synchronizing personal Web ↔ Device shelves seamlessly.
     - Users can still export (`PostgreSQL → willow25`) or re-import (`willow25 → PostgreSQL`) deterministically via `willow_*` coordinates and causal **`loro` (`1.18.0`)** CRDT frontiers.

## Sub-ADRs

- [001-neon](002-data/001-neon.md) — Neon PostgreSQL, Better Auth & Media Storage wiring
- [002-iroh](002-data/002-iroh.md) — `iroh` relay & native offline P2P sync/collaboration
- [003-crdt-report](002-data/003-crdt-report.md) — Decision Report: Global Open-Source Real-Time Co-Editing Engines (`Loro` Selected)
- [004-db-prefs](002-data/004-db-prefs.md) — Eliminating Redundant Browser `localStorage` in Favor of Neon PostgreSQL `user_profiles`
- [005-loro](002-data/005-loro.md) — Loro CRDT Real-Time Co-Editing, Movable Trees & History Architecture
