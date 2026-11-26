# 005-002. Prywatna bramka dokumentacji i ustawienia administratora

Data: 2026-11-19
Status: Zaakceptowano (w realizacji)
Nadrzędny: [005-docs-ui](../005-docs-ui.md)

## Decyzja

1. **Publiczna strona główna, trasa `/signin` i bramka Auth (Neon Better Auth)**:
   - Dokumentacja pozostaje ściśle prywatna i należy wyłącznie do aplikacji **ExoNovel** (`https://exonovel.exosystems.net`); żądania bez uprawnień otrzymują `401`/`403`.
   - Niezalogowani goście i roboty na `/` natychmiast otrzymują renderowaną serwerowo publiczną stronę **ExoNovel** (bez ściany logowania) opisującą platformę, karuzelę 4 slajdów, pięć formatów publikacji oraz linki do `/privacy`, `/terms` i `/signin`.
   - Kliknięcie **Zaloguj się** (`/signin` lub `?invite=`) otwiera logowanie przez **Google**, **GitHub** lub e-mail i hasło.
2. **Menu użytkownika w pasku, hub Profilu i Ustawienia**:
   - Zalogowani widzą miniaturę awatara 64px obok nazwy aktywnej persony z przejściem do **Profilu**, szybkim przełączaniem person, **Ustawieniami** i wylogowaniem.
   - **Ustawienia** zawierają preferencje konta, **Połączone aplikacje zewnętrzne** (rozdzielone od głównego logowania), motyw (`system`/`light`/`dark`), języki oraz przesyłanie awatarów do 2 MB (`64`/`256`/`1024` WebP/JPEG).
3. **Lista dostępu admina i role kont**:
   - Tylko konto administratora (`exonomyapp@gmail.com`) widzi panele zarządzania **Listą dostępu do dokumentacji** (`reader`, `moderator`, `pending`, `suspended`, `admin`) i **Zarejestrowanymi kontami** (`user` < `moderator` < `admin`) powiązanymi po `did:peer`.
4. **Cykl życia połączonych aplikacji (`001-auth` Sekcja 7)**:
   - Każde zalogowane konto widzi powiązane tożsamości SSO, aktywne zgody (jak **Google Drive**), przycisk **Połącz** z `login_hint` oraz **Odłącz i unieważnij** bez zamykania głównej sesji.
