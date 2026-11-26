# 005-004. Headless Inference Providers, Free-Tier Evaluation & Auto-Failover (`OpenCode`)

Date: 2026-11-21
Status: Accepted (`OpenCode` CLI + NVIDIA NIM active now; additional providers enabled only as API keys are provisioned)
Parent: [005-docs-ui](../005-docs-ui.md)

## Context

Under [005-003](003-cms.md), ExoNovel's core release scanner runs **100% in deterministic Rust code**. Only the optional **creative proposal synthesis** step (reading unstructured release notes/discussions and proposing unanticipated ExoNovel enhancements) uses natural-language inference via **`OpenCode`** (`opencode run --standalone --auto --format json`) backed by actively configured free remote endpoints.

Free-tier AI providers routinely **deprecate, rename, or remove `:free` and preview models** without warning, or trigger temporary `429` rate limits. Hardcoding retired checkpoints guarantees broken scans and clutters the UI. Therefore, ExoNovel renders and schedules **strictly providers with active provisioned credentials** (currently **OpenCode CLI** and **NVIDIA NIM** via `NVIDIA_NIM_API_KEY` / `NVIDIA_API_KEY`) and **current-generation active free models only** (`meta/llama-3.3-70b-instruct`, `deepseek-ai/deepseek-r1`, `nvidia/nemotron-3.5-lightning-30b-a3b`). Additional providers (`OpenRouter`, `Groq`, `Google AI Studio`, `Cerebras`, `Mistral`, or local `Ollama`) activate only once their keys are provisioned.

## Active & Evaluated Provider Policy (AOTA)

*Scope & UI Rule*: Free API tiers route **only public open-source library changelogs and public ADR metadata**—**never private user manuscripts or keys**. UI copy inside **AI Inference** shows strictly clean operational state and excludes retired models (such as `gemini-2.5` or `qwen2.5`) and internal policy rationale.

1. **OpenCode Headless CLI (`opencode run --standalone --auto --format json`)**:
   - Invokes the server-installed `opencode` binary (`/root/.opencode/bin/opencode` or `/root/.local/bin/opencode`) in isolated standalone JSONL mode, parsing streamed text parts cleanly and recording explicit diagnostics on error.
2. **NVIDIA NIM (`NVIDIA_NIM_API_KEY` / `NVIDIA_API_KEY` — Active)**:
   - *Free Offering*: Free developer prototyping access (`https://integrate.api.nvidia.com/v1`) across current open-weight models (`meta/llama-3.3-70b-instruct`, `deepseek-ai/deepseek-r1`, `nvidia/nemotron-3.5-lightning-30b-a3b`).
   - *Live Discovery*: Refreshes `GET https://integrate.api.nvidia.com/v1/models`, filters out deprecated/non-chat/embedding checkpoints, and executes direct fallback calls whenever needed.
3. **Additional Provision-on-Key Providers (AOTA)**:
   - **OpenRouter (`OPENROUTER_API_KEY`)**: Auto-enabled when keyed, discovering active `:free` chat models above 32K context.
   - **Google AI Studio (`GEMINI_API_KEY`)**: Auto-enabled when keyed, targeting current-generation active Flash/Pro models only.
   - **Groq (`GROQ_API_KEY`)**, **Cerebras (`CEREBRAS_API_KEY`)**, **Mistral (`MISTRAL_API_KEY`)**, & **Self-Hosted Ollama**: Enabled on key/endpoint provisioning without crowding the unprovisioned UI list.

## Decision: Live Catalog Discovery & Cross-Provider Failover

To guarantee ExoNovel's **`OpenCode`** Maturity Runner **never gets stuck on a removed or rate-limited model**:

1. **Database Catalog Sync (`provider_models`)**:
   - Stores active `(provider, model_id, is_free, context_window, supports_json, priority_score, cooldown_until, deprecated_at, last_verified_at)` rows strictly for currently configured providers and current free models.
2. **Automatic Live `/v1/models` Sync & Clean Failover**:
   - Queries each keyed provider's live catalog endpoint on refresh or after any `404` / `model_not_found` response, marking removed models deprecated and rotating immediately to the next healthy free model or provider without displaying stale retired entries.
