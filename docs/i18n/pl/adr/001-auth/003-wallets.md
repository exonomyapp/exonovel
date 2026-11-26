# 001-003. Technologie portfeli: NextGraph `ng-wallet` vs MetaMask i alternatywy

Data: 2026-11-19
Status: Zaakceptowano (AOTA)
Nadrzędny: [001-auth](../001-auth.md)

## Kontekst

Ustalenie roli portfela NextGraph (`ng-wallet`), **MetaMask** oraz alternatywnych rozwiązań w kontekście przyszłych paywalli i płatności—zgodnie z zasadą **AOTA („All of the Above“)**.

## Wyniki badań

### 1. Rozwiązanie NextGraph (`ng-wallet`)
- Własny portfel tożsamości w **Rust** (`ng-wallet`), zarządzający kluczami **Ed25519** dla `did:peer` w magazynie OS lub WebCrypto/IndexedDB.
- **Zaleta**: Zero opłat gas, pełne działanie offline na wszystkich 5 platformach.
- **Ograniczenie**: Nie obsługuje natywnie płatności EVM/Solana bez podpiętej szyny płatniczej.

### 2. Obsługiwane portfele i metody logowania (Spektrum AOTA)

| Opcja | Sposób działania | Mocne strony | Rola w ExoNovel |
| :--- | :--- | :--- | :--- |
| **Natywny `exo-wallet`** | Lokalne klucze **Ed25519** + `did:peer` | Zero opłat, pełne działanie offline | Rdzeń własności kryptograficznej |
| **MetaMask (SIWE / EIP-4361)** | Wtyczka przeglądarkowa i aplikacja mobilna | Znany użytkownikom Ethereum | Pierwszorzędna opcja Web3 i płatności |
| **Reown AppKit** *(WalletConnect)* | Otwarte SDK dla **MetaMask + 500+ portfeli**, **SIWE/SIWX** | Obsługa EVM, Solana, Bitcoin jednym protokołem | Główny łącznik portfeli zewnętrznych |
| **Privy / Dynamic** | Ukryte portfele non-custodial za Google/E-mail/Passkey | Wygodny start bez instalacji wtyczek | Adapter dla płatności Web3 bez rozszerzeń |
| **Passkeys (WebAuthn) + Biometria** | FaceID / TouchID / Windows Hello / klucze FIDO2 | Odporne na phishing szybkie odblokowanie | Główne wygodne odblokowanie |
| **Google, GitHub i E-mail (Neon Better Auth)** | Zarządzane OAuth2 i e-mail mapowane do `did:peer` | Znane logowanie Web2 z pełnią praw `did:peer` | Natychmiastowy start nowych twórców |

## Decyzja (Architektura AOTA)

Rozdzielenie **autorytatywnej tożsamości (`did:peer`)** od **adapterów logowania i płatności**, z jednoczesnym wsparciem wszystkich wymienionych opcji.
