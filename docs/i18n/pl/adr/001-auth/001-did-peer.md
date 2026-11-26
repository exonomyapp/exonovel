# 001-001. `did:peer` jako główny klucz własności i rozwiązywanie SSO

Data: 2026-11-19
Status: Zaakceptowano (zasada główna) / Zaproponowano (szczegóły przechowywania kluczy)
Nadrzędny: [001-auth](../001-auth.md)

## Kontekst

Wcześniejsze sformułowanie budziło wątpliwości, czy poza `did:peer` istnieje jeszcze inny klucz własności.

## Decyzja

1. **`did:peer` jest jedynym głównym kluczem własności**:
   - W ExoNovel nie istnieje żaden inny klucz własności.
   - Każdy obiekt autorski (`works`, `chapters`, `scenes`, `characters`, `plots`, `timelines`, `themes`, `forks`) przechowuje `owner_did` wskazujące na `did:peer` twórcy.
   - Identyfikatory dostawców (Google `sub`, GitHub `id`), adresy e-mail i UUID z Better Auth są wyłącznie aliasami uwierzytelniania.

2. **Wygodne logowanie SSO odblokowuje pełną ścieżkę główną**:
   - Po zalogowaniu przez **Google** lub **GitHub**:
     1. Backend odpytuje `identity_links(provider, subject) -> did_peer` w Neon PostgreSQL.
     2. **Istniejący użytkownik**: Odczytuje `did:peer` i przypina aktywne poświadczenie do sesji.
     3. **Nowy użytkownik**: Automatycznie generuje parę kluczy `did:peer`, zapisuje zaszyfrowany materiał kluczowy w PostgreSQL (lub Vault Transit), tworzy domyślną Personę i otwiera sesję `did:peer` z pełnymi prawami autorskimi i możliwością późniejszego eksportu klucza.
