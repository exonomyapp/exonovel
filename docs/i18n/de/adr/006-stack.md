# 006. Anwendungs-Stack: Eigenständiges Rust- & Leptos-Produkt

Datum: 2026-11-20
Status: Akzeptiert

## Kontext

**ExoNovel** (`https://exonovel.exosystems.net`, Repository `https://github.com/exonomyapp/exonovel`) ist ein vollständig von `exosystems.net` getrenntes Produkt mit eigener Codebasis, eigener Bereitstellung und eigenständigen Funktionen.

Die produktive ExoNovel-Anwendung—inklusive des integrierten privaten **Doku-Readers**—läuft **100 % in Rust und Leptos** ohne Node.js. Der Node.js/React-Teil im Repository dient ausschließlich der Vorschau in Umgebungen ohne Rust-Compiler.

## Entscheidung

### 1. Eigenständiges ExoNovel-Produkt & nativer Leptos-Doku-Reader
- **Full-Stack Rust & Leptos (`cargo-leptos` + Axum, Familie `0.8`)**:
  - **Rekursionslimit (`#![recursion_limit = "512"]`)**: In `crates/frontend`, `crates/app` und `crates/server` gesetzt, weil tief verschachtelte Leptos-`view!`-Makros das Standardlimit (`128`) von `rustc` überschreiten.
  - **Crate-Aufbau**: `crates/app` (gemeinsame UI + Router), `crates/frontend` (WASM-Hydrierung), `crates/server` (nativer Axum-Server unter `127.0.0.1:3001`).
  - **Build- & Paket-Pipeline**: GitHub Actions baut und testet auf `ubuntu-latest` (`exonovel-server` + `site/` + `docs/` + `AGENTS.md`) und überträgt das fertige Paket nach `/opt/exonovel` auf den Produktionsserver.
- **Native Rust-Anbindungen**:
  - **Neon PostgreSQL** via `sqlx` (`DATABASE_URL` / `DATABASE_URL_UNPOOLED`), **Neon Object Storage** via S3-API, **Neon Managed Better Auth** via JWKS (`jsonwebtoken`) und **dezentrale Identität/Sync** via `ed25519-dalek`, `willow25` und `iroh`.

### 2. Mobile Auslieferungsreihenfolge & Store-Konformität
- Android-Tests starten mit **F-Droid** (reproduzierbare **`foss`**-Variante ausschließlich mit **UnifiedPush**, ohne proprietäre Firebase-/Play-Services-Blobs), gefolgt von **Google Play** und **Apple App Store**.
- Öffentliche `/privacy`- und `/terms`-Endpunkte, Kontolöschungs-Parität und Regeln für externe Zahlungslinks sind für die App-Store-Einreichungen vorgesehen.

## Unter-ADRs

- [001-releases](006-stack/001-releases.md) — Abhängigkeits-Versionstracker, Upstream-Release-Watch & Upgrade-Protokoll
