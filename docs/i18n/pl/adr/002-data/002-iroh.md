# 002-002. Przekaźnik `iroh` i natywna synchronizacja P2P offline

Data: 2026-11-19
Status: Zaproponowano (kierunek zaakceptowany; szczegóły otwarte do wywiadu)
Nadrzędny: [002-data](../002-data.md)

## Kontekst

ExoNovel działa w przeglądarce oraz na pięciu platformach natywnych (Linux, macOS, Windows, Android, iOS) i wymaga niezawodnej pracy offline, współpracy na żywo oraz przekaźników chroniących prywatność.

## Ustalenia badawcze (`iroh`)

- **Wersja bazowa**: `iroh` **v1.4.1** oparty o QUIC i bezpośrednie adresowanie kryptograficznym `EndpointId`.
- **Bezstanowe szyfrowane przekaźniki domowe**: Pomagają przebić NAT, po czym ruch płynie bezpośrednio między węzłami.
- **Synchronizacja lokalna i offline**: `swarm-discovery` (mDNS w sieci LAN), `iroh-docs` + `iroh-blobs` (uzgadnianie zbiorów oparte na zakresach i skrótach SHA-256) oraz `iroh-gossip` (**v0.102.0**) do rozgłaszania obecności i delt CRDT.

## Decyzja

1. **Ślepy przekaźnik i poufność treści dla przekaźnika współpracy ORAZ osobistego syncu `willow25`**:
   - Darmowa strefa **`iroh` (`v1.4.x`)** z pełnym szyfrowaniem Noise + `ChaCha20-Poly1305` zapewniająca darmową synchronizację współpracy między przeglądarką a urządzeniami.
   - Wszystkie utwory, migawki/delty Loro, wiadomości i media są przed wysłaniem z urządzenia autora **dopełniane, solone losowymi nonce i szyfrowane AEAD (`XChaCha20-Poly1305` / MLS)**, dzięki czemu węzły synchronizacji osobistej i przekaźniki współpracy widzą wyłącznie nieczytelny szyfrogram.
2. **Wbudowany węzeł natywny**: Kompilacje desktopowe i mobilne osadzają natywny węzeł `iroh` (`iroh-docs` + `iroh-blobs` + `iroh-gossip`).
3. **Mostek Web i dwukierunkowa migracja Neon ↔ `willow25`**: Przeglądarki łączą się przez bezpieczne WebSockety (`wss://`), a aplikacje natywne działają bez Neon z możliwością późniejszego importu z powrotem do chmury.

## Ustalone i otwarte decyzje

- **Wybór CRDT (Ustalono)**: Wybrano **`loro` (`1.18.0`)** (zobacz [005-loro](005-loro.md)).
- **Hosting przekaźników (Otwarte)**: Dedykowane przekaźniki regionalne ExoNovel, własne przekaźniki autorów czy oba warianty (AOTA)?
