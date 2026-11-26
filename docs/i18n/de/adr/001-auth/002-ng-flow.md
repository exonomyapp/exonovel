# 001-002. Aufgeschlüsselter NextGraph-Auth-Ablauf & Technologie-Checkliste

Datum: 2026-11-19
Status: Akzeptiert (alle 14 NextGraph-Schritte vorerst beibehalten)
Übergeordnet: [001-auth](../001-auth.md)

## Kontext

Dieses Unter-ADR gliedert jeden Schritt und jede Technologie aus dem Authentifizierungs- und Wallet-Konzept von **NextGraph** (`nextgraph-rs` / `ng-wallet`) auf. Alle 14 Bausteine bleiben im Umfang erhalten und werden nach dem AOTA-Prinzip durch `iroh`-Relays, Neon Better Auth sowie Passkey-/Web3-Adapter ergänzt.

## Aufschlüsselung von Ablauf & Technologien

| # | Baustein | NextGraph-Technologie & Verhalten | Rolle in der Auth-Architektur | Entscheidung |
| :-: | :--- | :--- | :--- | :--- |
| 1 | **Lokale Wallet-Crate** | `ng-wallet` (Rust) verschlüsselter Tresor; Browser nutzt WebCrypto/IndexedDB, native Apps OS-App-Verzeichnisse | Hält private Schlüssel lokal ohne Serverkenntnis | Behalten |
| 2 | **Signatur-Kryptografie** | **Ed25519**-Schlüsselpaare + W3C-Data-Integrity-Nachweise | Leitet `did:peer` ab, signiert Commits, Auth-Challenges und Freigaben | Behalten |
| 3 | **Offline-Ersteinrichtung** | Wallet & `did:peer` werden offline vor jedem Serverkontakt erstellt | Garantiert serverunabhängige Existenz der Identität | Behalten |
| 4 | **Mnemonic-Seed-Phrase** | BIP-39-ähnliche 12/24-Wort-Phrase + druckbares Wiederherstellungs-PDF | Wurzel-Backup und geräteübergreifende Wiederherstellung | Behalten |
| 5 | **Pazzle-Entsperrung** | Visuelle Emoji-/Musterfolge (`load_pazzle_emojis`) | Schnelle, einprägsame lokale Entsperrung statt langer Passwörter | Behalten |
| 6 | **PIN / Biometrie-Sperre** | Kurze PIN / OS-Biometrie nach Inaktivität | Schnelle Wiederentsperrung auf Desktop und Mobilgeräten | Behalten |
| 7 | **Multi-Persona-Wallet** | Mehrere isolierte Identitäten (`did:peer` pro Persona) in einer Wallet | Trennt Pseudonyme sowie private und öffentliche Werke | Behalten |
| 8 | **QR-Code-Gerätekopplung** | Ephemerer verschlüsselter QR-Code zwischen Erst- und Mobilgerät | Überträgt Wallet-Schlüssel ohne Cloud-Schlüsselspeicher | Behalten |
| 9 | **TextCode-Gerätekopplung** | Einmalige Textzeichenfolge per Kopieren/Einfügen | Verbindet kamerafreie Desktop-Systeme (Linux/Windows/macOS) | Behalten |
| 10 | **Wallet-Dateiexport** | Verschlüsselte `.wallet`-Datei per USB oder lokalem Transfer | Kalte Offline-Sicherung und manueller Gerätewechsel | Behalten |
| 11 | **Relay-/Broker-Transport** | 2-stufiges Relay (Noise über WebSockets/QUIC) + **iroh (`v1.4.x`)** QUIC-Relays & `EndpointId` | NAT-Traversal, Offline-Sync und verschlüsseltes Peer-Routing | Behalten (+ iroh) |
| 12 | **Challenge-Response-Auth** | Client beweist `did:peer`-Besitz durch Signatur einer Server-Nonce mit Ed25519 | Passwortloser Login ohne Übertragung privater Schlüssel | Behalten |
| 13 | **Kryptografische OCAPs** | Object Capabilities (kryptografische Lese-/Schreib-/Fork-Tokens) | Feingranularer Zugriff auf Charaktere, Plots, Zeitachsen, Themen | Behalten |
| 14 | **PAKE / Passwort-Option** | `libopaque` (OPAQUE PAKE) neben **Neon Better Auth** (Google/GitHub SSO + E-Mail) | Passwort- und SSO-Komfort mit `did:peer`-Auflösung | Behalten (+ Better Auth / AOTA) |

## End-to-End-Ablaufübersicht

1. **Primärer Pfad**: App starten -> `exo-wallet` offline anlegen -> Mnemonic sichern -> per Pazzle/PIN/Biometrie entsperren -> `did:peer` ableiten -> per `iroh` / Noise / HTTPS verbinden -> Challenge signieren -> als `did:peer` authentifiziert.
2. **Gerätewechsel**: Neues Gerät öffnen -> QR scannen / TextCode einfügen / `.wallet` importieren / Mnemonic eingeben -> Wallet entschlüsseln -> Challenge signieren -> verschlüsselten Zustand über `iroh` + Neon abgleichen.
3. **Komfort-SSO & Wallets (AOTA)**: Mit Google, GitHub, E-Mail (Better Auth), `libopaque`, Passkeys, MetaMask/SIWE, Reown AppKit oder Privy anmelden -> Server löst `did:peer` auf oder erzeugt sie -> volle Fähigkeiten des Primärpfades aktiv.
