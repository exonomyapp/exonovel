# 001. Authentifizierungs- & Identitätsarchitektur

Datum: 2026-11-19
Status: Vorgeschlagen (zur Eigentümer-Prüfung)

## Kontext

ExoNovel ist eine kollaborative Plattform für kreatives Schreiben über Web, Desktop (Linux, macOS, Windows) und Mobilgeräte (Android, iOS), auf der **jede schreibende Person das Eigentum an den eigenen Inhalten behält**, auch bei gemeinsamer Arbeit an Manuskripten oder Story-Bausteinen.

Das Eigentum muss kryptografisch verankert, plattformübergreifend portabel und unabhängig von einzelnen OAuth-Anbietern sein—bei gleichzeitig reibungslosem Google- und GitHub-Login für alltägliche Nutzung.

## Übergreifende Architektur

1. **Maßgebliche Identität (`did:peer`)**:
   - Der `did:peer`-Bezeichner jedes Nutzers **ist** der primäre Eigentümerschlüssel in der gesamten Anwendung (`owner_did`).
   - Alle Inhalte—Romane, Szenen, Charaktere, Plots, Zeitachsen/Handlungsstränge, Themen und Forks—gehören einer `did:peer`.
   - E-Mail-Adressen, OAuth-IDs (Google/GitHub) und Better-Auth-UUIDs sind ausschließlich bequeme Login-Zuordnungen auf eine `did:peer`, niemals alternative Eigentümerschlüssel.

2. **Primärer Auth-Ablauf (Lokale Wallet nach NextGraph-Vorbild)**:
   - Angelehnt an NextGraph (`ng-wallet`) erzeugen Nutzende lokal eine kryptografische Wallet (Ed25519-Schlüsselpaare + `did:peer`) offline mit Mnemonic-Seed-Wiederherstellung, schneller lokaler Entsperrung („Pazzle“ / PIN / Biometrie), Multi-Persona-Identitäten sowie QR-/TextCode-/Datei-Kopplung zwischen Geräten.
   - Transport- und Relay-Handshakes laufen über **iroh** (`v1.4.x`) / Noise-gesicherte Kanäle mit Challenge-Response-Nachweis des `did:peer`-Besitzes.

3. **Komfort-Login (Google & GitHub SSO über Neon Better Auth)**:
   - Anmeldungen über **Google** oder **GitHub** (verwaltet durch **Neon Better Auth**) sind möglich.
   - Unmittelbar nach erfolgreichem SSO schlägt der Server die `did:peer` in PostgreSQL nach—oder erzeugt für neue Konten automatisch ein neues `did:peer`-Schlüsselpaar—und schaltet die **vollen Fähigkeiten des primären Pfades** frei.
   - Wer über SSO startet, kann die Eigenverwahrung des Schlüsselpaars später exportieren/beanspruchen, ohne `did:peer` oder Inhaltseigentum zu ändern.

4. **Personas (Mehrere Arbeitsidentitäten unter einem Eigentümer)**:
   - Jedes Konto besitzt eine Standard-Persona auf Basis der primären `did:peer` und kann beliebig viele sekundäre Personas (`did:peer`-Schlüsselpaare unter derselben `owner_did`) anlegen.
   - Ein Persona-Wechsel ändert Arbeitsbereichsansicht und Kollaborations-/Chat-Zuschreibung bei einheitlichem Eigentum (`owner_did`) über alle Inhalte hinweg.

5. **Berechtigungsstufen & Gruppenrollen**:
   - Die Plattform nutzt diskrete Stufen (`user` < `moderator` < `admin`) statt eines booleschen Schalters, sodass Moderation auf Werken ohne direkte Mitautorschaft möglich ist.
   - Kollaborations-**Gruppen** führen gruppenbezogene Rollen (`member`, `manager`); jedes Konto kann Gruppen anlegen, Mitwirkende einladen, Empfehlungen erfassen und Gruppenleitungen zuweisen.

6. **AOTA-Wallet- & Schlüssel-Infrastruktur (`All of the Above`)**:
   - Nach dem **AOTA**-Prinzip unterstützt ExoNovel **alle** evaluierten Wallets und Anmeldewege—native `exo-wallet` (`did:peer`), **MetaMask** / 500+ Web3-Wallets über **Reown AppKit (WalletConnect)** + **SIWE/SIWX**, eingebettete Wallets (**Privy** / **Dynamic**), **Passkeys (WebAuthn)** sowie **Google / GitHub / E-Mail** über **Neon Better Auth**—jeweils aufgelöst auf die maßgebliche `did:peer` (siehe [003-wallets](001-auth/003-wallets.md) & [004-pay](../004-pay.md)).
   - Evaluation von **HashiCorp Vault (Transit Secrets Engine)** für die Envelope-Verschlüsselung verwalteter `did:peer`-Schlüssel aus dem Komfort-SSO.

7. **Verbundene externe Apps, optionale Scopes & Widerruf in Präferenzen (`AOTA`)**:
   - **Minimaler Login vs. optionale App-Freigaben**: Primäre SSO-Logins (Google, GitHub, Microsoft, Apple, Discord) fordern **nur Identitäts-Scopes** (`email`, `profile`, `openid`). Erweiterte API-Rechte (z. B. Google Drive, Google Docs/Kalender, GitHub-Repositories, Microsoft OneDrive) werden **nie beim Login erzwungen**; Nutzende aktivieren sie explizit je Integration.
   - **Verifizierte Einschränkung von Hosted Neon Auth (`2026-11-23`)**: Protokolltests gegen den gehosteten Neon-Auth-Endpunkt und `neon_auth.account` bestätigten, dass Neons OAuth-Proxy Google-Redirects auf `scope=email+profile+openid` umschreibt, `access_type=offline` auslässt und `refreshToken` leer bleibt. Zudem trennt `unlinkAccount` das gesamte SSO-Login statt gezielt einzelne App-Rechte zu widerrufen.
   - **Produktionsarchitektur (`connected_apps` gebunden an `did:peer`)**: Das Produktions-Backend (`crates/server` auf `exonovel.exosystems.net`) trennt optionale App-Freigaben vom Primär-Login in einer eigenen Tabelle **`connected_apps`** (`did_peer`, `provider`, `app_key`, `granted_scopes`, `encrypted_refresh_token`, `status`, `connected_at`, `revoked_at`), abgesichert über [004-vault-kms](001-auth/004-vault-kms.md):
     1. Jeder Konnektor nutzt OAuth 2.0 Authorization Code + PKCE mit Offline-Zustimmung (`access_type=offline`, `prompt=consent`) auf ExoNovels fester Domain.
     2. Tokens liegen verschlüsselt in Neon PostgreSQL und aktualisieren sich still bis zum Widerruf.
     3. Unter **Präferenzen -> Verbundene Apps** können alle Konten verknüpfte SSO-Anbieter einsehen, optionale App-Zugriffe erteilen oder einzelne Integrationen **trennen & widerrufen** (Aufruf von `https://oauth2.googleapis.com/revoke` und Bereinigung von `connected_apps` **ohne Beeinträchtigung des Primär-Logins**).
   - **Grenze der Preview-Umgebung**: Dynamische Preview-URLs unterstützen keine serverseitigen OAuth-Callbacks ohne `redirect_uri_mismatch`; dort nutzt die Preview `listAccounts()`, `login_hint`, Firebase Auth und den Server-Proxy (`POST /api/oauth/google/revoke`) zur Vermeidung von CORS-Blockaden (siehe [005-002](../005-docs-ui/002-gate.md)).

## Unter-ADRs

- [001-did-peer](001-auth/001-did-peer.md) — `did:peer` als einziger primärer Eigentümerschlüssel & SSO-Auflösung
- [002-ng-flow](001-auth/002-ng-flow.md) — Aufgeschlüsselter NextGraph-Auth-Ablauf & Technologieliste
- [003-wallets](001-auth/003-wallets.md) — NextGraph `ng-wallet` vs. MetaMask vs. Reown AppKit / Privy / Passkeys
- [004-vault-kms](001-auth/004-vault-kms.md) — Abgrenzung HashiCorp Vault Transit KMS vs. Neon Better Auth
