# 006. Stos technologiczny aplikacji: Samodzielny produkt Rust i Leptos

Data: 2026-11-20
Status: Zaakceptowano

## Kontekst

**ExoNovel** (`https://exonovel.exosystems.net`, repozytorium `https://github.com/exonomyapp/exonovel`) jest produktem niezależnym od `exosystems.net` z własną bazą kodu, wdrożeniem i funkcjami.

Produkcja ExoNovel—wraz z wbudowanym prywatnym **Czytnikiem Dokumentacji**—działa **w 100% w Rust i Leptos** bez Node.js. Warstwa Node.js/React w repozytorium służy wyłącznie do podglądu w kontenerach pozbawionych kompilatora Rust.

## Decyzja

### 1. Samodzielny produkt ExoNovel i natywny czytnik Leptos
- **Full-Stack Rust i Leptos (`cargo-leptos` + Axum, rodzina `0.8`)**:
  - **Limit rekurencji (`#![recursion_limit = "512"]`)**: Zadeklarowany w `crates/frontend`, `crates/app` i `crates/server`, ponieważ zagnieżdżone makra `view!` przekraczają domyślny limit `128` kompilatora `rustc`.
  - **Struktura crate'ów**: `crates/app` (współdzielone UI i router), `crates/frontend` (hydracja WASM), `crates/server` (serwer Axum na `127.0.0.1:3001`).
  - **Potok budowania i pakowania**: GitHub Actions kompiluje i testuje wydanie w środowisku CI (`exonovel-server` + `site/` + `docs/` + `AGENTS.md`), po czym wdraża gotową paczkę do produkcyjnego katalogu aplikacji na serwerze produkcyjnym.
- **Natywne połączenia Rust**:
  - **Neon PostgreSQL** przez `sqlx` (`DATABASE_URL` / `DATABASE_URL_UNPOOLED`), **Neon Object Storage** przez S3 API, **Neon Managed Better Auth** przez JWKS (`jsonwebtoken`) oraz tożsamość/synchronizacja P2P przez `ed25519-dalek`, `willow25` i `iroh`.

### 2. Kolejność dystrybucji mobilnej i zgodność ze sklepami
- Testy na Androidzie startują od **F-Droid** (powtarzalny wariant **`foss`** wyłącznie z **UnifiedPush**, bez binarnych bibliotek Google Play / Firebase), następnie **Google Play** i **Apple App Store**.
- Publiczne adresy `/privacy` i `/terms`, funkcja usuwania konta oraz obsługa reguł płatności zewnętrznych wspierają przyszłe publikacje w sklepach.

## Dokumenty podrzędne (Sub-ADRs)

- [001-releases](006-stack/001-releases.md) — Rejestr wersji zależności, monitor wydań i dziennik aktualizacji
