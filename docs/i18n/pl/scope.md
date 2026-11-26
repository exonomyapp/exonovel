# ExoNovel — Zakres produktu

> **Status**: Wersja robocza oparta na wywiadzie wstępnym i badaniu formatów publikacji. Otwarta na dalsze doprecyzowanie.

---

## 1. Wizja produktu i koncepcja główna

**ExoNovel** to platforma i aplikacja do wspólnego pisania twórczego, w której **każdy współtwórca zachowuje pełną własność swoich treści**, nawet podczas pracy zespołowej nad wspólnymi projektami.

### Filary systemu

- **Indywidualna własność treści w projektach zespołowych**: Autorzy zachowują prawa do tworzonych przez siebie elementów i fragmentów nawet w ramach współdzielonych projektów.
- **Modułowe / wymiarowe udostępnianie**: System wspiera niezależne udostępnianie poszczególnych wymiarów projektu—np. opublikowanie lub udostępnienie biografii jednej postaci tak, aby inni autorzy mogli ją odkryć i za zgodą włączyć do własnych powieści.
- **Współpraca w stylu Google Docs i edycja na żywo**: Standardowe mechanizmy udostępniania i uprawnień połączone z jednoczesną edycją w czasie rzeczywistym.
- **Planowanie i tworzenie skoncentrowane na pisarzach**: Interfejs i procesy dopasowane do tego, jak pisarze faktycznie planują, porządkują i piszą utwory (powieści, scenariusze i inne formaty).

---

## 2. Zakres platformowy i techniczny

### Docelowe platformy wdrożenia

1. **Strona WWW (Aplikacja webowa)** — Pierwsze główne wydanie.
2. **Aplikacje desktopowe** (wszystkie trzy główne systemy):
   - **Linux**
   - **macOS**
   - **Windows**
3. **Aplikacje mobilne** (oba główne systemy mobilne):
   - **Android**
   - **iOS**

### Przechowywanie danych

- **Baza danych**: Hostowane rozwiązanie **PostgreSQL** (`pgsql`).

---

## 3. Pięć głównych formatów publikacji i naturalne struktury

Na podstawie zasięgu rynkowego i aktywności społeczności pisarskich zdefiniowano pięć podstawowych formatów publikacji dla ExoNovel wraz z ich naturalną strukturą **planowania** oraz **publikacji**.

### Format 1: Powieści (Długa proza fabularna i non-fiction)

Długie formy narracyjne stanowiące główny trzon publikacji tradycyjnych, niezależnych i self-publishingu.

#### Naturalna struktura planowania i pisania

- **Bazowa Biblia Fabuły**:
  - **Postacie**: Profile, biografie, łuki przemiany, motywacje, wady, rany i sieci relacji.
  - **Światotworzenie / Biblia Świata**: Lokacje, osie czasu, zasady i historia świata (trzymane w tle jako „góra lodowa“ podczas pisania).
  - **Fabuła / Konspekt**: Plan nadrzędny (np. struktura trzyaktowa, beat sheet, metoda płatka śniegu lub konspekt punktowy) mapujący punkt zwrotny, punkt środkowy, kulminację i rozwiązanie.
- **Hierarchia szkicu**:
  - **Części / Akty** (duże przesunięcia czasu, perspektywy lub fazy narracji)
  - **Rozdziały** (jednostki tempa, zazwyczaj 1 500–5 000 słów)
  - **Sceny i Sekwele** (atomowe jednostki akcji/konfliktu połączone z reakcją i przetworzeniem przez postać)

#### Struktura publikacji wyjściowej

- Strony wstępne (strona tytułowa, dedykacja, motto, spis treści)
- Uporządkowane **Części** (opcjonalnie) i **Rozdziały** z płynnymi przejściami
- Posłowie, podziękowania i biogram autora

---

### Format 2: Artykuły i Literatura Faktu (Eseje, Wspomnienia, Reportaże)

Poczytna literatura faktu obejmująca reportaż narracyjny, eseje osobiste, komentarz kulturowy i wspomnienia.

#### Naturalna struktura planowania i pisania

- **Główny kąt i cel**: Teza przewodnia (esej), haczyk informacyjny (artykuł) lub tematyczna prawda (wspomnienia/reportaż).
- **Konspekt strukturalny**:
  1. **Otwarcie / Lead**: Scena zmysłowa, zaskakująca dana lub streszczenie tezy.
  2. **Przejście / Kontekst**: Płynne przejście od otwarcia do sedna tematu.
  3. **Sekcje główne**: Argumenty, dowody, anegdoty lub sceny pod śródtytułami (`H2` / `H3`) w układzie odwróconej piramidy lub logicznej progresji.
  4. **Punkt zwrotny / Synteza**: Odpowiedź na kontrargumenty lub szczytowe wnioski.
  5. **Podsumowanie**: Synteza lub zapadający w pamięć obraz końcowy.
- **Notatki badawcze i źródła**: Przypisy, wywiady i weryfikacja faktów przypięte do sekcji.

#### Struktura publikacji wyjściowej

- Nagłówek główny i podtytuł
- Linia autorska i metadane
- Wstęp, śródtytułowane sekcje (z cytatami i źródłami) oraz zakończenie

---

### Format 3: Opowiadania i Flash Fiction

Zwięzła proza narracyjna od flash fiction (poniżej 1 000 słów) po klasyczne opowiadania (ok. 3 000–8 000 słów).

#### Naturalna struktura planowania i pisania

- **Pojedynczy rdzeń konfliktu**: Jedno główne napięcie, zwarta oś czasu i ograniczona obsada.
- **Skupiony konspekt**: Otwarcie *in media res*, oszczędne tło („technika góry lodowej“), eskalacja, punkt kulminacyjny i rezonans końcowy.
- **Grupowanie w antologie** (opcjonalnie): Łączenie powiązanych utworów w zbiory.

#### Struktura publikacji wyjściowej

- Tytuł, autor i zwarta proza (pojedyncza scena lub separatory `* * *`), publikowane osobno lub jako **Antologia / Zbiór opowiadań**.

---

### Format 4: Scenariusze (Film, Telewizja, Audio/Podcast)

Wizualno-dźwiękowe plany realizacyjne o ścisłym standardzie branżowym, gdzie ok. **1 strona scenariusza odpowiada 1 minucie czasu ekranowego**.

#### Naturalna struktura planowania i pisania

- **Dokumenty przedscenariuszowe**: Logline i przesłanka, treatment w czasie teraźniejszym (2–15+ stron), beat sheet i karty scen, zestawienia postaci i lokacji.
- **Elementy blokowe skryptu**: Nagłówki scen (`[INT./EXT.] [LOKACJA] - [PORA DNIA]`), linie akcji w czasie teraźniejszym, nazwy postaci (`(V.O.)`, `(O.S.)`), dialogi, didaskalia i przejścia.

#### Struktura publikacji / produkcji

- Strona tytułowa oraz standardowy układ czcionką stałoszerokościową (12pt Courier / Courier Prime, lewy margines 1,5", pozostałe 1") gotowy do prób czytanych i produkcji.

---

### Format 5: Poezja (Pojedyncze wiersze, Arkusze i Tomiki)

Twórczość oparta na wersach i strofach—od form klasycznych (sonety, villanelle, haiku) po wiersz wolny i spoken word.

#### Naturalna struktura planowania i pisania

- **Warsztat wersu i strofy**: Podział wersów, * przerzutnie*, strofy, rytm/metrum i zwrot (*volta*).
- **Kompozycja tomu**: Dobór i układ wierszy w **Mikro-arkusz** (~5–10), **Arkusz poetycki** (~15–30) lub **Pełny tom poetycki** (~40–80+ utworów).

#### Struktura publikacji wyjściowej

- **Pojedynczy wiersz**: Dokładne zachowanie wersyfikacji, wcięć i światła między strofami.
- **Tomik / Arkusz**: Strona tytułowa, spis treści, sekcje, każdy utwór na nowej stronie oraz posłowie.

---

## 4. Otwarte pytania do wywiadów

- Czy którykolwiek z tych **pięciu** formatów powinien zostać zmodyfikowany, zastąpiony lub przestawiony?
- Od którego z pięciu formatów rozpocząć szczegółowy wywiad dotyczący wymiarów planowania i reguł współwłasności?
