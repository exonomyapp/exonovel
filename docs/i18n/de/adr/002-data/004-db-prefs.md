# 002-004. Datenbank-zuerst-Präferenzen (Verzicht auf redundanten Browserspeicher)

Datum: 2026-11-21
Status: Vorgeschlagen (zur Eigentümer-Prüfung)
Übergeordnet: [002-data](../002-data.md)

## Kontext & Fragestellung

ExoNovel authentifiziert über Neon Managed Better Auth und speichert Benutzereinstellungen in **Neon PostgreSQL (`user_profiles`)** (`appearance_mode`, `preferred_format`, `locale`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, Benachrichtigungs-Flags). Redundantes Schreiben in Browser-`localStorage` erzeugt Drift zwischen Geräten.

## Antwort & Architekturentscheidung

**Sobald eine Anmeldung vorliegt, ist Browser-`localStorage` für persistente Nutzereinstellungen überflüssig und schädlich.**

### 1. Einzige Wahrheitsquelle: Neon PostgreSQL (`user_profiles`)
- Alle dauerhaften Einstellungen (`appearance_mode`, `ui_locale`, `default_input_locale`, `preferred_format`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, `docs_sidebar_collapsed`, `docs_sidebar_width`, `notify_prefs_json`) liegen ausschließlich in Neon PostgreSQL, werden mit `/api/me` geladen und über `PUT /api/profile` gespeichert.
- Serverseitiges Rendering (`shell()` in `crates/server/src/main.rs`) injiziert `<html lang="..." dir="..." data-appearance="...">` ohne `localStorage`.

### 2. Strikte Vor-Authentifizierungs-Ausnahmen
1. **Ausstehender Einladungscode (`sessionStorage`)**: Temporärer Wert (`exo_pending_invite`) über den OAuth-Redirect hinweg, unmittelbar nach `POST /api/invites/claim` gelöscht.
2. **Vor-Auth-Sprach-/Designauswahl**: Flüchtig im Arbeitsspeicher bzw. SSR-Cookie bis zur Übernahme in `user_profiles`.
3. **Lokaler Offline-Speicher (`willow25`)**: Auf nativen Geräten ohne Neon ersetzen reguläre Willow-Einträge unter `/personas/...` und `/profile` jeglichen `localStorage`.
