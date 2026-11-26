use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::FromRequestParts,
    http::{
        header::{ACCEPT_LANGUAGE, AUTHORIZATION, USER_AGENT},
        request::Parts,
        StatusCode,
    },
    Json,
};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    db::{
        is_authorized_for_docs, resolve_and_sync_identity, AuthCaptureContext, UserPersona,
        UserProfile,
    },
    AppState,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifiedSessionUser {
    pub auth_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub did_peer: Option<String>,
    pub account_role: String,
    pub account_status: String,
    pub authorized: bool,
    pub is_admin: bool,
    pub profile: Option<UserProfile>,
    pub personas: Vec<UserPersona>,
    pub active_persona: Option<UserPersona>,
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
    #[serde(alias = "emailVerified")]
    email_verified: Option<bool>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    image: Option<String>,
    locale: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

pub async fn verify_neon_auth_token(
    state: &AppState,
    token: &str,
    ip_address: String,
    user_agent: String,
    accept_language: Option<String>,
) -> Result<VerifiedSessionUser, (StatusCode, Json<serde_json::Value>)> {
    let (Some(base_url), Some(jwks_url)) = (&state.neon_auth_base_url, &state.neon_auth_jwks_url)
    else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Authentication service is not configured." })),
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

    let claims = decoded.claims;
    let email = claims
        .email
        .as_deref()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());

    let locale = claims
        .locale
        .clone()
        .or_else(|| {
            accept_language
                .as_deref()
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty());

    let mut raw_claims = serde_json::Map::new();
    for (k, v) in claims.extra {
        raw_claims.insert(k, v);
    }
    raw_claims.insert("sub".to_string(), serde_json::Value::String(claims.sub.clone()));
    if let Some(em) = &email {
        raw_claims.insert("email".to_string(), serde_json::Value::String(em.clone()));
    }
    if let Some(ev) = claims.email_verified {
        raw_claims.insert("email_verified".to_string(), serde_json::Value::Bool(ev));
    }
    if let Some(nm) = &claims.name {
        raw_claims.insert("name".to_string(), serde_json::Value::String(nm.clone()));
    }
    if let Some(gn) = &claims.given_name {
        raw_claims.insert("given_name".to_string(), serde_json::Value::String(gn.clone()));
    }
    if let Some(fnm) = &claims.family_name {
        raw_claims.insert("family_name".to_string(), serde_json::Value::String(fnm.clone()));
    }
    if let Some(loc) = &locale {
        raw_claims.insert("locale".to_string(), serde_json::Value::String(loc.clone()));
    }

    let capture = AuthCaptureContext {
        auth_user_id: claims.sub.clone(),
        email: email.clone(),
        email_verified: claims.email_verified,
        name: claims.name.clone(),
        given_name: claims.given_name.clone(),
        family_name: claims.family_name.clone(),
        picture: claims.picture.clone().or(claims.image.clone()),
        locale,
        ip_address,
        user_agent,
        raw_claims: serde_json::Value::Object(raw_claims),
    };

    let sync_out = if let Some(pool) = &state.db {
        match resolve_and_sync_identity(pool, &capture).await {
            Ok(out) => Some(out),
            Err(err) => {
                eprintln!("resolve_and_sync_identity warning: {err}");
                None
            }
        }
    } else {
        None
    };

    let (did_peer, account_role, account_status, profile, personas, active_persona) =
        match sync_out {
            Some(out) => {
                let active_did = out.profile.active_persona_did.clone();
                let active_p = out
                    .personas
                    .iter()
                    .find(|p| p.persona_did == active_did)
                    .or_else(|| out.personas.first())
                    .cloned();
                (
                    Some(out.did_peer),
                    out.account_role,
                    out.account_status,
                    Some(out.profile),
                    out.personas,
                    active_p,
                )
            }
            None => (
                None,
                "user".to_string(),
                "active".to_string(),
                None,
                Vec::new(),
                None,
            ),
        };

    let (authorized, is_admin) =
        is_authorized_for_docs(state.db.as_ref(), did_peer.as_deref(), email.as_deref()).await;

    Ok(VerifiedSessionUser {
        auth_user_id: claims.sub,
        email,
        name: claims.name,
        did_peer,
        account_role,
        account_status,
        authorized,
        is_admin,
        profile,
        personas,
        active_persona,
    })
}

pub struct AuthUser(pub VerifiedSessionUser);

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

        let ip_address = parts
            .headers
            .get("x-forwarded-for")
            .or_else(|| parts.headers.get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(str::trim)
            .unwrap_or("")
            .to_string();

        let user_agent = parts
            .headers
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let accept_language = parts
            .headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);

        let user =
            verify_neon_auth_token(state, token, ip_address, user_agent, accept_language).await?;
        Ok(AuthUser(user))
    }
}
