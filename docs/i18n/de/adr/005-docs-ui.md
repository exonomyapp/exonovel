# 005. Doku-Reader-UI, Auth-Gate & Admin-Zugriffskontrolle

Datum: 2026-11-19
Status: Akzeptiert (in aktiver Umsetzung)

## Kontext

ExoNovels Produktdokumentation (`docs/` und `AGENTS.md`) ist **privat**—kein öffentlicher Teil der Website und vollständig getrennt von `exosystems.net`. Die eigenständige Anwendung **ExoNovel** (`https://exonovel.exosystems.net`) enthält ihren eigenen integrierten **Leptos**-Markdown-Doku-Reader hinter Neon Managed Better Auth und einer vom Admin gesteuerten Zugriffsliste.

## Übergreifende Architektur

1. **ExoNovel-exklusiver Umfang & Leptos-Umsetzung**: Der Doku-Reader läuft nativ in Rust/Leptos (`comrak` + `ammonia`-Bereinigung), gesichert durch Axum-Endpunkte gegen `NEON_AUTH_JWKS_URL`.
2. **GitHub-inspiriertes UI-Design („GitHubish“)**: Alle Oberflächenelemente—Navbar, Sprachumschalter, Benutzermenü, unterstrichene Profil-Tabs, geteilte Präferenzen, Markdown-Reader und MADR-CMS-Karten—folgen einem ruhigen, umrahmten GitHub-Stil mit System-/Hell-/Dunkelmodus, Tabellenziffern und ohne unerbetene Hinweistexte oder Badge-Flut.
3. **Standardisierter Start, allgemeine Präferenzen & privates Doku-Gate**: Alle angemeldeten Konten erhalten eine sachliche **ExoNovel**-Arbeitsbereich-Startseite sowie Zugang zu **Präferenzen** (persönliche Präferenzen, Avatar-Uploads, profilgestützte Rückkehrnavigation und **Verbundene externe Apps** wie Google Drive). Sobald `exonomyapp@gmail.com` oder ein freigeschalteter `did:peer`-Eintrag in `docs_access_allowlist` angemeldet ist, erscheinen **Docs Reader** und **Google Drive** in der Navigation; Site-Admin-Bereiche in den Präferenzen bleiben auf `exonomyapp@gmail.com` beschränkt.
4. **Reader, Explorer, Google-Drive-Archiv & Reifegrad-CMS**: Autorisierte Konten nutzen GitHub Flavored Markdown mit einklapp- und größenverstellbarem Navigationsbaum, Suche, rechter Überschriftsgliederung, Google-Drive-Arbeitsbereich und dem **Neon PostgreSQL MADR 4.0.0 & Tooling Maturity CMS** ([003-cms](005-docs-ui/003-cms.md)).

## Unter-ADRs

- [001-reader](005-docs-ui/001-reader.md) — Linker Doku-Explorer & Markdown-Reader-Oberfläche
- [002-gate](005-docs-ui/002-gate.md) — Better-Auth-Zugangsschranke & Admin-Präferenzen
- [003-cms](005-docs-ui/003-cms.md) — Datengesteuertes ADR- & Tooling-Reifegrad-CMS (MADR 4.0.0 + Neon PostgreSQL)
- [004-inference](005-docs-ui/004-inference.md) — Headless-`OpenCode`-Inferenzanbieter, Live-`/v1/models`-Erkennung & Failover
