# 001-004. HashiCorp Vault vs Neon Better Auth

Date: 2026-11-19
Status: Proposed (for user decision)
Parent: [001-auth](../001-auth.md)

## Context

You asked whether **HashiCorp Vault** (`developer.hashicorp.com/vault`) is a good architectural fit alongside **Neon Better Auth**.

## Research Findings: Complementary Layers, Not Competitors

| System | Layer | What It Does | What It Does Not Do |
| :--- | :--- | :--- | :--- |
| **Neon Better Auth** | Application Auth & Sessions | Handles Google/GitHub OAuth callbacks, email/password, session cookies, JWTs (`neon_auth` schema in Neon Postgres) | Not an HSM/KMS for isolating master encryption keys outside the database |
| **HashiCorp Vault (Transit Engine)** | Infrastructure KMS / Cryptography-as-a-Service | Encrypts, decrypts, and signs payloads via API (`/transit/encrypt`, `/transit/sign`) with versioned, auto-rotated keys that **never leave Vault memory** | Does not manage user login flows, OAuth redirects, or browser sessions |

## Where Vault Fits in ExoNovel

Vault is **not needed for standard login sessions** (Neon Better Auth already handles that cleanly). However, Vault Transit solves one specific high-security problem created by our **Convenience SSO -> `did:peer`** requirement:

- When a new user signs up via Google/GitHub and the server mints a `did:peer` keypair on their behalf, the private key must be stored safely until the user exports it to a local device wallet.
- **Without Vault**: The private key is encrypted using an application env var (`MASTER_KEY`) and stored in Postgres.
- **With Vault Transit**: The server sends the newly minted `did:peer` private key to Vault Transit to encrypt; only the ciphertext is stored in Postgres. Even a full database leak cannot expose any custodial `did:peer` private keys because the wrapping key never enters the app database or app memory.

## Options for Interview Decision

- **Option A (Lean Start)**: Use Neon Better Auth + env-managed envelope encryption in Postgres now; add Vault Transit before production custodial key scale.
- **Option B (Vault From Day One)**: Provision HCP Vault / self-hosted Vault Transit immediately to wrap all SSO-minted `did:peer` keys and sensitive paywall/OAuth tokens.
