# 005-001. Czytnik Markdown oraz stanowo zwijana i skalowalna nawigacja lewa

Data: 2026-11-19
Status: Zaakceptowano
Nadrzędny: [005-docs-ui](../005-docs-ui.md)

## Decyzja

1. **Lewy eksplorator dokumentów**:
   - Wyświetla hierarchię `docs/` (nadrzędne ADR wraz z podfolderami) oraz `AGENTS.md`.
   - **Stanowo zwijany**: Przełącznik w nagłówku oraz automatyczne zwinięcie po wyborze dokumentu na wąskich ekranach mobilnych.
   - **Skalowalny**: Uchwyt zmiany szerokości po prawej krawędzi (minimum `210px`, obsługa strzałek klawiatury i dwukliku resetującego).
   - Wyszukiwanie na żywo po nazwach plików, tytułach i nagłówkach.
2. **Obszar treści Markdown**:
   - Pełna szerokość robocza z obsługą GitHub Flavored Markdown (tabele, listy zadań, cytaty, bloki kodu z kopiowaniem, linki wewnętrzne `.md`, kotwice nagłówków) i prawym spisem **Na tej stronie**.
3. **Przestrzeń Google Docs i Drive**:
   - Dostępna w **Preferencje -> Połączone aplikacje i Google Docs** dla wszystkich kont oraz na pasku **Dokumentacja i ADR** dla uprawnionych czytelników (`drive.file` + `userinfo.email`).
   - Pozwala łączyć lub odłączać konto Google bez wylogowania z aplikacji, konwertować Markdown do edytowalnych Dokumentów Google, zarządzać plikami/folderami, eksportować pojedyncze ADR i wykonywać pełną kopię zapasową **`ExoNovel Docs`**—wszystkie akcje destrukcyjne używają potwierdzeń w interfejsie zamiast `window.alert`/`window.confirm`.
