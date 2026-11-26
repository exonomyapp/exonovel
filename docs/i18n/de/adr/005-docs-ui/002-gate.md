# 005-002. Privates Doku-Auth-Gate & Admin-Präferenzen

Datum: 2026-11-19
Status: Akzeptiert (in Umsetzung)
Übergeordnet: [005-docs-ui](../005-docs-ui.md)

## Entscheidung

1. **Öffentliche Startseite, `/signin`-Route & Auth-Gate (Neon Better Auth)**:
   - Dokumentation bleibt strikt privat und gehört ausschließlich zur eigenständigen **ExoNovel**-Anwendung (`https://exonovel.exosystems.net`). Doku-Endpunkte lehnen unbefugte Zugriffe mit `401`/`403` ab.
   - Nicht angemeldete Besuchende und Crawler erhalten unter `/` direkt eine serverseitig gerenderte öffentliche **ExoNovel**-Startseite (ohne Login-Sperre) mit Plattformbeschreibung, 4-Karten-Karussell, fünf Publikationsformaten und Links zu `/privacy`, `/terms` sowie `/signin`.
   - Klick auf **Anmelden** (oder `/signin` bzw. `?invite=`) öffnet die Anmeldung mit **Google**, **GitHub** und E-Mail/Passwort.
2. **Navbar-Benutzermenü, Mobile-Avatar-Leiste, Profil-Hub & Präferenzen**:
   - Auf Desktop und Mobilgeräten zeigt die obere Leiste den runden Avatar des Kontos direkt neben Sprach-, Benachrichtigungs- und Theme-Schaltflächen (ohne klassisches Hamburger-Menü). Das Dropdown bündelt mobile Navigationslinks (`.exo-mobile-nav-section`), den Link zu **Profil**, den Persona-Wechsel (die aktive Persona wird herausgefiltert; existieren keine weiteren Personas, erscheint **„Personas im Profil anlegen“** samt direktem **`Personas`**-Button), **Präferenzen** und Abmeldung. Auf Viewports `<= 768px` öffnen die Menüs für Sprache, Benachrichtigungen und Avatar als scrollbare Viewport-Sheets mit Abdunklungshintergrund.
   - **Präferenzen** verwalten Konto-Präferenzen, **Verbundene externe Apps** (getrennt vom Primär-Login), Design (`system`/`light`/`dark`), Sprachen sowie serverseitige 2-MB-Avatar-Uploads (`64`/`256`/`1024` WebP/JPEG).
3. **Admin-Allowlist & Kontorollen**:
   - Nur das Admin-Konto (`exonomyapp@gmail.com`) sieht die Admin-Bereiche für **Doku-Zugriffsliste** (`reader`, `moderator`, `pending`, `suspended`, `admin`) und **Registrierte Konten** (`user` < `moderator` < `admin`), jeweils identifiziert über `did:peer`.
4. **Lebenszyklus verbundener Apps (`001-auth` Abschnitt 7)**:
   - Jedes angemeldete Konto sieht unter **Präferenzen** verknüpfte SSO-Identitäten, optionale App-Freigaben (wie **Google Drive**), eine **Verbinden**-Aktion mit `login_hint` sowie **Trennen & Widerrufen** ohne Beendigung der Primärsitzung.
