# 001-003. Wallet-Technologien: NextGraph `ng-wallet` vs. MetaMask & Moderne Alternativen

Datum: 2026-11-19
Status: Akzeptiert (AOTA)
Übergeordnet: [001-auth](../001-auth.md)

## Kontext

Klärung der Fragen:
1. Welche Wallet-Technologie NextGraph verwendet.
2. Ob **MetaMask** oder breitere Alternativen genutzt werden sollen—insbesondere im Hinblick auf Autoren-Paywalls und Zahlungen.
3. Nach ExoNovels **AOTA-Prinzip („All of the Above“)** darf keine Beschränkung auf einen einzigen Wallet- oder Anmeldeweg erzwungen werden.

## Rechercheergebnisse

### 1. Einsatz bei NextGraph (`ng-wallet`)
- NextGraph entwickelt eine eigene **Rust**-Identitätswallet (`ng-wallet` in `nextgraph-rs`), keine Ethereum-Browser-Erweiterung.
- Verwaltet **Ed25519**-Schlüssel für `did:peer`-Identitäten im OS-Speicher bzw. in Browser-WebCrypto/IndexedDB.
- **Stärke**: Keine Gas-Gebühren, native `did:peer`-Schlüssel, offlinefähig auf allen 5 Desktop-/Mobilplattformen.
- **Grenze**: Kann von sich aus keine EVM-/Solana-Beträge (USDC, ETH) ohne angebundene Zahlungsschiene abwickeln.

### 2. Unterstützte Wallets & Anmeldeoptionen (AOTA-Spektrum)

| Option | Funktionsweise | Stärken | Rolle in ExoNovel |
| :--- | :--- | :--- | :--- |
| **Native `exo-wallet` (`ng-wallet`-Stil)** | Lokales **Ed25519**-Schlüsselpaar + `did:peer` in WebCrypto/IndexedDB & OS-Keychains | Keine Transaktionsgebühren, Offline-First auf allen 5 OS | Kernverwahrung der kryptografischen Identität |
| **MetaMask (Direkt & SIWE / EIP-4361)** | Browser-Erweiterung & Mobile-App signieren SIWE-Nachrichten und Web3-Zahlungen | Vertraut für Ethereum-/MetaMask-Nutzende | Erstklassige Web3-Login- & Paywall-Option |
| **Reown AppKit** *(WalletConnect)* | Open-Source-SDK für **MetaMask + 500+ Wallets**, **SIWE/SIWX**, **E-Mail/Social Embedded Wallets**, Swaps und Onramps | Multi-Chain-Abdeckung (EVM, Solana, Bitcoin) über ein Protokoll | Breiter Konnektor für externe und mobile Wallets |
| **Privy / Dynamic** | Stille non-custodial Embedded Wallets hinter Google/E-Mail/Passkey plus externe Wallet-Verknüpfung | Reibungsfreies Onboarding für Nicht-Krypto-Lesende | Adapter für Web3-Paywalls ohne Erweiterungszwang |
| **Passkeys (WebAuthn) + OS-Biometrie** | FaceID / TouchID / Windows Hello / FIDO2-Hardware-Schlüssel | Phishing-resistente Entsperrung ohne Seed-Phrasen | Primäre Komfort-Entsperrung |
| **Google, GitHub & E-Mail (Neon Better Auth)** | Verwaltetes OAuth2 + E-Mail/Passwort mit automatischer `did:peer`-Zuordnung | Vertrauter Web2-Einstieg mit vollen Primärpfad-Rechten | Sofortiges Komfort-Onboarding |

## Entscheidung (AOTA-Architektur)

**Maßgebliche Identität (`did:peer`)** wird konsequent von **Authentifizierungs- und Zahlungsadaptern** getrennt, sodass alle genannten Verfahren zusammenarbeiten:

1. **Einheitlicher `did:peer`-Eigentumskern**: Jede Anmelde- und Wallet-Methode wird in Neon PostgreSQL und `exo-wallet` auf die `did:peer` (`owner_did`) abgebildet.
2. **Alle Wallet-Adapter aktiv**: Native `exo-wallet`, **MetaMask / SIWE**, **Reown AppKit (500+ Wallets)**, **Privy / Dynamic**, **Passkeys (WebAuthn)** und **Neon Better Auth** laufen parallel.
