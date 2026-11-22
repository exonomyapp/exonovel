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

1. **Blind Relay & Cryptographic Payload Confidentiality Across Mode 2 AND Mode 3 `willow25` Sync**:
   - Run **ExoNovel's free `iroh`** (`v1.4.x`) discovery & home relay zone for NAT traversal across decentralized clients. All direct QUIC connections and relay frames are end-to-end encrypted (Noise + `ChaCha20-Poly1305`) between sender and recipient `EndpointId`s.
   - **Why Client-Side Encryption Applies to Willow Synced Content Too**: Out of the box, **`willow25` + Meadowcap + Confidential Sync** protect transport, enforce path/subspace capabilities, and hide unshared path names via private interest overlap—**however**, a storage peer (such as ExoNovel's Mode 3 `willow25` sync node) *could* read plaintext entry bytes unless payloads are encrypted before entering Willow.
   - **ExoNovel Payload Confidentiality Rule**: All user works, Loro CRDT snapshots/deltas, chat messages, and media stored or relayed through ExoNovel's Mode 2 blind transit buffers **AND** Mode 3 managed `willow25` backup nodes are **padded, salted with nonces, and client-side AEAD encrypted (`XChaCha20-Poly1305` / MLS)** under workspace keys before leaving the author's device. Storage and relay peers hold only opaque ciphertext, `payload_digest`, byte lengths, and Meadowcap capabilities—keeping ExoNovel **100% blind** to user content. *(When a Mode 3 user chooses Web access, decryption runs inside their browser WebCrypto/WASM session with their unlocked key; only explicitly published/public works expose readable plaintext.)*
2. **Native Embedded Node**: Desktop (Linux, macOS, Windows) and Mobile (Android, iOS) installations embed a native `iroh` node (`iroh-docs` + `iroh-blobs` + `iroh-gossip`) so authors can draft offline, collaborate directly over LAN or P2P, and sync deltas cleanly when connectivity resumes.
3. **Web Bridge & Bidirectional Neon ↔ `willow25` Relocation**:
   - Web browsers without native QUIC sockets sync through ExoNovel's WebSocket/HTTPS bridge connected to Neon PostgreSQL using the exact same `loro` (`1.18.0`) CRDT model and `willow_*` path conventions.
   - Native builds (**Linux, macOS, Windows, Android, iOS**) start and operate cleanly **without Neon**. At onboarding/sign-up, native users choose between **Local-Only Device Mode (`willow25`)** and **Cloud + Local Sync (`Neon + willow25`)**.
   - Users who "flip the local switch" to pure Willow can later **import their local `willow25` store back into Neon** via idempotent coordinate/frontier reconciliation if they resume using the web app.

## Resolved & Remaining Decisions

- **CRDT Selection (Resolved)**: **`loro` (`1.18.0`)** selected (see [005-loro](005-loro.md)).
- **Relay Hosting (Open)**: Should ExoNovel host dedicated regional `iroh` relays, allow authors to point to custom/self-hosted relays, or both (AOTA)?
