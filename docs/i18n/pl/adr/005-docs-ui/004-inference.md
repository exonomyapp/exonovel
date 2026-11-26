# 005-004. Bezserwerowi dostawcy inferencji, darmowe limity i Failover (`OpenCode`)

Data: 2026-11-21
Status: Zaakceptowano (`OpenCode` CLI + NVIDIA NIM aktywne; kolejni dostawcy włączani po dodaniu kluczy)
Nadrzędny: [005-docs-ui](../005-docs-ui.md)

## Kontekst

Zgodnie z [005-003](003-cms.md) główny skaner wydań działa **w 100% w deterministycznym kodzie Rust**. Jedynie opcjonalna synteza propozycji korzysta z **`OpenCode`** (`opencode run --standalone --auto --format json`) na aktywnie skonfigurowanych darmowych punktach końcowych.

Ponieważ dostawcy bezpłatnych API często rotują lub wycofują starsze modele, ExoNovel wyświetla i wywołuje **wyłącznie dostawców z aktywnymi kluczami** (obecnie **OpenCode CLI** oraz **NVIDIA NIM** przez `NVIDIA_NIM_API_KEY` / `NVIDIA_API_KEY`) oraz **aktualne darmowe modele** (`meta/llama-3.3-70b-instruct`, `deepseek-ai/deepseek-r1`, `nvidia/nemotron-3.5-lightning-30b-a3b`). Kolejni dostawcy (`OpenRouter`, `Groq`, `Google AI Studio`, `Cerebras`, `Mistral`, `Ollama`) aktywują się dopiero po dostarczeniu ich kluczy.

## Aktywna polityka dostawców (AOTA)

*Zasada prywatności i UI*: Przez darmowe API przechodzą **wyłącznie publiczne listy zmian Open Source i publiczne metadane ADR**—**nigdy prywatne rękopisy ani klucze użytkowników**. Widok **Inferencja AI** pokazuje wyłącznie czysty stan operacyjny bez przestarzałych modeli (takich jak `gemini-2.5` czy `qwen2.5`) i bez wewnętrznych komentarzy.

1. **OpenCode Headless CLI (`opencode run --standalone --auto --format json`)**:
   - Uruchamia binarkę `opencode` (`/root/.opencode/bin/opencode` lub `/root/.local/bin/opencode`) w izolowanym trybie JSONL, odczytując fragmenty tekstu ze strumienia i zapisując dokładną diagnostykę w razie błędu.
2. **NVIDIA NIM (`NVIDIA_NIM_API_KEY` / `NVIDIA_API_KEY` — Aktywny)**:
   - Punkt końcowy zgodny z OpenAI (`https://integrate.api.nvidia.com/v1`) obsługujący aktualne otwarte modele (`meta/llama-3.3-70b-instruct`, `deepseek-ai/deepseek-r1`, `nvidia/nemotron-3.5-lightning-30b-a3b`).
   - Odświeżanie na żywo (`GET https://integrate.api.nvidia.com/v1/models`) odfiltrowuje modele przestarzałe i niekonwersacyjne.
3. **Dodatkowi dostawcy uruchamiani kluczem (AOTA)**:
   - **OpenRouter**, **Google AI Studio**, **Groq**, **Cerebras**, **Mistral** i lokalny **Ollama** aktywują się po dodaniu odpowiednich zmiennych środowiskowych.

## Decyzja: Odkrywanie katalogu na żywo i Failover

- **`provider_models` w Neon PostgreSQL**: Przechowuje aktywne modele wyłącznie dla skonfigurowanych dostawców i bieżących generacji.
- **Automatyczna synchronizacja `/v1/models`**: W przypadku `404` / `model_not_found` wycofany model zostaje oznaczony jako przestarzały, a zapytanie przełącza się natychmiast na kolejny aktywny darmowy model.
