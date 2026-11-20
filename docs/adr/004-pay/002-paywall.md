# 004-002. Author Paywall Gating & Entitlements

Date: 2026-11-19
Status: Proposed (for interview)
Parent: [004-pay](../004-pay.md)

## Decision Structure (Pending Interview)

- **Granularity**: Authors can gate a full published work, specific chapters/episodes, or curated story-architecture bundles.
- **Entitlement Storage**: Payment webhooks (Stripe or on-chain/WalletConnect verification) write an entitlement row `(buyer_did, resource_id, grant_type, expires_at)` to Neon Postgres.
- **Cross-Platform Access**: Because entitlements bind to the buyer's `did:peer`, purchasing on Web unlocks reading on Linux, macOS, Windows, Android, and iOS automatically.

## Open Interview Questions

- What paywall models should ExoNovel support (one-time purchase, recurring author membership, pay-per-chapter, tipping)?
- Should free previews (e.g., first N chapters or public character bio) be configurable per paywalled project?
