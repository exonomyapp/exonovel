# 003-002. Story-Architektur-Teile & Berechtigtes Forking

Datum: 2026-11-19
Status: Vorgeschlagen (offen für Interview)
Übergeordnet: [003-content](../003-content.md)

## Kontext

Teilen in ExoNovel konzentriert sich auf abgegrenzte Dimensionen einer Geschichte—etwa eine einzelne Charakterbiografie—, damit andere Schreibende sie gezielt entdecken und in eigene Werke einbauen können.

## Entscheidung

1. **Eigenständige teilbare Bausteine**:
   - **Charaktere**: Biografien, Merkmale, Handlungsbögen, Beziehungsgraphen.
   - **Plots**: Beat-Sequenzen, Handlungsstränge, Strukturgliederungen.
   - **Zeitachsen / Threads**: Chronologische Ereignisse, Handlungsfäden, Kontinuitätspläne.
   - **Themen**: Leitmotive, thematische Kernaussagen, symbolische Bögen.
2. **Google-Docs-ähnliche Freigabe & Live-Co-Editing**:
   - Einladungen, Link-Freigaben, Rollen (Lesen/Kommentieren/Bearbeiten) plus Live-Co-Editing.
3. **Berechtigtes Forking an neue Eigentümer**:
   - Jeder teilbare Baustein führt eine vom `owner_did` gesetzte `allow_fork`-Richtlinie.
   - Bei Freigabe erzeugt ein Fork eine eigenständige Kopie unter der `did:peer` des forkenden Autors, während `forked_from_id` und `origin_owner_did` für Herkunftsnachweise erhalten bleiben.

## Offene Interviewfragen

- Welche genauen Berechtigungsstufen sollen gelten (Viewer, Commenter, Editor, Fork-Allowed)?
- Sollen geforkte Teile entkoppelt bleiben oder optionale Benachrichtigungen für Upstream-Änderungen bieten?
- Wie sollen Nennung und ggf. Tantiemen bei kommerziellen Werken mit geforkten Charakteren geregelt werden?
