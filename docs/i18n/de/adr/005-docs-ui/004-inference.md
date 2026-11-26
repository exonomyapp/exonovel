# 005-004. Headless-Inferenzanbieter, kostenlose Kontingente & Failover (`OpenCode`)

Datum: 2026-11-21
Status: Akzeptiert (`OpenCode`-CLI + NVIDIA NIM aktiv; weitere Anbieter erst mit hinterlegtem Schlüssel)
Übergeordnet: [005-docs-ui](../005-docs-ui.md)

## Kontext

Unter [005-003](003-cms.md) läuft der Release-Scanner **100 % in deterministischem Rust-Code**. Nur die optionale Synthese neuer Vorschlagstexte nutzt natürliche Sprachinferenz über **`OpenCode`** (`opencode run --standalone --auto --format json`) auf aktiv konfigurierten kostenlosen Endpunkten.

Kostenlose Anbieter rotieren oder deaktivieren veraltete Checkpoints ohne Vorwarnung. Daher zeigt und plant ExoNovel ausschließlich **Anbieter mit aktiven Zugangsdaten** (aktuell **OpenCode-CLI** und **NVIDIA NIM** über `NVIDIA_NIM_API_KEY` / `NVIDIA_API_KEY`) sowie **aktuelle kostenlose Modelle** (`meta/llama-3.3-70b-instruct`, `deepseek-ai/deepseek-r1`, `nvidia/nemotron-3.5-lightning-30b-a3b`). Weitere Anbieter (`OpenRouter`, `Groq`, `Google AI Studio`, `Cerebras`, `Mistral` oder lokales `Ollama`) werden erst aktiviert, wenn deren Schlüssel vorliegen.

## Aktive Anbieterpolitik (AOTA)

*Datenschutz- & UI-Regel*: Über kostenlose Endpunkte laufen **ausschließlich öffentliche Open-Source-Changelogs und öffentliche ADR-Metadaten**—**niemals private Manuskripte oder Schlüssel**. Die UI-Ansicht **KI-Inferenz** zeigt nur saubere Betriebszustände ohne veraltete Modelle (wie `gemini-2.5` oder `qwen2.5`) und ohne interne Kommentare.

1. **OpenCode Headless-CLI (`opencode run --standalone --auto --format json`)**:
   - Ruft die auf dem Server installierte `opencode`-Binärdatei (`/root/.opencode/bin/opencode` bzw. `/root/.local/bin/opencode`) im isolierten JSONL-Modus auf, extrahiert gestreamte Textteile sauber und protokolliert bei Fehlern präzise Diagnosedaten.
2. **NVIDIA NIM (`NVIDIA_NIM_API_KEY` / `NVIDIA_API_KEY` — Aktiv)**:
   - OpenAI-kompatibler Endpunkt (`https://integrate.api.nvidia.com/v1`) für aktuelle Open-Weight-Modelle (`meta/llama-3.3-70b-instruct`, `deepseek-ai/deepseek-r1`, `nvidia/nemotron-3.5-lightning-30b-a3b`).
   - Live-Katalogabgleich über `GET https://integrate.api.nvidia.com/v1/models` filtert veraltete/reine Embedding-Checkpoints heraus.
3. **Weitere zuschaltbare Anbieter (AOTA)**:
   - **OpenRouter**, **Google AI Studio**, **Groq**, **Cerebras**, **Mistral** und **Ollama** schalten sich automatisch zu, sobald deren Schlüssel/Endpunkte in der Serverumgebung gesetzt sind.

## Entscheidung: Live-Katalogsync & Failover

- **`provider_models` in Neon PostgreSQL**: Führt aktive Modelle ausschließlich für konfigurierte Anbieter und aktuelle kostenlose Generationen.
- **Automatischer `/v1/models`-Abgleich**: Bei `404` / `model_not_found` oder Katalog-Aktualisierung werden entfernte Checkpoints als veraltet markiert und sofort durch das nächste aktive kostenlose Modell ersetzt.
