# 006-001. Rejestr wersji zależności i dziennik aktualizacji

Data: 2026-11-21
Status: Zaakceptowano (Aktywny dokument żywy)
Nadrzędny: [006-stack](../006-stack.md)

## Kontekst

ExoNovel rozwija się równolegle z dojrzewaniem kluczowych bibliotek Rust i Local-First. Dokument śledzi zadeklarowane wersje zależności, aktualne wydania źródłowe i odblokowane możliwości produktu.

---

## Bieżący inwentarz stosu

### 1. Aktywne zależności uruchomieniowe (`Cargo.toml` / CI)

| Technologia | Rola w ExoNovel | Zadeklarowana | Najnowsza zweryfikowana | Wpływ biznesowy |
| :--- | :--- | :--- | :--- | :--- |
| **`leptos` / `leptos_meta` / `leptos_router` / `leptos_axum`** | Full-Stack Rust SSR + WASM UI | `0.8` | **`0.8.23`** *(oraz `0.9.0-rc1`)* | Automatyczne poprawki `0.8.x`; ocena `0.9` po wydaniu stabilnym |
| **`cargo-leptos`** | Narzędzie budowania serwera i WASM | `latest` przez `taiki-e/install-action@v2` | `0.2.x` | Gotowy plik binarny oszczędza ~12 min w CI |
| **`axum`** | Router HTTP API i SSR | `0.8` | **`0.8.9`** | Poprawki routingu, limitów i WebSocketów |
| **`sqlx`** | Sterownik Neon PostgreSQL | `0.8` | **`0.9.1`** | Ocena migracji przy wyrównaniu MSRV |
| **`ed25519-dalek`** | Generowanie i podpisy `did:peer` | `2` | **`3.0.0`** | Ocena pod dowody W3C i parytet NextGraph |
| **`image`** | Przetwarzanie awatarów po stronie serwera (`64`/`256`/`1024`) | `0.25` | **`0.25.10`** | Czysty Rust bez zależności C |
| **`comrak`** i **`ammonia`** | Parser GFM Markdown i oczyszczanie HTML | `0.35` / `4` | **`0.55.0`** / **`4.x`** | Tabele, listy zadań i bezpieczne renderowanie |
| **Neon PostgreSQL i Managed Better Auth** | Baza w chmurze i logowanie OAuth/E-mail | Usługa zarządzana | Live | Weryfikacja JWT i gałęzie bazy danych |

### 2. Planowany stos Local-First, współpracy i płatności

| Technologia | Odblokowana funkcja | Stan źródłowy | Wyzwalacz integracji |
| :--- | :--- | :--- | :--- |
| **`willow25`** | Szyfrowany magazyn lokalny i sync | **`willow25 v0.7.11`** | Zastępuje wycofane `willow-data-model`/`meadowcap` |
| **`iroh` P2P (`iroh`, `iroh-docs`, `iroh-blobs`, `iroh-gossip`, `swarm-discovery`)** | Przekaźniki QUIC, sync LAN mDNS, obecność na żywo | `iroh 1.x` / `iroh-docs 0.101.0` / `iroh-gossip 0.102.0` | Osadzenie w kompilacjach desktop/mobile |
| **`loro` CRDT** | Jednoczesna edycja, drzewa ruchome, historia | **`loro 1.18.0` (Zaakceptowano)** | Integracja z deltami Neon i `willow25`/`iroh-gossip` |
| **`leptos-fluent` + Mozilla Fluent (`.ftl`)** | 29 języków UI i pisania, lustrzane RTL | **`leptos-fluent 0.3.1` (Zaakceptowano)** | Ładowanie `.ftl` na żądanie bez rozrostu WASM |
| **Powiadomienia Push i E-mail (`web-push`, `a2`, `fcm_v1`, Brevo v3)** | 5 platform + Web + Brevo (`notes@`/`noreply@`/`support@exosystems.net`) | Aktywne | Testy Android ruszają od **F-Droid (`foss`)** |
| **Globalne szyny płatnicze (AOTA)** | Poziom 1 (`dLocal`, `Stripe`, `Reown`), Poziom 2, Poziom 3 (`Airwallex`) | **Zaakceptowano** | Całość finansów w **Profilu** |
| **Global Live UI (`PgListener` + SSE)** | Aktualizacje DOM bez przeładowania | **`sqlx::PgListener` + SSE** | Zobacz [009-live-ui](../009-live-ui.md) |
| **Inferencja Headless (`OpenCode` + NVIDIA NIM)** | Opcjonalna synteza propozycji dojrzałości | `opencode run --standalone --auto --format json` | Zobacz [005-003](../005-docs-ui/003-cms.md) i [005-004](../005-docs-ui/004-inference.md) |

---

## Bieżący dziennik aktualizacji

| Data | Obszar | Zmiana | Uzasadnienie i korzyść |
| :--- | :--- | :--- | :--- |
| **2026-11-26** | **Mobilny awatar w pasku i przewijane arkusze, czysty ekran roboczy, filtr aktywnej persony + przycisk `Persony`, zmiana na `Preferencje` i neutralizacja marek (`crates/`, `style/main.css`, `docs/`)** | *(Wydanie UX, Mobile i Higieny Marki)* | Usunięto ikonę hamburgera i umieszczono okrągły awatar bezpośrednio w mobilnym pasku górnym; menu języka, powiadomień i awatara zamieniono na przewijane arkusze na ekranach `<= 768px` z przyciemnieniem tła; usunięto baner powitalny i duży awatar z `WorkspaceHomeView`; odfiltrowano aktywną personę z listy przełączania i dodano komunikat „Utwórz Persony w swoim Profilu” z bezpośrednim przyciskiem **`Persony`**; wdrożono `Preferences`/`Preferencje` (`preferences.rs`, `PreferencesSection`, `ActiveTab::Preferences`, wszystkie 29 języków, routing i baza danych bez starszych aliasów) oraz usunięto nazwy dostawców infrastruktury ze wszystkich tekstów UI, błędów API, pakietów Fluent i dokumentacji. |
| **2026-11-25** | **Rozdzielone zakładki CMS, `OpenCode` + NVIDIA NIM, `adr_translations` (`de`/`pl`), 100% i18n UI i widoki mobilne** | *(Wydanie CMS, i18n, AI i Mobile)* | Oddzielono **Narzędzia i Propozycje** od **Inferencji AI**, dodano przycisk **Utwórz szkic ADR z propozycji**, usunięto nieskonfigurowanych dostawców oraz modele `gemini-2.5`/`qwen2.5` (pozostawiając **`OpenCode` CLI** i **NVIDIA NIM**). Wdrożono `adr_translations` (`UNIQUE(adr_id, locale)`), zlokalizowane końcówki API oraz komplet 30 dokumentów po niemiecku (`docs/i18n/de/`) i polsku (`docs/i18n/pl/`). Osiągnięto 100% pokrycia kluczy Fluent w całym UI i dodano responsywne reguły CSS (`<= 960px`, `<= 768px`, `<= 480px`). |
| **2026-11-24** | **Połączone aplikacje i Google Docs, strona publiczna, logo marki, neutralność dostawców i poprawki CI** | *(Zgodność OAuth, Marka i CI)* | Dodano serwerowe OAuth Google (`drive.file`), eksport i archiwizację do Google Docs, publiczną stronę z karuzelą 4 slajdów, `/privacy` i `/terms`, przezroczyste logo ExoNovel, usunięcie nazw zewnętrznych firm hostingowych (`AGENTS.md` Reguła 16) oraz poprawki `pipefail` i pakowania wydań. |
| **2026-11-23** | **Rozdzielenie testów `ssr`/`hydrate`, `#![recursion_limit = "512"]`, Check Runs i higiena Dependabot** | `actions@v4 -> v7/v8`, `lockfile-only` | Usunięto błędy głębokości kompilatora, ostrzeżenia kompilacji i włączono automatyczne raportowanie logów błędów CI. |
| **2026-11-21** | **Global Live UI, ślepy przekaźnik Trybu 2, `loro 1.18.0`, 29 języków, Brevo i MADR CMS** | *(Baza architektoniczna)* | Ustanowiono główne dokumenty ADR (`007`, `008`, `009`, `002-005`, `004-003`, `005-003`) oraz `.github/dependabot.yml`. |
