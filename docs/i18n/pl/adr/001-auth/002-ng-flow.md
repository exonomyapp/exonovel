# 001-002. Szczegółowy przepływ NextGraph Auth i lista technologii

Data: 2026-11-19
Status: Zaakceptowano (zachowano wszystkie 14 kroków NextGraph)
Nadrzędny: [001-auth](../001-auth.md)

## Kontekst

Niniejszy dokument opisuje wszystkie 14 kroków i komponentów z ekosystemu **NextGraph** (`nextgraph-rs` / `ng-wallet`), uzupełnionych zgodnie z zasadą AOTA o kanały `iroh`, Neon Better Auth oraz adaptery Passkeys/Web3.

## Zestawienie przepływu i technologii

| # | Krok / Komponent | Technologia NextGraph | Rola w architekturze | Decyzja |
| :-: | :--- | :--- | :--- | :--- |
| 1 | **Lokalny portfel** | `ng-wallet` (Rust) szyfrowany magazyn lokalny (WebCrypto/IndexedDB lub katalog OS) | Przechowuje klucze prywatne lokalnie poza serwerem | Zachować |
| 2 | **Kryptografia podpisu** | Klucze **Ed25519** + dowody W3C Data Integrity | Wylicza `did:peer`, podpisuje zmiany i wyzwania logowania | Zachować |
| 3 | **Rejestracja offline** | Portfel i `did:peer` powstają bez dostępu do sieci | Gwarantuje niezależność tożsamości od serwerów | Zachować |
| 4 | **Fraza odzyskiwania** | Fraza 12/24 słów (BIP-39) + PDF ratunkowy | Kopia główna i odzyskiwanie między urządzeniami | Zachować |
| 5 | **Odblokowanie Pazzle** | Wizualna sekwencja emoji (`load_pazzle_emojis`) | Szybkie odblokowanie zamiast długiego hasła | Zachować |
| 6 | **Blokada PIN / Biometria** | Krótki PIN lub biometria systemu po bezczynności | Szybki powrót do sesji na desktopie i telefonie | Zachować |
| 7 | **Portfel wielopersonowy** | Wiele izolowanych tożsamości (`did:peer` na personę) | Rozdział pseudonimów i projektów prywatnych/publicznych | Zachować |
| 8 | **Parowanie QR** | Tymczasowy szyfrowany kod QR między urządzeniami | Przeniesienie portfela bez chmury kluczy | Zachować |
| 9 | **Parowanie TextCode** | Jednorazowy ciąg tekstowy kopiowany między urządzeniami | Łączenie komputerów bez kamery (Linux/Windows/Mac) | Zachować |
| 10 | **Eksport pliku `.wallet`** | Szyfrowany plik `.wallet` (USB / transfer lokalny) | Zimna kopia zapasowa i ręczna migracja | Zachować |
| 11 | **Transport Relay / Broker** | Dwupoziomowy relay (Noise / QUIC) + **iroh (`v1.4.x`)** | Przechodzenie przez NAT, sync offline i routing P2P | Zachować (+ iroh) |
| 12 | **Uwierzytelnianie Challenge-Response** | Klient podpisuje serwerowe nonce kluczem Ed25519 | Logowanie bez wysyłania klucza prywatnego | Zachować |
| 13 | **Kryptograficzne OCAP** | Object Capabilities (tokeny odczytu/zapisu/forkowania) | Precyzyjne uprawnienia do postaci, fabuł i motywów | Zachować |
| 14 | **PAKE / Hasło i SSO** | `libopaque` wraz z **Neon Better Auth** (Google/GitHub/E-mail) | Wygodne logowanie z automatycznym rozwiązaniem `did:peer` | Zachować (+ Better Auth / AOTA) |

## Podsumowanie przepływu końcowego

1. **Ścieżka podstawowa**: Utworzenie `exo-wallet` offline -> Zapis frazy -> Odblokowanie Pazzle/PIN/Biometria -> Wyprowadzenie `did:peer` -> Połączenie `iroh` / Noise / HTTPS -> Podpis wyzwania -> Aktywna sesja `did:peer`.
2. **Kolejne urządzenie**: Skan QR / TextCode / plik `.wallet` / fraza -> Deszyfrowanie portfela -> Synchronizacja przez `iroh` + Neon.
3. **Wygodne SSO i portfele (AOTA)**: Google, GitHub, E-mail, Passkeys, MetaMask/SIWE, Reown AppKit lub Privy -> Serwer rozwiązuje lub tworzy `did:peer`.
