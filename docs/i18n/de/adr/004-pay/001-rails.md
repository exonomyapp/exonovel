# 004-001. Zahlungsschienen: Fiat (Stripe) vs. Web3-Wallets

Datum: 2026-11-19
Status: Akzeptiert (AOTA)
Übergeordnet: [004-pay](../004-pay.md)

## Vergleich (Creator-Plattformen)

| Schiene | Anbieter / Standard | Stärken | Hinweise |
| :--- | :--- | :--- | :--- |
| **Fiat-Karten & Wallets** | **Stripe** (Checkout, Apple/Google Pay, Connect, Krypto-Onramp) | Höchste Konversion, globale Karten, Abos und Connect-Auszahlungen | Transaktionsgebühren, Rückbuchungen, KYC-Länderregeln |
| **Multi-Wallet Web3** | **Reown AppKit** *(WalletConnect)* + **MetaMask** + **USDC auf L2 (Base/Arbitrum)** | Verbindet MetaMask + 500+ Wallets + Embedded-Wallets; weltweite Sofortabwicklung ohne Rückbuchungen | Erfordert Krypto oder Fiat-Onramp |
| **Embedded Wallets** | **Privy** / **Dynamic** | Unsichtbare Non-Custodial-Wallet hinter Google/E-Mail plus Onramp | SDK-/Anbieterabhängigkeit |

## Entscheidung (AOTA)

Alle drei Schienen—**Stripe**, **Reown AppKit** und **Privy/Dynamic**—laufen parallel auf einem gemeinsamen `did:peer`-Anspruchsregister.

## Offene Interviewfrage

- Wie sollen Erlösbeteiligungen aufgeteilt werden, wenn mehrere `did:peer`-Konten gemeinsam ein kostenpflichtiges Werk schreiben oder einen Charakter forken?
