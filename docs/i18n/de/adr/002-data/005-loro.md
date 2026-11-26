# 002-005. Loro-CRDT-Echtzeit-Co-Editing & Historienarchitektur

Datum: 2026-11-21
Status: Akzeptiert (vom Eigentümer gewählt; Umsetzung gekoppelt an `003-content`)
Übergeordnet: [002-data](../002-data.md)

## Kontext

Gemäß [002-003](003-crdt-report.md) wurde **Loro (`loro` `1.18.0`)** als CRDT-Engine für simultane Zusammenarbeit und konfliktfreies Offline-Merging auf Web, Desktop und Mobilgeräten gewählt.

## Entscheidung: Loro-Bausteine & Stack-Integration

### 1. Zuordnung zu ExoNovel-Objekten
- **`LoroTree` (Verschiebbarer Baum)**: Modelliert **`Buch -> Teile -> Kapitel -> Szenen`** sowie **Story-Bibel**-Ordner ohne Duplikate oder Zyklen bei parallelem Verschieben.
- **`LoroText` (Fugue + Peritext Rich Text)**: Manuskript-Prosa, Dialoge und Beschreibungen mit Markierungen (Fett, Kursiv, Links, Kommentaranker).
- **`LoroMap`, `LoroList` & `LoroMovableList`**: Strukturierte Figurenmerkmale, Zeitachsen-Koordinaten, POV-/Orts-Tags und Kommentarstränge mit `loro 1.18.0`-Sicherheitsprüfungen.
- **DAG-Versionshistorie & Zeitreise (`Frontiers` / `VersionVector` / `checkout`)**: Kausaler Bearbeitungsgraph für Entwurfshistorie, Autoren-Attribution und Forking.
- **`EphemeralStore`**: Live-Cursor, Textauswahl und Tippindikatoren ohne Aufblähen permanenter Historien.

### 2. Duale Persistenz & Transport (Neon & `willow25` + `iroh`)
- **In Neon PostgreSQL**: `crdt_updates` (inkrementelle signierte Deltas) und `crdt_snapshots` (kompaktierte Snapshots für schnelle Web-Hydrierung).
- **Live-Transport**: Native Clients streamen direkt über **`iroh-gossip` (`v0.102.0`)**; Webbrowser verbinden über **`wss://`**-WebSockets und ExoNovels Relay-Brücke in denselben `iroh-gossip`-Kanal.
- **Bidirektionale Portabilität (Neon ↔ Willow)**: Identische `loro 1.18.0`-Crate in `crates/server` und `crates/frontend` garantiert bitgenaue Binärkompatibilität.
