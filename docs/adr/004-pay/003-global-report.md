# 004-003. Decision Report: Global Non-USD Payments, Local Rails & Wallets (AOTA)

Date: 2026-11-21
Status: Accepted (AOTA Tier 1 → Tier 2 → Tier 3 rollout approved; all author wallet/commercial controls live in Profile)
Parent: [004-pay](../004-pay.md)

## Purpose of This Report

ExoNovel's **AOTA ("All of the Above")** policy and global creative mission assume **USD and Euro are NOT the primary currencies** for most readers and authors worldwide. Large creative populations across India, Latin America, Southeast Asia, Mainland China, Africa, Eastern/Central Europe, and the Middle East transact daily using domestic instant bank rails, mobile money, QR super-apps, and borderless digital wallets—often without international Visa/Mastercard credit cards or USD bank accounts.

This report breaks down the global payment landscape by layer so you can select which regional processors, settlement hubs, and wallets ExoNovel should integrate first and next.

---

## 1. Major Non-USD Domestic & Regional Payment Rails

### Asia & India
- **UPI (Unified Payments Interface — India)**: Instant 24/7 bank-to-bank & QR mobile rail handling billions of monthly INR payments with near-zero merchant fees. Also adding cross-border corridors (UAE, Singapore, France, etc.). Essential for Indian readers & authors.
- **Alipay+ & WeChat Pay (Mainland China & Pan-Asia)**: QR/super-app ecosystems connecting Chinese CNY consumers plus regional e-wallets (GCash Philippines, Touch 'n Go Malaysia, KakaoPay Korea, TrueMoney Thailand) across borders.
- **QRIS / PromptPay / PayNow (Indonesia / Thailand / Singapore)**: National instant QR schemes dominant for IDR, THB, and SGD retail checkouts.

### Latin America
- **Pix (Brazil)**: Central Bank instant 24/7 BRL rail via QR or Pix Key. Surpassed cards for digital commerce across Brazil and supports recurring Pix Automático.
- **SPEI / OXXO (Mexico) & Mercado Pago**: Instant MXN bank transfers plus cash-voucher checkout for unbanked readers across LATAM.

### Europe (Non-Card & National Rails)
- **BLIK (Poland)**: Dominant 6-digit mobile code system used for the majority of Polish PLN e-commerce checkouts.
- **Wero / EPI & SEPA Instant + EuroPA Alliance**: Bank-backed European instant A2A wallet replacing reliance on US card networks across Germany, France, Belgium, Netherlands, expanding interoperability with **Bizum (Spain)**, **Bancomat (Italy)**, and **MB Way (Portugal)**.

### Africa
- **M-Pesa (Kenya / East Africa)**: Telecom-native mobile money (KES/TZS) allowing feature-phone and smartphone payments without traditional bank accounts.
- **Mobile Money & Local Bank Rails (Nigeria, Ghana, South Africa)**: MTN MoMo, Airtel Money, and instant bank transfers in NGN, GHS, ZAR.

### CIS / Sanctioned Corridors (Architectural Consideration)
- **SBP (Faster Payments System — Russia) & Mir**: Operate domestically in RUB disconnected from SWIFT/Visa/Mastercard. Because US/EU-domiciled processors (`Stripe`, `Adyen`, `Airwallex`) legally block sanctioned entities/rails, authors or readers in restricted corridors typically rely on peer-to-peer / self-custody digital rails (**Bitcoin Lightning**, **TON Wallet**, or self-custody stablecoins) rather than centralized fiat gateways.

---

## 2. API Orchestrators & Global Multi-Currency Processors

Because connecting directly to 20 national central banks is impractical, ExoNovel can combine a **multi-currency core treasury** with **regional specialists**:

- **Airwallex (Global Multi-Currency Hub)**:
  - Enables collecting, holding, and paying out natively across dozens of currencies (AUD, CNY, GBP, EUR, SGD, HKD, JPY, CAD, etc.) via local clearing networks without forced USD round-trips or double FX fees.
- **Adyen (Enterprise Local Acquiring)**:
  - Strongest single API for native local acquiring across Europe (Wero, iDEAL, BLIK, SEPA), APAC (Alipay+, WeChat Pay, GrabPay), and LATAM (Pix).
- **dLocal (Emerging Markets Specialist — LATAM, Africa, APAC, MENA)**:
  - Lets ExoNovel accept **Pix, UPI, M-Pesa, OXXO, SPEI, and local cards** in local currency and settle globally **without incorporating local subsidiaries in each country**.
- **Razorpay (India Specialist)**:
  - Deepest coverage of **UPI, RuPay, Indian netbanking, and INR recurring mandates**.
- **Flutterwave & Paystack (Pan-Africa Specialists)**:
  - Standard APIs for **M-Pesa, Mobile Money, Verve cards, and local African bank transfers** across Nigeria, Kenya, Ghana, and South Africa.
- **Paddle / Lemon Squeezy (Merchant of Record Option)**:
  - Acts as the reseller handling global VAT/GST/sales-tax compliance automatically in 100+ jurisdictions for subscriptions/digital unlocks, though they charge higher per-transaction fees (~5%) and support fewer hyper-local payout methods to authors.
- **Stripe (Connect & Global Cards/Apple Pay/Google Pay + Bridge)**:
  - Strong card/Express-Checkout baseline and Stripe Connect marketplace payouts, plus newly acquired **Bridge** stablecoin orchestration—but weak when serving users outside Visa/Mastercard/USD-friendly countries alone.

---

## 3. Borderless Crypto, Micro-Payment & Wallet Technologies (AOTA)

When banking corridors are expensive or unavailable—or for tiny per-chapter/tipping micro-payments (e.g., $0.05 equivalent)—non-custodial wallets shine:

### Wallet Connection Standards & Brands
- **Native `exo-wallet` (`did:peer`, Ed25519)**: Core authorship & ownership identity (no blockchain needed to sign in or own manuscripts).
- **Reown AppKit (formerly WalletConnect)**: Single open-source SDK supporting **MetaMask, Phantom, Trust Wallet, Rainbow, Coinbase Wallet, Binance Web3 Wallet, OKX Wallet, Safe, plus 500+ wallets** across EVM (Base, Arbitrum, Polygon, Ethereum), **Solana**, and **Bitcoin**, plus email/social embedded sign-in.
- **Privy / Dynamic**: Invisible non-custodial embedded wallets created automatically behind Email/Google/Passkey login so non-crypto readers can pay without installing browser extensions.
- **TON Wallet (Telegram Open Network)**: Native inside Telegram for over 900M+ users globally—high adoption across Eastern Europe, Asia, and Africa for instant micro-transfers in **TON / USDT-TON**.
- **Bitcoin Lightning Wallets (Phoenix, Breez, Wallet of Satoshi, Zeus / Lightspark Grid)**: Enables sub-cent, instant global micro-payments (pay-per-chapter, tipping in satoshis) and programmatic payouts into local fiat corridors.
- **EVM L2 + Solana Stablecoins (`USDC`, `USDT`, `EURC`, Regional Stablecoins like `BRZ` BRL / `XSGD` SGD)**:
  - Settlement on **Base, Arbitrum, Optimism, Polygon, or Solana** costs fractions of a cent in gas.
  - Off-ramp APIs (**Circle**, **Bridge**, **BlindPay**, **Lightspark Grid**) increasingly bridge stablecoin payouts directly into **Pix (BRL)**, **UPI (INR)**, or **SEPA Instant (EUR)**.

---

## 4. Core Economic Architecture Recommendation (Currency-Neutral)

Regardless of which payment gateways we activate in Phase 1 vs. Phase 2:
1. **Multi-Currency Native Pricing Ledger**: Store every price, balance, and payout split with exact `(minor_units BIGINT, currency_code TEXT)` (e.g., `INR`, `BRL`, `PLN`, `CNY`, `KES`, `NGN`, `EUR`, `SAT`, `USDC`) rather than assuming `USD` everywhere.
2. **Decouple Author Ownership (`did:peer`) from Payment Rail**: A collaborator in Warsaw can earn and cash out via **BLIK / EUR**, a co-author in Bangalore via **UPI / INR**, a reader in Nairobi can unlock chapters via **M-Pesa / KES**, and a reader on Telegram can tip via **TON / Lightning**—all writing unified entitlements to the buyer's `did:peer`.

## Proposed Decision Rollout (For Your Approval)

- **Tier 1 (Broadest Global Reach Fastest)**: Pair **dLocal** (instant coverage of Pix, UPI, M-Pesa, SPEI, emerging markets without local entities) + **Stripe** (cards, Apple/Google Pay, BLIK/SEPA/Connect) + **Reown AppKit (USDC/USDT on Base/Solana/Polygon)**.
- **Tier 2 (Deep Regional & Micro-Payments)**: Add **Razorpay** (full INR UPI AutoPay), **Flutterwave / Paystack** (Pan-African mobile money), **Alipay+ / WeChat Pay** (via Adyen/Airwallex), and **TON Wallet + Bitcoin Lightning** for sub-dollar chapter micro-unlocks and sanctioned-corridor self-custody.
- **Tier 3 (Treasury & FX Minimization)**: Add **Airwallex** multi-currency accounts so author payouts settle in native currency instead of converting through USD twice.
