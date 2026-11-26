# 002-001. Neon PostgreSQL, Better Auth & Objektspeicher

Datum: 2026-11-19
Status: Akzeptiert
Übergeordnet: [002-data](../002-data.md)

## Kontext

ExoNovel standardisiert Cloud-Persistenz, Authentifizierungs-Backend und Dateispeicher auf **Neon** (`org-still-credit-09220145`, Projekt `curly-pine-66858617`, Branch `production`).

## Entscheidung

1. **Konfiguration (`neon.ts`)**:
   - Deklariert `auth: true` (Managed Better Auth) und `buckets: { assets: { access: "private" } }` (Objektspeicher).
   - Bezieht Zugangsdaten (`DATABASE_URL`, `DATABASE_URL_UNPOOLED`, `NEON_AUTH_BASE_URL`, `NEON_AUTH_JWKS_URL`, `AWS_*` S3-Variablen) per `neon deploy`.
2. **Datenbanktreiber & Verbindungen**:
   - Nutzt gepooltes `DATABASE_URL` (`-pooler`) für reguläre Laufzeitabfragen.
   - Nutzt direktes `DATABASE_URL_UNPOOLED` für Schema-Migrationen, DDL und `PgListener`.
3. **Objektspeicher**:
   - S3-kompatibler Client mit `forcePathStyle: true` gegen `AWS_ENDPOINT_URL_S3`; in PostgreSQL liegen nur Objektschlüssel, Lesezugriffe erhalten kurzlebige signierte URLs.
4. **Managed Better Auth**:
   - Überprüft Sitzungs-JWTs gegen `NEON_AUTH_JWKS_URL` auf allen Server-Endpunkten.
   - Registriert `https://exonovel.exosystems.net` als vertrauenswürdige Produktions-Origin und synchronisiert benutzerdefinierte OAuth-Zugangsdaten (`GOOGLE_OAUTH_CLIENT_*`, `GITHUB_OAUTH_CLIENT_*`).
   - E-Mail-Versand von Neon Auth läuft über **Brevo** SMTP (`smtp-relay.brevo.com:587`) von **`noreply@exosystems.net`** (`"Exosystems"`).
