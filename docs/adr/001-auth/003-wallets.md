# 001-003. Wallet Tech: NextGraph `ng-wallet` vs MetaMask & Modern Alternatives

Date: 2026-11-19
Status: Accepted (AOTA)
Parent: [001-auth](../001-auth.md)

## Context

You asked:
1. Which wallet technology NextGraph uses.
2. Whether **MetaMask** should be used as our wallet / convenience sign-in, or if there are better/more popular alternatives—especially given upcoming author paywalls and payments.
3. Under ExoNovel's **AOTA ("All of the Above")** rule, ExoNovel must never force users into a single wallet or sign-in choice when supporting multiple options is reasonably possible.

## Research Findings

### 1. What NextGraph Uses (`ng-wallet`)
- NextGraph builds its own **Rust** identity wallet (`ng-wallet` in `nextgraph-rs`), NOT an Ethereum/Web3 browser-extension wallet.
- It manages **Ed25519** keys for `did:peer` identities, stored via OS storage on native apps and encrypted storage on Web (**WebCrypto + IndexedDB** recommended over `localStorage`).
- **Strength**: Zero blockchain/gas friction, true `did:peer` native keys, works offline on all 5 desktop/mobile platforms.
- **Limitation**: Cannot directly hold or spend EVM/Solana money (USDC, ETH) for paywalls out of the box without bridging to a payment rail.

### 2. Supported Wallets & Sign-In Options (AOTA Spectrum)

| Option | How It Works | Strengths | Role in ExoNovel |
| :--- | :--- | :--- | :--- |
| **Native `exo-wallet` (`ng-wallet` style)** | Local **Ed25519** keypair + `did:peer` stored in WebCrypto/IndexedDB & OS keychains | Zero gas, offline-first across all 5 desktop/mobile OSes | Core cryptographic identity custody |
| **MetaMask (Direct & SIWE / EIP-4361)** | Browser extension & mobile app signing SIWE messages and Web3 payments | Familiar to Ethereum/MetaMask users | First-class Web3 login & paywall option |
| **Reown AppKit** *(WalletConnect)* | Open-source SDK covering **MetaMask + 500+ wallets**, **SIWE/SIWX**, **email/social embedded wallets**, swaps, and onramps | Multi-chain (EVM, Solana, Bitcoin) breadth under one standard protocol | Broad external & mobile wallet connector |
| **Privy / Dynamic** | Silent non-custodial embedded wallets behind Google/Email/Passkey logins + external wallet linking | Frictionless onboarding for non-crypto readers & authors | Embedded wallet adapter for zero-extension Web3 paywalls |
| **Passkeys (WebAuthn) + OS Biometrics** | FaceID / TouchID / Windows Hello / FIDO2 hardware keys | Instant phishing-resistant unlock without seed phrases or extensions | Primary convenience unlock & passwordless sign-in |
| **Google, GitHub & Email (Neon Better Auth)** | Managed OAuth2 + email/password resolved or auto-minted to `did:peer` | Familiar Web2 sign-in with automatic full primary-path capabilities | Immediate convenience onboarding |

## Decision (AOTA Architecture)

Separate **Authoritative Identity (`did:peer`)** from **Authentication & Payment Adapters** and support **All of the Above (AOTA)**:

1. **Unified `did:peer` Ownership Core**: Every supported login and wallet method resolves to (or mints) the user's `did:peer` (`owner_did`) in Neon PostgreSQL and local `exo-wallet`. Switching or linking MetaMask, Reown, Privy, Passkeys, Google, or GitHub never fragments content ownership.
2. **All Wallet Adapters Enabled**: Support native `exo-wallet`, **MetaMask / SIWE**, **Reown AppKit (500+ wallets)**, **Privy / Dynamic** embedded wallets, **Passkeys (WebAuthn)**, and **Neon Better Auth** SSO together so every user chooses their preferred workflow.

