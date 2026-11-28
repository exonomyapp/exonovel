# 011. Entwurf für den Schreibtisch-Arbeitsbereich („Desk“: Demo-Platzhalter & geplante Module)

Datum: 2026-11-28
Status: Vorgeschlagen (interaktiver Demo-Entwurf in `crates/app/src/lib.rs` :: `WorkspaceHomeView`; wartet auf Prüfung & Interview durch den Eigentümer)
Verwandt: [003-content](003-content.md), [003-001](003-content/001-formats.md), [003-002](003-content/002-forking.md), [007-i18n](007-i18n.md), [010-dear-world](010-dear-world.md)

## Kontext

Nach der Anmeldung ersetzt ExoNovel den früheren Menüpunkt „Startseite“ durch **`Desk` (Schreibtisch)** (`ActiveTab::Workspace`). Gemäß Vorgabe des Eigentümers:
1. Entfernung der vier alten Schnellstart-Buttons (`Profil`, `Präferenzen`, `Docs & ADRs`, `Dear World`), da diese Ziele bereits in der Navbar bzw. im Benutzermenü liegen.
2. Entfernung der administrativen Metadatenzeilen (`ExoNovel Admin · Plan: free`, Klarname/Künstlername) aus dem Kopf des Schreibtischs.
3. Zusammenführung der früheren zwei Karten zu **einem einheitlichen Schreibtisch**, mit einem gekürzten `did:peer`-Chip samt Kopierbutton oben rechts.
4. Warme, nicht-technische Ansprache für **Schülerinnen/Schüler und Studierende**, fokussiert auf das eigene Schreiben statt auf Technik oder Sync-Tarife.
5. Konkrete **Arbeitsbereich-Platzhalter**, die geplante Funktionen sichtbar machen, bevor Datenbanktabellen und Editoren gebaut werden.

## Vorgeschlagene Architektur & aktueller Demo-Stand

Der aktuelle Demo-Code befindet sich in `WorkspaceHomeView` (`crates/app/src/lib.rs`, Zeilen 1125–1322) mit den zugehörigen Styles in `.exo-desk-*` (`style/main.css`, Zeilen 1286–1376). Derzeit läuft die Demo rein im Arbeitsspeicher des Browsers (noch ohne PostgreSQL-Speicherung), damit der Eigentümer den Aufbau direkt prüfen kann:

### 1. Schreibtisch-Kopf & `did:peer`-Chip (`lib.rs` Zeilen 1171–1193)
- Überschrift `Desk` und verständlicher Einführungstext für junge Schreibende; oben rechts die gekürzte `did:peer`-Vorschau mit Kopierbutton und vollständigem Tooltip.
- **Status**: Produktiv (liest die aktive Persona aus `/api/me`).

### 2. Platzhalter A — Format-Starter-Leiste („Start a New Project“, `lib.rs` Zeilen 1196–1231)
- Fünf auswählbare Formatkacheln passend zu [003-001](003-content/001-formats.md): **Romane**, **Artikel & Sachliteratur**, **Kurzgeschichten**, **Drehbücher & Skripte**, **Lyrik**.
- **Aktuelle Demo-Funktion**: Beim Klick wird die gewählte Kachel hervorgehoben und darunter eine Vorschau der zugehörigen Startstruktur angezeigt. Es wird noch kein Projekt in PostgreSQL angelegt.

### 3. Platzhalter B — Regal der Story-Bausteine („Your Shelf of Story Pieces“, `lib.rs` Zeilen 1233–1254)
- Vier Karten passend zu den wiederverwendbaren, teilbaren Bausteinen aus [003-002](003-content/002-forking.md): **Charaktere**, **Plots**, **Zeitachsen**, **Themen**.
- **Aktuelle Demo-Funktion**: Statische 2×2-Erklärungskarten; später direkte Verknüpfung zu eigenen und geteilten Bausteinen sowie Fork-Anfragen.

### 4. Platzhalter C — Schneller Ideen-Notizblock („Scratchpad“, `lib.rs` Zeilen 1256–1315)
- Niedrigschwelliges Notizfeld mit integrierter **`InputLocaleBar`** ([007-i18n](007-i18n.md)) für Schreibsprache und RTL/LTR-Richtung, Titelfeld, Textbereich sowie den Buttons **Idee anpinnen** und **Leeren**.
- **Aktuelle Demo-Funktion**: Hält den Text nur für die laufende Browser-Sitzung im Signal-Zustand (`scratch_title`, `scratch_body`, `scratch_saved`), ohne Speicherung in PostgreSQL.

## Offene Entscheidungsfragen an den Eigentümer

1. **Umfang & Reihenfolge**: Sollen alle drei Bereiche (Format-Starter, Story-Bausteine, Scratchpad) bleiben, vereinfacht oder z. B. um eine Liste *Aktuelle Geschichten & Einladungen* ergänzt werden?
2. **Verhalten der Format-Starter**: Soll ein Klick künftig direkt ein neues Projekt anlegen oder soll „Neues Projekt“ hinter einem Button liegen, damit bestehende Entwürfe zuerst erscheinen?
3. **Scratchpad-Persistenz**: Sollen angepinnte Notizen dauerhaft in PostgreSQL (`desk_notes`) gespeichert werden oder genügt ein späterer direkter Entwurfs-Editor?
4. **Platzierung der Story-Bausteine**: Gehören *Charaktere, Plots, Zeitachsen, Themen* direkt auf den Schreibtisch oder primär in den Projekt-Binder einzelner Geschichten?
