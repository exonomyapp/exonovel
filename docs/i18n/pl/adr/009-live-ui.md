# 009. Globalne aktualizacje UI na żywo i animacje pojawiania się treści

Data: 2026-11-21
Status: Zaakceptowano (Globalna reguła architektoniczna w całej aplikacji)

## Kontekst

Gdy nowe lub zmienione dane trafią do **Neon PostgreSQL** albo lokalnego magazynu **`willow25`** (np. poprzez synchronizację P2P `iroh`), użytkownik nigdy nie powinien przeładowywać widoku. Nowe wiersze i zaktualizowane pola otrzymują subtelne, punktowe wyróżnienie wejścia.

## Decyzja

### 1. Szyna zdarzeń niezależna od magazynu
Zarówno serwer WWW, jak i natywne instalacje `willow25` emitują ten sam typ zdarzenia **`LivePatchEvent`** (`entity_kind`, `entity_id`, `scope_key`, `op`, `changed_fields`, `payload`, `seq_ms`):
- **Źródło A — Neon PostgreSQL `LISTEN/NOTIFY` (`DATABASE_URL_UNPOOLED`)**: Odporny proces `PgListener` zasila kanał `tokio::sync::broadcast` i autoryzowany strumień **Axum SSE (`/api/events/stream`)**.
- **Źródło B — Natywny obserwator `willow25` / `iroh`**: Śledzi zapisy lokalne oraz przychodzące przez `iroh-docs` / `iroh-blobs`.

### 2. Globalna rekoncyliacja w Leptos (`LiveSyncContext`)
Zamontowana na poziomie `<App/>` aktualizuje otwarte listy punktowo w miejscu (`insert`, `update`, `delete`) bez resetu przewijania.

### 3. System animacji przybycia (CSS)
Współdzielone prymitywy (`<LiveRow>` i `<LiveField>` w `style/main.css`) stosują `.exo-live-insert` oraz `.exo-live-update`, z poszanowaniem `@media (prefers-reduced-motion: reduce)`.
