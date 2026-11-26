# 004-002. Bramkowanie paywallem i uprawnienia autorskie

Data: 2026-11-19
Status: Zaproponowano (do wywiadu)
Nadrzędny: [004-pay](../004-pay.md)

## Struktura decyzji (Otwarta do wywiadu)

- **Ziarnistość**: Autorzy mogą bramkować cały opublikowany utwór, wybrane rozdziały/odcinki albo pakiety architektury fabularnej.
- **Zapis uprawnień**: Webhooki płatnicze zapisują rekord `(buyer_did, resource_id, grant_type, expires_at)` w Neon PostgreSQL.
- **Dostęp wieloplatformowy**: Powiązanie uprawnień z `did:peer` sprawia, że zakup w przeglądarce odblokowuje dostęp na Linux, macOS, Windows, Android i iOS.

## Otwarte pytania

- Jakie modele cenowe wspierać (zakup jednorazowy, abonament autorski, płatność za rozdział, napiwki)?
- Czy darmowe fragmenty próbne (np. pierwsze N rozdziałów) powinny być konfigurowalne dla każdego projektu?
