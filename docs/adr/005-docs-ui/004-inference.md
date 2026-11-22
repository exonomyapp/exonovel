# 005-004. Headless Inference Providers, Free-Tier Evaluation & Auto-Failover (`OpenCode`)

Date: 2026-11-21
Status: Accepted (`OpenCode` + Google AI Studio + NVIDIA NIM + OpenRouter active; multi-provider auto-discovery & failover required)
Parent: [005-docs-ui](../005-docs-ui.md)

## Context

Under [005-003](003-cms.md), ExoNovel's core release scanner runs **100% in deterministic Rust code**. Only the optional **creative proposal synthesis** step (reading unstructured release notes/discussions and proposing unanticipated ExoNovel enhancements) uses natural-language inference via **`OpenCode`** (`opencode serve` headless mode).

Free-tier AI providers routinely **deprecate, rename, or remove `:free` and preview models** without warning, or trigger temporary `429` rate limits. Hardcoding a single model string guarantees broken cron runs. Meanwhile, the owner holds API keys for **Google AI Studio**, **NVIDIA NIM**, and **OpenRouter**, and excludes **Alibaba Cloud Model Studio** (whose free quota is strictly 1M tokens per model expiring after 90 days rather than ongoing).

## Verified Free-Tier Provider Landscape (AOTA)

*Scope Rule*: Free API tiers (especially Google AI Studio and Mistral Experiment) reserve product-improvement/training rights on prompts. Therefore, ExoNovel **only routes public open-source library changelogs and public ADR metadata** through free external providers—**never private user manuscripts or keys**.

1. **Google AI Studio (`GEMINI_API_KEY` — Key Provisioned)**:
   - *Free Offering*: Ongoing free tier on Gemini Flash / Flash-Lite models with up to 1M+ token context windows.
   - *Developer Outcomes*: Praised for massive context (entire changelogs + multiple ADRs fit in one prompt) and reliable JSON output; catch is strict per-minute quota ceilings and Google's free-tier data usage terms.
2. **NVIDIA NIM (`NVIDIA_NIM_API_KEY` — Key Provisioned)**:
   - *Free Offering*: Free developer prototyping access (`https://integrate.api.nvidia.com/v1`) to enterprise-grade open weights (Nemotron, Llama-3.x, DeepSeek/Qwen open microservices).
   - *Developer Outcomes*: Developers report consistently high instruction-following quality and clean OpenAI-compatible APIs; catch is prototyping/eval terms rather than guaranteed high-throughput production SLAs.
3. **OpenRouter (`OPENROUTER_API_KEY` — Key Provisioned)**:
   - *Free Offering*: Meta-router exposing ~20+ rotating `:free` open-weight and experimental models (`20 RPM / 50 RPD` without credit, scaling to `1,000 RPD` if the account holds `$10+` lifetime credits).
   - *Developer Outcomes*: Loved by developers specifically as a multi-model resilience layer; main frustration is frequent unannounced rotation/removal of `:free` model IDs—solved below via live catalog discovery.
4. **Groq (`GROQ_API_KEY` — Optional Ongoing Free Tier)**:
   - *Free Offering*: LPU-accelerated inference on Llama and open-weight reasoning models (`~30 RPM`, `1,000 RPD`, `~8K TPM / 200K TPD`).
   - *Developer Outcomes*: Praised widely for extreme speed (`300+ tok/s`) and explicit **no-training on developer inputs**; catch is tight tokens-per-minute caps on huge prompts.
5. **Cerebras (`CEREBRAS_API_KEY` — Optional Ongoing Free Tier)**:
   - *Free Offering*: **1,000,000 free tokens/day** (no credit card required) via OpenAI-compatible `https://api.cerebras.ai/v1`.
   - *Developer Outcomes*: Developers praise huge daily token volume and wafer-scale throughput (`2,000+ tok/s`); catch is strict burst throttling, ~8K context ceilings on some free models, and frequent model swaps.
6. **Mistral AI Experiment (`MISTRAL_API_KEY` — Optional Free Mode)**:
   - *Free Offering*: Free Experiment tier for open Mistral models (phone verification required).
   - *Developer Outcomes*: Solid European open-weight reasoning; catch is lower rate thresholds and data telemetry/improvement rights on free accounts.
7. **Cloudflare Workers AI (`10,000 Neurons/day` Free)** & **Self-Hosted Ollama (`Qwen2.5-Coder` / `Llama`, Zero Rate Limits)**:
   - Cloudflare works well for tiny edge routing summaries (10K Neurons exhausts quickly on long docs); local **Ollama** guarantees zero external dependency or quotas whenever local server RAM/GPU permits.
8. **Excluded — Alibaba Cloud Model Studio**:
   - Excluded because free quotas (1M tokens per model) expire in **90 days** or upon exhaustion and roll over to pay-as-you-go billing.

## Decision: Live Catalog Discovery & Cross-Provider Failover

To guarantee ExoNovel's **`OpenCode`** Maturity Runner **never gets stuck on a removed or rate-limited model**:

1. **Zero Hardcoded Static Model Strings**:
   - ExoNovel maintains a **`provider_models`** cache table in Neon PostgreSQL (`provider`, `model_id`, `is_free`, `context_window`, `supports_json`, `priority_score`, `cooldown_until`, `deprecated_at`, `last_verified_at`).
2. **Automatic Live `/v1/models` Sync**:
   - Before scheduling (every 6 hours) and immediately whenever any request returns `404` / `model_not_found` / `deprecated`, the Rust runner queries each configured provider's live catalog endpoint:
     - **Google AI Studio**: `GET /v1beta/models` (selecting active `generateContent` Flash/Pro free-eligible models).
     - **NVIDIA NIM**: `GET https://integrate.api.nvidia.com/v1/models`.
     - **OpenRouter**: `GET https://openrouter.ai/api/v1/models` (filtering `pricing.prompt == "0"` and `pricing.completion == "0"`, rejecting non-chat/tiny models below 32K context).
     - **Groq / Cerebras / Mistral**: Standard `GET /v1/models` discovery when their keys are configured.
3. **Self-Healing Failover Router**:
   - On **`404` / Removed Model**: Marks `deprecated_at = NOW()`, refreshes that provider's live `/v1/models` catalog to learn replacement IDs automatically, and retries immediately against the next highest-priority active model—**crossing provider boundaries cleanly** (e.g., Google AI Studio → NVIDIA NIM → OpenRouter `:free` → Groq → Cerebras → Mistral).
   - On **`429` Rate Limit / Quota Exhausted**: Sets `cooldown_until = NOW() + backoff` without deprecating the model, rotating instantaneously to the next keyed provider's active model so Maturity Scanner jobs complete without manual intervention.
