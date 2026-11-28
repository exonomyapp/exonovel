# 002-004. Datenbank-zuerst-Präferenzen (Verzicht auf redundanten Browserspeicher)

Datum: 2026-11-27
Status: Akzeptiert (PostgreSQL `user_profiles` + Muster für unsichtbare UI-Präferenzen)
Übergeordnet: [002-data](../002-data.md)

## Kontext & Fragestellung

ExoNovel authentifiziert über Managed Better Auth und speichert Benutzerpräferenzen in **PostgreSQL (`user_profiles`)** (`appearance_mode`, `preferred_format`, `locale`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, Benachrichtigungs-Flags). Redundantes Schreiben in Browser-`localStorage` erzeugt Drift zwischen Geräten. Zudem teilen sich **Profil** und **Präferenzen** künftig dieselbe integrierte (nicht schwebende), dauerhaft größenveränderbare linke Navigationsleiste, deren Breite ebenfalls ohne Browserspeicher und ohne zusätzliche Formularfelder gespeichert wird.

## Antwort & Architekturentscheidung

**Sobald eine Anmeldung vorliegt, ist Browser-`localStorage` für persistente Nutzerpräferenzen und Layout-Zustände ausgeschlossen.**

### 1. Einzige Wahrheitsquelle: PostgreSQL (`user_profiles`)
- Alle dauerhaften Präferenzen sowie **unsichtbare UI-Präferenzen** (`appearance_mode`, `ui_locale`, `default_input_locale`, `preferred_format`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, `docs_sidebar_collapsed`, `docs_sidebar_width`, `profile_nav_width` für **Profil**, `preferences_nav_width` für **Präferenzen**, `notify_prefs_json`) liegen ausschließlich in PostgreSQL, werden mit `/api/me` geladen und über `PUT /api/profile` gespeichert. Da **Profil** und **Präferenzen** unterschiedliche Inhalte haben, wird die Navigationsbreite getrennt pro Seite gespeichert.
- Serverseitiges Rendering (`shell()` in `crates/server/src/main.rs`) injiziert `<html lang="..." dir="..." data-appearance="...">` direkt ohne `localStorage`.

### 2. Muster für „unsichtbare Präferenzen“ (direkte Manipulation ohne Formular-Ballast)
- Ergonomie-Zustände, die direkt auf der Seite gesteuert werden (z. B. das Ziehen oder Tastatur-Verschieben der integrierten, nicht schwebenden linken Navigationsleiste getrennt auf **Profil** (`profile_nav_width`) und **Präferenzen** (`preferences_nav_width`) via `role="separator"`, das Einklappen/Skalieren der **Docs & ADRs**-Seitenleiste oder der zuletzt geöffnete Bereich/Pfad), werden in PostgreSQL persistiert — **ohne** zusätzliche Eingabefelder oder Schalter in **Präferenzen**-Formularen anzuzeigen.
- **Einheitliches Einstellungs-Layout**: Sowohl **Profil** (anstelle horizontaler Tabs) als auch **Präferenzen** nutzen dieselbe nicht schwebende, direkt in die Seite integrierte und pro Seite getrennt größenveränderbare linke Navigationsleiste (`ExoSettingsPage` + `ExoSettingsSplit`).

### 3. Strikte Vor-Authentifizierungs-Ausnahmen
1. **Ausstehender Einladungscode (`sessionStorage`)**: Temporärer Wert (`exo_pending_invite`) über den OAuth-Redirect hinweg, unmittelbar nach `POST /api/invites/claim` gelöscht.
2. **Vor-Auth-Sprach-/Designauswahl**: Flüchtig im Arbeitsspeicher bis zur Übernahme in `user_profiles`.
3. **Lokaler Offline-Speicher (`willow25`)**: Auf nativen Geräten ersetzen reguläre Willow-Einträge unter `/personas/...` und `/profile` jeglichen `localStorage`.
