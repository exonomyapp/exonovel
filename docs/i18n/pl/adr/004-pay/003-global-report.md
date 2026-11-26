# 004-003. Raport decyzyjny: Globalne płatności spoza USD, lokalne szyny i portfele (AOTA)

Data: 2026-11-21
Status: Zaakceptowano (Wdrożenie AOTA Poziom 1 -> 2 -> 3 zatwierdzone; wszystkie finanse w Profilu)
Nadrzędny: [004-pay](../004-pay.md)

## Cel raportu

Zasada **AOTA** zakłada, że USD i euro nie są podstawową walutą dla większości czytelników i autorów na świecie—w Polsce dominuje **BLIK**, w Brazylii **Pix**, w Indiach **UPI**, w Afryce Wschodniej **M-Pesa**, a w Azji **Alipay+ / WeChat Pay**.

---

## 1. Główne regionalne szyny płatnicze spoza USD

- **Azja i Indie**: **UPI** (Indie, 24/7), **Alipay+ i WeChat Pay**, **QRIS / PromptPay / PayNow**.
- **Ameryka Łacińska**: **Pix** (Brazylia), **SPEI / OXXO / Mercado Pago**.
- **Europa**: **BLIK** (Polska — dominujący 6-cyfrowy kod mobilny w PLN), **Wero / EPI + SEPA Instant + EuroPA** (Bizum, Bancomat, MB Way).
- **Afryka**: **M-Pesa** (Kenia / Afryka Wschodnia), **Mobile Money** (Nigeria, Ghana, RPA).
- **Korytarze objęte sankcjami**: Samodzielne portfele **Bitcoin Lightning**, **TON Wallet** oraz stablecoiny non-custodial.

---

## 2. Orkiestratory API i procesory globalne

- **Airwallex** (wypłaty w lokalnej walucie bez podwójnego FX), **Adyen** (BLIK, Wero, Pix, Alipay+), **dLocal** (Pix, UPI, M-Pesa, SPEI bez lokalnych spółek), **Razorpay**, **Flutterwave / Paystack**, **Paddle / Lemon Squeezy** oraz **Stripe** (Connect + karty + BLIK/SEPA + Apple/Google Pay).

---

## 3. Portfele bezgraniczne i mikropłatności

- **Natywny `exo-wallet` (`did:peer`)**, **Reown AppKit** (MetaMask + 500+ portfeli EVM/Solana/Bitcoin), **Privy / Dynamic**, **TON Wallet**, **Bitcoin Lightning** (mikropłatności poniżej 1 centa za rozdział) oraz **stablecoiny L2/Solana (`USDC`, `USDT`, `EURC`, `BRZ`, `XSGD`)**.

---

## 4. Zatwierdzony plan wdrożenia (AOTA)

- **Poziom 1**: **dLocal** + **Stripe** (m.in. BLIK/SEPA/karty) + **Reown AppKit (`USDC`/`USDT` na Base/Solana/Polygon)**.
- **Poziom 2**: **Razorpay**, **Flutterwave / Paystack**, **Alipay+ / WeChat Pay**, **TON Wallet + Bitcoin Lightning**.
- **Poziom 3**: Konta wielowalutowe **Airwallex** dla bezpośrednich rozliczeń w walutach lokalnych.
