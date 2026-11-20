# 001-002. Itemized NextGraph Auth Flow & Technology Checklist

Date: 2026-11-19
Status: Accepted (Keep all 14 NextGraph steps for now)
Parent: [001-auth](../001-auth.md)

## Context

This sub-ADR breaks down every step and technology in **NextGraph's** authentication and wallet story (`nextgraph-rs` / `ng-wallet`). Per your decision, we **keep all 14 steps/technologies** in scope for now while complementing them under our AOTA principle (`iroh` relays + Neon Better Auth + Passkeys/Web3 adapters).

## Itemized Flow & Technology Breakdown

| # | Step / Component | NextGraph Technology & Behavior | Role in Auth Story | Decision |
| :-: | :--- | :--- | :--- | :--- |
| 1 | **Local Wallet Crate** | `ng-wallet` (Rust) encrypted vault; browser uses encrypted IndexedDB/WebCrypto (`localStorage` fallback), native apps use OS app directory | Holds all private keys locally without server knowledge | Keep |
| 2 | **Signing Cryptography** | **Ed25519** keypairs + W3C Data Integrity proofs | Derives `did:peer`, signs commits, auth challenges, and shares | Keep |
| 3 | **Zero-Network Signup** | Wallet & `did:peer` created offline before contacting any server | Guarantees identity exists independently of central servers | Keep |
| 4 | **Mnemonic Seed Phrase** | BIP-39-style 12/24-word phrase + printable recovery PDF | Root backup and cross-device recovery | Keep |
| 5 | **Pazzle Unlock** | Emoji / visual pattern sequence (`load_pazzle_emojis`) | Fast, memorable local wallet unlock instead of long passwords | Keep |
| 6 | **PIN / Biometric Lock** | Short PIN / OS biometric prompt after inactivity timeout | Quick re-unlock on desktop & mobile | Keep |
| 7 | **Multi-Persona Wallet** | Multiple isolated identities (`did:peer` per persona) inside one wallet | Lets an author separate pen names or private vs public works | Keep |
| 8 | **QR Code Device Pairing** | Ephemeral encrypted QR shown on primary device, scanned on mobile/tablet | Transfers wallet keys to a new device without cloud key storage | Keep |
| 9 | **TextCode Device Pairing** | Short one-time text string copy/pasted between devices | Links headless/camera-less desktops (Linux/Windows/Mac) | Keep |
| 10 | **Wallet File Export** | Encrypted `.wallet` file via USB or local transfer | Offline cold backup & manual device migration | Keep |
| 11 | **Relay / Broker Transport** | 2-tier relay topology (Noise over WebSockets/QUIC) paired with **iroh (`v1.4.x`)** QUIC relays + `EndpointId` | NAT traversal, offline sync, and encrypted peer routing | Keep (+ iroh) |
| 12 | **Challenge-Response Auth** | Client proves `did:peer` ownership by signing a server nonce with Ed25519 | Passwordless login to ExoNovel web/API without sending private keys | Keep |
| 13 | **Cryptographic OCAPs** | Object Capabilities (cryptographic read/write/fork tokens) | Fine-grained access to shared characters, plots, timelines, themes | Keep |
| 14 | **PAKE / Password Option** | `libopaque` (OPAQUE PAKE) alongside **Neon Better Auth** (Google/GitHub SSO + email) | Password & SSO convenience resolving/minting `did:peer` | Keep (+ Better Auth / AOTA) |

## End-to-End Sequence Summary

1. **Primary User**: Launch app -> Create `exo-wallet` offline -> Back up Mnemonic -> Unlock with Pazzle / PIN / Biometrics -> Derive `did:peer` -> Connect via `iroh` / Noise / HTTPS -> Sign nonce challenge -> Authenticated as `did:peer`.
2. **Returning Multi-Device**: Open new device -> Scan QR / paste TextCode / import `.wallet` / enter Mnemonic -> Decrypt wallet -> Sign nonce challenge -> Sync encrypted state via `iroh` + Neon.
3. **Convenience SSO & Wallet Users (AOTA)**: Sign in with Google, GitHub, Email (Better Auth), `libopaque`, Passkeys, MetaMask/SIWE, Reown AppKit, or Privy -> Server resolves or mints `did:peer` -> Automatic key handle unlocked -> Authenticated as `did:peer` with full primary-path capabilities.

