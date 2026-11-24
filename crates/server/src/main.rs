#![recursion_limit = "512"]

mod auth;
mod cms;
mod db;
mod docs;
mod i18n;
mod live;
mod media;
mod notify;

use std::{fs, path::PathBuf};

use axum::{
    extract::{DefaultBodyLimit, FromRef, Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use exonovel_app::App;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_meta::MetaTags;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    auth::{AuthUser, JwksCache},
    cms::{
        api_cms_generate_proposal, api_cms_link_dependency, api_cms_list_adrs,
        api_cms_list_tasks, api_cms_refresh_models, api_cms_run_deterministic_scan,
        api_cms_toggle_task, api_cms_tooling_and_notes, api_cms_update_note_status,
        api_cms_upsert_adr, api_cms_upsert_tool, ensure_cms_schema, seed_cms_from_disk,
    },
    db::{
        claim_invite, connect_pool, create_group, create_invite, create_persona,
        delete_allowlist_entry, ensure_schema, get_admin_email, list_allowlist,
        list_groups_for_user, list_identities, list_invites_and_referrals, list_personas,
        load_media_variant, preview_invite, save_avatar_asset, set_group_member_role,
        update_identity_account_role, update_persona, update_user_profile, upsert_allowlist_entry,
        PersonaUpdateInput, ProfileUpdateInput,
    },
    docs::{
        extract_title_and_headings, get_docs_tree, render_safe_gfm_html, resolve_safe_doc_path,
        DocPayload,
    },
    i18n::{api_i18n_catalog, api_i18n_pack},
    live::{
        api_events_stream, api_issue_sse_ticket, emit_live_patch, spawn_pg_listener, LiveBus,
    },
    media::{build_avatar_derivatives, decode_avatar_data_url},
    notify::{
        api_activate_sync_plan, api_get_notification_prefs, api_list_billing,
        api_list_notifications, api_mark_notification_read, api_register_notification_device,
        api_test_notification, api_update_notification_prefs, ensure_notify_and_billing_schema,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub db: Option<PgPool>,
    pub neon_auth_base_url: Option<String>,
    pub neon_auth_jwks_url: Option<String>,
    pub jwks_cache: JwksCache,
    pub live_bus: LiveBus,
    pub root_dir: PathBuf,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::from_filename(".env.local");
    let _ = dotenvy::dotenv();

    let conf = get_configuration(None).expect("Failed to load Leptos configuration");
    let mut leptos_options = conf.leptos_options;

    if let Ok(port_str) = std::env::var("PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            leptos_options.site_addr.set_port(port);
        }
    }
    let addr = leptos_options.site_addr;

    let root_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let db = connect_pool().await;
    if let Some(pool) = &db {
        if let Err(err) = ensure_schema(pool).await {
            eprintln!("Database schema initialization warning: {err}");
        }
        if let Err(err) = ensure_notify_and_billing_schema(pool).await {
            eprintln!("Notify/billing schema warning: {err}");
        }
        if let Err(err) = ensure_cms_schema(pool).await {
            eprintln!("CMS schema warning: {err}");
        }
        seed_cms_from_disk(pool, &root_dir).await;
    }

    let live_bus = LiveBus::new();
    spawn_pg_listener(live_bus.clone());

    let neon_auth_base_url = std::env::var("NEON_AUTH_BASE_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let neon_auth_jwks_url = std::env::var("NEON_AUTH_JWKS_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let state = AppState {
        leptos_options: leptos_options.clone(),
        db,
        neon_auth_base_url,
        neon_auth_jwks_url,
        jwks_cache: JwksCache::new(),
        live_bus,
        root_dir,
    };

    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/auth-config", get(api_auth_config))
        .route("/api/me", get(api_me))
        .route("/api/events/ticket", post(api_issue_sse_ticket))
        .route("/api/events/stream", get(api_events_stream))
        .route("/api/i18n/catalog", get(api_i18n_catalog))
        .route("/api/i18n/{locale}", get(api_i18n_pack))
        .route("/api/profile", put(api_update_profile))
        .route(
            "/api/media/avatar",
            post(api_upload_avatar).layer(DefaultBodyLimit::max(4 * 1024 * 1024)),
        )
        .route("/api/media/{asset_id}/{variant}", get(api_serve_media))
        .route(
            "/api/personas",
            get(api_list_personas)
                .post(api_create_persona)
                .put(api_update_persona),
        )
        .route("/api/groups", get(api_list_groups).post(api_create_group))
        .route("/api/groups/member-role", put(api_set_group_member_role))
        .route("/api/invites", get(api_list_invites).post(api_create_invite))
        .route("/api/invites/preview", get(api_preview_invite))
        .route("/api/invites/claim", post(api_claim_invite))
        .route("/api/notifications", get(api_list_notifications))
        .route("/api/notifications/{id}/read", post(api_mark_notification_read))
        .route(
            "/api/notifications/preferences",
            get(api_get_notification_prefs).put(api_update_notification_prefs),
        )
        .route("/api/notifications/devices", post(api_register_notification_device))
        .route("/api/notifications/test", post(api_test_notification))
        .route("/api/billing", get(api_list_billing))
        .route("/api/billing/sync-plan", post(api_activate_sync_plan))
        .route("/api/docs", get(api_docs))
        .route("/api/doc", get(api_doc))
        .route("/api/cms/adrs", get(api_cms_list_adrs).post(api_cms_upsert_adr))
        .route("/api/cms/dependencies", post(api_cms_link_dependency))
        .route("/api/cms/tooling", get(api_cms_tooling_and_notes).post(api_cms_upsert_tool))
        .route("/api/cms/scan", post(api_cms_run_deterministic_scan))
        .route("/api/cms/models/refresh", post(api_cms_refresh_models))
        .route("/api/cms/proposals", post(api_cms_generate_proposal))
        .route("/api/cms/proposals/{id}", put(api_cms_update_note_status))
        .route("/api/cms/tasks", get(api_cms_list_tasks))
        .route("/api/cms/tasks/{task_key}/toggle", post(api_cms_toggle_task))
        .route(
            "/api/admin/allowlist",
            get(api_admin_list)
                .post(api_admin_upsert)
                .delete(api_admin_delete),
        )
        .route("/api/admin/identities", get(api_admin_identities))
        .route(
            "/api/admin/identities/profile",
            put(api_admin_update_identity_profile),
        )
        .leptos_routes(&state, routes, {
            let options = state.leptos_options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");
    println!("ExoNovel Rust + Leptos server listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server error");
}

fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" dir="ltr" data-appearance="system">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="application-name" content="ExoNovel"/>
                <meta name="description" content="ExoNovel is a collaborative creative writing and local-first publishing platform operated by Exosystems where authors retain cryptographic ownership of their novels, scripts, stories, essays, and poetry."/>
                <meta property="og:site_name" content="ExoNovel"/>
                <meta property="og:title" content="ExoNovel — Collaborative Creative Writing & Sovereign Publishing"/>
                <meta property="og:description" content="Collaborative creative writing and local-first publishing platform operated by Exosystems. Draft novels, scripts, articles, short stories, and poetry with verifiable author ownership."/>
                <meta property="og:type" content="website"/>
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin=""/>
                <link href="https://fonts.googleapis.com/css2?family=DM+Sans:ital,opsz,wght@0,9..40,400..700;1,9..40,400..600&family=JetBrains+Mono:wght@400;500;600&family=Plus+Jakarta+Sans:wght@500;600;700&display=swap" rel="stylesheet"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <link rel="icon" type="image/png" href="/exonovel-symbol.png" />
                <link rel="stylesheet" id="leptos" href="/pkg/exonovel.css" />
                <MetaTags />
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

fn db_unavailable() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({ "error": "Database unavailable." })),
    )
}

async fn api_health(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = if let Some(pool) = &state.db {
        sqlx::query("SELECT 1").execute(pool).await.is_ok()
    } else {
        false
    };
    let auth_ok = state.neon_auth_base_url.is_some() && state.neon_auth_jwks_url.is_some();

    Json(serde_json::json!({
        "ok": true,
        "runtime": "rust-leptos",
        "services": {
            "postgres": db_ok,
            "auth": auth_ok,
        }
    }))
}

async fn api_auth_config(State(state): State<AppState>) -> impl IntoResponse {
    let auth_configured =
        state.neon_auth_base_url.is_some() && state.neon_auth_jwks_url.is_some();
    Json(serde_json::json!({
        "authConfigured": auth_configured,
        "authUrl": state.neon_auth_base_url,
        "adminEmail": get_admin_email(),
    }))
}

async fn api_me(AuthUser(user): AuthUser) -> impl IntoResponse {
    Json(serde_json::json!({
        "authenticated": true,
        "user": user,
    }))
}

async fn api_update_profile(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<ProfileUpdateInput>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };

    if body.willow_sync_enabled == Some(true)
        && user
            .profile
            .as_ref()
            .is_none_or(|p| p.sync_plan_tier == "free")
    {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            Json(serde_json::json!({
                "error": "Active ExoNovel Sync subscription required in Profile -> Wallet, Paywalls & Purchases."
            })),
        ));
    }

    let profile = update_user_profile(pool, did_peer, &body)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    let personas = list_personas(pool, did_peer).await.unwrap_or_default();
    let active_persona = personas
        .iter()
        .find(|p| p.persona_did == profile.active_persona_did)
        .or_else(|| personas.first())
        .cloned();

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{did_peer}"),
        "profile",
        did_peer,
        "update",
        vec!["profile".to_string()],
        serde_json::json!({
            "profile": profile,
            "personas": personas,
            "activePersona": active_persona,
        }),
    )
    .await;

    Ok(Json(serde_json::json!({
        "profile": profile,
        "personas": personas,
        "activePersona": active_persona,
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadAvatarBody {
    data_url: String,
    persona_did: Option<String>,
}

async fn api_upload_avatar(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<UploadAvatarBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };

    let raw_bytes = decode_avatar_data_url(&body.data_url)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    let derivatives = tokio::task::spawn_blocking(move || build_avatar_derivatives(&raw_bytes))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Image worker error: {e}") })),
            )
        })?
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;

    let result = save_avatar_asset(pool, owner_did, body.persona_did.as_deref(), derivatives)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;

    Ok(Json(result))
}

async fn api_serve_media(
    State(state): State<AppState>,
    Path((asset_id, variant)): Path<(String, String)>,
) -> impl IntoResponse {
    let Some(pool) = state.db.as_ref() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let Some((mime, digest, bytes)) = load_media_variant(pool, &asset_id, &variant).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let mut headers = axum::http::HeaderMap::new();
    if let Ok(ct) = HeaderValue::from_str(&mime) {
        headers.insert(header::CONTENT_TYPE, ct);
    }
    if let Ok(etag) = HeaderValue::from_str(&format!("\"{digest}\"")) {
        headers.insert(header::ETAG, etag);
    }
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    (StatusCode::OK, headers, bytes).into_response()
}

async fn api_list_personas(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let personas = list_personas(pool, owner_did)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))))?;
    Ok(Json(serde_json::json!({ "personas": personas })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePersonaBody {
    display_name: String,
    handle: Option<String>,
    bio: Option<String>,
}

async fn api_create_persona(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreatePersonaBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let persona = create_persona(
        pool,
        owner_did,
        &body.display_name,
        body.handle.as_deref().unwrap_or(""),
        body.bio.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    let personas = list_personas(pool, owner_did).await.unwrap_or_default();
    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{owner_did}"),
        "persona",
        &persona.persona_did,
        "insert",
        vec!["personaDid".to_string(), "displayName".to_string()],
        serde_json::json!({ "persona": persona, "personas": personas }),
    )
    .await;
    Ok(Json(serde_json::json!({ "persona": persona, "personas": personas })))
}

async fn api_update_persona(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<PersonaUpdateInput>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let persona = update_persona(pool, owner_did, &body)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    let personas = list_personas(pool, owner_did).await.unwrap_or_default();
    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{owner_did}"),
        "persona",
        &persona.persona_did,
        "update",
        vec!["displayName".to_string(), "handle".to_string()],
        serde_json::json!({ "persona": persona, "personas": personas }),
    )
    .await;
    Ok(Json(serde_json::json!({ "persona": persona, "personas": personas })))
}

async fn api_list_groups(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let groups = list_groups_for_user(pool, owner_did)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))))?;
    Ok(Json(serde_json::json!({ "groups": groups })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateGroupBody {
    name: String,
    description: Option<String>,
    persona_did: Option<String>,
}

async fn api_create_group(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateGroupBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let persona_did = body
        .persona_did
        .as_deref()
        .filter(|s| user.personas.iter().any(|p| p.persona_did == *s))
        .or_else(|| user.active_persona.as_ref().map(|p| p.persona_did.as_str()))
        .unwrap_or(owner_did);

    let group = create_group(
        pool,
        owner_did,
        persona_did,
        &body.name,
        body.description.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{owner_did}"),
        "group",
        &group.group_id,
        "insert",
        vec!["name".to_string()],
        serde_json::json!({ "group": group }),
    )
    .await;
    Ok(Json(serde_json::json!({ "group": group })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetGroupRoleBody {
    group_id: String,
    member_owner_did: String,
    role: String,
}

async fn api_set_group_member_role(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<SetGroupRoleBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    set_group_member_role(
        pool,
        owner_did,
        body.group_id.trim(),
        body.member_owner_did.trim(),
        &body.role,
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{owner_did}"),
        "group",
        body.group_id.trim(),
        "update",
        vec!["role".to_string()],
        serde_json::json!({ "groupId": body.group_id, "memberOwnerDid": body.member_owner_did, "role": body.role }),
    )
    .await;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn api_list_invites(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let (invites, referrals, referred_by) = list_invites_and_referrals(pool, owner_did)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))))?;
    Ok(Json(serde_json::json!({
        "invites": invites,
        "referrals": referrals,
        "referredBy": referred_by,
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateInviteBody {
    group_id: Option<String>,
    target_group_role: Option<String>,
    note: Option<String>,
    persona_did: Option<String>,
}

async fn api_create_invite(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateInviteBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let persona_did = body
        .persona_did
        .as_deref()
        .filter(|s| user.personas.iter().any(|p| p.persona_did == *s))
        .or_else(|| user.active_persona.as_ref().map(|p| p.persona_did.as_str()))
        .unwrap_or(owner_did);

    let invite = create_invite(
        pool,
        owner_did,
        persona_did,
        body.group_id.as_deref(),
        body.target_group_role.as_deref().unwrap_or("member"),
        body.note.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{owner_did}"),
        "invite",
        &invite.code,
        "insert",
        vec!["code".to_string()],
        serde_json::json!({ "invite": invite }),
    )
    .await;
    Ok(Json(serde_json::json!({ "invite": invite })))
}

#[derive(Deserialize)]
struct InvitePreviewQuery {
    code: String,
}

async fn api_preview_invite(
    State(state): State<AppState>,
    Query(query): Query<InvitePreviewQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let preview = preview_invite(pool, &query.code)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    Ok(Json(preview))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaimInviteBody {
    code: String,
}

async fn api_claim_invite(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<ClaimInviteBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(owner_did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let persona_did = user
        .active_persona
        .as_ref()
        .map(|p| p.persona_did.as_str())
        .unwrap_or(owner_did);

    let claimed = claim_invite(pool, owner_did, persona_did, &body.code)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    if claimed {
        emit_live_patch(
            Some(pool),
            &state.live_bus,
            format!("user:{owner_did}"),
            "invite",
            body.code.trim(),
            "update",
            vec!["claimed".to_string()],
            serde_json::json!({ "claimed": true }),
        )
        .await;
    }
    Ok(Json(serde_json::json!({ "claimed": claimed })))
}

async fn api_docs(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.authorized {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Docs access denied." })),
        ));
    }
    let tree = get_docs_tree(&state.root_dir);
    Ok(Json(serde_json::json!({ "tree": tree })))
}

#[derive(Deserialize)]
struct DocQuery {
    path: String,
}

async fn api_doc(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<DocQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.authorized {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Docs access denied." })),
        ));
    }

    let Some(abs_path) = resolve_safe_doc_path(&state.root_dir, &query.path) else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found." })),
        ));
    };

    let markdown = fs::read_to_string(&abs_path).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Unable to read document." })),
        )
    })?;

    let file_name = abs_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("doc.md")
        .to_string();
    let (title, headings, word_count) = extract_title_and_headings(&markdown, &file_name);
    let reading_minutes = (word_count / 210).max(1);
    let html = render_safe_gfm_html(&markdown);

    let rel_path = abs_path
        .strip_prefix(fs::canonicalize(&state.root_dir).unwrap_or_else(|_| state.root_dir.clone()))
        .unwrap_or(&abs_path)
        .to_string_lossy()
        .replace('\\', "/");

    Ok(Json(DocPayload {
        path: rel_path,
        name: file_name,
        title,
        html,
        word_count,
        reading_minutes,
        headings,
    }))
}

async fn api_admin_list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let entries = list_allowlist(pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
    })?;
    Ok(Json(serde_json::json!({ "entries": entries })))
}

async fn api_admin_identities(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let identities = list_identities(pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
    })?;
    Ok(Json(serde_json::json!({ "identities": identities })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminUpdateIdentityProfileBody {
    did_peer: String,
    display_name: Option<String>,
    pen_name: Option<String>,
    bio: Option<String>,
    website: Option<String>,
    location: Option<String>,
    preferred_format: Option<String>,
    account_role: Option<String>,
    account_status: Option<String>,
}

async fn api_admin_update_identity_profile(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<AdminUpdateIdentityProfileBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let did = body.did_peer.trim();

    if body.account_role.is_some() || body.account_status.is_some() {
        update_identity_account_role(
            pool,
            did,
            body.account_role.as_deref(),
            body.account_status.as_deref(),
        )
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    }

    let input = ProfileUpdateInput {
        active_persona_did: None,
        display_name: body.display_name,
        pen_name: body.pen_name,
        bio: body.bio,
        avatar_asset_id: None,
        avatar_url: None,
        avatar_thumb_url: None,
        website: body.website,
        location: body.location,
        locale: None,
        timezone: None,
        preferred_format: body.preferred_format,
        appearance_mode: None,
        ui_locale: None,
        default_input_locale: None,
        preferred_ui_locales: None,
        preferred_input_locales: None,
        willow_sync_enabled: None,
        last_active_tab: None,
        last_doc_path: None,
    };
    let profile = update_user_profile(pool, did, &input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))))?;
    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "admin",
        "identity",
        did,
        "update",
        vec!["accountRole".to_string(), "displayName".to_string()],
        serde_json::json!({ "didPeer": did }),
    )
    .await;
    Ok(Json(serde_json::json!({ "profile": profile })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertAllowlistBody {
    did_peer: Option<String>,
    email: Option<String>,
    status: Option<String>,
    note: Option<String>,
}

async fn api_admin_upsert(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<UpsertAllowlistBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let added_by = user.email.unwrap_or_else(get_admin_email);
    let status = body.status.as_deref().unwrap_or("reader");
    let entry = upsert_allowlist_entry(
        pool,
        body.did_peer.as_deref(),
        body.email.as_deref(),
        status,
        body.note.as_deref().unwrap_or(""),
        &added_by,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "admin",
        "allowlist",
        &entry.email,
        "update",
        vec!["status".to_string()],
        serde_json::json!({ "entry": entry }),
    )
    .await;

    Ok(Json(serde_json::json!({ "entry": entry })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteAllowlistQuery {
    did_peer: Option<String>,
    email: Option<String>,
}

async fn api_admin_delete(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<DeleteAllowlistQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required." })),
        ));
    }
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let removed =
        delete_allowlist_entry(pool, query.did_peer.as_deref(), query.email.as_deref())
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
            })?;
    emit_live_patch(
        Some(pool),
        &state.live_bus,
        "admin",
        "allowlist",
        query.did_peer.as_deref().or(query.email.as_deref()).unwrap_or(""),
        "delete",
        vec![],
        serde_json::json!({ "removed": removed }),
    )
    .await;
    Ok(Json(serde_json::json!({ "removed": removed })))
}
