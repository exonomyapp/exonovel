# 001-001. `did:peer` als primärer Eigentümerschlüssel & Komfort-SSO-Auflösung

Datum: 2026-11-19
Status: Akzeptiert (Kernprinzip) / Vorgeschlagen (Details zur Schlüsselverwahrung)
Übergeordnet: [001-auth](../001-auth.md)

## Kontext

Eine frühere Formulierung („auf eine `did:peer` abgebildet, niemals der primäre Eigentümerschlüssel“) erzeugte Unklarheit darüber, ob noch ein weiterer Eigentümerschlüssel existiert.

## Entscheidung

1. **`did:peer` ist der einzige primäre Eigentümerschlüssel**:
   - Es gibt **keinen** anderen Eigentümerschlüssel in ExoNovel.
   - Jedes erstellte Objekt (`works`, `chapters`, `scenes`, `characters`, `plots`, `timelines`, `themes`, `forks`) speichert `owner_did` als Verweis auf die `did:peer` des Autors.
   - Provider-IDs (Google `sub`, GitHub `id`), E-Mail-Adressen und Better-Auth-UUIDs sind **reine Authentifizierungs-Aliase**—sie schlagen die `did:peer` nach oder entsperren sie und besitzen selbst niemals Inhalte.

2. **Komfort-SSO schaltet den vollen primären Pfad frei**:
   - Eine Anmeldung über **Google** oder **GitHub** (Neon Better Auth) begründet kein zweitrangiges Konto.
   - Direkt nach Abschluss von SSO gilt:
     1. Das Backend sucht in Neon PostgreSQL nach `identity_links(provider, subject) -> did_peer`.
     2. **Bestehendes Konto**: Löst die `did:peer` auf, entsperrt die verwaltete Schlüssel-Referenz oder verknüpft den aktiven `did:peer`-Anspruch mit der Sitzung.
     3. **Neues Konto**: Führt automatisch die primäre Initialisierung aus—erzeugt ein `did:peer`-Schlüsselpaar, speichert öffentliche `did:peer` sowie verschlüsseltes Schlüsselmaterial in PostgreSQL (oder Vault Transit), legt Standard-Wallet/Persona an und liefert eine aktive `did:peer`-Sitzung.
   - Ergebnis: SSO-Nutzende erhalten exakt dieselben Eigentums-, Freigabe-, Forking- und Paywall-Fähigkeiten wie Konten mit direkter lokaler Wallet-Erstellung und können später jederzeit auf Eigenverwahrung wechseln.
