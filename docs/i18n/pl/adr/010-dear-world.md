# 010. Przewodniki „Dear World”, magazyn PostgreSQL i publiczny start użytkownika

Data: 2026-11-27
Status: Zaakceptowano (baza PostgreSQL ze współrzędnymi `willow_*` aktywna teraz; replikacja `willow25` odłożona do startu aplikacji natywnych)

## Kontekst

ExoNovel wprowadza uczniów szkół średnich i studentów w nowy sposób wspólnego pisania: klocki fabularne (`Postacie`, `Wątki`, `Osie czasu`, `Motywy`), wiele `Person`, rozwidlanie elementów z zachowaniem autorstwa oraz płynną współpracę między przeglądarką a aplikacjami na urządzenia. Wzorując się na przystępności przewodnika GitHub Docs **„Hello World”** (`https://docs.github.com/en/get-started/using-github/hello-world`), ExoNovel potrzebuje własnych **Przewodników użytkownika** — oddzielonych od wewnętrznych ADR-ów i dostępnych publicznie bez logowania.

Ponieważ pełna replikacja peer-to-peer `willow25` (`v0.7.11`) jest obecnie odłożona do etapu aplikacji desktopowych i mobilnych, przewodniki działają na bazie PostgreSQL z gotowymi polami współrzędnych Willow na przyszłą synchronizację.

## Decyzja

1. **Koncepcja redakcyjna „Dear World”**:
   - Nazwa serii: **„Dear World”**, utrzymana w ciepłym, życzliwym tonie doświadczonego pisarza wprowadzającego młodszych twórców w **nowy sposób korzystania z ExoNovel** (bez teoretycznych wykładów o samym pisaniu).
   - Startowe 4 rozdziały po angielsku (`docs/guides/`), niemiecku (`docs/i18n/de/guides/`) i polsku (`docs/i18n/pl/guides/`): `01-dear-world`, `02-first-story-and-blocks`, `03-personas-groups-and-ownership`, `04-web-device-and-sharing`.

2. **Publiczny dostęp i jeden punkt nawigacji (wyłącznie górny pasek)**:
   - Dostępne publicznie pod `/dear-world` (`/dear-world/{slug}`) oraz przez API `GET /api/guides` i `GET /api/guides/{slug}` **bez logowania**.
   - Link **`Dear World`** występuje w **jednym miejscu**: wyłącznie w górnym pasku nawigacji (oraz jego mobilnym menu) przed logowaniem i po zalogowaniu (obok **`Biurka`**, bez przycisku „Start”/„Home”), bez powielania przycisków ani linków na **Biurku**, w stopce czy na stronach poniżej górnego paska.

3. **Model danych PostgreSQL ze współrzędnymi `willow25` (`crates/server/src/guides.rs`)**:
   - Tabele `dear_world_guides` oraz `dear_world_guide_translations` (`PRIMARY KEY (slug, locale)`) przechowują bezpiecznie wyrenderowany HTML GFM (`comrak` + `ammonia`) oraz pola `willow_namespace = 'exonovel'`, `willow_subspace = 'public/dear-world'`, `willow_path = '/dear-world/{slug}'`, `willow_timestamp_us`.
   - Dzięki temu po włączeniu `willow25` biblioteka „Dear World” dołączy do publicznego magazynu Willow bez zmiany schematu bazy.

## Konsekwencje

- Uczniowie i studenci mogą spokojnie poznać ExoNovel w **Dear World** jeszcze przed utworzeniem konta.
- Odłożenie `willow25` nie blokuje wdrożenia przewodników i zachowuje pełną gotowość do przyszłej synchronizacji P2P.
