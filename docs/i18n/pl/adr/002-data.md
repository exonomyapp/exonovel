# 002. Architektura danych, trwałości i synchronizacji offline

Data: 2026-11-19
Status: Zaakceptowano (stos Neon) / Zaproponowano (szczegóły synchronizacji `iroh`)

## Kontekst

ExoNovel działa w przeglądarce oraz natywnie na **Linux, macOS, Windows, Android i iOS**, zapewniając trwały zapis w chmurze, jednoczesną edycję na żywo i synchronizację P2P offline.

## Założenia nadrzędne

1. **Zarządzana chmura z przenośnością Willow (Neon)**:
   - **PostgreSQL**: Centralny magazyn relacyjny dla `did:peer`, `user_personas`, `user_profiles`, `media_assets`, grup, zaproszeń, powiązań SSO i architektury fabularnej.
   - **Współrzędne protokołu Willow**: Tabele zachowują deterministyczne pola `willow_namespace`, `willow_subspace`, `willow_path`, `willow_timestamp_us`, `payload_digest` i `byte_length`, umożliwiając bezstratną migrację do lokalnego magazynu Rust `willow25`.
   - **Wielorozmiarowe media**: Przesyłane obrazy do 2 MB są przetwarzane po stronie serwera do wariantów `64×64`, `256×256` oraz `<= 1024px` (WebP/JPEG).
2. **Trzy tryby pracy, węzły `did:peer` ExoNovel i dwukierunkowa przenośność**:
   - **Tryb 1 — ExoNovel w sieci (`Darmowy`)**: Przeglądarkowa obsługa oparta na **Neon PostgreSQL**.
   - **Tryb 2 — ExoNovel na urządzeniu (`Darmowy`)**: Aplikacje natywne uruchamiają się lokalnie **bez wymagania Neon**, używając **`willow25`** (`0.7.11`) i darmowej strefy **relay/discovery `iroh`** (`v1.4.x`). **Zasada ślepego przekaźnika**: wszystkie dane buforowane w trybie 2 są szyfrowane end-to-end na urządzeniu nadawcy.
   - **Tryb 3 — Sieć + Urządzenie z zarządzanym synciem (`Płatny: równowartość 9 USD/mies. lub 90 USD/rok w walucie lokalnej`)**:
     - Aktywacja i rozliczenie w **Profilu** (ExoNovel działa jako partner z własnym `did:peer`), odblokowując przełącznik **Synchronizacji w Ustawieniach** oparty o delegację uprawnień **Meadowcap** w `willow25` i pełne szyfrowanie AEAD po stronie klienta.
     - Dwukierunkowy eksport (`Neon -> willow25`) i import (`willow25 -> Neon`) z zachowaniem przyczynowej historii **`loro` (`1.18.0`)**.

## Dokumenty podrzędne (Sub-ADRs)

- [001-neon](002-data/001-neon.md) — Neon PostgreSQL, Better Auth i magazyn obiektów
- [002-iroh](002-data/002-iroh.md) — Przekaźniki `iroh` i natywna synchronizacja P2P offline
- [003-crdt-report](002-data/003-crdt-report.md) — Raport decyzyjny: silniki CRDT współedycji w czasie rzeczywistym (wybrano `Loro`)
- [004-db-prefs](002-data/004-db-prefs.md) — Eliminacja nadmiarowego `localStorage` na rzecz `user_profiles` w Neon PostgreSQL
- [005-loro](002-data/005-loro.md) — Architektura współedycji na żywo, drzew ruchomych i historii w Loro CRDT
