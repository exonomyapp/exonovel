# 001. Auth & Identity Architecture

Date: 2026-11-19
Status: Proposed (for user review)

## Context

ExoNovel is a collaborative creative writing platform across Web, Desktop (Linux, macOS, Windows), and Mobile (Android, iOS) where **every author retains ownership of their own content** even while collaborating on shared manuscripts or story parts.

Ownership must be cryptographically grounded, portable across platforms, and independent of any single OAuth provider, while still offering frictionless Google and GitHub sign-in for everyday writers.

## High-Level Story

1. **Authoritative Identity (`did:peer`)**:
   - Every user's `did:peer` identifier **is** the primary ownership key across the entire application (`owner_did`).
   - All content—novels, scenes, characters, plots, timelines/threads, themes, and forks—is owned by a `did:peer`.
   - Email addresses, OAuth IDs (Google/GitHub), and Better Auth user UUIDs are strictly convenience login mappings that resolve to a `did:peer`, never alternative ownership keys.

2. **Primary Auth Flow (NextGraph-Style Local-First Wallet)**:
   - Inspired by NextGraph (`ng-wallet`), users can generate a local cryptographic wallet (Ed25519 keypairs + `did:peer`) offline with mnemonic seed recovery, fast local unlock ("Pazzle" / PIN / biometrics), multi-persona identities, and QR/TextCode/file pairing across devices.
   - Transport and relay handshakes can run over **iroh** (`v1.4.x`) / Noise-secured channels with challenge-response proof of `did:peer` ownership.

3. **Convenience Auth Flow (Google & GitHub SSO via Neon Better Auth)**:
   - Users can sign in with **Google** or **GitHub** (managed by Neon's **Better Auth**).
   - Immediately upon SSO completion, the server looks up the user's `did:peer` in PostgreSQL—or automatically mints a new `did:peer` keypair on behalf of a newly registered user—and unlocks the **full capabilities of the primary path** automatically.
   - Users who start with SSO can later export/claim self-custody of their keypair without changing their `did:peer` or losing content ownership.

4. **AOTA Wallet & Secrets Infrastructure (`All of the Above`)**:
   - Under ExoNovel's **AOTA** principle, support **all** evaluated wallet and sign-in mechanisms—native `exo-wallet` (`did:peer`), **MetaMask** / 500+ Web3 wallets via **Reown AppKit (WalletConnect)** + **SIWE/SIWX**, embedded consumer wallets (**Privy** / **Dynamic**), **Passkeys (WebAuthn)**, and **Google / GitHub / Email** via **Neon Better Auth**—each resolving to the user's authoritative `did:peer` (see [003-wallets](001-auth/003-wallets.md) & [004-pay](../004-pay.md)).
   - Evaluate **HashiCorp Vault (Transit Secrets Engine)** for envelope-encrypting managed/custodial `did:peer` keys created during Convenience SSO, complementing Neon Better Auth.

## Sub-ADRs

- [001-did-peer](001-auth/001-did-peer.md) — `did:peer` as the sole primary ownership key & SSO auto-minting/unlocking
- [002-ng-flow](001-auth/002-ng-flow.md) — Itemized NextGraph auth flow & technologies (Keep / Modify / Drop checklist)
- [003-wallets](001-auth/003-wallets.md) — NextGraph `ng-wallet` vs MetaMask vs Reown AppKit / Privy / Passkeys
- [004-vault-kms](001-auth/004-vault-kms.md) — HashiCorp Vault Transit KMS vs Neon Better Auth boundaries
