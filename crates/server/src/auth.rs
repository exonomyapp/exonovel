use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    Json,
};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    db::{is_email_authorized_for_docs, resolve_or_mint_did_peer},
    AppState,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifiedSessionUser {
    pub auth_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub did_peer: Option<String>,
    pub authorized: bool,
    pub is_admin: bool,
}

#[derive(Clone)]
pub struct JwksCache {
    http: reqwest::Client,
    inner: Arc<RwLock<Option<(Instant, JwkSet)>>>,
}

impl JwksCache {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            inner: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_jwks(&self, url: &str, force_refresh: bool) -> Result<JwkSet, String> {
        if !force_refresh {
            let guard = self.inner.read().await;
            if let Some((fetched_at, set)) = guard.as_ref() {
                if fetched_at.elapsed() < Duration::from_secs(3600) {
                    return Ok(set.clone());
                }
            }
        }

        let jwks = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {e}"))?
            .error_for_status()
            .map_err(|e| format!("JWKS endpoint error: {e}"))?
            .json::<JwkSet>()
            .await
            .map_err(|e| format!("Invalid JWKS payload: {e}"))?;

        let mut guard = self.inner.write().await;
        *guard = Some((Instant::now(), jwks.clone()));
        Ok(jwks)
    }
}

#[derive(Debug, Deserialize)]
struct TokenClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
}

pub async fn verify_neon_auth_token(
    state: &AppState,
    token: &str,
) -> Result<VerifiedSessionUser, (StatusCode, Json<serde_json::Value>)> {
    let (Some(base_url), Some(jwks_url)) = (&state.neon_auth_base_url, &state.neon_auth_jwks_url)
    else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Neon Auth is not configured." })),
        ));
    };

    let header = decode_header(token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Malformed session token." })),
        )
    })?;

    let mut jwks = state
        .jwks_cache
        .get_jwks(jwks_url, false)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Unable to load auth keys." })),
            )
        })?;

    let mut matched = header
        .kid
        .as_deref()
        .and_then(|kid| jwks.find(kid))
        .or_else(|| jwks.keys.first());

    if matched.is_none() {
        if let Ok(fresh) = state.jwks_cache.get_jwks(jwks_url, true).await {
            jwks = fresh;
            matched = header
                .kid
                .as_deref()
                .and_then(|kid| jwks.find(kid))
                .or_else(|| jwks.keys.first());
        }
    }

    let jwk = matched.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "No matching signing key." })),
        )
    })?;

    let decoding_key = DecodingKey::from_jwk(jwk).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid JWK key." })),
        )
    })?;

    let mut validation = Validation::new(header.alg);
    validation.validate_aud = false;
    if let Ok(parsed) = reqwest::Url::parse(base_url) {
        let origin = parsed.origin().ascii_serialization();
        validation.set_issuer(&[base_url.as_str(), origin.as_str()]);
    }

    let decoded = decode::<TokenClaims>(token, &decoding_key, &validation).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid or expired session token." })),
        )
    })?;

    let email = decoded
        .claims
        .email
        .as_deref()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());

    let did_peer = if let Some(pool) = &state.db {
        match resolve_or_mint_did_peer(
            pool,
            &decoded.claims.sub,
            email.as_deref(),
            "neon-better-auth",
        )
        .await
        {
            Ok(identity) => Some(identity.did_peer),
            Err(_) => None,
        }
    } else {
        None
    };

    let (authorized, is_admin) =
        is_email_authorized_for_docs(state.db.as_ref(), email.as_deref()).await;

    Ok(VerifiedSessionUser {
        auth_user_id: decoded.claims.sub,
        email,
        name: decoded.claims.name,
        did_peer,
        authorized,
        is_admin,
    })
}

pub struct AuthUser(pub VerifiedSessionUser);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header_val = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let token = header_val
            .strip_prefix("Bearer ")
            .map(str::trim)
            .unwrap_or("");

        if token.is_empty() {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Authentication required." })),
            ));
        }

        let user = verify_neon_auth_token(state, token).await?;
        Ok(AuthUser(user))
    }
}
