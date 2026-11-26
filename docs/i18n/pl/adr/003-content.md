# 003. Modele treści i udostępnianie architektury fabuły

Data: 2026-11-19
Status: Zaproponowano (otwarte do szczegółowych wywiadów)

## Kontekst

Pisarze porządkują materiały inaczej w zależności od formatu publikacji, a jednocześnie współpraca w ExoNovel opiera się na modułowych klockach fabularnych, które można udostępniać, współedytować i wykorzystywać ponownie bez utraty autorstwa.

## Założenia nadrzędne

1. **Tworzenie natywne dla formatu**: Obsługa pięciu głównych formatów (**Powieści**, **Artykuły i Literatura Faktu**, **Opowiadania i Flash Fiction**, **Scenariusze**, **Poezja**) z rozróżnieniem **struktury planowania** od **struktury publikacji**.
2. **Modułowa architektura fabuły**: **Postacie**, **Fabuły**, **Osie czasu / Wątki** i **Motywy** jako niezależnie udostępniane byty należące do `did:peer`.
3. **Uprawniony forking**: Za wyraźną zgodą właściciela `did:peer` inny autor może wykonać **fork** elementu do własnego projektu pod nowym `owner_did` z zachowaniem atrybucji twórcy źródłowego (`origin_owner_did`).

## Dokumenty podrzędne (Sub-ADRs)

- [001-formats](003-content/001-formats.md) — 5 głównych formatów publikacji oraz struktury planowania i wydania
- [002-forking](003-content/002-forking.md) — Części architektury fabularnej i uprawniony forking
