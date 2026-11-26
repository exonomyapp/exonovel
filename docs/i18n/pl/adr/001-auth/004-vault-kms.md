# 001-004. HashiCorp Vault vs Neon Better Auth

Data: 2026-11-19
Status: Zaproponowano (do decyzji właściciela)
Nadrzędny: [001-auth](../001-auth.md)

## Kontekst

Ocena wykorzystania **HashiCorp Vault** obok **Neon Better Auth**.

## Warstwy uzupełniające, nie konkurencyjne

| System | Warstwa | Zadania | Czego nie robi |
| :--- | :--- | :--- | :--- |
| **Neon Better Auth** | Uwierzytelnianie i sesje | Obsługa Google/GitHub OAuth, logowania e-mail, ciasteczek i JWT (`neon_auth`) | Nie stanowi HSM/KMS izolującego klucze szyfrujące poza bazą |
| **HashiCorp Vault (Transit Engine)** | Infrastrukturalny KMS / Kryptografia jako usługa | Szyfruje, deszyfruje i podpisuje przez API z rotowanymi kluczami nieopuszczającymi pamięci Vault | Nie obsługuje ekranów logowania ani sesji przeglądarkowych |

## Zastosowanie w ExoNovel

- Vault zabezpiecza klucze prywatne `did:peer` generowane podczas rejestracji przez SSO do czasu ich eksportu na urządzenie użytkownika oraz tokeny odświeżania w `connected_apps.encrypted_refresh_token`.
- **Opcja A (Lekki start)**: Szyfrowanie kopertowe z użyciem klucza środowiskowego w Postgres teraz; włączenie Vault Transit przed skalowaniem produkcyjnym.
- **Opcja B (Vault od pierwszego dnia)**: Natychmiastowe wdrożenie Vault Transit do otaczania wszystkich kluczy powierniczych `did:peer` oraz tokenów `connected_apps`.
