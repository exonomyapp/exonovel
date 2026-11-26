# 002-003. Entscheidungsbericht: Globale Echtzeit-Co-Editing-Engines (CRDT vs. OT)

Datum: 2026-11-21
Status: Akzeptiert (Option A — `loro 1.18.0` vom Eigentümer gewählt; siehe [005-loro](005-loro.md))
Übergeordnet: [002-data](../002-data.md)

## Zweck dieses Berichts

ExoNovel benötigt gleichzeitiges Co-Editing im Stil von Google Docs über Web (WASM/Leptos), Desktop (Linux, macOS, Windows) und Mobilgeräte (Android, iOS) sowie offlinefähige Bearbeitung mit späterer konfliktfreier Zusammenführung unter Rust + **Willow-Protokoll (`willow25`)**.

---

## Evaluierte Open-Source-Kandidaten (Globaler Markt)

### 1. Loro (`loro` — Weltweites Open Source aus China, Rust + WASM)
- **Herkunft & Version**: Team Loro (MIT-Lizenz). Verifizierte Rust-Crate: **`1.18.0`**.
- **Funktionsweise**: Nativer Rust-Kern kompiliert nach WASM und nativen Targets; **Fugue**-Rich-Text-CRDT plus DAG-Versionshistorie.
- **Besondere Stärken für ExoNovel**:
  - **Erstklassige verschiebbare Bäume (Movable Trees)**: Gleichzeitiges Verschieben von Kapiteln zwischen Teilen oder Neuordnen von Szenen und Story-Bibel-Ordnern ohne Duplikate.
  - **Integrierte Zeitreise & Versionierung**: Autorenbezogenes Rückgängigmachen, Historien-Snapshots und Branchen/Mergen.
  - **Rich Text (Peritext-inspiriert)** & **kompakte binäre Snapshots/Deltas** für **`iroh-blobs` / `willow25`**.
- **Abwägungen**: Größeres WASM-Paket (~150–300 kB gzipped vs. ~18 kB bei Yjs) und jüngeres Editor-Plugin-Ökosystem.

### 2. Yrs / Yjs (`yrs` — Open Source aus Deutschland, Rust + JS/WASM)
- **Herkunft & Version**: Kevin Jahns (MIT-Lizenz). `yrs` ist der offizielle Rust-Port (`0.28.1`).
- **Stärken**: Breites Editor-Ökosystem (TipTap, ProseMirror, Lexical, CodeMirror, Monaco) und winzige Web-Größe.
- **Abwägungen**: Kein nativer Movable-Tree-CRDT und standardmäßiges Verwerfen der vollständigen Bearbeitungshistorie.

### 3. Automerge (`automerge` — UK / Ink & Switch, Rust + WASM)
- **Version**: **`0.12.0`**, Git-ähnlicher unveränderlicher Änderungsgraph (`automerge-repo`).
- **Abwägungen**: Höherer Speicherbedarf auf sehr großen Einzeldokumenten und kein integrierter Movable-Tree-Typ.

### 4. Diamond Types (`diamond-types` — Australien, Rust) & 5. Cola (`cola`)
- Extreme Durchsatzrate für **reinen Klartext** ohne Rich-Text-Markierungen oder Bäume.

### 6. Zentrales Operational Transform (`ShareDB`)
- Erfordert zentralen Koordinationsserver, widerspricht Offline-First und nativem Rust/`iroh`.

---

## Vergleich anhand der ExoNovel-Anforderungen

- **Nativ in Rust Server + WASM-Leptos kompilierbar**: Loro, Yrs, Automerge, Diamond Types und Cola.
- **Abdeckung von Romanstrukturen (`Teile -> Kapitel -> Szenen`)**: **Loro** bietet verschiebbare Bäume, Rich Text und Zeitreise nativ aus einer Hand.

## Entscheidungsoptionen

- **Option A — Loro (`1.18.0`)**: Empfohlen für **Kapitel-/Szenenbaum-Sortierung, Rich Text, integrierte Zeitreise und einheitlichen Rust+WASM-Kern**.
- **Option B — Yrs (`0.28.1`)**: Minimales Web-Bundle und fertige Editor-Plugins bei eigener Baumlogik.
- **Option C — Automerge (`0.12.0`)**: Fokus auf lückenlosen Commit-Graphen je Tastenanschlag.
- **Option D — Hybrid**: Manuskriptbäume in `Loro`/`Yrs` und Klartext-Drehbücher als Markdown/Fountain.
