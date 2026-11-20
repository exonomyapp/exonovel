mod auth;
mod db;
mod docs;

use std::{fs, path::PathBuf};

use axum::{
    extract::{FromRef, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
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
    db::{
        connect_pool, delete_allowlist_entry, ensure_schema, get_admin_email, list_allowlist,
        upsert_allowlist_entry,
    },
    docs::{
        extract_title_and_headings, get_docs_tree, render_safe_gfm_html, resolve_safe_doc_path,
        DocPayload,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub db: Option<PgPool>,
    pub neon_auth_base_url: Option<String>,
    pub neon_auth_jwks_url: Option<String>,
    pub jwks_cache: JwksCache,
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

    let db = connect_pool().await;
    if let Some(pool) = &db {
        if let Err(err) = ensure_schema(pool).await {
            eprintln!("Database schema initialization warning: {err}");
        }
    }

    let neon_auth_base_url = std::env::var("NEON_AUTH_BASE_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let neon_auth_jwks_url = std::env::var("NEON_AUTH_JWKS_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let root_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let state = AppState {
        leptos_options: leptos_options.clone(),
        db,
        neon_auth_base_url,
        neon_auth_jwks_url,
        jwks_cache: JwksCache::new(),
        root_dir,
    };

    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/auth-config", get(api_auth_config))
        .route("/api/me", get(api_me))
        .route("/api/docs", get(api_docs))
        .route("/api/doc", get(api_doc))
        .route(
            "/api/admin/allowlist",
            get(api_admin_list)
                .post(api_admin_upsert)
                .delete(api_admin_delete),
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
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <link rel="stylesheet" id="leptos" href="/pkg/exonovel.css" />
                <MetaTags />
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
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
    let Some(pool) = &state.db else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let entries = list_allowlist(pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
    })?;
    Ok(Json(serde_json::json!({ "entries": entries })))
}

#[derive(Deserialize)]
struct UpsertAllowlistBody {
    email: String,
    enabled: Option<bool>,
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
    let Some(pool) = &state.db else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let added_by = user.email.unwrap_or_else(get_admin_email);
    let entry = upsert_allowlist_entry(
        pool,
        &body.email,
        body.enabled.unwrap_or(true),
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

    Ok(Json(serde_json::json!({ "entry": entry })))
}

#[derive(Deserialize)]
struct DeleteAllowlistQuery {
    email: String,
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
    let Some(pool) = &state.db else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let removed = delete_allowlist_entry(pool, &query.email)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
        })?;
    Ok(Json(serde_json::json!({ "removed": removed })))
}
