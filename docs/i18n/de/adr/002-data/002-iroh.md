# 002-002. `iroh`-Relay & Nativer Offline-P2P-Sync

Datum: 2026-11-19
Status: Vorgeschlagen (Richtung akzeptiert; Details offen für Interview)
Übergeordnet: [002-data](../002-data.md)

## Kontext

ExoNovel erscheint im Web sowie auf fünf nativen Plattformen (Linux, macOS, Windows, Android, iOS) und benötigt Offline-Synchronisation, Live-Kollaboration und eine datenschutzfreundliche Relay-Schicht.

## Recherchestand (`iroh`)

- **Kern-Release**: `iroh` **v1.4.1**, QUIC-basiertes P2P-Netzwerk mit direkter Adressierung über kryptografische `EndpointId`.
- **Zustandslose verschlüsselte Home-Relays**: Hole-Punching und NAT-Traversal; sobald ein direkter QUIC-Pfad steht, fließt der Verkehr Peer-to-Peer.
- **Offline- & Lokal-Sync**:
  - `swarm-discovery` liefert mDNS-LAN-Erkennung für lokale Zusammenarbeit ohne Internet.
  - `iroh-docs` gleicht Schlüssel-Wert-*Replicas* über **Range-Based Set Reconciliation** ab, gestützt auf `iroh-blobs`.
  - `iroh-gossip` (**v0.102.0**) stellt epidemische Broadcast-Bäume (*HyParView* / *PlumTree*) über ALPN für Live-Präsenz und CRDT-Deltas bereit.

## Entscheidung

1. **Blind-Relay & Inhaltsvertraulichkeit für Team-Relay & persönliche `willow25`-Sync**:
   - Betrieb von **ExoNovels kostenloser `iroh`**-Zone (`v1.4.x`) für NAT-Traversal und kostenlose Web↔Gerät-Zusammenarbeit zwischen Schreibenden mit durchgängiger Noise + `ChaCha20-Poly1305`-Verschlüsselung.
   - **Clientseitige Nutzdatenverschlüsselung**: Während `willow25` + Meadowcap Transport und Pfadberechtigungen absichern, könnte ein Speicher-Peer für die persönliche Synchronisation unverschlüsselte Bytes lesen. Daher werden alle Werke, Loro-Snapshots/Deltas, Chats und Medien vor Verlassen des Autorengeräts **gepolstert, mit Nonces gesalzen und per AEAD (`XChaCha20-Poly1305` / MLS) verschlüsselt**. Relays und Backup-Knoten halten ausschließlich opaken Chiffretext.
2. **Eingebetteter nativer Knoten**: Desktop- und Mobil-Installationen betten einen nativen `iroh`-Knoten (`iroh-docs` + `iroh-blobs` + `iroh-gossip`) ein.
3. **Web-Brücke & Bidirektionale Verlagerung Neon ↔ `willow25`**:
   - Webbrowser ohne rohe UDP-Sockets synchronisieren per `wss://`-Brücke mit Neon PostgreSQL unter identischem `loro` (`1.18.0`)-CRDT-Modell.
   - Native Clients starten sauber **ohne Neon** (`Local-Only Device Mode` oder `Neon + willow25`) und erlauben jederzeit den idempotenten Re-Import zurück nach Neon.

## Geklärte & offene Entscheidungen

- **CRDT-Auswahl (Geklärt)**: **`loro` (`1.18.0`)** ausgewählt (siehe [005-loro](005-loro.md)).
- **Relay-Hosting (Offen)**: Dedizierte regionale `iroh`-Relays von ExoNovel, selbstgehostete Autoren-Relays oder beides (AOTA)?
