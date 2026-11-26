# 002-004. Preferencje oparte o bazę danych (Eliminacja nadmiarowego `localStorage`)

Data: 2026-11-21
Status: Zaproponowano (do przeglądu przez właściciela)
Nadrzędny: [002-data](../002-data.md)

## Kontekst i pytanie projektowe

ExoNovel przechowuje preferencje w **Neon PostgreSQL (`user_profiles`)** (`appearance_mode`, `preferred_format`, `locale`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, ustawienia powiadomień). Równoległe zapisywanie ich do `localStorage` powoduje rozjazd między urządzeniami.

## Odpowiedź i decyzja architektoniczna

**Po zalogowaniu przeglądarkowe `localStorage` jest zbędne i szkodliwe dla trwałych preferencji.**

### 1. Jedno źródło prawdy: Neon PostgreSQL (`user_profiles`)
- Wszystkie trwałe ustawienia (`appearance_mode`, `ui_locale`, `default_input_locale`, `preferred_format`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, `docs_sidebar_collapsed`, `docs_sidebar_width`, `notify_prefs_json`) są pobierane w `/api/me` i aktualizowane przez `PUT /api/profile`.
- Renderowanie serwerowe (`shell()` w `crates/server/src/main.rs`) wstrzykuje `<html lang="..." dir="..." data-appearance="...">` bezpośrednio z profilu.

### 2. Jedyne wyjątki przed uwierzytelnieniem
1. **Oczekujący kod zaproszenia (`sessionStorage`)**: Tymczasowa wartość (`exo_pending_invite`) przenoszona przez przekierowanie OAuth i usuwana natychmiast po `POST /api/invites/claim`.
2. **Wybór języka/motywu przed logowaniem**: Tymczasowo w pamięci / krótkim ciasteczku SSR do chwili zapisu w `user_profiles`.
3. **Lokalny magazyn natywny (`willow25`)**: Na aplikacjach bez Neon ustawienia trafiają bezpośrednio pod `/personas/...` i `/profile`.
