# 002-002. `iroh` Relay & Native Offline P2P Sync

Date: 2026-11-19
Status: Proposed (accepted direction; details open for interview)
Parent: [002-data](../002-data.md)

## Context

ExoNovel deploys across Web plus five native platforms (Linux, macOS, Windows, Android, iOS) and requires offline syncing, live collaboration, and a privacy-preserving relay layer.

## Research Findings (`iroh` Late-2026 State)

- **Core Release**: `iroh` **v1.4.1** (post-1.0 stable), QUIC-based P2P networking where peers dial directly by cryptographic `EndpointId`.
- **Stateless Encrypted Home Relays**: Peers connect to a home relay for hole-punching and NAT traversal; once a direct QUIC path is established, traffic flows peer-to-peer. Relays see only end-to-end encrypted packets.
- **Offline & Local Sync**:
  - `swarm-discovery` provides built-in mDNS LAN discovery so nearby writers sync with zero internet access.
  - `iroh-docs` synchronizes key-value *Replicas* via **range-based set reconciliation**, backed by `iroh-blobs` for content-addressed payloads.
  - `iroh-gossip` (**v0.102.0**) provides epidemic broadcast trees (*HyParView* / *PlumTree*) over ALPN for live co-editing presence and CRDT delta broadcasts.

## Decision

1. **Relay Infrastructure**: Run **iroh** (`v1.4.x`) home relay servers for encrypted NAT traversal and peer routing across all clients.
2. **Native Embedded Node**: Desktop (Linux, macOS, Windows) and Mobile (Android, iOS) installations embed a native `iroh` node (`iroh-docs` + `iroh-blobs` + `iroh-gossip`) so authors can draft offline, collaborate directly over LAN or P2P, and sync deltas cleanly when connectivity resumes.
3. **Web Bridge**: Web browsers without native QUIC sockets sync through ExoNovel's WebSocket/HTTPS bridge (`server.ts` / Neon) that participates in the same CRDT/document model.

## Open Interview Questions

- Which CRDT library (e.g., Automerge, Loro, Yjs) should sit inside `iroh-docs`/`iroh-blobs` for simultaneous co-editing?
- Should ExoNovel host dedicated regional `iroh` relays or allow authors to point to custom/self-hosted relays?
