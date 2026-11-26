# 005. Interfejs Czytnika Dokumentacji, Bramka Logowania i Kontrola Dostępu Admina

Data: 2026-11-19
Status: Zaakceptowano (w aktywnym wdrożeniu)

## Kontekst

Dokumentacja produktowa ExoNovel (`docs/` oraz `AGENTS.md`) jest **prywatna**—nie stanowi publicznej części strony i jest całkowicie oddzielona od `exosystems.net`. Samodzielna aplikacja **ExoNovel** (`https://exonovel.exosystems.net`) posiada wbudowany **Czytnik Dokumentacji w Leptos** chroniony przez Neon Managed Better Auth i listę dostępu kontrolowaną przez administratora.

## Założenia nadrzędne

1. **Zakres wyłącznie w ExoNovel i implementacja w Leptos**: Czytnik działa natywnie w Rust/Leptos (`comrak` + `ammonia`) za punktami końcowymi Axum weryfikującymi tokeny Neon Better Auth (`NEON_AUTH_JWKS_URL`).
2. **Motyw inspirowany GitHub („GitHubish“)**: Wszystkie elementy UI—pasek nawigacji, selektor języka, menu użytkownika, podkreślane zakładki Profilu, boczne menu Ustawień, czytnik Markdown oraz karty CMS MADR—stosują spokojny, obramowany styl GitHub w trybie systemowym/jasnym/ciemnym, bez zbędnych nakładek i pigułek statusu.
3. **Przyjazny ekran startowy, powszechne Ustawienia i prywatna dokumentacja**: Każdy zalogowany użytkownik widzi ekran roboczy oraz pełne **Ustawienia** (preferencje, awatary, **Połączone aplikacje zewnętrzne** takie jak Google Drive). Po zalogowaniu `exonomyapp@gmail.com` lub wpisu `did:peer` z `docs_access_allowlist` w pasku pojawiają się **Dokumentacja** i **Google Drive**, a panele administratora pozostają ograniczone do `exonomyapp@gmail.com`.
4. **Czytnik, Eksplorator, Archiwum Google Drive i CMS Dojrzałości**: Zgodnie z [003-cms](005-docs-ui/003-cms.md) czytnik działa jako **CMS MADR 4.0.0 i Dojrzałości Narzędzi w Neon PostgreSQL** ze zwijanym drzewem nawigacji, wyszukiwaniem, spisem treści i eksportem do Google Docs.

## Dokumenty podrzędne (Sub-ADRs)

- [001-reader](005-docs-ui/001-reader.md) — Lewy eksplorator dokumentów i interfejs czytnika Markdown
- [002-gate](005-docs-ui/002-gate.md) — Bramka logowania Better Auth i ustawienia administratora
- [003-cms](005-docs-ui/003-cms.md) — Oparty o dane CMS MADR 4.0.0 i Dojrzałości Narzędzi (Neon PostgreSQL)
- [004-inference](005-docs-ui/004-inference.md) — Bezserwerowi dostawcy inferencji `OpenCode`, odkrywanie `/v1/models` i failover
