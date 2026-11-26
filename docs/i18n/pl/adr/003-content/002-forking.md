# 003-002. Części architektury fabularnej i uprawniony forking

Data: 2026-11-19
Status: Zaproponowano (otwarte do wywiadu)
Nadrzędny: [003-content](../003-content.md)

## Kontekst

Udostępnianie w ExoNovel koncentruje się na wyodrębnionych wymiarach historii—na przykład samej biografii postaci—aby inni autorzy mogli ją odkryć i wykorzystać we własnym utworze.

## Decyzja

1. **Pierwszorzędne elementy udostępniane**:
   - **Postacie**: Biografie, cechy, łuki fabularne, grafy relacji.
   - **Fabuły**: Sekwencje beatów, linie fabularne, konspekty strukturalne.
   - **Osie czasu / Wątki**: Zdarzenia chronologiczne i ciągłość wydarzeń.
   - **Motywy**: Motywy przewodnie, tezy i łuki symboliczne.
2. **Udostępnianie jak w Google Docs i edycja na żywo**:
   - Zaproszenia, linki, role (podgląd/komentarz/edycja) oraz współedycja na żywo.
3. **Uprawniony forking na nowego właściciela**:
   - Każdy element posiada flagę `allow_fork` określaną przez `owner_did`.
   - Dozwolony fork tworzy niezależną kopię z `owner_did` autora forkującego oraz zapisuje `forked_from_id` i `origin_owner_did` dla pełnego rodowodu.

## Otwarte pytania

- Jakie dokładne poziomy uprawnień powinny obowiązywać (Viewer, Commenter, Editor, Fork-Allowed)?
- Czy zforkowane elementy mają pozostać niezależne, czy oferować powiadomienia o zmianach źródłowych?
- Jak rozliczać atrybucję / tantiemy przy komercyjnym użyciu zforkowanej postaci?
