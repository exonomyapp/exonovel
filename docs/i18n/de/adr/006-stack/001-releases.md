# 006-001. Abhängigkeits-Versionstracker & Upgrade-Protokoll

Datum: 2026-11-21
Status: Akzeptiert (Aktives lebendes Dokument)
Übergeordnet: [006-stack](../006-stack.md)

## Kontext

ExoNovel befindet sich in aktiver Vorentwicklung, während zentrale Rust- und Local-First-Bibliotheken reifen. Dieses Dokument verfolgt deklarierte Versionen, aktuelle Upstream-Stände und die konkret freigeschalteten Produktfähigkeiten.

---

## Aktueller Stack-Bestand

### 1. Aktive Laufzeit-Abhängigkeiten (`Cargo.toml` / CI)

| Technologie | Rolle in ExoNovel | Deklariert | Aktuell verifiziert | Upgrade-Nutzen & Auswirkung |
| :--- | :--- | :--- | :--- | :--- |
| **`leptos` / `leptos_meta` / `leptos_router` / `leptos_axum`** | Full-Stack Rust SSR + WASM UI | `0.8` | **`0.8.23`** *(und `0.9.0-rc1`)* | Bezieht automatisch `0.8.x`-Fixes; `0.9` wird nach Release geprüft |
| **`cargo-leptos`** | Build-CLI für Server + WASM | `latest` via `taiki-e/install-action@v2` | `0.2.x` | Vorkompiliertes Binary spart ~12 Min. pro CI-Lauf |
| **`axum`** | HTTP-API & SSR-Router | `0.8` | **`0.8.9`** | Routing-, Limit- und WebSocket-Stabilität |
| **`sqlx`** | Neon PostgreSQL Async-Treiber | `0.8` | **`0.9.1`** | Prüfung nach Toolchain-Abgleich |
| **`ed25519-dalek`** | `did:peer`-Schlüsselerzeugung & Signatur | `2` | **`3.0.0`** | Evaluierung für W3C-Proofs & NextGraph-Parität |
| **`image`** | Serverseitige Avatar-Verarbeitung (`64`/`256`/`1024`) | `0.25` | **`0.25.10`** | Reines Rust ohne C-Abhängigkeiten |
| **`comrak`** & **`ammonia`** | GFM-Markdown-Parser & HTML-Sanitizer | `0.35` / `4` | **`0.55.0`** / **`4.x`** | GFM-Tabellen, Aufgabenlisten und sichere Bereinigung |
| **Neon PostgreSQL & Managed Better Auth** | Cloud-DB & OAuth/E-Mail-Auth | Managed SaaS | Live | JWT-Prüfung, OAuth-Verknüpfung & Branching |

### 2. Geplanter Local-First-, Kollaborations- & Zahlungs-Stack

| Technologie | Freigeschaltete Fähigkeit | Upstream-Stand | Integrationsauslöser |
| :--- | :--- | :--- | :--- |
| **`willow25`** | Verschlüsselter lokaler Gerätespeicher & Sync | **`willow25 v0.7.11`** | Ersetzt veraltete `willow-data-model`/`meadowcap`-Crates |
| **`iroh` P2P (`iroh`, `iroh-docs`, `iroh-blobs`, `iroh-gossip`, `swarm-discovery`)** | QUIC-Relays, mDNS-LAN-Sync, Live-Präsenz | `iroh 1.x` / `iroh-docs 0.101.0` / `iroh-gossip 0.102.0` | Einbettung in Desktop-/Mobile-Builds |
| **`loro` CRDT-Engine** | Simultanes Co-Editing, verschiebbare Bäume, Zeitreise | **`loro 1.18.0` (Akzeptiert)** | Integration in Neon-Deltas & `willow25`/`iroh-gossip` |
| **`leptos-fluent` + Mozilla Fluent (`.ftl`)** | 29-Sprachen-UI & Eingabesprache, RTL-Spiegelung | **`leptos-fluent 0.3.1` (Akzeptiert)** | On-Demand-`.ftl`-Laden ohne WASM-Aufblähung |
| **Push & E-Mail (`web-push`, `a2`, `fcm_v1`, Brevo v3)** | 5 Plattformen + Web + Brevo-Multi-Sender | Aktiv (`notes@`/`noreply@`/`support@exosystems.net`) | Android-Test startet mit **F-Droid (`foss`)** |
| **Globale Zahlungsschienen (AOTA)** | Tier 1 (`dLocal`, `Stripe`, `Reown`), Tier 2, Tier 3 (`Airwallex`) | **Akzeptiert** | Alle Finanzfunktionen im **Profil** |
| **Live-UI (`PgListener` + SSE)** | Neuladefreie DOM-Updates (`<LiveRow>`, `<LiveField>`) | **`sqlx::PgListener` + SSE** | Siehe [009-live-ui](../009-live-ui.md) |
| **Headless-Inferenz (`OpenCode` + NVIDIA NIM)** | Optionale kreative Reifegrad-Vorschläge | `opencode run --standalone --auto --format json` | Siehe [005-003](../005-docs-ui/003-cms.md) & [005-004](../005-docs-ui/004-inference.md) |

---

## Laufendes Upgrade-Protokoll

| Datum | Bereich | Änderung | Begründung & Nutzen |
| :--- | :--- | :--- | :--- |
| **2026-11-25** | **Getrennte CMS-Tabs, `OpenCode` + NVIDIA NIM, `adr_translations` (`de`/`pl`), 100 % App-UI-i18n & Mobile-UI** | *(CMS-, i18n-, Inferenz- & Mobile-Release)* | **Tooling & Vorschläge** sauber von **KI-Inferenz** getrennt, Aktion **ADR aus Vorschlag entwerfen** ergänzt, nicht konfigurierte Anbieter sowie `gemini-2.5`/`qwen2.5` entfernt, sodass nur aktive **`OpenCode`-CLI** und **NVIDIA NIM** erscheinen. `adr_translations` (`UNIQUE(adr_id, locale)`), lokalisierte `/api/cms/adrs`-/Doc-Endpunkte und vollständige deutsche (`docs/i18n/de/`) sowie polnische (`docs/i18n/pl/`) Ausgaben aller 30 Dokumente umgesetzt. 100 % Fluent-Schlüsselabdeckung und responsive Breakpoints (`<= 960px`, `<= 768px`, `<= 480px`) hinzugefügt. |
| **2026-11-24** | **Verbundene Apps & Google Docs, öffentliche Startseite, Marken-Logo, Vendor-Neutralität & CI-Fixes** | *(OAuth-Konformität, Marke & CI)* | Serverseitiges Google-OAuth (`drive.file`), Google-Docs-Export/Archiv, öffentliche Startseite mit 4-Karten-Karussell, `/privacy` & `/terms`, transparentes ExoNovel-Logo, Bereinigung von Fremdmarken (`AGENTS.md` Regel 16) sowie `pipefail`- und Packager-Fixes umgesetzt. |
| **2026-11-23** | **CI-Test-Scoping, `#![recursion_limit = "512"]`, Check Runs & Dependabot-Hygiene** | `actions@v4 -> v7/v8`, `lockfile-only` | `ssr`- und `hydrate`-Prüfungen getrennt, Compiler-Warnungen bereinigt und automatische CI-Fehlerberichte aktiviert. |
| **2026-11-21** | **Global Live UI, Mode-2-Blind-Relay, `loro 1.18.0`, 29 Sprachen, Brevo & MADR CMS** | *(Architektur-Baseline)* | Grundlegende ADRs (`007`, `008`, `009`, `002-005`, `004-003`, `005-003`) und `.github/dependabot.yml` etabliert. |
