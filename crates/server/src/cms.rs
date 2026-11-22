use std::{fs, path::Path};

use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::{
    auth::AuthUser,
    db::now_timestamp_us,
    docs::{extract_title_and_headings, render_safe_gfm_html, DocHeading},
    live::emit_live_patch,
    AppState,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdrToolDependencyView {
    pub tool_key: String,
    pub tool_name: String,
    pub installed_version: String,
    pub latest_version: String,
    pub required_constraint: String,
    pub blocking_capability: String,
    pub is_satisfied: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdrRecord {
    pub id: i32,
    pub adr_number: Option<String>,
    pub parent_adr_id: Option<i32>,
    pub doc_kind: String,
    pub title: String,
    pub slug: String,
    pub rel_path: String,
    pub status: String,
    pub superseded_by_id: Option<i32>,
    pub context_problem: String,
    pub decision_drivers: String,
    pub considered_options: serde_json::Value,
    pub decision_outcome: String,
    pub consequences_pros_cons: String,
    pub confirmation_criteria: String,
    pub decision_makers: String,
    pub consulted: String,
    pub informed: String,
    pub raw_markdown: String,
    pub html: String,
    pub word_count: usize,
    pub reading_minutes: usize,
    pub headings: Vec<DocHeading>,
    pub author_did: String,
    pub dependencies: Vec<AdrToolDependencyView>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogItem {
    pub tool_key: String,
    pub name: String,
    pub ecosystem: String,
    pub package_name: String,
    pub repo_url: String,
    pub installed_version: String,
    pub latest_version: String,
    pub release_notes_url: String,
    pub purpose: String,
    pub last_checked_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaturityNoteItem {
    pub id: i32,
    pub adr_id: Option<i32>,
    pub adr_title: Option<String>,
    pub tool_key: String,
    pub target_version: String,
    pub note_kind: String,
    pub title: String,
    pub rationale: String,
    pub proposed_feature_changes: String,
    pub provider_used: String,
    pub model_used: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelInfo {
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    pub is_free: bool,
    pub supports_tools: bool,
    pub is_active: bool,
    pub key_configured: bool,
    pub cooldown_until: Option<String>,
    pub deprecated_at: Option<String>,
    pub last_verified_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerExternalTask {
    pub task_key: String,
    pub category: String,
    pub owner_actor: String,
    pub title: String,
    pub step_by_step: String,
    pub copy_snippet: String,
    pub is_done: bool,
    pub completed_by: String,
    pub completed_at: Option<String>,
    pub sort_order: i32,
}

pub async fn ensure_cms_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS adr_records (
            id SERIAL PRIMARY KEY,
            adr_number TEXT,
            parent_adr_id INTEGER REFERENCES adr_records(id) ON DELETE SET NULL,
            doc_kind TEXT NOT NULL DEFAULT 'adr',
            title TEXT NOT NULL,
            slug TEXT NOT NULL,
            rel_path TEXT NOT NULL UNIQUE,
            status TEXT NOT NULL DEFAULT 'proposed',
            superseded_by_id INTEGER REFERENCES adr_records(id) ON DELETE SET NULL,
            context_problem TEXT NOT NULL DEFAULT '',
            decision_drivers TEXT NOT NULL DEFAULT '',
            considered_options_json JSONB NOT NULL DEFAULT '[]'::jsonb,
            decision_outcome TEXT NOT NULL DEFAULT '',
            consequences_pros_cons TEXT NOT NULL DEFAULT '',
            confirmation_criteria TEXT NOT NULL DEFAULT '',
            decision_makers TEXT NOT NULL DEFAULT '',
            consulted TEXT NOT NULL DEFAULT '',
            informed TEXT NOT NULL DEFAULT '',
            raw_markdown TEXT NOT NULL DEFAULT '',
            cms_edited BOOLEAN NOT NULL DEFAULT FALSE,
            author_did TEXT NOT NULL DEFAULT '',
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT '',
            willow_path TEXT NOT NULL DEFAULT '',
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tooling_catalog (
            tool_key TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            ecosystem TEXT NOT NULL,
            package_name TEXT NOT NULL,
            repo_url TEXT NOT NULL DEFAULT '',
            installed_version TEXT NOT NULL DEFAULT '',
            latest_version TEXT NOT NULL DEFAULT '',
            release_notes_url TEXT NOT NULL DEFAULT '',
            purpose TEXT NOT NULL DEFAULT '',
            last_checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tooling_versions (
            id SERIAL PRIMARY KEY,
            tool_key TEXT NOT NULL REFERENCES tooling_catalog(tool_key) ON DELETE CASCADE,
            version TEXT NOT NULL,
            release_url TEXT NOT NULL DEFAULT '',
            summary TEXT NOT NULL DEFAULT '',
            is_prerelease BOOLEAN NOT NULL DEFAULT FALSE,
            unlocked_capabilities_json JSONB NOT NULL DEFAULT '[]'::jsonb,
            discovered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (tool_key, version)
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS adr_tooling_dependencies (
            adr_id INTEGER NOT NULL REFERENCES adr_records(id) ON DELETE CASCADE,
            tool_key TEXT NOT NULL REFERENCES tooling_catalog(tool_key) ON DELETE CASCADE,
            required_constraint TEXT NOT NULL DEFAULT '',
            blocking_capability TEXT NOT NULL DEFAULT '',
            is_satisfied BOOLEAN NOT NULL DEFAULT FALSE,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (adr_id, tool_key)
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS maturity_proposal_notes (
            id SERIAL PRIMARY KEY,
            adr_id INTEGER REFERENCES adr_records(id) ON DELETE SET NULL,
            tool_key TEXT NOT NULL REFERENCES tooling_catalog(tool_key) ON DELETE CASCADE,
            target_version TEXT NOT NULL DEFAULT '',
            note_kind TEXT NOT NULL DEFAULT 'new-opportunity',
            title TEXT NOT NULL,
            rationale TEXT NOT NULL DEFAULT '',
            proposed_feature_changes TEXT NOT NULL DEFAULT '',
            provider_used TEXT NOT NULL DEFAULT 'deterministic',
            model_used TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'open',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS provider_models (
            provider TEXT NOT NULL,
            model_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            is_free BOOLEAN NOT NULL DEFAULT TRUE,
            supports_tools BOOLEAN NOT NULL DEFAULT TRUE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            cooldown_until TIMESTAMPTZ,
            deprecated_at TIMESTAMPTZ,
            last_verified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (provider, model_id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS owner_external_tasks (
            task_key TEXT PRIMARY KEY,
            category TEXT NOT NULL,
            owner_actor TEXT NOT NULL DEFAULT 'Owner',
            title TEXT NOT NULL,
            step_by_step TEXT NOT NULL DEFAULT '',
            copy_snippet TEXT NOT NULL DEFAULT '',
            is_done BOOLEAN NOT NULL DEFAULT FALSE,
            completed_by TEXT NOT NULL DEFAULT '',
            completed_at TIMESTAMPTZ,
            sort_order INTEGER NOT NULL DEFAULT 100,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn extract_madr_section(md: &str, heading_needle: &str) -> String {
    let mut capture = false;
    let mut buf = Vec::new();
    let needle = heading_needle.to_lowercase();
    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            let h = trimmed.trim_start_matches("## ").trim().to_lowercase();
            if capture {
                break;
            }
            if h.contains(&needle) {
                capture = true;
                continue;
            }
        } else if capture {
            buf.push(line);
        }
    }
    buf.join("\n").trim().to_string()
}

fn infer_status_from_md(md: &str, rel_path: &str) -> &'static str {
    let head: String = md.lines().take(28).collect::<Vec<_>>().join("\n").to_lowercase();
    if head.contains("on hold") || head.contains("on-hold") || head.contains("awaiting") {
        "on-hold"
    } else if head.contains("ready to build") || head.contains("ready-to-build") {
        "ready-to-build"
    } else if head.contains("superseded") {
        "superseded"
    } else if head.contains("deprecated") {
        "deprecated"
    } else if head.contains("rejected") {
        "rejected"
    } else if head.contains("accepted") {
        "accepted"
    } else if rel_path.contains("report") {
        "accepted"
    } else {
        "proposed"
    }
}

fn infer_doc_kind(rel_path: &str) -> &'static str {
    if rel_path == "AGENTS.md" {
        "policy"
    } else if rel_path.contains("report") {
        "report"
    } else if rel_path.matches('/').count() >= 3 && rel_path.starts_with("docs/adr/") {
        "sub-adr"
    } else if rel_path.starts_with("docs/adr/") {
        "adr"
    } else {
        "guide"
    }
}

fn extract_adr_number(rel_path: &str) -> Option<String> {
    let fname = Path::new(rel_path).file_stem()?.to_str()?;
    let prefix: String = fname.chars().take_while(|c| c.is_ascii_digit()).collect();
    if prefix.is_empty() {
        None
    } else {
        Some(prefix)
    }
}

fn collect_md_files(dir: &Path, rel_prefix: &str, out: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut sorted: Vec<_> = entries.flatten().collect();
    sorted.sort_by_key(|e| e.file_name());
    for entry in sorted {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let rel = if rel_prefix.is_empty() {
            name.clone()
        } else {
            format!("{rel_prefix}/{name}")
        };
        if path.is_dir() {
            collect_md_files(&path, &rel, out);
        } else if path.is_file() && name.to_lowercase().ends_with(".md") {
            if let Ok(md) = fs::read_to_string(&path) {
                out.push((rel, md));
            }
        }
    }
}

pub async fn seed_cms_from_disk(pool: &PgPool, root_dir: &Path) {
    let mut files: Vec<(String, String)> = Vec::new();
    let docs_dir = root_dir.join("docs");
    if docs_dir.is_dir() {
        collect_md_files(&docs_dir, "docs", &mut files);
    }
    let agents_path = root_dir.join("AGENTS.md");
    if agents_path.is_file() {
        if let Ok(md) = fs::read_to_string(&agents_path) {
            files.push(("AGENTS.md".to_string(), md));
        }
    }

    // Sort shorter paths first so parent ADRs insert before sub-ADRs
    files.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then(a.0.cmp(&b.0)));

    let ts_us = now_timestamp_us();
    for (rel_path, md) in files {
        let file_name = Path::new(&rel_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("doc.md");
        let (title, _, _) = extract_title_and_headings(&md, file_name);
        let doc_kind = infer_doc_kind(&rel_path);
        let status = infer_status_from_md(&md, &rel_path);
        let adr_number = extract_adr_number(&rel_path);
        let slug = rel_path
            .trim_end_matches(".md")
            .replace("docs/adr/", "")
            .replace(['/', '.'], "-");
        let context_problem = extract_madr_section(&md, "context");
        let decision_drivers = extract_madr_section(&md, "drivers");
        let decision_outcome = extract_madr_section(&md, "decision outcome");
        let consequences = extract_madr_section(&md, "consequences");
        let confirmation = extract_madr_section(&md, "confirmation");

        let parent_adr_id: Option<i32> = if doc_kind == "sub-adr" {
            if let Some(parent_dir) = Path::new(&rel_path).parent() {
                let candidate_rel = format!("{}.md", parent_dir.to_string_lossy().replace('\\', "/"));
                sqlx::query_scalar::<_, i32>(
                    "SELECT id FROM adr_records WHERE rel_path = $1 LIMIT 1",
                )
                .bind(candidate_rel)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
            } else {
                None
            }
        } else {
            None
        };

        let _ = sqlx::query(
            r#"
            INSERT INTO adr_records (
                adr_number, parent_adr_id, doc_kind, title, slug, rel_path, status,
                context_problem, decision_drivers, decision_outcome,
                consequences_pros_cons, confirmation_criteria, raw_markdown,
                willow_namespace, willow_path, willow_timestamp_us
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'exonovel', $14, $15)
            ON CONFLICT (rel_path) DO UPDATE
            SET title = CASE WHEN adr_records.cms_edited THEN adr_records.title ELSE EXCLUDED.title END,
                status = CASE WHEN adr_records.cms_edited THEN adr_records.status ELSE EXCLUDED.status END,
                parent_adr_id = COALESCE(adr_records.parent_adr_id, EXCLUDED.parent_adr_id),
                raw_markdown = CASE WHEN adr_records.cms_edited THEN adr_records.raw_markdown ELSE EXCLUDED.raw_markdown END,
                willow_timestamp_us = EXCLUDED.willow_timestamp_us,
                updated_at = NOW()
            "#,
        )
        .bind(adr_number)
        .bind(parent_adr_id)
        .bind(doc_kind)
        .bind(&title)
        .bind(&slug)
        .bind(&rel_path)
        .bind(status)
        .bind(&context_problem)
        .bind(&decision_drivers)
        .bind(&decision_outcome)
        .bind(&consequences)
        .bind(&confirmation)
        .bind(&md)
        .bind(format!("/cms/{rel_path}"))
        .bind(ts_us)
        .execute(pool)
        .await;
    }

    // Seed essential Tooling Catalog from ADR 006 / 002
    let seed_tools = [
        ("leptos", "Leptos", "crates.io", "leptos", "https://github.com/leptos-rs/leptos", "0.8.x", "0.8.x", "Reactive Rust SSR + WASM web framework"),
        ("axum", "Axum", "crates.io", "axum", "https://github.com/tokio-rs/axum", "0.8.x", "0.8.x", "HTTP + SSE backend router"),
        ("sqlx", "SQLx", "crates.io", "sqlx", "https://github.com/launchbadge/sqlx", "0.8.x", "0.8.x", "Async Postgres + PgListener driver"),
        ("loro", "Loro CRDT", "crates.io", "loro", "https://github.com/loro-dev/loro", "1.10.x", "1.10.x", "Real-time collaborative manuscript CRDT engine"),
        ("iroh", "iroh", "crates.io", "iroh", "https://github.com/n0-computer/iroh", "0.97.x", "0.97.x", "Peer-to-peer QUIC networking & relays"),
        ("iroh-gossip", "iroh-gossip", "crates.io", "iroh-gossip", "https://github.com/n0-computer/iroh-gossip", "0.97.x", "0.97.x", "Ephemeral presence & cursor gossip"),
        ("willow25", "Willow (willow25)", "crates.io", "willow25", "https://github.com/earthstar-project/willow-rs", "0.3.x", "0.3.x", "Decentralized Meadowcap store & sync protocol"),
    ];

    for (key, name, eco, pkg, repo, inst, lat, purpose) in seed_tools {
        let _ = sqlx::query(
            r#"
            INSERT INTO tooling_catalog (tool_key, name, ecosystem, package_name, repo_url, installed_version, latest_version, release_notes_url, purpose)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $5 || '/releases', $8)
            ON CONFLICT (tool_key) DO NOTHING
            "#,
        )
        .bind(key)
        .bind(name)
        .bind(eco)
        .bind(pkg)
        .bind(repo)
        .bind(inst)
        .bind(lat)
        .bind(purpose)
        .execute(pool)
        .await;
    }

    // Link key ADRs to Tooling dependencies if not already linked
    let link_pairs = [
        ("docs/adr/002-data/005-loro.md", "loro", ">=1.10.0", "Rich-text LoroText + EphemeralStore cursor sync", true),
        ("docs/adr/002-data/002-iroh.md", "willow25", ">=0.4.0", "Stable production SPSC Willow engine & Meadowcap sync", false),
        ("docs/adr/002-data/002-iroh.md", "iroh", ">=0.97.0", "ExoNovel relay zone discovery", true),
    ];
    for (rel_path, tool_key, constraint, cap, satisfied) in link_pairs {
        if let Ok(Some(adr_id)) =
            sqlx::query_scalar::<_, i32>("SELECT id FROM adr_records WHERE rel_path = $1 LIMIT 1")
                .bind(rel_path)
                .fetch_optional(pool)
                .await
        {
            let _ = sqlx::query(
                r#"
                INSERT INTO adr_tooling_dependencies (adr_id, tool_key, required_constraint, blocking_capability, is_satisfied)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (adr_id, tool_key) DO NOTHING
                "#,
            )
            .bind(adr_id)
            .bind(tool_key)
            .bind(constraint)
            .bind(cap)
            .bind(satisfied)
            .execute(pool)
            .await;
        }
    }

    // Seed multi-provider free inference model catalog (ADR 005-004 — NO Alibaba)
    let seed_models = [
        ("google_ai_studio", "gemini-2.5-flash", "Gemini 2.5 Flash (AI Studio Free)", true, true),
        ("google_ai_studio", "gemini-2.5-flash-lite", "Gemini 2.5 Flash-Lite (AI Studio Free)", true, true),
        ("nvidia_nim", "meta/llama-3.3-70b-instruct", "Llama 3.3 70B Instruct (NVIDIA NIM)", true, true),
        ("nvidia_nim", "qwen/qwen2.5-coder-32b-instruct", "Qwen 2.5 Coder 32B (NVIDIA NIM)", true, true),
        ("openrouter", "deepseek/deepseek-chat-v3-0324:free", "DeepSeek V3 Free (OpenRouter)", true, true),
        ("openrouter", "google/gemma-3-27b-it:free", "Gemma 3 27B Free (OpenRouter)", true, false),
        ("groq", "llama-3.3-70b-versatile", "Llama 3.3 70B Versatile (Groq Free)", true, true),
    ];
    for (prov, model_id, display, free, tools) in seed_models {
        let _ = sqlx::query(
            r#"
            INSERT INTO provider_models (provider, model_id, display_name, is_free, supports_tools, is_active)
            VALUES ($1, $2, $3, $4, $5, TRUE)
            ON CONFLICT (provider, model_id) DO NOTHING
            "#,
        )
        .bind(prov)
        .bind(model_id)
        .bind(display)
        .bind(free)
        .bind(tools)
        .execute(pool)
        .await;
    }

    // Seed external tasks only reachable outside AI Studio (preserves owner's is_done checkbox state)
    let external_tasks = [
        (
            "postfix-dkim-or-brevo-relay",
            "Mail Server (43.157.89.31)",
            "Tencent Agent",
            "Verified: Roundcube & Postfix outbound mail both relay via Brevo SMTP (Gmail Inbox & DMARC pass)",
            "Verified 2026-11-21: Roundcube submits directly to smtp-relay.brevo.com:587, and Postfix on 43.157.89.31 now also relays all outbound mail via [smtp-relay.brevo.com]:587 (SASL plain/login from /opt/exosystems_net/env, TLS encrypt). Live test from root@exosystems.net to Gmail landed in Inbox with mailed-by exomail.exosystems.net and signed-by exosystems.net.",
            "",
            20_i32,
            true,
            "Tencent Agent + Gmail Verified",
        ),
        (
            "brevo-verify-senders",
            "Brevo Dashboard",
            "Owner",
            "Confirm all 3 exosystems.net senders are active in Brevo",
            "Open Brevo -> Settings (bottom left) -> Senders, Domains & IPs -> Senders tab, and confirm notes@exosystems.net (Notes from ExoNovel), noreply@exosystems.net (Exosystems), and support@exosystems.net (ExoSupport) are all green/verified.",
            "",
            30_i32,
            false,
            "",
        ),
        (
            "opencode-ai-keys",
            "Tencent Server (/opt/exonovel/env)",
            "Owner",
            "Install OpenCode CLI and add AI failover keys to /opt/exonovel/env",
            "Install the opencode CLI on the Tencent instance and add GEMINI_API_KEY, NVIDIA_NIM_API_KEY, OPENROUTER_API_KEY (plus optional GROQ_API_KEY) to /opt/exonovel/env so Maturity Proposal generation has multi-provider failover.",
            "",
            40_i32,
            false,
            "",
        ),
        (
            "tencent-iroh-udp-firewall",
            "Tencent Firewall",
            "Owner",
            "Open UDP/QUIC port in Tencent Lighthouse Firewall (when enabling iroh P2P relays)",
            "In Tencent Cloud Lighthouse Console -> click your instance -> Firewall tab -> Add Firewall Rule -> Type: Custom, Protocol: UDP, Source: 0.0.0.0/0, Port: 7842 (or your chosen iroh relay UDP port), Policy: Allow -> Save.",
            "",
            50_i32,
            false,
            "",
        ),
        (
            "github-oauth-later",
            "Auth (Deferred)",
            "Owner",
            "Register GitHub OAuth app after Google OAuth wrap-up",
            "In GitHub Settings -> Developer settings -> OAuth Apps -> New OAuth App, set Authorization callback URL to https://ep-wispy-wave-bgqyejp5.neonauth.c-9.eu-central-1.aws.neon.tech/exonovel/auth/callback/github and add Client ID/Secret in Neon Console -> Auth.",
            "https://ep-wispy-wave-bgqyejp5.neonauth.c-9.eu-central-1.aws.neon.tech/exonovel/auth/callback/github",
            60_i32,
            false,
            "",
        ),
        (
            "google-custom-brand",
            "Google Cloud Branding",
            "Owner",
            "Re-submit Google Brand Verification after pushing /privacy so login displays 'ExoNovel'",
            "Neon Auth is already verified using your custom Google Cloud Client ID & Secret (902812495809-...). Right now Google falls back to displaying 'neon.tech' only because Brand Verification failed on the missing /privacy page. After pushing to main so https://exonovel.exosystems.net/privacy goes live, open Google Auth Platform -> Branding verification issues, select 'I have fixed the issues', and click Proceed.",
            "https://exonovel.exosystems.net/privacy",
            70_i32,
            false,
            "",
        ),
        (
            "payment-gateways-kyc",
            "Mode 3 Payments",
            "Owner",
            "Complete payment gateway KYC/webhooks & set crypto wallet receive addresses",
            "Complete merchant verification and copy webhook signing secrets into /opt/exonovel/env for your active AOTA payment processors (dLocal / Stripe / Razorpay / Flutterwave / Lightning / TON / USDC) before turning on live $9/mo ($90/yr) Mode 3 sync plans.",
            "",
            80_i32,
            false,
            "",
        ),
    ];

    for (key, cat, actor, title, steps, snippet, sort_order, default_done, default_by) in external_tasks {
        let _ = sqlx::query(
            r#"
            INSERT INTO owner_external_tasks (task_key, category, owner_actor, title, step_by_step, copy_snippet, sort_order, is_done, completed_by, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CASE WHEN $8 THEN NOW() ELSE NULL END)
            ON CONFLICT (task_key) DO UPDATE SET
                category = EXCLUDED.category,
                owner_actor = EXCLUDED.owner_actor,
                title = EXCLUDED.title,
                step_by_step = EXCLUDED.step_by_step,
                copy_snippet = EXCLUDED.copy_snippet,
                sort_order = EXCLUDED.sort_order,
                updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(cat)
        .bind(actor)
        .bind(title)
        .bind(steps)
        .bind(snippet)
        .bind(sort_order)
        .bind(default_done)
        .bind(default_by)
        .execute(pool)
        .await;
    }
}

pub async fn api_cms_list_adrs(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.authorized {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Docs access denied." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let rows = sqlx::query(
        r#"
        SELECT id, adr_number, parent_adr_id, doc_kind, title, slug, rel_path, status, superseded_by_id,
               context_problem, decision_drivers, considered_options_json, decision_outcome,
               consequences_pros_cons, confirmation_criteria, decision_makers, consulted, informed,
               raw_markdown, author_did,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        FROM adr_records
        ORDER BY rel_path ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let dep_rows = sqlx::query(
        r#"
        SELECT d.adr_id, d.tool_key, tc.name AS tool_name, tc.installed_version, tc.latest_version,
               d.required_constraint, d.blocking_capability, d.is_satisfied
        FROM adr_tooling_dependencies d
        JOIN tooling_catalog tc ON tc.tool_key = d.tool_key
        ORDER BY d.tool_key ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut deps_by_adr: std::collections::HashMap<i32, Vec<AdrToolDependencyView>> =
        std::collections::HashMap::new();
    for dr in dep_rows {
        let adr_id: i32 = dr.try_get("adr_id").unwrap_or(0);
        deps_by_adr
            .entry(adr_id)
            .or_default()
            .push(AdrToolDependencyView {
                tool_key: dr.try_get("tool_key").unwrap_or_default(),
                tool_name: dr.try_get("tool_name").unwrap_or_default(),
                installed_version: dr.try_get("installed_version").unwrap_or_default(),
                latest_version: dr.try_get("latest_version").unwrap_or_default(),
                required_constraint: dr.try_get("required_constraint").unwrap_or_default(),
                blocking_capability: dr.try_get("blocking_capability").unwrap_or_default(),
                is_satisfied: dr.try_get("is_satisfied").unwrap_or(false),
            });
    }

    let records: Vec<AdrRecord> = rows
        .into_iter()
        .map(|r| {
            let id: i32 = r.try_get("id").unwrap_or(0);
            let rel_path: String = r.try_get("rel_path").unwrap_or_default();
            let raw_markdown: String = r.try_get("raw_markdown").unwrap_or_default();
            let (_, headings, word_count) = extract_title_and_headings(&raw_markdown, &rel_path);
            let html = render_safe_gfm_html(&raw_markdown);
            AdrRecord {
                id,
                adr_number: r.try_get("adr_number").unwrap_or(None),
                parent_adr_id: r.try_get("parent_adr_id").unwrap_or(None),
                doc_kind: r.try_get("doc_kind").unwrap_or_else(|_| "adr".to_string()),
                title: r.try_get("title").unwrap_or_default(),
                slug: r.try_get("slug").unwrap_or_default(),
                rel_path,
                status: r.try_get("status").unwrap_or_else(|_| "proposed".to_string()),
                superseded_by_id: r.try_get("superseded_by_id").unwrap_or(None),
                context_problem: r.try_get("context_problem").unwrap_or_default(),
                decision_drivers: r.try_get("decision_drivers").unwrap_or_default(),
                considered_options: r
                    .try_get("considered_options_json")
                    .unwrap_or_else(|_| serde_json::json!([])),
                decision_outcome: r.try_get("decision_outcome").unwrap_or_default(),
                consequences_pros_cons: r.try_get("consequences_pros_cons").unwrap_or_default(),
                confirmation_criteria: r.try_get("confirmation_criteria").unwrap_or_default(),
                decision_makers: r.try_get("decision_makers").unwrap_or_default(),
                consulted: r.try_get("consulted").unwrap_or_default(),
                informed: r.try_get("informed").unwrap_or_default(),
                raw_markdown,
                html,
                word_count,
                reading_minutes: (word_count / 210).max(1),
                headings,
                author_did: r.try_get("author_did").unwrap_or_default(),
                dependencies: deps_by_adr.remove(&id).unwrap_or_default(),
                updated_at: r.try_get("updated_at").unwrap_or_default(),
            }
        })
        .collect();

    Ok(Json(serde_json::json!({ "adrs": records })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAdrBody {
    pub id: Option<i32>,
    pub adr_number: Option<String>,
    pub parent_adr_id: Option<i32>,
    pub doc_kind: Option<String>,
    pub title: String,
    pub rel_path: String,
    pub status: String,
    pub context_problem: Option<String>,
    pub decision_drivers: Option<String>,
    pub decision_outcome: Option<String>,
    pub consequences_pros_cons: Option<String>,
    pub confirmation_criteria: Option<String>,
    pub decision_makers: Option<String>,
    pub raw_markdown: Option<String>,
}

pub async fn api_cms_upsert_adr(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<UpsertAdrBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin privileges required to modify ADR records." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let valid_status = [
        "proposed",
        "accepted",
        "on-hold",
        "ready-to-build",
        "rejected",
        "deprecated",
        "superseded",
    ];
    if !valid_status.contains(&body.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid MADR 4.0.0 status value." })),
        ));
    }
    let clean_title = body.title.trim();
    let clean_path = body.rel_path.trim().trim_start_matches('/').to_string();
    if clean_title.is_empty() || clean_path.is_empty() || clean_path.contains("..") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Title and valid relative document path are required." })),
        ));
    }

    let ctx_prob = body.context_problem.unwrap_or_default();
    let drivers = body.decision_drivers.unwrap_or_default();
    let outcome = body.decision_outcome.unwrap_or_default();
    let consequences = body.consequences_pros_cons.unwrap_or_default();
    let confirm = body.confirmation_criteria.unwrap_or_default();
    let makers = body.decision_makers.unwrap_or_default();

    let composed_md = match body.raw_markdown.filter(|s| !s.trim().is_empty()) {
        Some(raw) => raw,
        None => format!(
            "---\nstatus: {status}\ndecision-makers: {makers}\n---\n\n# {clean_title}\n\n\
             ## Context and Problem Statement\n\n{ctx_prob}\n\n\
             ## Decision Drivers\n\n{drivers}\n\n\
             ## Decision Outcome\n\n{outcome}\n\n\
             ## Consequences\n\n{consequences}\n\n\
             ## Confirmation\n\n{confirm}\n",
            status = body.status,
        ),
    };

    let slug = clean_path
        .trim_end_matches(".md")
        .replace("docs/adr/", "")
        .replace(['/', '.'], "-");
    let ts_us = now_timestamp_us();
    let author_did = user.did_peer.unwrap_or_default();
    let doc_kind = body.doc_kind.unwrap_or_else(|| "adr".to_string());

    let row_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO adr_records (
            adr_number, parent_adr_id, doc_kind, title, slug, rel_path, status,
            context_problem, decision_drivers, decision_outcome, consequences_pros_cons,
            confirmation_criteria, decision_makers, raw_markdown, cms_edited, author_did,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, TRUE, $15, 'exonovel', $15, $16, $17)
        ON CONFLICT (rel_path) DO UPDATE
        SET title = EXCLUDED.title,
            status = EXCLUDED.status,
            doc_kind = EXCLUDED.doc_kind,
            parent_adr_id = EXCLUDED.parent_adr_id,
            context_problem = EXCLUDED.context_problem,
            decision_drivers = EXCLUDED.decision_drivers,
            decision_outcome = EXCLUDED.decision_outcome,
            consequences_pros_cons = EXCLUDED.consequences_pros_cons,
            confirmation_criteria = EXCLUDED.confirmation_criteria,
            decision_makers = EXCLUDED.decision_makers,
            raw_markdown = EXCLUDED.raw_markdown,
            cms_edited = TRUE,
            willow_timestamp_us = EXCLUDED.willow_timestamp_us,
            updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(body.adr_number)
    .bind(body.parent_adr_id)
    .bind(&doc_kind)
    .bind(clean_title)
    .bind(&slug)
    .bind(&clean_path)
    .bind(&body.status)
    .bind(&ctx_prob)
    .bind(&drivers)
    .bind(&outcome)
    .bind(&consequences)
    .bind(&confirm)
    .bind(&makers)
    .bind(&composed_md)
    .bind(&author_did)
    .bind(format!("/cms/{clean_path}"))
    .bind(ts_us)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "authorized",
        "adr",
        row_id.to_string(),
        if body.id.is_some() { "update" } else { "insert" },
        vec!["title".to_string(), "status".to_string(), "rawMarkdown".to_string()],
        serde_json::json!({ "id": row_id, "relPath": clean_path, "title": clean_title, "status": body.status }),
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true, "id": row_id, "relPath": clean_path })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkDependencyBody {
    pub adr_id: i32,
    pub tool_key: String,
    pub required_constraint: String,
    pub blocking_capability: String,
    pub is_satisfied: bool,
}

pub async fn api_cms_link_dependency(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<LinkDependencyBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    sqlx::query(
        r#"
        INSERT INTO adr_tooling_dependencies (adr_id, tool_key, required_constraint, blocking_capability, is_satisfied)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (adr_id, tool_key) DO UPDATE
        SET required_constraint = EXCLUDED.required_constraint,
            blocking_capability = EXCLUDED.blocking_capability,
            is_satisfied = EXCLUDED.is_satisfied,
            updated_at = NOW()
        "#,
    )
    .bind(body.adr_id)
    .bind(body.tool_key.trim())
    .bind(body.required_constraint.trim())
    .bind(body.blocking_capability.trim())
    .bind(body.is_satisfied)
    .execute(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "authorized",
        "adr",
        body.adr_id.to_string(),
        "update",
        vec!["dependencies".to_string()],
        serde_json::json!({ "adrId": body.adr_id, "toolKey": body.tool_key }),
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn api_cms_tooling_and_notes(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.authorized {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Docs access denied." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let t_rows = sqlx::query(
        r#"
        SELECT tool_key, name, ecosystem, package_name, repo_url, installed_version, latest_version,
               release_notes_url, purpose,
               to_char(last_checked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_checked_at
        FROM tooling_catalog
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let tools: Vec<ToolCatalogItem> = t_rows
        .into_iter()
        .map(|r| ToolCatalogItem {
            tool_key: r.try_get("tool_key").unwrap_or_default(),
            name: r.try_get("name").unwrap_or_default(),
            ecosystem: r.try_get("ecosystem").unwrap_or_default(),
            package_name: r.try_get("package_name").unwrap_or_default(),
            repo_url: r.try_get("repo_url").unwrap_or_default(),
            installed_version: r.try_get("installed_version").unwrap_or_default(),
            latest_version: r.try_get("latest_version").unwrap_or_default(),
            release_notes_url: r.try_get("release_notes_url").unwrap_or_default(),
            purpose: r.try_get("purpose").unwrap_or_default(),
            last_checked_at: r.try_get("last_checked_at").unwrap_or_default(),
        })
        .collect();

    let n_rows = sqlx::query(
        r#"
        SELECT n.id, n.adr_id, a.title AS adr_title, n.tool_key, n.target_version, n.note_kind,
               n.title, n.rationale, n.proposed_feature_changes, n.provider_used, n.model_used, n.status,
               to_char(n.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM maturity_proposal_notes n
        LEFT JOIN adr_records a ON a.id = n.adr_id
        ORDER BY n.created_at DESC
        LIMIT 80
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let notes: Vec<MaturityNoteItem> = n_rows
        .into_iter()
        .map(|r| MaturityNoteItem {
            id: r.try_get("id").unwrap_or(0),
            adr_id: r.try_get("adr_id").unwrap_or(None),
            adr_title: r.try_get("adr_title").unwrap_or(None),
            tool_key: r.try_get("tool_key").unwrap_or_default(),
            target_version: r.try_get("target_version").unwrap_or_default(),
            note_kind: r.try_get("note_kind").unwrap_or_default(),
            title: r.try_get("title").unwrap_or_default(),
            rationale: r.try_get("rationale").unwrap_or_default(),
            proposed_feature_changes: r.try_get("proposed_feature_changes").unwrap_or_default(),
            provider_used: r.try_get("provider_used").unwrap_or_default(),
            model_used: r.try_get("model_used").unwrap_or_default(),
            status: r.try_get("status").unwrap_or_default(),
            created_at: r.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let m_rows = sqlx::query(
        r#"
        SELECT provider, model_id, display_name, is_free, supports_tools, is_active,
               to_char(cooldown_until AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS cooldown_until,
               to_char(deprecated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS deprecated_at,
               to_char(last_verified_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_verified_at
        FROM provider_models
        ORDER BY provider ASC, model_id ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let models: Vec<ProviderModelInfo> = m_rows
        .into_iter()
        .map(|r| {
            let provider: String = r.try_get("provider").unwrap_or_default();
            let key_configured = match provider.as_str() {
                "google_ai_studio" => env_non_empty("GEMINI_API_KEY"),
                "nvidia_nim" => env_non_empty("NVIDIA_NIM_API_KEY"),
                "openrouter" => env_non_empty("OPENROUTER_API_KEY"),
                "groq" => env_non_empty("GROQ_API_KEY"),
                "cerebras" => env_non_empty("CEREBRAS_API_KEY"),
                "mistral" => env_non_empty("MISTRAL_API_KEY"),
                _ => false,
            };
            ProviderModelInfo {
                provider,
                model_id: r.try_get("model_id").unwrap_or_default(),
                display_name: r.try_get("display_name").unwrap_or_default(),
                is_free: r.try_get("is_free").unwrap_or(true),
                supports_tools: r.try_get("supports_tools").unwrap_or(true),
                is_active: r.try_get("is_active").unwrap_or(true),
                key_configured,
                cooldown_until: r.try_get("cooldown_until").unwrap_or(None),
                deprecated_at: r.try_get("deprecated_at").unwrap_or(None),
                last_verified_at: r.try_get("last_verified_at").unwrap_or_default(),
            }
        })
        .collect();

    Ok(Json(serde_json::json!({
        "tools": tools,
        "notes": notes,
        "providerModels": models,
    })))
}

fn env_non_empty(name: &str) -> bool {
    std::env::var(name)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddToolBody {
    pub tool_key: String,
    pub name: String,
    pub ecosystem: String,
    pub package_name: String,
    pub repo_url: Option<String>,
    pub installed_version: Option<String>,
    pub purpose: Option<String>,
}

pub async fn api_cms_upsert_tool(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<AddToolBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let clean_key = body.tool_key.trim().to_lowercase();
    if clean_key.is_empty() || body.package_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Tool key and package name are required." })),
        ));
    }

    sqlx::query(
        r#"
        INSERT INTO tooling_catalog (tool_key, name, ecosystem, package_name, repo_url, installed_version, latest_version, purpose)
        VALUES ($1, $2, $3, $4, $5, $6, $6, $7)
        ON CONFLICT (tool_key) DO UPDATE
        SET name = EXCLUDED.name,
            ecosystem = EXCLUDED.ecosystem,
            package_name = EXCLUDED.package_name,
            repo_url = EXCLUDED.repo_url,
            installed_version = EXCLUDED.installed_version,
            purpose = EXCLUDED.purpose,
            updated_at = NOW()
        "#,
    )
    .bind(&clean_key)
    .bind(body.name.trim())
    .bind(body.ecosystem.trim())
    .bind(body.package_name.trim())
    .bind(body.repo_url.as_deref().unwrap_or("").trim())
    .bind(body.installed_version.as_deref().unwrap_or("0.1.0").trim())
    .bind(body.purpose.as_deref().unwrap_or("").trim())
    .execute(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "authorized",
        "tooling",
        &clean_key,
        "update",
        vec!["toolKey".to_string()],
        serde_json::json!({ "toolKey": clean_key }),
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Deterministic scan against crates.io API for all crates.io dependencies in tooling_catalog
pub async fn api_cms_run_deterministic_scan(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let client = reqwest::Client::builder()
        .user_agent("ExoNovelMaturityScanner/1.0 (+https://exonovel.exosystems.net)")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let tools = sqlx::query(
        "SELECT tool_key, ecosystem, package_name, latest_version FROM tooling_catalog WHERE ecosystem = 'crates.io'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut updated_count = 0_usize;
    let mut new_notes = 0_usize;

    for row in tools {
        let tool_key: String = row.try_get("tool_key").unwrap_or_default();
        let pkg: String = row.try_get("package_name").unwrap_or_default();
        let prev_latest: String = row.try_get("latest_version").unwrap_or_default();

        // Only allow strict alphanumeric/hyphen/underscore crate names against hardcoded host crates.io
        if !pkg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            continue;
        }
        let url = format!("https://crates.io/api/v1/crates/{pkg}");
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(max_ver) = json
                        .get("crate")
                        .and_then(|c| c.get("max_stable_version").or_else(|| c.get("max_version")))
                        .and_then(|v| v.as_str())
                    {
                        let _ = sqlx::query(
                            "UPDATE tooling_catalog SET latest_version = $2, last_checked_at = NOW(), updated_at = NOW() WHERE tool_key = $1",
                        )
                        .bind(&tool_key)
                        .bind(max_ver)
                        .execute(pool)
                        .await;

                        let _ = sqlx::query(
                            r#"
                            INSERT INTO tooling_versions (tool_key, version, release_url, summary)
                            VALUES ($1, $2, $3, 'Verified via deterministic crates.io registry scan.')
                            ON CONFLICT (tool_key, version) DO NOTHING
                            "#,
                        )
                        .bind(&tool_key)
                        .bind(max_ver)
                        .bind(format!("https://crates.io/crates/{pkg}/{max_ver}"))
                        .execute(pool)
                        .await;

                        updated_count += 1;

                        if !prev_latest.is_empty() && prev_latest != max_ver {
                            let _ = sqlx::query(
                                r#"
                                INSERT INTO maturity_proposal_notes (
                                    tool_key, target_version, note_kind, title, rationale,
                                    proposed_feature_changes, provider_used, status
                                )
                                VALUES ($1, $2, 'new-opportunity', $3, $4, $5, 'deterministic', 'open')
                                "#,
                            )
                            .bind(&tool_key)
                            .bind(max_ver)
                            .bind(format!("{pkg} {max_ver} released (was {prev_latest})"))
                            .bind("Detected deterministically via crates.io release API without AI inference cost.")
                            .bind(format!("Review release notes for {pkg} {max_ver} and verify linked ADR constraints."))
                            .execute(pool)
                            .await;
                            new_notes += 1;
                        }
                    }
                }
            }
        }
    }

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "authorized",
        "maturity_note",
        "scan",
        "insert",
        vec!["latestVersion".to_string()],
        serde_json::json!({ "updatedTools": updated_count, "newNotes": new_notes }),
    )
    .await;

    Ok(Json(serde_json::json!({
        "ok": true,
        "updatedTools": updated_count,
        "newNotes": new_notes,
    })))
}

/// Live /v1/models discovery across Google AI Studio, NVIDIA NIM, OpenRouter (:free), and Groq
/// Automatically marks missing models as deprecated_at = NOW() so OpenCode never gets stuck on removed models.
pub async fn api_cms_refresh_models(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let client = reqwest::Client::new();
    let mut verified_total = 0_usize;

    // 1. NVIDIA NIM live models
    if let Ok(key) = std::env::var("NVIDIA_NIM_API_KEY") {
        let key = key.trim();
        if !key.is_empty() {
            if let Ok(resp) = client
                .get("https://integrate.api.nvidia.com/v1/models")
                .bearer_auth(key)
                .send()
                .await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = json.get("data").and_then(|d| d.as_array()) {
                        let mut live_ids = Vec::new();
                        for item in arr.iter().take(40) {
                            if let Some(id) = item.get("id").and_then(|s| s.as_str()) {
                                live_ids.push(id.to_string());
                                let _ = sqlx::query(
                                    r#"
                                    INSERT INTO provider_models (provider, model_id, display_name, is_free, supports_tools, is_active, deprecated_at, last_verified_at)
                                    VALUES ('nvidia_nim', $1, $1, TRUE, TRUE, TRUE, NULL, NOW())
                                    ON CONFLICT (provider, model_id) DO UPDATE
                                    SET deprecated_at = NULL, is_active = TRUE, last_verified_at = NOW()
                                    "#,
                                )
                                .bind(id)
                                .execute(pool)
                                .await;
                                verified_total += 1;
                            }
                        }
                        if !live_ids.is_empty() {
                            let _ = sqlx::query(
                                "UPDATE provider_models SET is_active = FALSE, deprecated_at = NOW() WHERE provider = 'nvidia_nim' AND NOT (model_id = ANY($1))",
                            )
                            .bind(&live_ids)
                            .execute(pool)
                            .await;
                        }
                    }
                }
            }
        }
    }

    // 2. OpenRouter live :free models
    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        let key = key.trim();
        if !key.is_empty() {
            if let Ok(resp) = client
                .get("https://openrouter.ai/api/v1/models")
                .bearer_auth(key)
                .send()
                .await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = json.get("data").and_then(|d| d.as_array()) {
                        let mut live_ids = Vec::new();
                        for item in arr {
                            if let Some(id) = item.get("id").and_then(|s| s.as_str()) {
                                if id.ends_with(":free") {
                                    live_ids.push(id.to_string());
                                    let name = item
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or(id);
                                    let _ = sqlx::query(
                                        r#"
                                        INSERT INTO provider_models (provider, model_id, display_name, is_free, supports_tools, is_active, deprecated_at, last_verified_at)
                                        VALUES ('openrouter', $1, $2, TRUE, TRUE, TRUE, NULL, NOW())
                                        ON CONFLICT (provider, model_id) DO UPDATE
                                        SET display_name = EXCLUDED.display_name, deprecated_at = NULL, is_active = TRUE, last_verified_at = NOW()
                                        "#,
                                    )
                                    .bind(id)
                                    .bind(name)
                                    .execute(pool)
                                    .await;
                                    verified_total += 1;
                                }
                            }
                        }
                        if !live_ids.is_empty() {
                            let _ = sqlx::query(
                                "UPDATE provider_models SET is_active = FALSE, deprecated_at = NOW() WHERE provider = 'openrouter' AND NOT (model_id = ANY($1))",
                            )
                            .bind(&live_ids)
                            .execute(pool)
                            .await;
                        }
                    }
                }
            }
        }
    }

    // 3. Google AI Studio live Gemini models
    if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        let key = key.trim();
        if !key.is_empty() {
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={key}");
            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = json.get("models").and_then(|d| d.as_array()) {
                        for item in arr.iter().take(25) {
                            if let Some(raw_name) = item.get("name").and_then(|s| s.as_str()) {
                                let id = raw_name.trim_start_matches("models/");
                                if id.contains("gemini") {
                                    let display = item
                                        .get("displayName")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or(id);
                                    let _ = sqlx::query(
                                        r#"
                                        INSERT INTO provider_models (provider, model_id, display_name, is_free, supports_tools, is_active, deprecated_at, last_verified_at)
                                        VALUES ('google_ai_studio', $1, $2, TRUE, TRUE, TRUE, NULL, NOW())
                                        ON CONFLICT (provider, model_id) DO UPDATE
                                        SET display_name = EXCLUDED.display_name, deprecated_at = NULL, is_active = TRUE, last_verified_at = NOW()
                                        "#,
                                    )
                                    .bind(id)
                                    .bind(display)
                                    .execute(pool)
                                    .await;
                                    verified_total += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "authorized",
        "tooling",
        "provider_models",
        "update",
        vec!["providerModels".to_string()],
        serde_json::json!({ "verified": verified_total }),
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true, "verified": verified_total })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateProposalBody {
    pub tool_key: String,
    pub adr_id: Option<i32>,
}

/// Cross-provider OpenAI-compatible + Gemini failover generator for new Maturity Proposal Notes.
/// Tries Google AI Studio -> NVIDIA NIM -> OpenRouter :free -> Groq in health order,
/// marking 404/410 deprecated models & 429 cooldown models automatically.
pub async fn api_cms_generate_proposal(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<GenerateProposalBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let tool_row = sqlx::query(
        "SELECT name, installed_version, latest_version, purpose FROM tooling_catalog WHERE tool_key = $1 LIMIT 1",
    )
    .bind(body.tool_key.trim())
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;
    let Some(tr) = tool_row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tool not found in catalog." })),
        ));
    };

    let tool_name: String = tr.try_get("name").unwrap_or_default();
    let inst_v: String = tr.try_get("installed_version").unwrap_or_default();
    let lat_v: String = tr.try_get("latest_version").unwrap_or_default();
    let purpose: String = tr.try_get("purpose").unwrap_or_default();

    let prompt = format!(
        "Tool: {tool_name} ({purpose}). Installed: {inst_v}. Latest: {lat_v}.\n\
         In 3 concise bullet points (plain text, no markdown tables), explain what ExoNovel features or architectural capabilities this upgrade unlocks and suggest one opportunistic product improvement aligned with sovereign local-first creative writing."
    );

    let candidates = sqlx::query(
        r#"
        SELECT provider, model_id
        FROM provider_models
        WHERE is_active = TRUE
          AND deprecated_at IS NULL
          AND (cooldown_until IS NULL OR cooldown_until < NOW())
        ORDER BY CASE provider
            WHEN 'google_ai_studio' THEN 0
            WHEN 'nvidia_nim' THEN 1
            WHEN 'openrouter' THEN 2
            ELSE 3 END, last_verified_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let client = reqwest::Client::new();
    let mut chosen_provider = String::new();
    let mut chosen_model = String::new();
    let mut generated_text: Option<String> = None;

    for cand in candidates {
        let prov: String = cand.try_get("provider").unwrap_or_default();
        let mid: String = cand.try_get("model_id").unwrap_or_default();

        let (endpoint_opt, api_key_opt, is_gemini) = match prov.as_str() {
            "google_ai_studio" => (
                Some(format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{mid}:generateContent"
                )),
                std::env::var("GEMINI_API_KEY").ok(),
                true,
            ),
            "nvidia_nim" => (
                Some("https://integrate.api.nvidia.com/v1/chat/completions".to_string()),
                std::env::var("NVIDIA_NIM_API_KEY").ok(),
                false,
            ),
            "openrouter" => (
                Some("https://openrouter.ai/api/v1/chat/completions".to_string()),
                std::env::var("OPENROUTER_API_KEY").ok(),
                false,
            ),
            "groq" => (
                Some("https://api.groq.com/openai/v1/chat/completions".to_string()),
                std::env::var("GROQ_API_KEY").ok(),
                false,
            ),
            _ => (None, None, false),
        };

        let (Some(url), Some(key)) = (endpoint_opt, api_key_opt.filter(|k| !k.trim().is_empty())) else {
            continue;
        };

        let res = if is_gemini {
            client
                .post(&url)
                .header("x-goog-api-key", key.trim())
                .json(&serde_json::json!({
                    "contents": [{ "parts": [{ "text": &prompt }] }]
                }))
                .send()
                .await
        } else {
            client
                .post(&url)
                .bearer_auth(key.trim())
                .json(&serde_json::json!({
                    "model": mid,
                    "messages": [{ "role": "user", "content": &prompt }],
                    "max_tokens": 350
                }))
                .send()
                .await
        };

        match res {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    let txt = if is_gemini {
                        json.get("candidates")
                            .and_then(|c| c.get(0))
                            .and_then(|c| c.get("content"))
                            .and_then(|c| c.get("parts"))
                            .and_then(|p| p.get(0))
                            .and_then(|p| p.get("text"))
                            .and_then(|t| t.as_str())
                            .map(str::to_string)
                    } else {
                        json.get("choices")
                            .and_then(|c| c.get(0))
                            .and_then(|c| c.get("message"))
                            .and_then(|m| m.get("content"))
                            .and_then(|t| t.as_str())
                            .map(str::to_string)
                    };
                    if let Some(clean_txt) = txt.filter(|t| !t.trim().is_empty()) {
                        chosen_provider = prov;
                        chosen_model = mid;
                        generated_text = Some(clean_txt);
                        break;
                    }
                }
            }
            Ok(resp) => {
                let code = resp.status().as_u16();
                if code == 404 || code == 410 {
                    let _ = sqlx::query(
                        "UPDATE provider_models SET is_active = FALSE, deprecated_at = NOW() WHERE provider = $1 AND model_id = $2",
                    )
                    .bind(&prov)
                    .bind(&mid)
                    .execute(pool)
                    .await;
                } else if code == 429 {
                    let _ = sqlx::query(
                        "UPDATE provider_models SET cooldown_until = NOW() + INTERVAL '5 minutes' WHERE provider = $1 AND model_id = $2",
                    )
                    .bind(&prov)
                    .bind(&mid)
                    .execute(pool)
                    .await;
                }
            }
            Err(_) => continue,
        }
    }

    let proposal_body = generated_text.unwrap_or_else(|| {
        chosen_provider = "deterministic".to_string();
        chosen_model = "none-configured".to_string();
        format!(
            "No active remote inference keys answered right now. Deterministic recommendation: audit {tool_name} changelog between {inst_v} and {lat_v}, verify tests on upgrade branch, and review blocked ADRs."
        )
    });

    let note_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO maturity_proposal_notes (
            adr_id, tool_key, target_version, note_kind, title, rationale,
            proposed_feature_changes, provider_used, model_used, status
        )
        VALUES ($1, $2, $3, 'new-opportunity', $4, $5, $6, $7, $8, 'open')
        RETURNING id
        "#,
    )
    .bind(body.adr_id)
    .bind(body.tool_key.trim())
    .bind(&lat_v)
    .bind(format!("{tool_name} {lat_v}: Feature Opportunity Proposal"))
    .bind(format!("Generated via OpenCode failover pipeline ({chosen_provider} / {chosen_model})."))
    .bind(&proposal_body)
    .bind(&chosen_provider)
    .bind(&chosen_model)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "authorized",
        "maturity_note",
        note_id.to_string(),
        "insert",
        vec!["title".to_string(), "providerUsed".to_string()],
        serde_json::json!({ "id": note_id, "provider": chosen_provider, "model": chosen_model }),
    )
    .await;

    Ok(Json(serde_json::json!({
        "ok": true,
        "noteId": note_id,
        "provider": chosen_provider,
        "model": chosen_model,
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteStatusBody {
    pub status: String,
}

pub async fn api_cms_update_note_status(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    AxumPath(id): AxumPath<i32>,
    Json(body): Json<UpdateNoteStatusBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    if !["open", "accepted", "dismissed"].contains(&body.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Status must be open, accepted, or dismissed." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let _ = sqlx::query("UPDATE maturity_proposal_notes SET status = $2 WHERE id = $1")
        .bind(id)
        .bind(&body.status)
        .execute(pool)
        .await;

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "authorized",
        "maturity_note",
        id.to_string(),
        "update",
        vec!["status".to_string()],
        serde_json::json!({ "id": id, "status": body.status }),
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn api_cms_list_tasks(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.authorized {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Docs access denied." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let rows = sqlx::query(
        r#"
        SELECT task_key, category, owner_actor, title, step_by_step, copy_snippet,
               is_done, completed_by,
               to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS completed_at,
               sort_order
        FROM owner_external_tasks
        ORDER BY is_done ASC, sort_order ASC, task_key ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let tasks: Vec<OwnerExternalTask> = rows
        .into_iter()
        .map(|r| OwnerExternalTask {
            task_key: r.try_get("task_key").unwrap_or_default(),
            category: r.try_get("category").unwrap_or_default(),
            owner_actor: r.try_get("owner_actor").unwrap_or_default(),
            title: r.try_get("title").unwrap_or_default(),
            step_by_step: r.try_get("step_by_step").unwrap_or_default(),
            copy_snippet: r.try_get("copy_snippet").unwrap_or_default(),
            is_done: r.try_get("is_done").unwrap_or(false),
            completed_by: r.try_get("completed_by").unwrap_or_default(),
            completed_at: r.try_get("completed_at").ok(),
            sort_order: r.try_get("sort_order").unwrap_or(100),
        })
        .collect();

    Ok(Json(serde_json::json!({ "tasks": tasks })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleTaskBody {
    pub is_done: bool,
}

pub async fn api_cms_toggle_task(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    AxumPath(task_key): AxumPath<String>,
    Json(body): Json<ToggleTaskBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required to update tasks." })),
        ));
    }
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };

    let who = user
        .email
        .clone()
        .or(user.did_peer.clone())
        .unwrap_or_else(|| "admin".to_string());

    let res = sqlx::query(
        r#"
        UPDATE owner_external_tasks
        SET is_done = $2,
            completed_by = CASE WHEN $2 THEN $3 ELSE '' END,
            completed_at = CASE WHEN $2 THEN NOW() ELSE NULL END,
            updated_at = NOW()
        WHERE task_key = $1
        "#,
    )
    .bind(task_key.trim())
    .bind(body.is_done)
    .bind(&who)
    .execute(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if res.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Task not found." })),
        ));
    }

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "authorized",
        "task",
        task_key.trim().to_string(),
        "update",
        vec!["isDone".to_string()],
        serde_json::json!({ "taskKey": task_key.trim(), "isDone": body.is_done }),
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true, "isDone": body.is_done })))
}
