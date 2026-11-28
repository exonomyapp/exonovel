# 002-004. Preferencje oparte o bazę danych (Eliminacja nadmiarowego `localStorage`)

Data: 2026-11-27
Status: Zaakceptowano (PostgreSQL `user_profiles` + wzorzec niewidocznych preferencji UI)
Nadrzędny: [002-data](../002-data.md)

## Kontekst i pytanie projektowe

ExoNovel przechowuje preferencje w **PostgreSQL (`user_profiles`)** (`appearance_mode`, `preferred_format`, `locale`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, preferencje powiadomień). Równoległe zapisywanie ich do `localStorage` powoduje rozjazd między urządzeniami. Zarówno **Profil**, jak i **Preferencje** będą korzystać z jednego, wbudowanego w stronę (niepływającego) lewego paska nawigacji z trwałą zmianą szerokości zapisywaną w bazie zamiast w przeglądarce i bez dodawania zbędnych kontrolek formularza.

## Odpowiedź i decyzja architektoniczna

**Po zalogowaniu przeglądarkowe `localStorage` jest wykluczone dla trwałych preferencji i ustawień układu.**

### 1. Jedno źródło prawdy: PostgreSQL (`user_profiles`)
- Wszystkie trwałe preferencje oraz **niewidoczne ustawienia UI** (`appearance_mode`, `ui_locale`, `default_input_locale`, `preferred_format`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, `docs_sidebar_collapsed`, `docs_sidebar_width`, `profile_nav_width` dla **Profilu**, `preferences_nav_width` dla **Preferencji**, `notify_prefs_json`) są pobierane w `/api/me` i aktualizowane przez `PUT /api/profile`. Ponieważ **Profil** i **Preferencje** mają inną zawartość, szerokość lewego paska jest zapisywana osobno dla każdej strony.
- Renderowanie serwerowe (`shell()` w `crates/server/src/main.rs`) wstrzykuje `<html lang="..." dir="..." data-appearance="...">` bezpośrednio z profilu.

### 2. Wzorzec „niewidocznych preferencji” (bezpośrednia manipulacja bez zbędnego UI)
- Ustawienia ergonomii zmieniane bezpośrednio na stronie (np. przesuwanie uchwytu szerokości niepływającej lewej nawigacji osobno w **Profilu** (`profile_nav_width`) i **Preferencjach** (`preferences_nav_width`) przez `role="separator"`, szerokość/zwinięcie paska **Docs & ADRs** lub ostatnio otwarta sekcja/dokument) zapisują się w PostgreSQL — **bez** dokładania dodatkowych pól czy przełączników w formularzach **Preferencji**.
- **Spójny układ Profilu i Preferencji**: **Profil** (zastępujący poziome zakładki) oraz **Preferencje** dzielą ten sam niepływający, zintegrowany ze stroną lewy panel z niezależnie zapisywaną szerokością dla każdej strony (`ExoSettingsPage` + `ExoSettingsSplit`).

### 3. Jedyne wyjątki przed uwierzytelnieniem
1. **Oczekujący kod zaproszenia (`sessionStorage`)**: Tymczasowa wartość (`exo_pending_invite`) przenoszona przez przekierowanie OAuth i usuwana natychmiast po `POST /api/invites/claim`.
2. **Wybór języka/motywu przed logowaniem**: Tymczasowo w pamięci do chwili zapisu w `user_profiles`.
3. **Lokalny magazyn natywny (`willow25`)**: Na aplikacjach natywnych preferencje oraz niewidoczne ustawienia UI trafiają bezpośrednio pod `/personas/...` i `/profile`.
