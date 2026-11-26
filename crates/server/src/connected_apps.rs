use std::{collections::HashMap, fs, time::{SystemTime, UNIX_EPOCH}};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use base64::{
    engine::general_purpose::{STANDARD as B64_STD, URL_SAFE_NO_PAD as B64_URL},
    Engine as _,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use url::{form_urlencoded::byte_serialize, Url};

use crate::{
    auth::AuthUser,
    db::{now_timestamp_us, open_secret, random_hex_token, seal_secret},
    docs::resolve_safe_doc_path,
    live::emit_live_patch,
    AppState,
};

pub const GOOGLE_DRIVE_PROVIDER: &str = "google_drive";
pub const DEFAULT_GOOGLE_SCOPES: &str =
    "https://www.googleapis.com/auth/drive.file https://www.googleapis.com/auth/userinfo.email";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoAccountSummary {
    pub provider_id: String,
    pub account_id: String,
    pub scopes: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppItem {
    pub provider: String,
    pub status: String,
    pub provider_email: String,
    pub provider_account_id: String,
    pub scopes: String,
    pub connected_at: String,
    pub revoked_at: Option<String>,
    pub willow_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFileItem {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub modified_time: Option<String>,
    pub size: Option<String>,
    pub web_view_link: Option<String>,
    pub trashed: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthStatePayload {
    did_peer: String,
    auth_user_id: String,
    redirect_uri: String,
    exp: i64,
    nonce: String,
    return_tab: String,
    return_doc_path: String,
}

fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn google_oauth_credentials() -> Option<(String, String)> {
    let client_id = non_empty_env("GOOGLE_OAUTH_CLIENT_ID")?;
    let client_secret = non_empty_env("GOOGLE_OAUTH_CLIENT_SECRET")?;
    Some((client_id, client_secret))
}

fn google_drive_scopes() -> String {
    non_empty_env("GOOGLE_DRIVE_SCOPES").unwrap_or_else(|| DEFAULT_GOOGLE_SCOPES.to_string())
}

fn sanitize_host_header(raw: &str) -> Option<String> {
    let first = raw.split(',').next()?.trim();
    if first.is_empty() || first.len() > 253 {
        return None;
    }
    if first
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
    {
        Some(first.to_string())
    } else {
        None
    }
}

pub fn resolve_google_redirect_uri(headers: &HeaderMap) -> String {
    if let Some(explicit) = non_empty_env("GOOGLE_DRIVE_REDIRECT_URI") {
        return explicit;
    }

    let host = headers
        .get("x-forwarded-host")
        .and_then(|v| v.to_str().ok())
        .and_then(sanitize_host_header)
        .or_else(|| {
            headers
                .get("host")
                .and_then(|v| v.to_str().ok())
                .and_then(sanitize_host_header)
        });

    if let Some(h) = host {
        let is_local = h.starts_with("localhost") || h.starts_with("127.0.0.1");
        let proto = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(str::trim)
            .filter(|p| *p == "http" || *p == "https")
            .unwrap_or(if is_local { "http" } else { "https" });
        return format!("{proto}://{h}/api/oauth/google/callback");
    }

    let base = non_empty_env("PRODUCTION_APP_URL")
        .or_else(|| non_empty_env("APP_URL"))
        .unwrap_or_else(|| "https://exonovel.exosystems.net".to_string());
    format!("{}/api/oauth/google/callback", base.trim_end_matches('/'))
}

pub fn sanitize_drive_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 160 {
        return None;
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub fn escape_drive_query_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn url_encode_component(raw: &str) -> String {
    byte_serialize(raw.as_bytes()).collect()
}

fn seal_oauth_state(payload: &OAuthStatePayload) -> Result<String, String> {
    let json = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    let sealed = seal_secret(&json)?;
    Ok(B64_URL.encode(sealed.as_bytes()))
}

fn open_oauth_state(token: &str) -> Result<OAuthStatePayload, String> {
    let raw_bytes = B64_URL
        .decode(token.trim())
        .map_err(|_| "Invalid OAuth state encoding.".to_string())?;
    let sealed =
        String::from_utf8(raw_bytes).map_err(|_| "Invalid OAuth state UTF-8.".to_string())?;
    let json = open_secret(&sealed)?;
    let payload: OAuthStatePayload =
        serde_json::from_str(&json).map_err(|_| "Malformed OAuth state payload.".to_string())?;
    if payload.exp < now_unix_secs() {
        return Err("OAuth authorization session expired. Please try again.".to_string());
    }
    if !payload.did_peer.starts_with("did:peer:") {
        return Err("Invalid OAuth identity binding.".to_string());
    }
    Ok(payload)
}

async fn wrap_oauth_token(plaintext: &str) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    if let (Some(vault_addr), Some(vault_token)) =
        (non_empty_env("VAULT_ADDR"), non_empty_env("VAULT_TOKEN"))
    {
        let key_name = non_empty_env("VAULT_TRANSIT_KEY").unwrap_or_else(|| "exonovel".to_string());
        let url = format!(
            "{}/v1/transit/encrypt/{}",
            vault_addr.trim_end_matches('/'),
            key_name
        );
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("X-Vault-Token", vault_token)
            .json(&serde_json::json!({
                "plaintext": B64_STD.encode(plaintext.as_bytes()),
            }))
            .send()
            .await
            .map_err(|e| format!("Token envelope encrypt error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let _err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Token envelope encrypt rejected ({status})."));
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Token envelope parse error: {e}"))?;
        let ct = body
            .pointer("/data/ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Token envelope missing ciphertext.".to_string())?;
        return Ok(ct.to_string());
    }
    seal_secret(plaintext)
}

async fn unwrap_oauth_token(ciphertext: &str) -> Result<String, String> {
    let trimmed = ciphertext.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if trimmed.starts_with("vault:") {
        let vault_addr = non_empty_env("VAULT_ADDR")
            .ok_or_else(|| "Token encryption service address is not configured.".to_string())?;
        let vault_token = non_empty_env("VAULT_TOKEN")
            .ok_or_else(|| "Token encryption service credentials are not configured.".to_string())?;
        let key_name = non_empty_env("VAULT_TRANSIT_KEY").unwrap_or_else(|| "exonovel".to_string());
        let url = format!(
            "{}/v1/transit/decrypt/{}",
            vault_addr.trim_end_matches('/'),
            key_name
        );
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("X-Vault-Token", vault_token)
            .json(&serde_json::json!({
                "ciphertext": trimmed,
            }))
            .send()
            .await
            .map_err(|e| format!("Token envelope decrypt error: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!(
                "Token envelope decrypt failed ({}).",
                resp.status()
            ));
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Token envelope decrypt parse error: {e}"))?;
        let b64_plain = body
            .pointer("/data/plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Token envelope missing plaintext.".to_string())?;
        let bytes = B64_STD
            .decode(b64_plain)
            .map_err(|_| "Token envelope invalid base64 plaintext.".to_string())?;
        return String::from_utf8(bytes)
            .map_err(|_| "Token envelope plaintext is not UTF-8.".to_string());
    }
    open_secret(trimmed)
}

pub async fn ensure_connected_apps_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS connected_apps (
            id SERIAL PRIMARY KEY,
            owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            provider TEXT NOT NULL,
            provider_account_id TEXT NOT NULL DEFAULT '',
            provider_email TEXT NOT NULL DEFAULT '',
            scopes TEXT NOT NULL DEFAULT '',
            encrypted_access_token TEXT NOT NULL DEFAULT '',
            encrypted_refresh_token TEXT NOT NULL DEFAULT '',
            token_expires_at TIMESTAMPTZ,
            status TEXT NOT NULL DEFAULT 'connected',
            connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            revoked_at TIMESTAMPTZ,
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT '',
            willow_path TEXT NOT NULL DEFAULT '',
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (owner_did, provider)
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn db_unavailable() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({ "error": "Database unavailable." })),
    )
}

fn err_json(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": msg.into() })))
}

pub async fn api_list_connected_apps(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };

    let sso_rows = sqlx::query(
        r#"
        SELECT
            COALESCE("providerId", '') AS provider_id,
            COALESCE("accountId", '') AS account_id,
            COALESCE("scope", '') AS scopes,
            COALESCE(to_char("updatedAt" AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'), '') AS updated_at
        FROM neon_auth.account
        WHERE "userId"::text = $1
        ORDER BY "updatedAt" DESC
        "#,
    )
    .bind(&user.auth_user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let sso_accounts: Vec<SsoAccountSummary> = sso_rows
        .into_iter()
        .map(|r| SsoAccountSummary {
            provider_id: r.try_get("provider_id").unwrap_or_default(),
            account_id: r.try_get("account_id").unwrap_or_default(),
            scopes: r.try_get("scopes").unwrap_or_default(),
            updated_at: r.try_get("updated_at").unwrap_or_default(),
        })
        .collect();

    let app_rows = sqlx::query(
        r#"
        SELECT
            provider,
            status,
            provider_email,
            provider_account_id,
            scopes,
            to_char(connected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS connected_at,
            CASE WHEN revoked_at IS NOT NULL
                 THEN to_char(revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
                 ELSE NULL END AS revoked_at,
            willow_path
        FROM connected_apps
        WHERE owner_did = $1
        ORDER BY updated_at DESC
        "#,
    )
    .bind(did_peer)
    .fetch_all(pool)
    .await
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let connected_apps: Vec<ConnectedAppItem> = app_rows
        .into_iter()
        .map(|r| ConnectedAppItem {
            provider: r.try_get("provider").unwrap_or_default(),
            status: r.try_get("status").unwrap_or_default(),
            provider_email: r.try_get("provider_email").unwrap_or_default(),
            provider_account_id: r.try_get("provider_account_id").unwrap_or_default(),
            scopes: r.try_get("scopes").unwrap_or_default(),
            connected_at: r.try_get("connected_at").unwrap_or_default(),
            revoked_at: r.try_get("revoked_at").unwrap_or(None),
            willow_path: r.try_get("willow_path").unwrap_or_default(),
        })
        .collect();

    let oauth_configured = google_oauth_credentials().is_some();
    let redirect_uri = resolve_google_redirect_uri(&headers);

    Ok(Json(serde_json::json!({
        "ssoAccounts": sso_accounts,
        "connectedApps": connected_apps,
        "oauthConfigured": oauth_configured,
        "redirectUri": redirect_uri,
        "recommendedScopes": google_drive_scopes(),
    })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleStartBody {
    pub return_tab: Option<String>,
    pub return_doc_path: Option<String>,
}

pub async fn api_google_oauth_start(
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Json(body): Json<GoogleStartBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(did_peer) = user.did_peer else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };
    let Some((client_id, _)) = google_oauth_credentials() else {
        return Err(err_json(
            StatusCode::SERVICE_UNAVAILABLE,
            "Google Drive & Docs integration is not configured on the server.",
        ));
    };

    let redirect_uri = resolve_google_redirect_uri(&headers);
    let return_tab = match body.return_tab.as_deref() {
        Some("docs") if user.authorized => "docs".to_string(),
        _ => "preferences".to_string(),
    };
    let return_doc_path = body
        .return_doc_path
        .unwrap_or_default()
        .trim()
        .replace("..", "")
        .chars()
        .take(220)
        .collect::<String>();

    let state_payload = OAuthStatePayload {
        did_peer,
        auth_user_id: user.auth_user_id,
        redirect_uri: redirect_uri.clone(),
        exp: now_unix_secs() + 600,
        nonce: random_hex_token(12),
        return_tab,
        return_doc_path,
    };
    let state_token =
        seal_oauth_state(&state_payload).map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let scopes = google_drive_scopes();
    let mut query_pairs = vec![
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", scopes.as_str()),
        ("access_type", "offline"),
        ("include_granted_scopes", "true"),
        ("prompt", "consent"),
        ("state", state_token.as_str()),
    ];
    let login_hint_str = user.email.unwrap_or_default();
    if !login_hint_str.trim().is_empty() {
        query_pairs.push(("login_hint", login_hint_str.trim()));
    }

    let auth_url = Url::parse_with_params(
        "https://accounts.google.com/o/oauth2/v2/auth",
        &query_pairs,
    )
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "authUrl": auth_url.to_string(),
        "redirectUri": redirect_uri,
    })))
}

#[derive(Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

fn callback_redirect_url(
    return_tab: &str,
    return_doc_path: &str,
    connected: bool,
    oauth_error: Option<&str>,
) -> String {
    let base = if return_tab == "docs" {
        if return_doc_path.is_empty() {
            "/?tab=docs".to_string()
        } else {
            format!("/?tab=docs&doc={}", url_encode_component(return_doc_path))
        }
    } else {
        "/?tab=preferences&section=connected-apps".to_string()
    };

    if let Some(err) = oauth_error {
        let safe_err: String = err
            .chars()
            .filter(|c| !c.is_control())
            .take(180)
            .collect();
        return format!("{base}&oauth_error={}", url_encode_component(&safe_err));
    }
    if connected {
        return format!("{base}&connected=google");
    }
    base
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: Option<String>,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
struct GoogleUserInfoResponse {
    id: Option<String>,
    email: Option<String>,
}

pub async fn api_google_oauth_callback(
    State(state): State<AppState>,
    Query(query): Query<GoogleCallbackQuery>,
) -> impl IntoResponse {
    let state_payload = match query.state.as_deref() {
        Some(s) => match open_oauth_state(s) {
            Ok(p) => p,
            Err(err) => {
                let target = callback_redirect_url("preferences", "", false, Some(&err));
                return Redirect::to(&target).into_response();
            }
        },
        None => {
            let target = callback_redirect_url(
                "preferences",
                "",
                false,
                Some("Missing OAuth state parameter."),
            );
            return Redirect::to(&target).into_response();
        }
    };

    if let Some(oauth_err) = query.error {
        let msg = query
            .error_description
            .filter(|d| !d.trim().is_empty())
            .unwrap_or(oauth_err);
        let target = callback_redirect_url(
            &state_payload.return_tab,
            &state_payload.return_doc_path,
            false,
            Some(&msg),
        );
        return Redirect::to(&target).into_response();
    }

    let Some(code) = query.code.filter(|c| !c.trim().is_empty()) else {
        let target = callback_redirect_url(
            &state_payload.return_tab,
            &state_payload.return_doc_path,
            false,
            Some("Missing authorization code from Google."),
        );
        return Redirect::to(&target).into_response();
    };

    let Some((client_id, client_secret)) = google_oauth_credentials() else {
        let target = callback_redirect_url(
            &state_payload.return_tab,
            &state_payload.return_doc_path,
            false,
            Some("Server Google OAuth credentials are not configured."),
        );
        return Redirect::to(&target).into_response();
    };

    let Some(pool) = state.db.as_ref() else {
        let target = callback_redirect_url(
            &state_payload.return_tab,
            &state_payload.return_doc_path,
            false,
            Some("Database unavailable during OAuth callback."),
        );
        return Redirect::to(&target).into_response();
    };

    let http = reqwest::Client::new();
    let token_res = http
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri", state_payload.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await;

    let token_body = match token_res {
        Ok(resp) => {
            let status_ok = resp.status().is_success();
            let parsed = resp.json::<GoogleTokenResponse>().await.ok();
            if !status_ok || parsed.as_ref().and_then(|p| p.access_token.as_ref()).is_none() {
                let err_msg = parsed
                    .and_then(|p| p.error_description.or(p.error))
                    .unwrap_or_else(|| "Google token exchange failed.".to_string());
                let target = callback_redirect_url(
                    &state_payload.return_tab,
                    &state_payload.return_doc_path,
                    false,
                    Some(&err_msg),
                );
                return Redirect::to(&target).into_response();
            }
            parsed.unwrap()
        }
        Err(e) => {
            let target = callback_redirect_url(
                &state_payload.return_tab,
                &state_payload.return_doc_path,
                false,
                Some(&format!("Google token request error: {e}")),
            );
            return Redirect::to(&target).into_response();
        }
    };

    let access_token = token_body.access_token.unwrap_or_default();
    let expires_in = token_body.expires_in.unwrap_or(3500).clamp(60, 86_400);
    let scopes = token_body.scope.unwrap_or_else(google_drive_scopes);

    let userinfo = http
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&access_token)
        .send()
        .await
        .ok()
        .and_then(|r| if r.status().is_success() { Some(r) } else { None });
    let info_json = match userinfo {
        Some(r) => r.json::<GoogleUserInfoResponse>().await.ok(),
        None => None,
    };

    let provider_email = info_json
        .as_ref()
        .and_then(|i| i.email.clone())
        .unwrap_or_default();
    let provider_account_id = info_json
        .as_ref()
        .and_then(|i| i.id.clone())
        .unwrap_or_default();

    let sealed_access = match wrap_oauth_token(&access_token).await {
        Ok(s) => s,
        Err(e) => {
            let target = callback_redirect_url(
                &state_payload.return_tab,
                &state_payload.return_doc_path,
                false,
                Some(&e),
            );
            return Redirect::to(&target).into_response();
        }
    };

    let sealed_refresh = match token_body.refresh_token.as_deref() {
        Some(rt) if !rt.trim().is_empty() => match wrap_oauth_token(rt).await {
            Ok(s) => s,
            Err(e) => {
                let target = callback_redirect_url(
                    &state_payload.return_tab,
                    &state_payload.return_doc_path,
                    false,
                    Some(&e),
                );
                return Redirect::to(&target).into_response();
            }
        },
        _ => String::new(),
    };

    let ts_us = now_timestamp_us();
    let willow_path = format!("/connected_apps/{GOOGLE_DRIVE_PROVIDER}");
    let save_res = sqlx::query(
        r#"
        INSERT INTO connected_apps (
            owner_did, provider, provider_account_id, provider_email, scopes,
            encrypted_access_token, encrypted_refresh_token,
            token_expires_at, status, connected_at, revoked_at,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            NOW() + make_interval(secs => $8::double precision),
            'connected', NOW(), NULL,
            'exonovel', $1, $9, $10, NOW()
        )
        ON CONFLICT (owner_did, provider) DO UPDATE SET
            provider_account_id = CASE WHEN EXCLUDED.provider_account_id <> '' THEN EXCLUDED.provider_account_id ELSE connected_apps.provider_account_id END,
            provider_email = CASE WHEN EXCLUDED.provider_email <> '' THEN EXCLUDED.provider_email ELSE connected_apps.provider_email END,
            scopes = EXCLUDED.scopes,
            encrypted_access_token = EXCLUDED.encrypted_access_token,
            encrypted_refresh_token = CASE WHEN EXCLUDED.encrypted_refresh_token <> '' THEN EXCLUDED.encrypted_refresh_token ELSE connected_apps.encrypted_refresh_token END,
            token_expires_at = EXCLUDED.token_expires_at,
            status = 'connected',
            connected_at = NOW(),
            revoked_at = NULL,
            willow_timestamp_us = EXCLUDED.willow_timestamp_us,
            updated_at = NOW()
        "#,
    )
    .bind(&state_payload.did_peer)
    .bind(GOOGLE_DRIVE_PROVIDER)
    .bind(&provider_account_id)
    .bind(&provider_email)
    .bind(&scopes)
    .bind(&sealed_access)
    .bind(&sealed_refresh)
    .bind(expires_in as f64)
    .bind(&willow_path)
    .bind(ts_us)
    .execute(pool)
    .await;

    if let Err(e) = save_res {
        let target = callback_redirect_url(
            &state_payload.return_tab,
            &state_payload.return_doc_path,
            false,
            Some(&format!("Failed to persist Google connection: {e}")),
        );
        return Redirect::to(&target).into_response();
    }

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{}", state_payload.did_peer),
        "connected_app",
        GOOGLE_DRIVE_PROVIDER,
        "update",
        vec!["status".to_string(), "providerEmail".to_string()],
        serde_json::json!({
            "provider": GOOGLE_DRIVE_PROVIDER,
            "status": "connected",
            "providerEmail": provider_email,
        }),
    )
    .await;

    let target = callback_redirect_url(
        &state_payload.return_tab,
        &state_payload.return_doc_path,
        true,
        None,
    );
    Redirect::to(&target).into_response()
}

pub async fn api_google_oauth_revoke(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };

    let row = sqlx::query(
        r#"
        SELECT encrypted_access_token, encrypted_refresh_token
        FROM connected_apps
        WHERE owner_did = $1 AND provider = $2
        LIMIT 1
        "#,
    )
    .bind(did_peer)
    .bind(GOOGLE_DRIVE_PROVIDER)
    .fetch_optional(pool)
    .await
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut revoked_upstream = false;
    if let Some(r) = row {
        let enc_refresh: String = r.try_get("encrypted_refresh_token").unwrap_or_default();
        let enc_access: String = r.try_get("encrypted_access_token").unwrap_or_default();
        let refresh_plain = unwrap_oauth_token(&enc_refresh).await.unwrap_or_default();
        let access_plain = unwrap_oauth_token(&enc_access).await.unwrap_or_default();
        let tok_to_revoke = if !refresh_plain.is_empty() {
            refresh_plain
        } else {
            access_plain
        };

        if !tok_to_revoke.is_empty() {
            if let Ok(resp) = reqwest::Client::new()
                .post("https://oauth2.googleapis.com/revoke")
                .form(&[("token", tok_to_revoke.as_str())])
                .send()
                .await
            {
                revoked_upstream = resp.status().is_success() || resp.status().as_u16() == 400;
            }
        }
    }

    let ts_us = now_timestamp_us();
    sqlx::query(
        r#"
        UPDATE connected_apps
        SET status = 'revoked',
            encrypted_access_token = '',
            encrypted_refresh_token = '',
            token_expires_at = NULL,
            revoked_at = NOW(),
            willow_timestamp_us = $2,
            updated_at = NOW()
        WHERE owner_did = $1 AND provider = 'google_drive'
        "#,
    )
    .bind(did_peer)
    .bind(ts_us)
    .execute(pool)
    .await
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{did_peer}"),
        "connected_app",
        GOOGLE_DRIVE_PROVIDER,
        "update",
        vec!["status".to_string()],
        serde_json::json!({
            "provider": GOOGLE_DRIVE_PROVIDER,
            "status": "revoked",
        }),
    )
    .await;

    Ok(Json(serde_json::json!({
        "ok": true,
        "revokedUpstream": revoked_upstream,
    })))
}

async fn mark_google_revoked(pool: &PgPool, did_peer: &str) {
    let ts_us = now_timestamp_us();
    let _ = sqlx::query(
        r#"
        UPDATE connected_apps
        SET status = 'revoked',
            encrypted_access_token = '',
            encrypted_refresh_token = '',
            token_expires_at = NULL,
            revoked_at = NOW(),
            willow_timestamp_us = $2,
            updated_at = NOW()
        WHERE owner_did = $1 AND provider = 'google_drive'
        "#,
    )
    .bind(did_peer)
    .bind(ts_us)
    .execute(pool)
    .await;
}

async fn get_valid_google_access_token(
    pool: &PgPool,
    did_peer: &str,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let row = sqlx::query(
        r#"
        SELECT
            status,
            encrypted_access_token,
            encrypted_refresh_token,
            COALESCE(EXTRACT(EPOCH FROM token_expires_at)::BIGINT, 0) AS expires_epoch
        FROM connected_apps
        WHERE owner_did = $1 AND provider = $2
        LIMIT 1
        "#,
    )
    .bind(did_peer)
    .bind(GOOGLE_DRIVE_PROVIDER)
    .fetch_optional(pool)
    .await
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let Some(r) = row else {
        return Err(err_json(
            StatusCode::PRECONDITION_REQUIRED,
            "Google Drive & Docs is not connected. Open Preferences -> Connected Apps & Google Docs to connect.",
        ));
    };

    let status: String = r.try_get("status").unwrap_or_default();
    if status != "connected" {
        return Err(err_json(
            StatusCode::PRECONDITION_REQUIRED,
            "Google Drive & Docs connection was disconnected. Please reconnect in Preferences -> Connected Apps & Google Docs.",
        ));
    }

    let enc_access: String = r.try_get("encrypted_access_token").unwrap_or_default();
    let enc_refresh: String = r.try_get("encrypted_refresh_token").unwrap_or_default();
    let expires_epoch: i64 = r.try_get("expires_epoch").unwrap_or(0);

    if !enc_access.is_empty() && expires_epoch - 90 > now_unix_secs() {
        if let Ok(plain) = unwrap_oauth_token(&enc_access).await {
            if !plain.is_empty() {
                return Ok(plain);
            }
        }
    }

    let refresh_plain = unwrap_oauth_token(&enc_refresh).await.unwrap_or_default();
    if refresh_plain.is_empty() {
        mark_google_revoked(pool, did_peer).await;
        return Err(err_json(
            StatusCode::UNAUTHORIZED,
            "Google session expired without a refresh token. Please reconnect in Preferences -> Connected Apps & Google Docs.",
        ));
    }

    let Some((client_id, client_secret)) = google_oauth_credentials() else {
        return Err(err_json(
            StatusCode::SERVICE_UNAVAILABLE,
            "Server Google OAuth credentials missing.",
        ));
    };

    let resp = reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_plain.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| err_json(StatusCode::BAD_GATEWAY, format!("Token refresh failed: {e}")))?;

    let status_code = resp.status();
    let parsed: GoogleTokenResponse = resp
        .json()
        .await
        .map_err(|e| err_json(StatusCode::BAD_GATEWAY, format!("Invalid refresh response: {e}")))?;

    if !status_code.is_success() || parsed.access_token.as_deref().unwrap_or("").is_empty() {
        if parsed.error.as_deref() == Some("invalid_grant") {
            mark_google_revoked(pool, did_peer).await;
            return Err(err_json(
                StatusCode::UNAUTHORIZED,
                "Google authorization was revoked or expired. Please reconnect in Preferences.",
            ));
        }
        return Err(err_json(
            StatusCode::UNAUTHORIZED,
            parsed
                .error_description
                .or(parsed.error)
                .unwrap_or_else(|| "Google token refresh failed.".to_string()),
        ));
    }

    let fresh_access = parsed.access_token.unwrap_or_default();
    let expires_in = parsed.expires_in.unwrap_or(3500).clamp(60, 86_400);
    let sealed_access = wrap_oauth_token(&fresh_access)
        .await
        .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let _ = sqlx::query(
        r#"
        UPDATE connected_apps
        SET encrypted_access_token = $2,
            token_expires_at = NOW() + make_interval(secs => $3::double precision),
            updated_at = NOW()
        WHERE owner_did = $1 AND provider = 'google_drive'
        "#,
    )
    .bind(did_peer)
    .bind(&sealed_access)
    .bind(expires_in as f64)
    .execute(pool)
    .await;

    Ok(fresh_access)
}

async fn drive_send_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, (StatusCode, Json<serde_json::Value>)> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let code = if status.as_u16() == 401 {
            StatusCode::UNAUTHORIZED
        } else if status.as_u16() == 403 {
            StatusCode::FORBIDDEN
        } else if status.as_u16() == 404 {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_GATEWAY
        };
        return Err(err_json(
            code,
            if body.trim().is_empty() {
                format!("Google Drive API error ({status})")
            } else {
                body
            },
        ));
    }
    resp.json::<T>()
        .await
        .map_err(|e| err_json(StatusCode::BAD_GATEWAY, format!("Google Drive JSON parse error: {e}")))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDriveFile {
    id: String,
    name: String,
    mime_type: String,
    modified_time: Option<String>,
    size: Option<String>,
    web_view_link: Option<String>,
    trashed: Option<bool>,
}

impl From<RawDriveFile> for DriveFileItem {
    fn from(r: RawDriveFile) -> Self {
        Self {
            id: r.id,
            name: r.name,
            mime_type: r.mime_type,
            modified_time: r.modified_time,
            size: r.size,
            web_view_link: r.web_view_link,
            trashed: r.trashed.unwrap_or(false),
        }
    }
}

#[derive(Deserialize)]
struct RawDriveList {
    files: Option<Vec<RawDriveFile>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveListQuery {
    pub folder_id: Option<String>,
    pub q: Option<String>,
    pub trashed: Option<bool>,
    pub all: Option<bool>,
}

pub async fn api_drive_list_files(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<DriveListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };

    let access_token = get_valid_google_access_token(pool, did_peer).await?;
    let trashed_only = query.trashed.unwrap_or(false);
    let mut clauses = vec![if trashed_only {
        "trashed = true".to_string()
    } else {
        "trashed = false".to_string()
    }];

    if let Some(q_text) = query.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        clauses.push(format!("name contains '{}'", escape_drive_query_value(q_text)));
    } else if !trashed_only && !query.all.unwrap_or(false) {
        if let Some(fid) = query.folder_id.as_deref().and_then(sanitize_drive_id) {
            clauses.push(format!("'{fid}' in parents"));
        }
    }

    let q_str = clauses.join(" and ");
    let url = Url::parse_with_params(
        "https://www.googleapis.com/drive/v3/files",
        &[
            ("q", q_str.as_str()),
            ("pageSize", "60"),
            ("orderBy", "folder,modifiedTime desc"),
            ("fields", "files(id,name,mimeType,modifiedTime,size,webViewLink,trashed)"),
            ("supportsAllDrives", "true"),
            ("includeItemsFromAllDrives", "true"),
        ],
    )
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let resp = reqwest::Client::new()
        .get(url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| err_json(StatusCode::BAD_GATEWAY, format!("Drive list error: {e}")))?;

    let data: RawDriveList = drive_send_json(resp).await?;
    let files: Vec<DriveFileItem> = data
        .files
        .unwrap_or_default()
        .into_iter()
        .map(DriveFileItem::from)
        .collect();

    Ok(Json(serde_json::json!({ "files": files })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDriveFolderBody {
    pub name: String,
    pub parent_id: Option<String>,
}

async fn find_drive_item_by_name(
    http: &reqwest::Client,
    access_token: &str,
    name: &str,
    parent_id: Option<&str>,
    mime_type: Option<&str>,
    exclude_folders: bool,
) -> Result<Option<DriveFileItem>, (StatusCode, Json<serde_json::Value>)> {
    let mut clauses = vec![
        "trashed = false".to_string(),
        format!("name = '{}'", escape_drive_query_value(name)),
    ];
    let safe_parent = parent_id
        .and_then(sanitize_drive_id)
        .unwrap_or_else(|| "root".to_string());
    clauses.push(format!("'{safe_parent}' in parents"));

    if let Some(mt) = mime_type {
        clauses.push(format!("mimeType = '{}'", escape_drive_query_value(mt)));
    } else if exclude_folders {
        clauses.push("mimeType != 'application/vnd.google-apps.folder'".to_string());
    }

    let q_str = clauses.join(" and ");
    let url = Url::parse_with_params(
        "https://www.googleapis.com/drive/v3/files",
        &[
            ("q", q_str.as_str()),
            ("pageSize", "1"),
            ("fields", "files(id,name,mimeType,modifiedTime,size,webViewLink,trashed)"),
            ("supportsAllDrives", "true"),
            ("includeItemsFromAllDrives", "true"),
        ],
    )
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let resp = http
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| err_json(StatusCode::BAD_GATEWAY, format!("Drive lookup failed: {e}")))?;

    let list: RawDriveList = drive_send_json(resp).await?;
    Ok(list
        .files
        .unwrap_or_default()
        .into_iter()
        .next()
        .map(DriveFileItem::from))
}

async fn create_drive_folder_remote(
    http: &reqwest::Client,
    access_token: &str,
    name: &str,
    parent_id: Option<&str>,
) -> Result<DriveFileItem, (StatusCode, Json<serde_json::Value>)> {
    let mut metadata = serde_json::json!({
        "name": name,
        "mimeType": "application/vnd.google-apps.folder",
    });
    if let Some(pid) = parent_id.and_then(sanitize_drive_id) {
        if pid != "root" {
            metadata["parents"] = serde_json::json!([pid]);
        }
    }
    let resp = http
        .post("https://www.googleapis.com/drive/v3/files?supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,webViewLink,trashed")
        .bearer_auth(access_token)
        .json(&metadata)
        .send()
        .await
        .map_err(|e| err_json(StatusCode::BAD_GATEWAY, format!("Drive folder create error: {e}")))?;
    let raw: RawDriveFile = drive_send_json(resp).await?;
    Ok(DriveFileItem::from(raw))
}

async fn ensure_drive_folder_remote(
    http: &reqwest::Client,
    access_token: &str,
    name: &str,
    parent_id: Option<&str>,
) -> Result<DriveFileItem, (StatusCode, Json<serde_json::Value>)> {
    if let Some(found) = find_drive_item_by_name(
        http,
        access_token,
        name,
        parent_id,
        Some("application/vnd.google-apps.folder"),
        false,
    )
    .await?
    {
        return Ok(found);
    }
    create_drive_folder_remote(http, access_token, name, parent_id).await
}

pub async fn api_drive_create_folder(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateDriveFolderBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };
    let clean_name = body.name.trim();
    if clean_name.is_empty() || clean_name.len() > 120 || clean_name.contains('/') {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "Valid folder name is required.",
        ));
    }

    let access_token = get_valid_google_access_token(pool, did_peer).await?;
    let http = reqwest::Client::new();
    let folder =
        create_drive_folder_remote(&http, &access_token, clean_name, body.parent_id.as_deref())
            .await?;

    Ok(Json(serde_json::json!({ "folder": folder })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveContentQuery {
    pub mime_type: Option<String>,
}

pub async fn api_drive_read_content(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(file_id): Path<String>,
    Query(query): Query<DriveContentQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };
    let Some(safe_id) = sanitize_drive_id(&file_id) else {
        return Err(err_json(StatusCode::BAD_REQUEST, "Invalid Google Drive file ID."));
    };

    let access_token = get_valid_google_access_token(pool, did_peer).await?;
    let http = reqwest::Client::new();

    let effective_mime = match query.mime_type.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(m) => m.to_string(),
        None => {
            let meta_url = format!(
                "https://www.googleapis.com/drive/v3/files/{safe_id}?supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,size,webViewLink,trashed"
            );
            let meta_resp = http
                .get(&meta_url)
                .bearer_auth(&access_token)
                .send()
                .await
                .map_err(|e| err_json(StatusCode::BAD_GATEWAY, e.to_string()))?;
            let meta: RawDriveFile = drive_send_json(meta_resp).await?;
            meta.mime_type
        }
    };

    let (fetch_url, exported_mime) = if effective_mime == "application/vnd.google-apps.document" {
        (
            format!(
                "https://www.googleapis.com/drive/v3/files/{safe_id}/export?mimeType=text/markdown"
            ),
            "text/markdown",
        )
    } else if effective_mime == "application/vnd.google-apps.spreadsheet" {
        (
            format!("https://www.googleapis.com/drive/v3/files/{safe_id}/export?mimeType=text/csv"),
            "text/csv",
        )
    } else if effective_mime == "application/vnd.google-apps.presentation" {
        (
            format!(
                "https://www.googleapis.com/drive/v3/files/{safe_id}/export?mimeType=text/plain"
            ),
            "text/plain",
        )
    } else {
        (
            format!(
                "https://www.googleapis.com/drive/v3/files/{safe_id}?alt=media&supportsAllDrives=true"
            ),
            effective_mime.as_str(),
        )
    };

    let resp = http
        .get(&fetch_url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| err_json(StatusCode::BAD_GATEWAY, format!("Drive content read failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let txt = resp.text().await.unwrap_or_default();
        return Err(err_json(
            StatusCode::BAD_GATEWAY,
            format!("Drive content export failed ({status}): {txt}"),
        ));
    }

    let content = resp
        .text()
        .await
        .map_err(|e| err_json(StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": safe_id,
        "content": content,
        "exportedMimeType": exported_mime,
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFileActionBody {
    pub action: String,
    pub name: Option<String>,
}

pub async fn api_drive_file_action(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(file_id): Path<String>,
    Json(body): Json<DriveFileActionBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };
    let Some(safe_id) = sanitize_drive_id(&file_id) else {
        return Err(err_json(StatusCode::BAD_REQUEST, "Invalid Google Drive file ID."));
    };

    let access_token = get_valid_google_access_token(pool, did_peer).await?;
    let http = reqwest::Client::new();

    match body.action.trim() {
        "rename" => {
            let new_name = body
                .name
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && s.len() <= 180)
                .ok_or_else(|| err_json(StatusCode::BAD_REQUEST, "Valid new file name required."))?;
            let url = format!(
                "https://www.googleapis.com/drive/v3/files/{safe_id}?supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,size,webViewLink,trashed"
            );
            let resp = http
                .patch(&url)
                .bearer_auth(&access_token)
                .json(&serde_json::json!({ "name": new_name }))
                .send()
                .await
                .map_err(|e| err_json(StatusCode::BAD_GATEWAY, e.to_string()))?;
            let updated: RawDriveFile = drive_send_json(resp).await?;
            Ok(Json(serde_json::json!({
                "ok": true,
                "file": DriveFileItem::from(updated),
            })))
        }
        "trash" | "restore" => {
            let trashed_flag = body.action.trim() == "trash";
            let url = format!(
                "https://www.googleapis.com/drive/v3/files/{safe_id}?supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,size,webViewLink,trashed"
            );
            let resp = http
                .patch(&url)
                .bearer_auth(&access_token)
                .json(&serde_json::json!({ "trashed": trashed_flag }))
                .send()
                .await
                .map_err(|e| err_json(StatusCode::BAD_GATEWAY, e.to_string()))?;
            let updated: RawDriveFile = drive_send_json(resp).await?;
            Ok(Json(serde_json::json!({
                "ok": true,
                "file": DriveFileItem::from(updated),
            })))
        }
        "delete" => {
            let url = format!(
                "https://www.googleapis.com/drive/v3/files/{safe_id}?supportsAllDrives=true"
            );
            let resp = http
                .delete(&url)
                .bearer_auth(&access_token)
                .send()
                .await
                .map_err(|e| err_json(StatusCode::BAD_GATEWAY, e.to_string()))?;
            if !resp.status().is_success() {
                let st = resp.status();
                let txt = resp.text().await.unwrap_or_default();
                return Err(err_json(
                    StatusCode::BAD_GATEWAY,
                    format!("Delete failed ({st}): {txt}"),
                ));
            }
            Ok(Json(serde_json::json!({ "ok": true })))
        }
        _ => Err(err_json(StatusCode::BAD_REQUEST, "Unsupported Drive action.")),
    }
}

async fn upload_or_update_multipart_doc(
    http: &reqwest::Client,
    access_token: &str,
    name: &str,
    markdown_content: &str,
    parent_id: Option<&str>,
    as_google_doc: bool,
) -> Result<(DriveFileItem, &'static str), (StatusCode, Json<serde_json::Value>)> {
    let target_mime = if as_google_doc {
        Some("application/vnd.google-apps.document")
    } else {
        None
    };

    let existing = find_drive_item_by_name(
        http,
        access_token,
        name,
        parent_id,
        target_mime,
        true,
    )
    .await?;

    let boundary = format!("exonovel_gdoc_{}", random_hex_token(8));

    if let Some(ex) = existing {
        let meta = serde_json::json!({ "name": name });
        let body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n--{boundary}\r\nContent-Type: text/markdown; charset=UTF-8\r\n\r\n{}\r\n--{boundary}--",
            meta,
            markdown_content
        );
        let safe_id = sanitize_drive_id(&ex.id)
            .ok_or_else(|| err_json(StatusCode::BAD_GATEWAY, "Malformed existing Drive ID."))?;
        let url = format!(
            "https://www.googleapis.com/upload/drive/v3/files/{safe_id}?uploadType=multipart&supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,size,webViewLink,trashed"
        );
        let resp = http
            .patch(&url)
            .bearer_auth(access_token)
            .header(
                "Content-Type",
                format!("multipart/related; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await
            .map_err(|e| err_json(StatusCode::BAD_GATEWAY, e.to_string()))?;
        let raw: RawDriveFile = drive_send_json(resp).await?;
        Ok((DriveFileItem::from(raw), "updated"))
    } else {
        let mut meta = serde_json::json!({
            "name": name,
            "mimeType": if as_google_doc { "application/vnd.google-apps.document" } else { "text/markdown" },
        });
        if let Some(pid) = parent_id.and_then(sanitize_drive_id) {
            if pid != "root" {
                meta["parents"] = serde_json::json!([pid]);
            }
        }
        let body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n--{boundary}\r\nContent-Type: text/markdown; charset=UTF-8\r\n\r\n{}\r\n--{boundary}--",
            meta,
            markdown_content
        );
        let resp = http
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,size,webViewLink,trashed")
            .bearer_auth(access_token)
            .header(
                "Content-Type",
                format!("multipart/related; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await
            .map_err(|e| err_json(StatusCode::BAD_GATEWAY, e.to_string()))?;
        let raw: RawDriveFile = drive_send_json(resp).await?;
        Ok((DriveFileItem::from(raw), "created"))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDriveDocBody {
    pub rel_path: Option<String>,
    pub title: Option<String>,
    pub file_name: Option<String>,
    pub markdown: Option<String>,
    pub folder_name: Option<String>,
    pub parent_id: Option<String>,
    pub format: Option<String>,
}

pub async fn api_drive_save_doc(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<SaveDriveDocBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };

    let format_str = match body.format.as_deref() {
        Some("markdown") => "markdown",
        _ => "google-doc",
    };
    let as_google_doc = format_str == "google-doc";

    let (resolved_name, resolved_markdown) = if let Some(rel) = body
        .rel_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !user.authorized {
            return Err(err_json(
                StatusCode::FORBIDDEN,
                "Docs access authorization is required to export ExoNovel ADR documents.",
            ));
        }
        let db_doc = sqlx::query(
            "SELECT title, raw_markdown FROM adr_records WHERE rel_path = $1 LIMIT 1",
        )
        .bind(rel)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let (doc_title, md) = if let Some(row) = db_doc {
            let t: String = row.try_get("title").unwrap_or_default();
            let raw: String = row.try_get("raw_markdown").unwrap_or_default();
            (t, raw)
        } else if let Some(abs_path) = resolve_safe_doc_path(&state.root_dir, rel) {
            let raw = fs::read_to_string(&abs_path)
                .map_err(|_| err_json(StatusCode::NOT_FOUND, "Document file not found."))?;
            let fname = abs_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Document")
                .to_string();
            (fname, raw)
        } else {
            return Err(err_json(StatusCode::NOT_FOUND, "Document not found."));
        };

        let base_leaf = rel
            .rsplit('/')
            .next()
            .unwrap_or("document.md")
            .trim_end_matches(".md");
        let name = if as_google_doc {
            if doc_title.trim().is_empty() {
                base_leaf.to_string()
            } else {
                doc_title
            }
        } else {
            format!("{base_leaf}.md")
        };
        (name, md)
    } else {
        let md = body
            .markdown
            .unwrap_or_default();
        if md.trim().is_empty() {
            return Err(err_json(
                StatusCode::BAD_REQUEST,
                "Document content cannot be empty.",
            ));
        }
        if md.len() > 4 * 1024 * 1024 {
            return Err(err_json(
                StatusCode::BAD_REQUEST,
                "Document exceeds 4 MB size limit.",
            ));
        }
        let raw_title = body
            .file_name
            .as_deref()
            .or(body.title.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("ExoNovel Draft");
        let sanitized_leaf = raw_title
            .replace(['/', '\\'], "-")
            .trim_end_matches(".md")
            .chars()
            .take(140)
            .collect::<String>();
        let name = if as_google_doc {
            sanitized_leaf
        } else {
            format!("{sanitized_leaf}.md")
        };
        (name, md)
    };

    let access_token = get_valid_google_access_token(pool, did_peer).await?;
    let http = reqwest::Client::new();

    let (target_parent_id, ensured_folder) = match body
        .folder_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(fname) => {
            let folder =
                ensure_drive_folder_remote(&http, &access_token, fname, body.parent_id.as_deref())
                    .await?;
            (Some(folder.id.clone()), Some(folder))
        }
        None => {
            if let Some(pid) = body.parent_id.as_deref().and_then(sanitize_drive_id) {
                (Some(pid), None)
            } else {
                let folder =
                    ensure_drive_folder_remote(&http, &access_token, "ExoNovel Docs", Some("root"))
                        .await?;
                (Some(folder.id.clone()), Some(folder))
            }
        }
    };

    let (file, action) = upload_or_update_multipart_doc(
        &http,
        &access_token,
        &resolved_name,
        &resolved_markdown,
        target_parent_id.as_deref(),
        as_google_doc,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "file": file,
        "action": action,
        "folder": ensured_folder,
        "format": format_str,
    })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BackupArchiveBody {
    pub root_folder_name: Option<String>,
    pub parent_id: Option<String>,
    pub format: Option<String>,
}

pub async fn api_drive_backup_archive(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<BackupArchiveBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !user.authorized {
        return Err(err_json(
            StatusCode::FORBIDDEN,
            "Docs access authorization is required to back up ExoNovel Docs.",
        ));
    }
    let pool = state.db.as_ref().ok_or_else(db_unavailable)?;
    let Some(did_peer) = user.did_peer.as_deref() else {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "did:peer identity unavailable.",
        ));
    };

    let format_str = match body.format.as_deref() {
        Some("markdown") => "markdown",
        _ => "google-doc",
    };
    let as_google_doc = format_str == "google-doc";
    let root_folder_name = body
        .root_folder_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("ExoNovel Docs");

    let rows = sqlx::query(
        "SELECT rel_path, title, raw_markdown FROM adr_records ORDER BY rel_path ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let access_token = get_valid_google_access_token(pool, did_peer).await?;
    let http = reqwest::Client::new();

    let archive_folder = ensure_drive_folder_remote(
        &http,
        &access_token,
        root_folder_name,
        body.parent_id.as_deref().or(Some("root")),
    )
    .await?;

    let mut folder_cache: HashMap<String, String> = HashMap::new();
    folder_cache.insert(String::new(), archive_folder.id.clone());

    let mut created_count = 0_usize;
    let mut updated_count = 0_usize;
    let total = rows.len();

    for row in rows {
        let rel_path: String = row.try_get("rel_path").unwrap_or_default();
        let title: String = row.try_get("title").unwrap_or_default();
        let raw_markdown: String = row.try_get("raw_markdown").unwrap_or_default();

        let normalized = rel_path.trim_start_matches("docs/");
        let mut parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let file_leaf = parts.pop().unwrap_or("document.md");

        let mut current_parent_id = archive_folder.id.clone();
        let mut cache_key = String::new();
        for segment in parts {
            cache_key = if cache_key.is_empty() {
                segment.to_string()
            } else {
                format!("{cache_key}/{segment}")
            };
            if let Some(cached_id) = folder_cache.get(&cache_key) {
                current_parent_id = cached_id.clone();
            } else {
                let sub = ensure_drive_folder_remote(
                    &http,
                    &access_token,
                    segment,
                    Some(&current_parent_id),
                )
                .await?;
                current_parent_id = sub.id.clone();
                folder_cache.insert(cache_key.clone(), current_parent_id.clone());
            }
        }

        let target_name = if as_google_doc {
            let stem = file_leaf.trim_end_matches(".md");
            if title.trim().is_empty() {
                stem.to_string()
            } else {
                format!("{stem} — {}", title.trim())
            }
        } else {
            file_leaf.to_string()
        };

        let (_, action) = upload_or_update_multipart_doc(
            &http,
            &access_token,
            &target_name,
            &raw_markdown,
            Some(&current_parent_id),
            as_google_doc,
        )
        .await?;

        if action == "created" {
            created_count += 1;
        } else {
            updated_count += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "archiveFolder": archive_folder,
        "createdCount": created_count,
        "updatedCount": updated_count,
        "total": total,
        "format": format_str,
    })))
}

#[cfg(test)]
mod tests {
    use super::{
        escape_drive_query_value, now_unix_secs, open_oauth_state, sanitize_drive_id,
        seal_oauth_state, OAuthStatePayload,
    };

    #[test]
    fn oauth_state_roundtrip_and_sanitization() {
        let payload = OAuthStatePayload {
            did_peer: "did:peer:0z6Mktest123".to_string(),
            auth_user_id: "user_1".to_string(),
            redirect_uri: "https://exonovel.exosystems.net/api/oauth/google/callback".to_string(),
            exp: now_unix_secs() + 300,
            nonce: "abcdef".to_string(),
            return_tab: "preferences".to_string(),
            return_doc_path: "docs/adr/001-auth.md".to_string(),
        };
        let tok = seal_oauth_state(&payload).expect("state seal should succeed");
        let parsed = open_oauth_state(&tok).expect("state open should succeed");
        assert_eq!(parsed.did_peer, "did:peer:0z6Mktest123");
        assert_eq!(parsed.return_tab, "preferences");
        assert_eq!(sanitize_drive_id("1AbC_xyz-99"), Some("1AbC_xyz-99".to_string()));
        assert_eq!(sanitize_drive_id("../etc/passwd"), None);
        assert_eq!(escape_drive_query_value("Bob's \\ Notes"), "Bob\\'s \\\\ Notes");
    }
}
