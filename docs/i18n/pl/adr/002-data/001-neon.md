# 002-001. Neon PostgreSQL, Better Auth i magazyn obiektów

Data: 2026-11-19
Status: Zaakceptowano
Nadrzędny: [002-data](../002-data.md)

## Kontekst

ExoNovel standaryzuje bazę danych, backend uwierzytelniania i magazyn plików na **Neon** (`org-still-credit-09220145`, projekt `curly-pine-66858617`, gałąź `production`).

## Decyzja

1. **Konfiguracja (`neon.ts`)**: Deklaracja `auth: true` i prywatnego zasobnika obiektów (`assets`) oraz pobieranie zmiennych `DATABASE_URL`, `DATABASE_URL_UNPOOLED`, `NEON_AUTH_*`, `AWS_*` przez `neon deploy`.
2. **Połączenia z bazą danych**: Połączenie z pulą (`-pooler`) dla zwykłych zapytań oraz bezpośrednie (`DATABASE_URL_UNPOOLED`) dla migracji DDL i `PgListener`.
3. **Magazyn obiektów S3**: Klient S3-kompatybilny z `forcePathStyle: true`; w PostgreSQL przechowywane są wyłącznie klucze obiektów.
4. **Managed Better Auth**: Weryfikacja JWT przeciw `NEON_AUTH_JWKS_URL`, rejestracja zaufanego pochodzenia `https://exonovel.exosystems.net` oraz wysyłka poczty przez SMTP **Brevo** z adresu **`noreply@exosystems.net`** (`"Exosystems"`).
