# 002. Daten-, Persistenz- & Offline-Sync-Architektur

Datum: 2026-11-19
Status: Akzeptiert (Neon-Stack) / Vorgeschlagen (Details zu `iroh`-Sync)

## Kontext

ExoNovel läuft als Webanwendung sowie nativ auf **Linux, macOS, Windows, Android und iOS**. Schreibende erwarten:
1. Beständige Cloud-Persistenz, relationale Abfragen und Medienablage.
2. Gleichzeitiges Live-Co-Editing nach Art von Google Docs.
3. Offline-fähiges Schreiben und Peer-to-Peer-Synchronisation auf nativen Geräten.

## Übergreifende Architektur

1. **Verwaltete Cloud-Persistenz mit Willow-Portabilität (Neon)**:
   - **PostgreSQL**: Zentraler relationaler Speicher für `did:peer`-Identitäten, `user_personas`, `user_profiles`, `media_assets`, `groups`, `group_members`, `invites`, `referrals`, SSO-Verknüpfungen, Projektmetadaten, Story-Bausteine und Zugriffssteuerung.
   - **Willow-Protokoll-Koordinaten**: Zentrale Tabellen führen deterministische Willow-Koordinaten (`willow_namespace`, `willow_subspace`, `willow_path`, `willow_timestamp_us` sowie SHA-256 `payload_digest` und `byte_length`), sodass Nutzende ihren Bestand verlustfrei in einen lokalen Rust-Willow-Store überführen können.
   - **Mehrgrößen-Medienassets**: Bild-Uploads bis 2 MB werden serverseitig in passende Größen (`thumb` 64×64, `medium` 256×256, `large` <= 1024px WebP/JPEG) mit Inhalts-Digests umgerechnet.
   - **Neon Better Auth**: Verwaltete Auth-Tabellen im Schema `neon_auth`.
2. **Web, Gerät, Team-Sync vs. Persönliche Geräte-Sync (`did:peer`-Portabilität)**:
   - **Im Web (`Kostenlos`)**: Cloud-gestützter Arbeitsbereich über PostgreSQL, in jedem Browser verfügbar.
   - **Auf deinem Gerät (`Kostenlos`)**: Native Builds für **Linux, macOS, Windows, Android und iOS** starten lokal **ohne Cloud-Datenbank-Zwang** mittels **`willow25`** (`0.7.11`) und **ExoNovels kostenloser `iroh`-Discovery-&-Relay-Zone** (`v1.4.x`). **Blind-Relay-Gebot**: Alle über ExoNovels Relays geleiteten Nutzdaten werden vor Versand auf dem Gerät Ende-zu-Ende verschlüsselt.
   - **Kostenlose automatische Team-Synchronisation über Web & Gerät**: Sobald Mitwirkende hinzugefügt werden, synchronisiert ExoNovel die gemeinsame Arbeit automatisch zwischen allen Co-Autorinnen — unabhängig davon, ob eine Person im Browser und eine andere lokal auf ihrem Gerät schreibt.
   - **Persönliche Geräte-Synchronisation (`Kostenpflichtig: Gegenwert von 9 USD/Monat oder 90 USD/Jahr in Landeswährung`)**:
     - Deckt **ausschließlich** das fortlaufende Synchronisieren der eigenen Offline-Geräte-Entwürfe mit dem eigenen Web-Profil ab.
     - Aktivierung und Abrechnung erfolgen über **Profil → Tarife** (mit Zahlungswegen in **Profil → Wallet**, wobei ExoNovel als Peer mit eigener `did:peer` und AOTA-Wallets agiert).
     - Registriert die `did:peer`-IDs mittels **Meadowcap-Fähigkeitsdelegation** in `willow25` auf ExoNovels verwaltetem Sync-Dienst — mit clientseitiger AEAD-Verschlüsselung, sodass ExoNovel **kryptografisch blind** bleibt.
     - Verlustfreier Export (`PostgreSQL -> willow25`) und Re-Import (`willow25 -> PostgreSQL`) über `willow_*`-Koordinaten und kausale **`loro` (`1.18.0`)**-CRDT-Frontiers.

## Unter-ADRs

- [001-neon](002-data/001-neon.md) — Neon PostgreSQL, Better Auth & Objektspeicher
- [002-iroh](002-data/002-iroh.md) — `iroh`-Relay & nativer Offline-P2P-Sync
- [003-crdt-report](002-data/003-crdt-report.md) — Entscheidungsbericht: Open-Source-Echtzeit-Co-Editing-Engines (`Loro` gewählt)
- [004-db-prefs](002-data/004-db-prefs.md) — Verzicht auf redundantes Browser-`localStorage` zugunsten von Neon PostgreSQL `user_profiles`
- [005-loro](002-data/005-loro.md) — Loro-CRDT-Echtzeit-Co-Editing, verschiebbare Bäume & Versionshistorie
