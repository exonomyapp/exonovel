# 005-001. Markdown-Reader & zustandsbehaftet einklapp-/skalierbare linke Navigation

Datum: 2026-11-19
Status: Akzeptiert
Übergeordnet: [005-docs-ui](../005-docs-ui.md)

## Entscheidung

1. **Linker Doku-Explorer**:
   - Zeigt die `docs/`-Hierarchie (übergeordnete ADRs gepaart mit Unter-ADR-Ordnern) sowie `AGENTS.md`.
   - **Zustandsbehaftet einklappbar**: Umschaltung im Kopf bzw. automatisches Einklappen nach Dokumentauswahl auf schmalen Mobilbildschirmen.
   - **Größenverstellbar**: Rechter Ziehgriff (Minimum `210px`, ohne Obergrenze, Tastatur `ArrowLeft`/`ArrowRight`, Doppelklick zum Zurücksetzen).
   - Live-Suche filtert Dateinamen, Titel und Überschriften.
2. **Markdown-Inhaltsbereich**:
   - Volle verfügbare Breite mit GitHub Flavored Markdown (Tabellen, Checklisten, Blockquotes, Codeblöcke mit Kopierfunktion, interne `.md`-Navigation, Überschriftsanker).
   - Fixierte rechte Gliederung **Auf dieser Seite**, Theme-Umschalter und Live-Aktualisierung.
3. **Google Docs & Drive-Arbeitsbereich**:
   - Verfügbar in **Präferenzen -> Verbundene Apps & Google Docs** für alle Konten sowie in der **Docs & ADRs**-Werkzeugleiste für berechtigte Lesende (`drive.file` + `userinfo.email`).
   - Ermöglicht Verbinden/Widerrufen ohne Abmeldung, Umwandlung von Markdown in bearbeitbare Google Docs, Ordnerverwaltung, Dateisuche/-vorschau/-umbenennung/-löschung, direkten ADR-Export sowie vollständige **`ExoNovel Docs`**-Archivsicherungen—stets mit Inline-Bestätigung ohne `window.alert`/`window.confirm`.
