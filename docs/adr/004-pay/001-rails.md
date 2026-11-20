# 004-001. Payment Rails: Fiat (Stripe) vs Web3 Wallets

Date: 2026-11-19
Status: Accepted (AOTA)
Parent: [004-pay](../004-pay.md)

## Research Comparison (2025–2026 Creator Platforms)

| Rail | Provider / Standard | Strengths | Considerations |
| :--- | :--- | :--- | :--- |
| **Fiat Cards & Wallets** | **Stripe** (Checkout, Apple/Google Pay, Connect, Crypto Onramp) | Highest reader conversion; global cards, recurring subscriptions, and Connect author payouts | Platform fees, chargebacks, KYC/payout country rules |
| **Multi-Wallet Web3** | **Reown AppKit** *(WalletConnect)* + **MetaMask** + **USDC on L2 (Base/Arbitrum)** | Connects MetaMask + 500+ wallets + email embedded wallets; instant global settlement, zero chargebacks | Readers need crypto or fiat-to-crypto onramp |
| **Consumer Embedded Wallet** | **Privy** / **Dynamic** | Invisible non-custodial wallet from Google/Email + fiat onramp | Third-party SDK/pricing dependency |

## Decision (AOTA)

Support **all three rails side-by-side**—**Stripe**, **Reown AppKit (MetaMask + 500+ wallets)**, and **Privy/Dynamic embedded wallets**—normalized into a single `did:peer` entitlement ledger so neither authors nor readers are forced onto one payment mechanism.

## Open Interview Question

- How should collaborative revenue splits work when multiple `did:peer` authors co-write a paywalled work or fork a character?
