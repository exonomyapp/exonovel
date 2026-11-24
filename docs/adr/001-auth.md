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

4. **Personas (Multiple Workspace Identities Under One Owner)**:
   - Every user has a default Persona anchored on their primary `did:peer` and can create unlimited secondary Personas (`did:peer` keypairs scoped under the same `owner_did`).
   - Switching active Persona switches workspace views and collaboration/chat attribution while preserving unified single-user ownership (`owner_did`) across all created content and chat histories.

5. **Authorization Tiers & Group Roles**:
   - Platform authorization uses discrete levels (`user` < `moderator` < `admin`) rather than a boolean gate, enabling moderators to review/comment on creative works they do not directly collaborate on.
   - Collaboration **Groups** carry group-scoped roles (`member`, `manager`) where any user can create groups, invite collaborators, record referrals, and assign group managers.

6. **AOTA Wallet & Secrets Infrastructure (`All of the Above`)**:
   - Under ExoNovel's **AOTA** principle, support **all** evaluated wallet and sign-in mechanisms—native `exo-wallet` (`did:peer`), **MetaMask** / 500+ Web3 wallets via **Reown AppKit (WalletConnect)** + **SIWE/SIWX**, embedded consumer wallets (**Privy** / **Dynamic**), **Passkeys (WebAuthn)**, and **Google / GitHub / Email** via **Neon Better Auth**—each resolving to the user's authoritative `did:peer` (see [003-wallets](001-auth/003-wallets.md) & [004-pay](../004-pay.md)).
   - Evaluate **HashiCorp Vault (Transit Secrets Engine)** for envelope-encrypting managed/custodial `did:peer` keys created during Convenience SSO, complementing Neon Better Auth.

7. **Connected External Apps, Optional Provider Scopes & Settings Revocation (`AOTA`)**:
   - **Minimal Login vs. Optional App Consent**: Primary SSO login (Google, GitHub, Microsoft, Apple, Discord) requests **identity-only scopes** (`email`, `profile`, `openid`). Extended third-party API permissions (e.g., Google Drive, Google Docs/Calendar, GitHub repositories, Microsoft OneDrive) are **never forced at login**; users opt in explicitly per external app when they want that integration.
   - **Verified Constraint on Hosted Neon Auth (`2026-11-23`)**: Live protocol testing against ExoNovel's hosted Neon Auth endpoint and `neon_auth.account` confirmed that Neon's managed OAuth proxy rewrites Google redirects to `scope=email+profile+openid` (ignoring client-supplied `scopes`) and omits `access_type=offline`, leaving `refreshToken` unpopulated (`has_refresh: false`). Additionally, Better Auth's `unlinkAccount` removes the entire SSO login linkage rather than selectively revoking an external app grant.
   - **Production Architecture (`connected_apps` Keyed to `did:peer`)**: To keep external apps connected across sessions until the user chooses to disconnect—while supporting multiple SSO providers under **AOTA**—ExoNovel's production backend (`crates/server` on `exonovel.exosystems.net`, or self-hosted Better Auth) isolates optional app grants from primary login in a dedicated **`connected_apps`** table (`did_peer`, `provider`, `app_key`, `granted_scopes`, `encrypted_refresh_token`, `status`, `connected_at`, `revoked_at`) wrapped via [004-vault-kms](001-auth/004-vault-kms.md):
     1. Each provider connector runs OAuth 2.0 Authorization Code + PKCE with offline consent (`access_type=offline`, `prompt=consent`, incremental scopes) directly on ExoNovel's fixed origin.
     2. Tokens persist encrypted in Neon PostgreSQL and refresh silently until the user revokes them.
     3. **ExoNovel Settings -> Connected Apps** lets any user inspect linked SSO providers, grant optional app access, or **Disconnect & Revoke** a single integration—calling the provider's official revocation endpoint (e.g., `https://oauth2.googleapis.com/revoke`) and purging the stored `connected_apps` tokens **without breaking primary SSO sign-in**.
   - **Preview Environment Boundary**: Because Google AI Studio preview URLs (`ais-dev-*`) are dynamic and cannot host server-side OAuth redirect callbacks without `redirect_uri_mismatch`, the preview app detects Neon Auth Google sign-in via `listAccounts()`, pre-fills `login_hint`, launches Google Drive consent in-memory via Firebase Auth on user click, and exposes **Settings -> Connected Apps** status and live token revocation routed through ExoNovel's authenticated server proxy (`POST /api/oauth/google/revoke` -> `oauth2.googleapis.com/revoke`) to avoid browser CORS blocks (see [005-002](../005-docs-ui/002-gate.md)).

## Sub-ADRs

- [001-did-peer](001-auth/001-did-peer.md) — `did:peer` as the sole primary ownership key & SSO auto-minting/unlocking
- [002-ng-flow](001-auth/002-ng-flow.md) — Itemized NextGraph auth flow & technologies (Keep / Modify / Drop checklist)
- [003-wallets](001-auth/003-wallets.md) — NextGraph `ng-wallet` vs MetaMask vs Reown AppKit / Privy / Passkeys
- [004-vault-kms](001-auth/004-vault-kms.md) — HashiCorp Vault Transit KMS vs Neon Better Auth boundaries
