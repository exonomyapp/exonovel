# 011. Propozycja Biurka Pisarza („Desk”: makieta demonstracyjna i planowane moduły)

Data: 2026-11-28
Status: Zaproponowano (interaktywny kod demonstracyjny w `crates/app/src/lib.rs` :: `WorkspaceHomeView`; oczekuje na ocenę i decyzję właściciela)
Powiązane: [003-content](003-content.md), [003-001](003-content/001-formats.md), [003-002](003-content/002-forking.md), [007-i18n](007-i18n.md), [010-dear-world](010-dear-world.md)

## Kontekst

Po zalogowaniu ExoNovel zastępuje ogólny przycisk „Strona główna” widokiem **`Desk` (Biurko)** (`ActiveTab::Workspace`). Zgodnie z wytycznymi właściciela:
1. Usunięto cztery zbędne przyciski skrótów (`Profil`, `Preferencje`, `Docs & ADRs`, `Dear World`), ponieważ są już dostępne w górnym pasku i menu użytkownika.
2. Usunięto wiersze administracyjne (`ExoNovel Admin · Plan: free`, imię/pseudonim) z nagłówka Biurka.
3. Połączono dawne dwie sekcje w **jedno spójne Biurko Pisarza**, pozostawiając w prawym górnym rogu skrócony identyfikator `did:peer` z przyciskiem kopiowania.
4. Tekst skierowano ciepłym, nietechnicznym językiem do **uczniów szkół średnich i studentów**, skupiając się na samym pisaniu.
5. Dodano **makiety przyszłych narzędzi pisarskich**, aby właściciel mógł obejrzeć propozycję przed budową tabel bazodanowych i edytorów.

## Proponowana architektura i obecny kod demonstracyjny

Obecny kod demonstracyjny znajduje się w komponencie `WorkspaceHomeView` (`crates/app/src/lib.rs`, linie 1125–1322) oraz w stylach `.exo-desk-*` (`style/main.css`, linie 1286–1376). Obecnie działa wyłącznie w pamięci przeglądarki (bez zapisu w PostgreSQL), aby można było ocenić układ i interakcję:

### 1. Nagłówek Biurka i identyfikator `did:peer` (`lib.rs` linie 1171–1193)
- Nagłówek `Desk` oraz przystępny opis dla młodych autorów; w prawym górnym rogu skrócony `did:peer` z przyciskiem kopiowania i pełną podpowiedzią po najechaniu.
- **Status**: Działa z prawdziwymi danymi z `/api/me`.

### 2. Makieta A — Półka startowa formatów („Start a New Project”, `lib.rs` linie 1196–1231)
- Pięć kafelków formatów z [003-001](003-content/001-formats.md): **Powieści**, **Artykuły i Literatura Faktu**, **Opowiadania**, **Scenariusze**, **Poezja**.
- **Działanie w demie**: Kliknięcie podświetla kafelek i wyświetla poniżej listę przykładowych sekcji startowych dla wybranego formatu. Nie tworzy jeszcze projektu w bazie danych.

### 3. Makieta B — Półka elementów fabuły („Your Shelf of Story Pieces”, `lib.rs` linie 1233–1254)
- Cztery karty odpowiadające modułowym, współdzielonym elementom z [003-002](003-content/002-forking.md): **Postacie**, **Fabuły**, **Osie czasu**, **Motywy**.
- **Działanie w demie**: Statyczne karty opisowe 2×2; docelowo mają prowadzić do własnych i udostępnionych bibliotek elementów oraz zaproszeń do forka.

### 4. Makieta C — Szybki Brudnopis („Scratchpad”, `lib.rs` linie 1256–1315)
- Pole szybkich notatek z paskiem **`InputLocaleBar`** ([007-i18n](007-i18n.md)) do zmiany języka pisania i kierunku RTL/LTR, polem tytułu, polem szkicu sceny oraz przyciskami **Przypnij pomysł** i **Wyczyść**.
- **Działanie w demie**: Przechowuje tekst wyłącznie w pamięci bieżącej sesji (`scratch_title`, `scratch_body`, `scratch_saved`), bez zapisu w PostgreSQL.

## Otwarte pytania do decyzji właściciela

1. **Zakres i kolejność sekcji**: Czy wszystkie trzy bloki (Półka formatów, Elementy fabuły, Brudnopis) odpowiadają wizji Biurka, czy któryś należy usunąć, uprościć lub poprzedzić listą *Ostatnie historie i zaproszenia*?
2. **Tworzenie projektów**: Czy kliknięcie formatu ma od razu otwierać formularz nowego projektu, czy lista istniejących szkiców powinna być główną częścią Biurka?
3. **Zapis Brudnopisu**: Czy przypięte pomysły powinny trwale zapisywać się w PostgreSQL (`desk_notes`), czy Biurko powinno prowadzić prosto do właściwego edytora szkiców?
4. **Miejsce Elementów fabuły**: Czy *Postacie, Fabuły, Osie czasu, Motywy* powinny być widoczne bezpośrednio na Biurku, czy głównie wewnątrz konkretnych projektów?
