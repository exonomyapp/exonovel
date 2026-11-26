# 004-001. Szyny płatnicze: Fiat (Stripe) vs portfele Web3

Data: 2026-11-19
Status: Zaakceptowano (AOTA)
Nadrzędny: [004-pay](../004-pay.md)

## Porównanie rynkowe

| Szyna | Dostawca / Standard | Zalety | Uwagi |
| :--- | :--- | :--- | :--- |
| **Karty i portfele Fiat** | **Stripe** (Checkout, Apple/Google Pay, BLIK/SEPA, Connect) | Najwyższa konwersja, subskrypcje i wypłaty Connect | Prowizje, obciążenia zwrotne, reguły KYC |
| **Multi-Wallet Web3** | **Reown AppKit** *(WalletConnect)* + **MetaMask** + **USDC L2** | Obsługa MetaMask + 500+ portfeli; natychmiastowe rozliczenie globalne bez chargebacków | Wymaga kryptowaluty lub onrampu |
| **Embedded Wallets** | **Privy** / **Dynamic** | Niewidoczny portfel non-custodial pod Google/E-mail z onrampem | Zależność od SDK dostawcy |

## Decyzja (AOTA)

Wsparcie **wszystkich trzech ścieżek równolegle**—**Stripe**, **Reown AppKit** i **Privy/Dynamic**—zapisujących uprawnienia do wspólnego rejestru `did:peer`.

## Otwarte pytanie

- Jak rozliczać podział przychodów, gdy wielu współautorów `did:peer` tworzy płatny utwór lub korzysta ze zforkowanej postaci?
