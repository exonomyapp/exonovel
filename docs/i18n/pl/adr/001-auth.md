# 001. Architektura uwierzytelniania i tożsamości

Data: 2026-11-19
Status: Zaproponowano (do przeglądu przez właściciela)

## Kontekst

ExoNovel jest platformą wspólnego pisania na Web, Desktop (Linux, macOS, Windows) i Mobile (Android, iOS), w której **każdy autor zachowuje własność swoich treści** nawet podczas współtworzenia projektów.

Własność musi być oparta na kryptografii, przenośna między platformami i niezależna od pojedynczego dostawcy OAuth, przy zachowaniu wygodnego logowania przez Google i GitHub.

## Założenia nadrzędne

1. **Autorytatywna tożsamość (`did:peer`)**:
   - Identyfikator `did:peer` każdego użytkownika **jest** jedynym głównym kluczem własności w całej aplikacji (`owner_did`).
   - Wszystkie zasoby—powieści, sceny, postacie, wątki, osie czasu, motywy i forki—należą do `did:peer`.
   - Adresy e-mail, identyfikatory OAuth oraz UUID z Better Auth są wyłącznie aliasami logowania mapującymi na `did:peer`.

2. **Ścieżka podstawowa (Lokalny portfel w stylu NextGraph)**:
   - Wzorowana na NextGraph (`ng-wallet`)—użytkownik może wygenerować lokalny portfel kryptograficzny (klucze Ed25519 + `did:peer`) offline z frazą odzyskiwania, szybkim odblokowaniem lokalnym („Pazzle“ / PIN / biometria), wieloma personami oraz parowaniem QR/TextCode/pliku między urządzeniami.
   - Transport i uzgodnienia działają przez kanały **iroh** (`v1.4.x`) / Noise z dowodem Challenge-Response posiadania `did:peer`.

3. **Wygodne logowanie SSO (Google i GitHub przez Neon Better Auth)**:
   - Po udanym logowaniu SSO serwer odnajduje `did:peer` w PostgreSQL—lub automatycznie tworzy nową parę kluczy `did:peer` dla nowego konta—odblokowując **pełne możliwości ścieżki podstawowej**.
   - Użytkownik rozpoczynający od SSO może później wyeksportować klucze do pełnej samokontroli bez zmiany `did:peer`.

4. **Persony (Wiele tożsamości roboczych dla jednego właściciela)**:
   - Każde konto posiada domyślną Personę na głównym `did:peer` oraz nielimitowaną liczbę dodatkowych Person (`did:peer` powiązanych z tym samym `owner_did`).

5. **Poziomy uprawnień i role grupowe**:
   - Uprawnienia platformy korzystają z poziomów `user` < `moderator` < `admin`, a grupy współpracy posiadają role `member` i `manager`.

6. **Infrastruktura portfeli i kluczy AOTA (`All of the Above`)**:
   - Zgodnie z zasadą **AOTA** platforma wspiera natywny `exo-wallet` (`did:peer`), **MetaMask** / ponad 500 portfeli przez **Reown AppKit (WalletConnect)** + **SIWE/SIWX**, portfele wbudowane (**Privy** / **Dynamic**), **Passkeys (WebAuthn)** oraz **Google / GitHub / E-mail** (zobacz [003-wallets](001-auth/003-wallets.md) i [004-pay](../004-pay.md)).

7. **Połączone aplikacje zewnętrzne, opcjonalne zakresy i unieważnianie w Ustawieniach (`AOTA`)**:
   - **Minimalne logowanie a opcjonalne zgody**: Podstawowe logowanie SSO żąda wyłącznie uprawnień tożsamości (`email`, `profile`, `openid`). Rozszerzone dostępy (np. Google Drive/Docs, repozytoria GitHub) są udzielane świadomie oddzielnie.
   - **Ograniczenie Neon Auth zweryfikowane (`2026-11-23`)**: Zarządzane proxy OAuth Neon przepisuje przekierowania Google do `scope=email+profile+openid` bez `access_type=offline`, a `unlinkAccount` odpina całe logowanie.
   - **Architektura produkcyjna (`connected_apps` na `did:peer`)**: Backend produkcyjny (`crates/server`) przechowuje opcjonalne integracje w tabeli **`connected_apps`** szyfrowanej zgodnie z [004-vault-kms](001-auth/004-vault-kms.md), umożliwiając odłączenie i wywołanie `https://oauth2.googleapis.com/revoke` z poziomu **Ustawienia -> Połączone aplikacje** bez zrywania sesji głównej.

## Dokumenty podrzędne (Sub-ADRs)

- [001-did-peer](001-auth/001-did-peer.md) — `did:peer` jako wyłączny klucz własności i automatyczne mapowanie SSO
- [002-ng-flow](001-auth/002-ng-flow.md) — Szczegółowy przepływ NextGraph Auth i lista technologii
- [003-wallets](001-auth/003-wallets.md) — NextGraph `ng-wallet` vs MetaMask vs Reown AppKit / Privy / Passkeys
- [004-vault-kms](001-auth/004-vault-kms.md) — Granice HashiCorp Vault Transit KMS i Neon Better Auth
