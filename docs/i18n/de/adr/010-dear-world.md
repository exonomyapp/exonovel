# 010. „Dear World“-Benutzerhandbücher, PostgreSQL-Speicher & Öffentliches Onboarding

Datum: 2026-11-27
Status: Akzeptiert (PostgreSQL-Speicher mit `willow_*`-Koordinaten aktiv; `willow25`-Peer-Replikation bis zum nativen Geräte-Rollout zurückgestellt)

## Kontext

ExoNovel führt junge Autorinnen und Autoren an Schulen und Universitäten in eine neue Art des gemeinsamen Schreibens ein: Geschichten-Bausteine (`Figuren`, `Handlungen`, `Zeitleisten`, `Themen`), mehrere `Personas`, lizenzierbares Remixing und nahtlose Zusammenarbeit zwischen Webbrowser und Offline-Geräte-Apps. Angelehnt an GitHub Docs' **„Hello World“**-Leitfaden (`https://docs.github.com/en/get-started/using-github/hello-world`) benötigt ExoNovel ein eigenes, herzliches **Benutzerhandbuch** — getrennt von internen Architektur-ADRs und öffentlich zugänglich ohne vorherige Anmeldung.

Da die direkte `willow25` (`v0.7.11`)-Peer-Replikation bis zum Start der nativen Desktop- und Mobile-Clients zurückgestellt bleibt, laufen die Handbücher heute direkt über PostgreSQL und tragen bereits alle Koordinaten für die spätere Willow-Synchronisation.

## Entscheidung

1. **Redaktionelles Konzept „Dear World“**:
   - Name: **„Dear World“**, geschrieben im warmen, ermutigenden Ton einer erfahrenen Autorin bzw. eines erfahrenen Autors.
   - Inhaltlicher Fokus ausschließlich auf **die Nutzung von ExoNovel für eine neue Art des Schreibens** — keine allgemeinen Schreibkurse.
   - Startumfang mit 4 kompakten Kapiteln in Englisch (`docs/guides/`), Deutsch (`docs/i18n/de/guides/`) und Polnisch (`docs/i18n/pl/guides/`): `01-dear-world`, `02-first-story-and-blocks`, `03-personas-groups-and-ownership`, `04-web-device-and-sharing`.

2. **Öffentlicher Zugang & Einziger Navigations-Einstieg (Nur obere Navigationsleiste)**:
   - Öffentlich unter `/dear-world` (`/dear-world/{slug}`) sowie über `GET /api/guides` und `GET /api/guides/{slug}` **ohne Anmeldung** abrufbar.
   - **`Dear World`** befindet sich an **genau einer Stelle**: direkt in der oberen Navigationsleiste (inkl. mobilem Navigationsmenü) für abgemeldete Besucherinnen und angemeldete Schreibende (angemeldet neben **`Schreibtisch`**, ohne „Home“/„Startseite“-Button). Keine doppelten Einstiegs-Buttons im Schreibtisch, im Footer oder auf Unterseiten unter derselben Navigationsleiste.

3. **PostgreSQL-Datenmodell mit vorbereiteten `willow25`-Koordinaten (`crates/server/src/guides.rs`)**:
   - Tabellen `dear_world_guides` und `dear_world_guide_translations` (`PRIMARY KEY (slug, locale)`) speichern bereinigtes GFM-HTML (`comrak` + `ammonia`), Metadaten und deterministische `willow_*`-Felder (`willow_namespace = 'exonovel'`, `willow_subspace = 'public/dear-world'`, `willow_path = '/dear-world/{slug}'`, `willow_timestamp_us`).
   - Damit schließt sich „Dear World“ bei Aktivierung von `willow25` ohne Schema-Migration nahtlos an ExoNovels öffentlichen schreibgeschützten Willow-Store an.

## Konsequenzen

- Neue Schreibende können **Dear World** vor der Kontoerstellung lesen und direkt vom Leitfaden in die Anmeldung oder an ihren **Schreibtisch** wechseln.
- Die spätere Anbindung an `willow25` bleibt ohne Umstrukturierung garantiert.
