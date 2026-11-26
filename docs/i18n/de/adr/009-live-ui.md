# 009. Globale App-weite Live-UI-Updates & Eingangsanimationen

Datum: 2026-11-21
Status: Akzeptiert (Globale Architekturregel über alle ExoNovel-Ansichten)

## Kontext

Sobald neue oder geänderte Daten in **Neon PostgreSQL** geschrieben werden oder in einem lokalen **`willow25`-Store** eintreffen (etwa über `iroh`-P2P-Sync auf Desktop und Mobilgeräten), darf kein manuelles Neuladen erforderlich sein. Neu eintreffende Zeilen und aktualisierte Felder erhalten eine dezente, lokale Eingangsbewegung.

## Entscheidung

### 1. Speicherunabhängiger Live-Änderungsbus
Sowohl der zentrale Web-Server als auch native `willow25`-Installationen senden identische typisierte **`LivePatchEvent`**-Nachrichten (`entity_kind`, `entity_id`, `scope_key`, `op`, `changed_fields`, `payload`, `seq_ms`):
- **Quelle A — Neon PostgreSQL `LISTEN/NOTIFY` (`DATABASE_URL_UNPOOLED`)**: Ein dauerhafter `PgListener` speist einen `tokio::sync::broadcast`-Kanal hin zum autorisierten **Axum-SSE-Stream (`/api/events/stream`)**.
- **Quelle B — Nativer `willow25` / `iroh` Store-Watcher**: Beobachtet lokale und über `iroh-docs`/`iroh-blobs` empfangene Schreibvorgänge und emittiert dasselbe `LivePatchEvent`.

### 2. Globale Abgleichsschicht in Leptos (`LiveSyncContext`)
Auf `<App/>`-Ebene eingehängt, aktualisiert `LiveSyncContext` geöffnete Listen gezielt und stabil ohne Sprünge der Scrollposition (`insert`, `update`, `delete`).

### 3. Visuelles Bewegungssystem
Geteilte Komponenten (`<LiveRow>` und `<LiveField>` in `style/main.css`) nutzen `.exo-live-insert` (sanftes Einblenden + kurzes Akzent-Leuchten) sowie `.exo-live-update` und respektieren `@media (prefers-reduced-motion: reduce)`.
