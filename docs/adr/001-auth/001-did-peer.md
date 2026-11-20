# 001-001. `did:peer` as Primary Ownership Key & Convenience SSO Resolution

Date: 2026-11-19
Status: Accepted (core principle) / Proposed (key custody details)
Parent: [001-auth](../001-auth.md)

## Context

Clarification from review: earlier phrasing ("mapped to a `did:peer`, never the primary ownership key") created ambiguity around whether another ownership key exists.

## Decision

1. **`did:peer` Is the Sole Primary Ownership Key**:
   - There is **no** other ownership key in ExoNovel.
   - Every authored artifact (`works`, `chapters`, `scenes`, `characters`, `plots`, `timelines`, `themes`, `forks`) stores `owner_did` referencing the author's `did:peer` string.
   - Provider IDs (Google `sub`, GitHub `id`), emails, and Better Auth `user.id` UUIDs are **authentication/login aliases only**—they look up or unlock the user's `did:peer`, and never own content directly.

2. **Convenience SSO Unlocks the Full Primary Path**:
   - Signing in via **Google** or **GitHub** (Neon Better Auth) is not a second-class account tier.
   - As soon as SSO completes:
     1. Backend queries Neon Postgres for an existing `identity_links(provider, subject) -> did_peer`.
     2. **Existing user**: Resolves their `did:peer`, unlocks their managed key handle (if custodial) or attaches their active `did:peer` claim to the session.
     3. **New user**: Automatically runs the primary-path initialization on their behalf—mints a `did:peer` keypair, stores the public `did:peer` and encrypted key material in Postgres (or Vault Transit), creates the default wallet/persona record, and returns an active `did:peer` session.
   - Result: An SSO user receives the exact same `did:peer` ownership, sharing, forking, and paywall capabilities as a user who initialized a local wallet directly—and may export/claim self-custody later without altering their `did:peer`.
