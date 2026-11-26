# 004-003. Entscheidungsbericht: Globale Nicht-USD-Zahlungen, lokale Schienen & Wallets (AOTA)

Datum: 2026-11-21
Status: Akzeptiert (AOTA Stufe 1 -> 2 -> 3 genehmigt; alle kommerziellen Steuerungen im Profil)
Übergeordnet: [004-pay](../004-pay.md)

## Zweck dieses Berichts

ExoNovels **AOTA-Prinzip** berücksichtigt, dass USD und Euro weltweit für viele Lesende und Schreibende nicht die Alltagswährung sind. In Indien, Lateinamerika, Südostasien, Festlandchina, Afrika, Ost-/Mitteleuropa und dem Nahen Osten dominieren nationale Sofortüberweisungen, Mobile Money, QR-Super-Apps und digitale Wallets.

---

## 1. Zentrale Nicht-USD-Inlands- & Regionalschienen

- **Asien & Indien**: **UPI** (Indien, 24/7 Sofortzahlung), **Alipay+ & WeChat Pay** (China & Panasien inkl. GCash/TrueMoney/KakaoPay), **QRIS / PromptPay / PayNow**.
- **Lateinamerika**: **Pix** (Brasilien, dominanter 24/7-Standard inkl. Pix Automático), **SPEI / OXXO / Mercado Pago** (Mexiko & LATAM).
- **Europa**: **BLIK** (Polen, führender 6-stelliger Mobile-Code in PLN), **Wero / EPI & SEPA Instant + EuroPA** (inkl. Bizum, Bancomat, MB Way).
- **Afrika**: **M-Pesa** (Kenia/Ostafrika, KES/TZS) sowie **Mobile Money** (MTN MoMo, Airtel Money in NGN, GHS, ZAR).
- **Eingeschränkte Korridore**: In vom internationalen Kartenverkehr getrennten Korridoren (z. B. SBP/Mir) stehen Selbstverwahrungs-Schienen (**Bitcoin Lightning**, **TON Wallet**, Self-Custody-Stablecoins) bereit.

---

## 2. API-Orchestratoren & Globale Prozessoren

- **Airwallex**: Mehrwährungs-Treasury und direkte Landeswährungs-Auszahlung ohne doppelte USD-Konvertierung.
- **Adyen**: Einheitliche API für Europa (Wero, BLIK, SEPA), APAC (Alipay+, WeChat Pay) und LATAM (Pix).
- **dLocal**: Sofortige Akzeptanz von Pix, UPI, M-Pesa, SPEI und OXXO ohne lokale Niederlassungen.
- **Razorpay** (Indien UPI AutoPay) & **Flutterwave / Paystack** (Panafrika Mobile Money).
- **Paddle / Lemon Squeezy** (optionaler Merchant of Record) & **Stripe** (Connect + Karten + Apple/Google Pay + Bridge).

---

## 3. Grenzenlose Krypto-, Mikro-Zahlungs- & Wallet-Technologien

- **Native `exo-wallet` (`did:peer`, Ed25519)** als Urheberkern ohne Blockchain-Zwang.
- **Reown AppKit** (MetaMask + 500+ Wallets auf EVM, Solana, Bitcoin), **Privy / Dynamic**, **TON Wallet**, **Bitcoin Lightning** (Sub-Cent-Kapitelzahlungen) sowie **L2- & Solana-Stablecoins (`USDC`, `USDT`, `EURC`, `BRZ`, `XSGD`)** mit Off-Ramps nach Pix/UPI/SEPA.

---

## 4. Genehmigter Stufenplan (AOTA)

- **Stufe 1**: **dLocal** + **Stripe** + **Reown AppKit (`USDC`/`USDT` auf Base/Solana/Polygon)**.
- **Stufe 2**: **Razorpay**, **Flutterwave / Paystack**, **Alipay+ / WeChat Pay**, **TON Wallet + Bitcoin Lightning**.
- **Stufe 3**: **Airwallex** Mehrwährungskonten für direkte Auszahlung in Landeswährung.
