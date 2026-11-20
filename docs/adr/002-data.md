# 002. Data, Persistence & Offline Sync Architecture

Date: 2026-11-19
Status: Accepted (Neon stack) / Proposed (iroh sync details)

## Context

ExoNovel spans a web application and native installations across **Linux, macOS, Windows, Android, and iOS**. Writers expect:
1. Durable cloud persistence, relational querying, and asset storage.
2. Google Docs-style live simultaneous co-editing.
3. Offline-capable local authoring and peer-to-peer sync on native desktop/mobile apps.

## High-Level Story

1. **Managed Cloud Persistence (Neon)**:
   - **Lakebase PostgreSQL**: Authoritative relational store for `did:peer` identities, SSO links, project metadata, story architecture parts, sharing/fork lineage, paywall entitlements, and docs access control.
   - **Neon Better Auth**: Managed authentication tables in `neon_auth` branching with the database.
   - **Neon Object Storage**: S3-compatible buckets (`assets`) branching alongside Postgres for document attachments, exports, and media.
2. **Relay & Native P2P Sync (`iroh`)**:
   - Use the latest **iroh** (`v1.4.x`) for NAT-traversing encrypted QUIC relays and direct peer connections by `EndpointId`.
   - Native apps (Linux, macOS, Windows, Android, iOS) embed an `iroh` node (`iroh-docs`, `iroh-blobs`, `iroh-gossip`, `swarm-discovery`) so writers can collaborate over LAN or offline and reconcile state automatically upon reconnection.

## Sub-ADRs

- [001-neon](002-data/001-neon.md) — Neon PostgreSQL, Better Auth & Object Storage wiring
- [002-iroh](002-data/002-iroh.md) — `iroh` relay & native offline P2P sync/collaboration
