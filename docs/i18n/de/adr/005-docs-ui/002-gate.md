# 005-002. Privates Doku-Auth-Gate & Admin-Einstellungen

Datum: 2026-11-19
Status: Akzeptiert (in Umsetzung)
Übergeordnet: [005-docs-ui](../005-docs-ui.md)

## Entscheidung

1. **Öffentliche Startseite, `/signin`-Route & Auth-Gate (Neon Better Auth)**:
   - Dokumentation bleibt strikt privat und gehört ausschließlich zur eigenständigen **ExoNovel**-Anwendung (`https://exonovel.exosystems.net`). Doku-Endpunkte lehnen unbefugte Zugriffe mit `401`/`403` ab.
   - Nicht angemeldete Besuchende und Crawler erhalten unter `/` direkt eine serverseitig gerenderte öffentliche **ExoNovel**-Startseite (ohne Login-Sperre) mit Plattformbeschreibung, 4-Karten-Karussell, fünf Publikationsformaten und Links zu `/privacy`, `/terms` sowie `/signin`.
   - Klick auf **Anmelden** (oder `/signin` bzw. `?invite=`) öffnet die Anmeldung mit **Google**, **GitHub** und E-Mail/Passwort.
2. **Navbar-Benutzermenü, Profil-Hub & Einstellungen**:
   - Angemeldete Konten sehen ihren 64px-Avatar neben dem aktiven Persona-Namen; das Menü führt zu **Profil**, Persona-Wechsel, **Einstellungen** und Abmeldung.
   - **Einstellungen** verwaltet Kontoeinstellungen, **Verbundene externe Apps** (getrennt vom Primär-Login), Design (`system`/`light`/`dark`), Sprachen sowie serverseitige 2-MB-Avatar-Uploads (`64`/`256`/`1024` WebP/JPEG).
3. **Admin-Allowlist & Kontorollen**:
   - Nur das Admin-Konto (`exonomyapp@gmail.com`) sieht die Admin-Bereiche für **Doku-Zugriffsliste** (`reader`, `moderator`, `pending`, `suspended`, `admin`) und **Registrierte Konten** (`user` < `moderator` < `admin`), jeweils identifiziert über `did:peer`.
4. **Lebenszyklus verbundener Apps (`001-auth` Abschnitt 7)**:
   - Jedes angemeldete Konto sieht unter **Einstellungen** verknüpfte SSO-Identitäten, optionale App-Freigaben (wie **Google Drive**), eine **Verbinden**-Aktion mit `login_hint` sowie **Trennen & Widerrufen** ohne Beendigung der Primärsitzung.
