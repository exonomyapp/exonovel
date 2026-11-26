# 001-004. HashiCorp Vault vs. Neon Better Auth

Datum: 2026-11-19
Status: Vorgeschlagen (zur Eigentümer-Entscheidung)
Übergeordnet: [001-auth](../001-auth.md)

## Kontext

Einordnung von **HashiCorp Vault** (`developer.hashicorp.com/vault`) als Ergänzung zu **Neon Better Auth**.

## Recherche: Komplementäre Schichten statt Konkurrenten

| System | Schicht | Aufgaben | Nicht-Aufgaben |
| :--- | :--- | :--- | :--- |
| **Neon Better Auth** | Anwendungs-Auth & Sitzungen | Verarbeitet Google/GitHub-OAuth-Callbacks, E-Mail/Passwort, Session-Cookies, JWTs (`neon_auth`-Schema in Neon Postgres) | Kein HSM/KMS zur Isolierung von Hauptschlüsseln außerhalb der Datenbank |
| **HashiCorp Vault (Transit Engine)** | Infrastruktur-KMS / Cryptography-as-a-Service | Ver-, entschlüsselt und signiert per API (`/transit/encrypt`, `/transit/sign`) mit rotierten Schlüsseln, die **Vault-Speicher nie verlassen** | Verwaltet keine Login-Abläufe, OAuth-Redirects oder Browser-Sitzungen |

## Einsatzort von Vault in ExoNovel

Vault wird **nicht für reguläre Login-Sitzungen benötigt**, löst aber gezielt die Absicherung serverseitiger Schlüsselmaterialien:

- Beim Erst-Login über Google/GitHub erzeugt der Server ein `did:peer`-Schlüsselpaar im Namen des Nutzers, bis dieser die Eigenverwahrung übernimmt.
- Beim Verbinden optionaler Drittanbieter-Apps (`001-auth` Abschnitt 7) müssen `connected_apps.encrypted_refresh_token`-Werte bis zum Widerruf sicher ruhend gespeichert bleiben.
- **Ohne Vault**: Verschlüsselung über einen Anwendungsschlüssel (`MASTER_KEY`) vor Ablage in PostgreSQL.
- **Mit Vault Transit**: Verschlüsselung über die Vault-Transit-API; in PostgreSQL landet ausschließlich Chiffretext, sodass selbst ein Datenbank-Dump keinen Wrap-Key preisgibt.

## Optionen für die Interview-Entscheidung

- **Option A (Schlanker Start)**: Neon Better Auth + umgebungsverwaltete Envelope-Verschlüsselung in Postgres jetzt; Vault Transit bei Skalierung verwalteter Schlüssel nachrüsten.
- **Option B (Vault ab Tag Eins)**: HCP Vault / selbstgehostetes Vault Transit sofort für alle SSO-erzeugten `did:peer`-Schlüssel und `connected_apps`-Refresh-Tokens einbinden.
