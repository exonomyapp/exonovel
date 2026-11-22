use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{
    pkcs8::{spki::der::pem::LineEnding, EncodePrivateKey},
    SigningKey,
};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};

use crate::media::ImageDerivative;

pub fn now_timestamp_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

fn random_token(prefix: &str, num_bytes: usize) -> String {
    let mut bytes = vec![0_u8; num_bytes];
    OsRng.fill_bytes(&mut bytes);
    format!("{prefix}_{}", bs58::encode(&bytes).into_string())
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub did_peer: String,
    pub active_persona_did: String,
    pub display_name: String,
    pub pen_name: String,
    pub bio: String,
    pub avatar_asset_id: String,
    pub avatar_url: String,
    pub avatar_thumb_url: String,
    pub website: String,
    pub location: String,
    pub locale: String,
    pub timezone: String,
    pub preferred_format: String,
    pub appearance_mode: String,
    pub ui_locale: String,
    pub default_input_locale: String,
    pub preferred_ui_locales: Vec<String>,
    pub preferred_input_locales: Vec<String>,
    pub sync_plan_tier: String,
    pub willow_sync_enabled: bool,
    pub last_active_tab: String,
    pub last_doc_path: String,
    pub provider_given_name: String,
    pub provider_family_name: String,
    pub provider_account_id: String,
    pub provider_name: String,
    pub provider_scopes: String,
    pub email_verified: bool,
    pub auth_created_at: Option<String>,
    pub last_sign_in_at: String,
    pub last_sign_in_ip: String,
    pub last_user_agent: String,
    pub provider_raw_json: serde_json::Value,
    pub willow_namespace: String,
    pub willow_subspace: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPersona {
    pub persona_did: String,
    pub owner_did: String,
    pub public_key: String,
    pub is_default: bool,
    pub display_name: String,
    pub handle: String,
    pub bio: String,
    pub avatar_asset_id: String,
    pub avatar_url: String,
    pub avatar_thumb_url: String,
    pub willow_namespace: String,
    pub willow_subspace: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityRecord {
    pub did_peer: String,
    pub public_key: String,
    pub email: Option<String>,
    pub auth_user_id: String,
    pub provider: String,
    pub account_role: String,
    pub account_status: String,
    pub created_at: String,
    pub profile: UserProfile,
    pub docs_status: String,
    pub docs_allowed: bool,
    pub docs_role: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowlistEntry {
    pub did_peer: Option<String>,
    pub display_name: String,
    pub email: String,
    pub status: String,
    pub enabled: bool,
    pub role: String,
    pub note: String,
    pub added_by: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberView {
    pub member_owner_did: String,
    pub member_persona_did: String,
    pub display_name: String,
    pub avatar_thumb_url: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupView {
    pub group_id: String,
    pub owner_did: String,
    pub created_by_persona_did: String,
    pub name: String,
    pub description: String,
    pub my_role: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
    pub created_at: String,
    pub members: Vec<GroupMemberView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteView {
    pub code: String,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub target_group_role: String,
    pub note: String,
    pub use_count: i32,
    pub active: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralView {
    pub referred_owner_did: String,
    pub referred_persona_did: String,
    pub display_name: String,
    pub avatar_thumb_url: String,
    pub invite_code: String,
    pub group_name: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitePreview {
    pub code: String,
    pub valid: bool,
    pub inviter_display_name: String,
    pub inviter_avatar_thumb_url: String,
    pub group_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvatarUploadResult {
    pub asset_id: String,
    pub thumb_url: String,
    pub medium_url: String,
    pub large_url: String,
}

#[derive(Clone, Debug, Default)]
pub struct AuthCaptureContext {
    pub auth_user_id: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub raw_claims: serde_json::Value,
}

pub fn get_admin_email() -> String {
    std::env::var("ADMIN_EMAIL")
        .unwrap_or_else(|_| "exonomyapp@gmail.com".to_string())
        .trim()
        .to_lowercase()
}

pub async fn connect_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    if url.trim().is_empty() {
        return None;
    }
    PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await
        .ok()
}

pub async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_identities (
            id SERIAL PRIMARY KEY,
            did_peer TEXT NOT NULL UNIQUE,
            public_key TEXT NOT NULL,
            encrypted_private_key TEXT NOT NULL,
            auth_user_id TEXT UNIQUE,
            email TEXT,
            provider TEXT NOT NULL DEFAULT 'better-auth',
            account_role TEXT NOT NULL DEFAULT 'user',
            account_status TEXT NOT NULL DEFAULT 'active',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE user_identities ADD COLUMN IF NOT EXISTS account_role TEXT NOT NULL DEFAULT 'user';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_identities ADD COLUMN IF NOT EXISTS account_status TEXT NOT NULL DEFAULT 'active';")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_profiles (
            did_peer TEXT PRIMARY KEY REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            active_persona_did TEXT NOT NULL DEFAULT '',
            display_name TEXT NOT NULL DEFAULT '',
            pen_name TEXT NOT NULL DEFAULT '',
            bio TEXT NOT NULL DEFAULT '',
            avatar_asset_id TEXT NOT NULL DEFAULT '',
            avatar_url TEXT NOT NULL DEFAULT '',
            avatar_thumb_url TEXT NOT NULL DEFAULT '',
            website TEXT NOT NULL DEFAULT '',
            location TEXT NOT NULL DEFAULT '',
            locale TEXT NOT NULL DEFAULT '',
            timezone TEXT NOT NULL DEFAULT '',
            preferred_format TEXT NOT NULL DEFAULT 'novel',
            appearance_mode TEXT NOT NULL DEFAULT 'system',
            last_active_tab TEXT NOT NULL DEFAULT 'workspace',
            last_doc_path TEXT NOT NULL DEFAULT '',
            provider_given_name TEXT NOT NULL DEFAULT '',
            provider_family_name TEXT NOT NULL DEFAULT '',
            provider_account_id TEXT NOT NULL DEFAULT '',
            provider_name TEXT NOT NULL DEFAULT '',
            provider_scopes TEXT NOT NULL DEFAULT '',
            email_verified BOOLEAN NOT NULL DEFAULT FALSE,
            auth_created_at TIMESTAMPTZ,
            last_sign_in_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            last_sign_in_ip TEXT NOT NULL DEFAULT '',
            last_user_agent TEXT NOT NULL DEFAULT '',
            provider_raw_json JSONB NOT NULL DEFAULT '{}'::jsonb,
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT '',
            willow_path TEXT NOT NULL DEFAULT '/profile',
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE user_profiles DROP COLUMN IF EXISTS width_mode;")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS active_persona_did TEXT NOT NULL DEFAULT '';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS avatar_asset_id TEXT NOT NULL DEFAULT '';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS avatar_thumb_url TEXT NOT NULL DEFAULT '';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS willow_namespace TEXT NOT NULL DEFAULT 'exonovel';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS willow_subspace TEXT NOT NULL DEFAULT '';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS willow_path TEXT NOT NULL DEFAULT '/profile';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS willow_timestamp_us BIGINT NOT NULL DEFAULT 0;")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS ui_locale TEXT NOT NULL DEFAULT 'en';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS default_input_locale TEXT NOT NULL DEFAULT 'en';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS preferred_ui_locales TEXT[] NOT NULL DEFAULT ARRAY['en']::text[];")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS preferred_input_locales TEXT[] NOT NULL DEFAULT ARRAY['en']::text[];")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS sync_plan_tier TEXT NOT NULL DEFAULT 'free';")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE user_profiles ADD COLUMN IF NOT EXISTS willow_sync_enabled BOOLEAN NOT NULL DEFAULT FALSE;")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_personas (
            persona_did TEXT PRIMARY KEY,
            owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            public_key TEXT NOT NULL,
            encrypted_private_key TEXT NOT NULL,
            is_default BOOLEAN NOT NULL DEFAULT FALSE,
            display_name TEXT NOT NULL DEFAULT '',
            handle TEXT NOT NULL DEFAULT '',
            bio TEXT NOT NULL DEFAULT '',
            avatar_asset_id TEXT NOT NULL DEFAULT '',
            avatar_url TEXT NOT NULL DEFAULT '',
            avatar_thumb_url TEXT NOT NULL DEFAULT '',
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

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_personas_owner ON user_personas(owner_did);")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS media_assets (
            asset_id TEXT NOT NULL,
            variant TEXT NOT NULL,
            owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            persona_did TEXT NOT NULL DEFAULT '',
            mime_type TEXT NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            byte_length BIGINT NOT NULL,
            payload_digest TEXT NOT NULL,
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT '',
            willow_path TEXT NOT NULL DEFAULT '',
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            bytes BYTEA NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (asset_id, variant)
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS docs_access_allowlist (
            email TEXT PRIMARY KEY,
            did_peer TEXT,
            status TEXT NOT NULL DEFAULT 'reader',
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            role TEXT NOT NULL DEFAULT 'reader',
            note TEXT NOT NULL DEFAULT '',
            added_by TEXT NOT NULL DEFAULT 'system',
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE docs_access_allowlist ADD COLUMN IF NOT EXISTS did_peer TEXT;")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE docs_access_allowlist ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'reader';")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_docs_allowlist_did_peer ON docs_access_allowlist(did_peer) WHERE did_peer IS NOT NULL;",
    )
    .execute(pool)
    .await?;

    let _ = sqlx::query(
        r#"
        UPDATE docs_access_allowlist
        SET status = CASE
            WHEN role = 'admin' THEN 'admin'
            WHEN enabled = FALSE THEN 'suspended'
            WHEN role = 'moderator' THEN 'moderator'
            ELSE 'reader'
        END
        WHERE status NOT IN ('admin', 'moderator', 'reader', 'pending', 'suspended');
        "#,
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        r#"
        UPDATE docs_access_allowlist a
        SET did_peer = ui.did_peer
        FROM user_identities ui
        WHERE a.did_peer IS NULL
          AND ui.email IS NOT NULL
          AND LOWER(ui.email) = LOWER(a.email);
        "#,
    )
    .execute(pool)
    .await;

    let admin_email = get_admin_email();
    sqlx::query(
        r#"
        INSERT INTO docs_access_allowlist (email, status, enabled, role, note, added_by)
        VALUES ($1, 'admin', TRUE, 'admin', 'Primary ExoNovel documentation owner', 'system')
        ON CONFLICT (email) DO UPDATE
        SET status = 'admin', enabled = TRUE, role = 'admin', updated_at = NOW();
        "#,
    )
    .bind(&admin_email)
    .execute(pool)
    .await?;

    let _ = sqlx::query(
        "UPDATE user_identities SET account_role = 'admin', account_status = 'active' WHERE LOWER(email) = LOWER($1);",
    )
    .bind(&admin_email)
    .execute(pool)
    .await;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            group_id TEXT PRIMARY KEY,
            owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            created_by_persona_did TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
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
        CREATE TABLE IF NOT EXISTS group_members (
            group_id TEXT NOT NULL REFERENCES groups(group_id) ON DELETE CASCADE,
            member_owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            member_persona_did TEXT NOT NULL DEFAULT '',
            role TEXT NOT NULL DEFAULT 'member',
            invited_by_did TEXT NOT NULL DEFAULT '',
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT '',
            willow_path TEXT NOT NULL DEFAULT '',
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (group_id, member_owner_did)
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS invites (
            code TEXT PRIMARY KEY,
            inviter_owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            inviter_persona_did TEXT NOT NULL DEFAULT '',
            group_id TEXT REFERENCES groups(group_id) ON DELETE SET NULL,
            target_group_role TEXT NOT NULL DEFAULT 'member',
            note TEXT NOT NULL DEFAULT '',
            use_count INTEGER NOT NULL DEFAULT 0,
            active BOOLEAN NOT NULL DEFAULT TRUE,
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT '',
            willow_path TEXT NOT NULL DEFAULT '',
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS referrals (
            id SERIAL PRIMARY KEY,
            referrer_owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            referrer_persona_did TEXT NOT NULL DEFAULT '',
            referred_owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            referred_persona_did TEXT NOT NULL DEFAULT '',
            invite_code TEXT NOT NULL REFERENCES invites(code) ON DELETE CASCADE,
            group_id TEXT REFERENCES groups(group_id) ON DELETE SET NULL,
            willow_namespace TEXT NOT NULL DEFAULT 'exonovel',
            willow_subspace TEXT NOT NULL DEFAULT '',
            willow_path TEXT NOT NULL DEFAULT '',
            willow_timestamp_us BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (invite_code, referred_owner_did)
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn wrapping_key_bytes() -> [u8; 32] {
    let secret = std::env::var("DID_KEY_ENCRYPTION_SECRET")
        .or_else(|_| std::env::var("NEON_AUTH_COOKIE_SECRET"))
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "exonovel-dev-wrapping-fallback".to_string());
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().into()
}

pub fn mint_did_peer_keypair() -> Result<(String, String, String), String> {
    let signing_key = SigningKey::generate(&mut OsRng);
    let raw_pub = signing_key.verifying_key().to_bytes();

    let mut multicodec = Vec::with_capacity(34);
    multicodec.extend_from_slice(&[0xed, 0x01]);
    multicodec.extend_from_slice(&raw_pub);

    let multibase = format!("z{}", bs58::encode(&multicodec).into_string());
    let did_peer = format!("did:peer:0{multibase}");

    let pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|e| format!("Failed to encode PKCS8 key: {e}"))?;

    let key = wrapping_key_bytes();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Invalid key length: {e}"))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let sealed = cipher
        .encrypt(&nonce, pem.as_bytes())
        .map_err(|e| format!("Failed to seal private key: {e}"))?;

    let tag_split = sealed.len().saturating_sub(16);
    let (ciphertext, tag) = sealed.split_at(tag_split);
    let encrypted = format!(
        "{}.{}.{}",
        B64.encode(nonce.as_slice()),
        B64.encode(tag),
        B64.encode(ciphertext)
    );

    Ok((did_peer, multibase, encrypted))
}

struct NeonAuthSnapshot {
    name: Option<String>,
    email_verified: Option<bool>,
    image: Option<String>,
    auth_created_at: Option<String>,
    provider_id: Option<String>,
    account_id: Option<String>,
    scopes: Option<String>,
    user_json: serde_json::Value,
    account_json: serde_json::Value,
}

fn sanitize_neon_account_json(val: serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::Object(mut map) => {
            for secret_key in ["password", "accessToken", "refreshToken", "idToken"] {
                map.remove(secret_key);
            }
            serde_json::Value::Object(map)
        }
        other => other,
    }
}

async fn fetch_neon_auth_snapshot(pool: &PgPool, auth_user_id: &str) -> Option<NeonAuthSnapshot> {
    let row = sqlx::query(
        r#"
        SELECT
            to_jsonb(u) AS user_json,
            to_jsonb(a) AS account_json
        FROM neon_auth."user" u
        LEFT JOIN neon_auth.account a ON u.id = a."userId"
        WHERE u.id::text = $1
        ORDER BY CASE WHEN COALESCE(a."providerId", '') <> 'credential' THEN 0 ELSE 1 END
        LIMIT 1
        "#,
    )
    .bind(auth_user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    let user_json: serde_json::Value = row.try_get("user_json").unwrap_or(serde_json::Value::Null);
    let raw_account: serde_json::Value = row.try_get("account_json").unwrap_or(serde_json::Value::Null);
    let account_json = sanitize_neon_account_json(raw_account);

    let get_str = |v: &serde_json::Value, key: &str| -> Option<String> {
        v.get(key)
            .and_then(|x| x.as_str())
            .map(str::to_string)
    };

    Some(NeonAuthSnapshot {
        name: get_str(&user_json, "name"),
        email_verified: user_json.get("emailVerified").and_then(|x| x.as_bool()),
        image: get_str(&user_json, "image"),
        auth_created_at: get_str(&user_json, "createdAt"),
        provider_id: get_str(&account_json, "providerId"),
        account_id: get_str(&account_json, "accountId"),
        scopes: get_str(&account_json, "scope"),
        user_json,
        account_json,
    })
}

fn default_display_name(ctx: &AuthCaptureContext, snap: Option<&NeonAuthSnapshot>) -> String {
    if let Some(n) = ctx.name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return n.to_string();
    }
    if let Some(n) = snap.and_then(|s| s.name.as_deref()).map(str::trim).filter(|s| !s.is_empty()) {
        return n.to_string();
    }
    if let Some(email) = ctx.email.as_deref() {
        if let Some(local) = email.split('@').next() {
            if !local.is_empty() {
                return local.to_string();
            }
        }
    }
    "ExoNovel Writer".to_string()
}

pub struct IdentitySyncOutput {
    pub did_peer: String,
    pub account_role: String,
    pub account_status: String,
    pub profile: UserProfile,
    pub personas: Vec<UserPersona>,
}

pub async fn resolve_and_sync_identity(
    pool: &PgPool,
    ctx: &AuthCaptureContext,
) -> Result<IdentitySyncOutput, String> {
    let snap = fetch_neon_auth_snapshot(pool, &ctx.auth_user_id).await;
    let provider_name = snap
        .as_ref()
        .and_then(|s| s.provider_id.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "neon-better-auth".to_string());

    let admin_email = get_admin_email();
    let is_owner_admin = ctx
        .email
        .as_deref()
        .is_some_and(|em| em.eq_ignore_ascii_case(&admin_email));

    let existing_row = sqlx::query(
        r#"
        SELECT did_peer, public_key, encrypted_private_key, email, account_role, account_status
        FROM user_identities
        WHERE auth_user_id = $1
        LIMIT 1
        "#,
    )
    .bind(&ctx.auth_user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("DB lookup failed: {e}"))?;

    let (did_peer, public_key, encrypted_private_key, account_role, account_status) =
        if let Some(row) = existing_row {
            let existing_did: String = row.try_get("did_peer").map_err(|e| e.to_string())?;
            let pub_k: String = row.try_get("public_key").unwrap_or_default();
            let enc_k: String = row.try_get("encrypted_private_key").unwrap_or_default();
            let existing_email: Option<String> = row.try_get("email").unwrap_or(None);
            let mut acc_role: String = row
                .try_get("account_role")
                .unwrap_or_else(|_| "user".to_string());
            let mut acc_status: String = row
                .try_get("account_status")
                .unwrap_or_else(|_| "active".to_string());
            if is_owner_admin {
                acc_role = "admin".to_string();
                acc_status = "active".to_string();
            }

            let next_email = ctx.email.as_deref().or(existing_email.as_deref());
            let _ = sqlx::query(
                "UPDATE user_identities SET email = COALESCE($1, email), provider = $2, account_role = $3, account_status = $4, updated_at = NOW() WHERE did_peer = $5",
            )
            .bind(next_email)
            .bind(&provider_name)
            .bind(&acc_role)
            .bind(&acc_status)
            .bind(&existing_did)
            .execute(pool)
            .await;

            (existing_did, pub_k, enc_k, acc_role, acc_status)
        } else {
            let (minted_did, pub_b58, encrypted_priv) = mint_did_peer_keypair()?;
            let init_role = if is_owner_admin { "admin" } else { "user" };
            let inserted = sqlx::query(
                r#"
                INSERT INTO user_identities (did_peer, public_key, encrypted_private_key, auth_user_id, email, provider, account_role, account_status)
                VALUES ($1, $2, $3, $4, $5, $6, $7, 'active')
                ON CONFLICT (auth_user_id) DO UPDATE
                SET email = COALESCE(EXCLUDED.email, user_identities.email),
                    provider = EXCLUDED.provider,
                    updated_at = NOW()
                RETURNING did_peer, public_key, encrypted_private_key, account_role, account_status
                "#,
            )
            .bind(&minted_did)
            .bind(&pub_b58)
            .bind(&encrypted_priv)
            .bind(&ctx.auth_user_id)
            .bind(ctx.email.as_deref())
            .bind(&provider_name)
            .bind(init_role)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("DB insert failed: {e}"))?;

            (
                inserted.try_get("did_peer").map_err(|e| e.to_string())?,
                inserted.try_get("public_key").unwrap_or(pub_b58),
                inserted.try_get("encrypted_private_key").unwrap_or(encrypted_priv),
                inserted.try_get("account_role").unwrap_or_else(|_| init_role.to_string()),
                inserted.try_get("account_status").unwrap_or_else(|_| "active".to_string()),
            )
        };

    if let Some(email) = ctx.email.as_deref() {
        let _ = sqlx::query(
            "UPDATE docs_access_allowlist SET did_peer = $1 WHERE LOWER(email) = LOWER($2) AND (did_peer IS NULL OR did_peer = '')",
        )
        .bind(&did_peer)
        .bind(email)
        .execute(pool)
        .await;
    }

    let ts_us = now_timestamp_us();
    let seed_display_name = default_display_name(ctx, snap.as_ref());
    let seed_avatar = ctx
        .picture
        .as_deref()
        .or_else(|| snap.as_ref().and_then(|s| s.image.as_deref()))
        .unwrap_or("")
        .trim()
        .to_string();
    let given_name = ctx.given_name.as_deref().unwrap_or("").trim().to_string();
    let family_name = ctx.family_name.as_deref().unwrap_or("").trim().to_string();
    let locale = ctx.locale.as_deref().unwrap_or("").trim().to_string();
    let email_verified = ctx
        .email_verified
        .or_else(|| snap.as_ref().and_then(|s| s.email_verified))
        .unwrap_or(false);
    let account_id = snap
        .as_ref()
        .and_then(|s| s.account_id.as_deref())
        .unwrap_or("")
        .to_string();
    let scopes = snap
        .as_ref()
        .and_then(|s| s.scopes.as_deref())
        .unwrap_or("")
        .to_string();
    let auth_created_at = snap.as_ref().and_then(|s| s.auth_created_at.clone());
    let raw_json = serde_json::json!({
        "jwtClaims": ctx.raw_claims,
        "neonAuthUser": snap.as_ref().map(|s| &s.user_json),
        "neonAuthAccount": snap.as_ref().map(|s| &s.account_json),
    });

    // Seed default persona anchored on owner's did:peer so every user starts with a default persona
    let _ = sqlx::query(
        r#"
        INSERT INTO user_personas (
            persona_did, owner_did, public_key, encrypted_private_key,
            is_default, display_name, handle, avatar_url, avatar_thumb_url,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $1, $2, $3, TRUE, $4, '', $5, $5, 'exonovel', $1, $6, $7)
        ON CONFLICT (persona_did) DO UPDATE
        SET display_name = CASE WHEN user_personas.display_name = '' THEN EXCLUDED.display_name ELSE user_personas.display_name END,
            avatar_url = CASE WHEN user_personas.avatar_url = '' THEN EXCLUDED.avatar_url ELSE user_personas.avatar_url END,
            avatar_thumb_url = CASE WHEN user_personas.avatar_thumb_url = '' THEN EXCLUDED.avatar_thumb_url ELSE user_personas.avatar_thumb_url END
        "#,
    )
    .bind(&did_peer)
    .bind(&public_key)
    .bind(&encrypted_private_key)
    .bind(&seed_display_name)
    .bind(&seed_avatar)
    .bind(format!("/personas/{did_peer}"))
    .bind(ts_us)
    .execute(pool)
    .await;

    let row = sqlx::query(
        r#"
        INSERT INTO user_profiles (
            did_peer, active_persona_did, display_name, avatar_url, avatar_thumb_url, locale,
            provider_given_name, provider_family_name, provider_account_id,
            provider_name, provider_scopes, email_verified,
            auth_created_at, last_sign_in_at, last_sign_in_ip, last_user_agent,
            provider_raw_json, willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES (
            $1, $1, $2, $3, $3, $4,
            $5, $6, $7,
            $8, $9, $10,
            CASE WHEN $11 ~ '^[0-9]{4}-[0-9]{2}-[0-9]{2}' THEN $11::timestamptz ELSE NULL END, NOW(), $12, $13,
            $14, 'exonovel', $1, '/profile', $15
        )
        ON CONFLICT (did_peer) DO UPDATE
        SET active_persona_did = CASE WHEN user_profiles.active_persona_did = '' THEN $1 ELSE user_profiles.active_persona_did END,
            display_name = CASE WHEN user_profiles.display_name = '' THEN EXCLUDED.display_name ELSE user_profiles.display_name END,
            avatar_url = CASE WHEN user_profiles.avatar_url = '' THEN EXCLUDED.avatar_url ELSE user_profiles.avatar_url END,
            avatar_thumb_url = CASE WHEN user_profiles.avatar_thumb_url = '' THEN EXCLUDED.avatar_thumb_url ELSE user_profiles.avatar_thumb_url END,
            locale = CASE WHEN user_profiles.locale = '' THEN EXCLUDED.locale ELSE user_profiles.locale END,
            provider_given_name = CASE WHEN EXCLUDED.provider_given_name <> '' THEN EXCLUDED.provider_given_name ELSE user_profiles.provider_given_name END,
            provider_family_name = CASE WHEN EXCLUDED.provider_family_name <> '' THEN EXCLUDED.provider_family_name ELSE user_profiles.provider_family_name END,
            provider_account_id = CASE WHEN EXCLUDED.provider_account_id <> '' THEN EXCLUDED.provider_account_id ELSE user_profiles.provider_account_id END,
            provider_name = CASE WHEN EXCLUDED.provider_name <> '' THEN EXCLUDED.provider_name ELSE user_profiles.provider_name END,
            provider_scopes = CASE WHEN EXCLUDED.provider_scopes <> '' THEN EXCLUDED.provider_scopes ELSE user_profiles.provider_scopes END,
            email_verified = EXCLUDED.email_verified OR user_profiles.email_verified,
            auth_created_at = COALESCE(user_profiles.auth_created_at, EXCLUDED.auth_created_at),
            last_sign_in_at = NOW(),
            last_sign_in_ip = EXCLUDED.last_sign_in_ip,
            last_user_agent = EXCLUDED.last_user_agent,
            provider_raw_json = EXCLUDED.provider_raw_json,
            willow_subspace = $1,
            willow_timestamp_us = $15,
            updated_at = NOW()
        RETURNING
            did_peer, active_persona_did, display_name, pen_name, bio,
            avatar_asset_id, avatar_url, avatar_thumb_url, website, location,
            locale, timezone, preferred_format, appearance_mode,
            ui_locale, default_input_locale, preferred_ui_locales, preferred_input_locales,
            sync_plan_tier, willow_sync_enabled,
            last_active_tab, last_doc_path, provider_given_name, provider_family_name,
            provider_account_id, provider_name, provider_scopes, email_verified,
            to_char(auth_created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS auth_created_at,
            to_char(last_sign_in_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_sign_in_at,
            last_sign_in_ip, last_user_agent, provider_raw_json,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        "#,
    )
    .bind(&did_peer)
    .bind(&seed_display_name)
    .bind(&seed_avatar)
    .bind(&locale)
    .bind(&given_name)
    .bind(&family_name)
    .bind(&account_id)
    .bind(&provider_name)
    .bind(&scopes)
    .bind(email_verified)
    .bind(auth_created_at.as_deref().unwrap_or(""))
    .bind(&ctx.ip_address)
    .bind(&ctx.user_agent)
    .bind(&raw_json)
    .bind(ts_us)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Profile sync failed: {e}"))?;

    let mut profile = profile_from_row(&row)?;
    let personas = list_personas(pool, &did_peer).await?;
    if !personas.iter().any(|p| p.persona_did == profile.active_persona_did) {
        profile.active_persona_did = did_peer.clone();
    }

    Ok(IdentitySyncOutput {
        did_peer,
        account_role,
        account_status,
        profile,
        personas,
    })
}

fn profile_from_row(row: &sqlx::postgres::PgRow) -> Result<UserProfile, String> {
    Ok(UserProfile {
        did_peer: row.try_get("did_peer").map_err(|e| e.to_string())?,
        active_persona_did: row.try_get("active_persona_did").unwrap_or_default(),
        display_name: row.try_get("display_name").unwrap_or_default(),
        pen_name: row.try_get("pen_name").unwrap_or_default(),
        bio: row.try_get("bio").unwrap_or_default(),
        avatar_asset_id: row.try_get("avatar_asset_id").unwrap_or_default(),
        avatar_url: row.try_get("avatar_url").unwrap_or_default(),
        avatar_thumb_url: row.try_get("avatar_thumb_url").unwrap_or_default(),
        website: row.try_get("website").unwrap_or_default(),
        location: row.try_get("location").unwrap_or_default(),
        locale: row.try_get("locale").unwrap_or_default(),
        timezone: row.try_get("timezone").unwrap_or_default(),
        preferred_format: row.try_get("preferred_format").unwrap_or_else(|_| "novel".to_string()),
        appearance_mode: row.try_get("appearance_mode").unwrap_or_else(|_| "system".to_string()),
        ui_locale: row.try_get("ui_locale").unwrap_or_else(|_| "en".to_string()),
        default_input_locale: row.try_get("default_input_locale").unwrap_or_else(|_| "en".to_string()),
        preferred_ui_locales: row.try_get("preferred_ui_locales").unwrap_or_else(|_| vec!["en".to_string()]),
        preferred_input_locales: row.try_get("preferred_input_locales").unwrap_or_else(|_| vec!["en".to_string()]),
        sync_plan_tier: row.try_get("sync_plan_tier").unwrap_or_else(|_| "free".to_string()),
        willow_sync_enabled: row.try_get("willow_sync_enabled").unwrap_or(false),
        last_active_tab: row.try_get("last_active_tab").unwrap_or_else(|_| "workspace".to_string()),
        last_doc_path: row.try_get("last_doc_path").unwrap_or_default(),
        provider_given_name: row.try_get("provider_given_name").unwrap_or_default(),
        provider_family_name: row.try_get("provider_family_name").unwrap_or_default(),
        provider_account_id: row.try_get("provider_account_id").unwrap_or_default(),
        provider_name: row.try_get("provider_name").unwrap_or_default(),
        provider_scopes: row.try_get("provider_scopes").unwrap_or_default(),
        email_verified: row.try_get("email_verified").unwrap_or(false),
        auth_created_at: row.try_get("auth_created_at").unwrap_or(None),
        last_sign_in_at: row.try_get("last_sign_in_at").unwrap_or_default(),
        last_sign_in_ip: row.try_get("last_sign_in_ip").unwrap_or_default(),
        last_user_agent: row.try_get("last_user_agent").unwrap_or_default(),
        provider_raw_json: row.try_get("provider_raw_json").unwrap_or_else(|_| serde_json::json!({})),
        willow_namespace: row.try_get("willow_namespace").unwrap_or_else(|_| "exonovel".to_string()),
        willow_subspace: row.try_get("willow_subspace").unwrap_or_default(),
        willow_path: row.try_get("willow_path").unwrap_or_else(|_| "/profile".to_string()),
        willow_timestamp_us: row.try_get("willow_timestamp_us").unwrap_or(0),
    })
}

fn persona_from_row(row: &sqlx::postgres::PgRow) -> Result<UserPersona, String> {
    Ok(UserPersona {
        persona_did: row.try_get("persona_did").map_err(|e| e.to_string())?,
        owner_did: row.try_get("owner_did").map_err(|e| e.to_string())?,
        public_key: row.try_get("public_key").unwrap_or_default(),
        is_default: row.try_get("is_default").unwrap_or(false),
        display_name: row.try_get("display_name").unwrap_or_default(),
        handle: row.try_get("handle").unwrap_or_default(),
        bio: row.try_get("bio").unwrap_or_default(),
        avatar_asset_id: row.try_get("avatar_asset_id").unwrap_or_default(),
        avatar_url: row.try_get("avatar_url").unwrap_or_default(),
        avatar_thumb_url: row.try_get("avatar_thumb_url").unwrap_or_default(),
        willow_namespace: row.try_get("willow_namespace").unwrap_or_else(|_| "exonovel".to_string()),
        willow_subspace: row.try_get("willow_subspace").unwrap_or_default(),
        willow_path: row.try_get("willow_path").unwrap_or_default(),
        willow_timestamp_us: row.try_get("willow_timestamp_us").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or_default(),
    })
}

pub async fn list_personas(pool: &PgPool, owner_did: &str) -> Result<Vec<UserPersona>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            persona_did, owner_did, public_key, is_default,
            display_name, handle, bio, avatar_asset_id, avatar_url, avatar_thumb_url,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM user_personas
        WHERE owner_did = $1
        ORDER BY is_default DESC, created_at ASC
        "#,
    )
    .bind(owner_did)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter().map(|r| persona_from_row(&r)).collect()
}

pub async fn create_persona(
    pool: &PgPool,
    owner_did: &str,
    display_name: &str,
    handle: &str,
    bio: &str,
) -> Result<UserPersona, String> {
    let clean_name = display_name.trim();
    if clean_name.is_empty() {
        return Err("Persona display name is required.".to_string());
    }
    let (persona_did, public_key, encrypted_private_key) = mint_did_peer_keypair()?;
    let ts_us = now_timestamp_us();
    let willow_path = format!("/personas/{persona_did}");

    let row = sqlx::query(
        r#"
        INSERT INTO user_personas (
            persona_did, owner_did, public_key, encrypted_private_key,
            is_default, display_name, handle, bio,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $2, $3, $4, FALSE, $5, $6, $7, 'exonovel', $2, $8, $9)
        RETURNING
            persona_did, owner_did, public_key, is_default,
            display_name, handle, bio, avatar_asset_id, avatar_url, avatar_thumb_url,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        "#,
    )
    .bind(&persona_did)
    .bind(owner_did)
    .bind(&public_key)
    .bind(&encrypted_private_key)
    .bind(clean_name)
    .bind(handle.trim())
    .bind(bio.trim())
    .bind(&willow_path)
    .bind(ts_us)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to create persona: {e}"))?;

    persona_from_row(&row)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaUpdateInput {
    pub persona_did: String,
    pub display_name: Option<String>,
    pub handle: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_thumb_url: Option<String>,
    pub avatar_asset_id: Option<String>,
}

pub async fn update_persona(
    pool: &PgPool,
    owner_did: &str,
    input: &PersonaUpdateInput,
) -> Result<UserPersona, String> {
    if let Some(av) = &input.avatar_url {
        if !is_allowed_avatar_url(av) {
            return Err("Avatar URL must start with https://, http://, or /api/media/.".to_string());
        }
    }
    if let Some(th) = &input.avatar_thumb_url {
        if !is_allowed_avatar_url(th) {
            return Err("Thumbnail URL must start with https://, http://, or /api/media/.".to_string());
        }
    }
    let ts_us = now_timestamp_us();
    let row = sqlx::query(
        r#"
        UPDATE user_personas
        SET display_name = COALESCE(NULLIF($3, ''), display_name),
            handle = COALESCE($4, handle),
            bio = COALESCE($5, bio),
            avatar_url = COALESCE($6, avatar_url),
            avatar_thumb_url = COALESCE($7, avatar_thumb_url),
            avatar_asset_id = COALESCE($8, avatar_asset_id),
            willow_timestamp_us = $9,
            updated_at = NOW()
        WHERE persona_did = $1 AND owner_did = $2
        RETURNING
            persona_did, owner_did, public_key, is_default,
            display_name, handle, bio, avatar_asset_id, avatar_url, avatar_thumb_url,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        "#,
    )
    .bind(input.persona_did.trim())
    .bind(owner_did)
    .bind(input.display_name.as_deref().map(str::trim))
    .bind(input.handle.as_deref().map(str::trim))
    .bind(input.bio.as_deref().map(str::trim))
    .bind(input.avatar_url.as_deref().map(str::trim))
    .bind(input.avatar_thumb_url.as_deref().map(str::trim))
    .bind(input.avatar_asset_id.as_deref().map(str::trim))
    .bind(ts_us)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to update persona: {e}"))?;

    let updated = persona_from_row(&row)?;
    if updated.is_default {
        let _ = sqlx::query(
            r#"
            UPDATE user_profiles
            SET display_name = $2,
                pen_name = $3,
                bio = $4,
                avatar_asset_id = $5,
                avatar_url = $6,
                avatar_thumb_url = $7,
                willow_timestamp_us = $8,
                updated_at = NOW()
            WHERE did_peer = $1
            "#,
        )
        .bind(owner_did)
        .bind(&updated.display_name)
        .bind(&updated.handle)
        .bind(&updated.bio)
        .bind(&updated.avatar_asset_id)
        .bind(&updated.avatar_url)
        .bind(&updated.avatar_thumb_url)
        .bind(ts_us)
        .execute(pool)
        .await;
    }

    Ok(updated)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdateInput {
    pub active_persona_did: Option<String>,
    pub display_name: Option<String>,
    pub pen_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_asset_id: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_thumb_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub preferred_format: Option<String>,
    pub appearance_mode: Option<String>,
    pub ui_locale: Option<String>,
    pub default_input_locale: Option<String>,
    pub preferred_ui_locales: Option<Vec<String>>,
    pub preferred_input_locales: Option<Vec<String>>,
    pub willow_sync_enabled: Option<bool>,
    pub last_active_tab: Option<String>,
    pub last_doc_path: Option<String>,
}

fn is_allowed_avatar_url(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("/api/media/")
        || lower.starts_with("https://")
        || lower.starts_with("http://")
}

fn is_allowed_website_url(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

pub async fn update_user_profile(
    pool: &PgPool,
    did_peer: &str,
    input: &ProfileUpdateInput,
) -> Result<UserProfile, String> {
    if let Some(avatar) = &input.avatar_url {
        if !is_allowed_avatar_url(avatar) {
            return Err("Avatar must be an http(s) URL or uploaded through ExoNovel media.".to_string());
        }
    }
    if let Some(thumb) = &input.avatar_thumb_url {
        if !is_allowed_avatar_url(thumb) {
            return Err("Thumbnail URL must be valid.".to_string());
        }
    }
    if let Some(site) = &input.website {
        if !is_allowed_website_url(site) {
            return Err("Website must start with https:// or http://.".to_string());
        }
    }
    if let Some(fmt) = &input.preferred_format {
        let v = fmt.trim();
        if !["novel", "article", "short-story", "screenplay", "poetry"].contains(&v) {
            return Err("Unsupported preferred writing format.".to_string());
        }
    }
    if let Some(app) = &input.appearance_mode {
        let v = app.trim();
        if !["system", "light", "dark"].contains(&v) {
            return Err("Unsupported appearance mode.".to_string());
        }
    }
    if let Some(tab) = &input.last_active_tab {
        let v = tab.trim();
        if !["workspace", "docs", "profile", "settings"].contains(&v) {
            return Err("Unsupported tab identifier.".to_string());
        }
    }

    if let Some(target_persona) = input.active_persona_did.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let owns_persona = sqlx::query(
            "SELECT 1 FROM user_personas WHERE persona_did = $1 AND owner_did = $2 LIMIT 1",
        )
        .bind(target_persona)
        .bind(did_peer)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        if owns_persona.is_none() {
            return Err("Selected persona does not belong to your account.".to_string());
        }
    }

    if let Some(ui_loc) = &input.ui_locale {
        if !crate::i18n::is_supported_locale(ui_loc) {
            return Err("Unsupported UI language code.".to_string());
        }
    }
    if let Some(in_loc) = &input.default_input_locale {
        if !crate::i18n::is_supported_locale(in_loc) {
            return Err("Unsupported input language code.".to_string());
        }
    }

    let ts_us = now_timestamp_us();
    let row = sqlx::query(
        r#"
        UPDATE user_profiles
        SET active_persona_did = COALESCE(NULLIF($2, ''), active_persona_did),
            display_name = COALESCE($3, display_name),
            pen_name = COALESCE($4, pen_name),
            bio = COALESCE($5, bio),
            avatar_asset_id = COALESCE($6, avatar_asset_id),
            avatar_url = COALESCE($7, avatar_url),
            avatar_thumb_url = COALESCE($8, avatar_thumb_url),
            website = COALESCE($9, website),
            location = COALESCE($10, location),
            locale = COALESCE($11, locale),
            timezone = COALESCE($12, timezone),
            preferred_format = COALESCE($13, preferred_format),
            appearance_mode = COALESCE($14, appearance_mode),
            last_active_tab = COALESCE($15, last_active_tab),
            last_doc_path = COALESCE($16, last_doc_path),
            ui_locale = COALESCE($18, ui_locale),
            default_input_locale = COALESCE($19, default_input_locale),
            preferred_ui_locales = COALESCE($20, preferred_ui_locales),
            preferred_input_locales = COALESCE($21, preferred_input_locales),
            willow_sync_enabled = COALESCE($22, willow_sync_enabled),
            willow_timestamp_us = $17,
            updated_at = NOW()
        WHERE did_peer = $1
        RETURNING
            did_peer, active_persona_did, display_name, pen_name, bio,
            avatar_asset_id, avatar_url, avatar_thumb_url, website, location,
            locale, timezone, preferred_format, appearance_mode,
            ui_locale, default_input_locale, preferred_ui_locales, preferred_input_locales,
            sync_plan_tier, willow_sync_enabled,
            last_active_tab, last_doc_path, provider_given_name, provider_family_name,
            provider_account_id, provider_name, provider_scopes, email_verified,
            to_char(auth_created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS auth_created_at,
            to_char(last_sign_in_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_sign_in_at,
            last_sign_in_ip, last_user_agent, provider_raw_json,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        "#,
    )
    .bind(did_peer)
    .bind(input.active_persona_did.as_deref().map(str::trim))
    .bind(input.display_name.as_deref().map(str::trim))
    .bind(input.pen_name.as_deref().map(str::trim))
    .bind(input.bio.as_deref().map(str::trim))
    .bind(input.avatar_asset_id.as_deref().map(str::trim))
    .bind(input.avatar_url.as_deref().map(str::trim))
    .bind(input.avatar_thumb_url.as_deref().map(str::trim))
    .bind(input.website.as_deref().map(str::trim))
    .bind(input.location.as_deref().map(str::trim))
    .bind(input.locale.as_deref().map(str::trim))
    .bind(input.timezone.as_deref().map(str::trim))
    .bind(input.preferred_format.as_deref().map(str::trim))
    .bind(input.appearance_mode.as_deref().map(str::trim))
    .bind(input.last_active_tab.as_deref().map(str::trim))
    .bind(input.last_doc_path.as_deref().map(str::trim))
    .bind(ts_us)
    .bind(input.ui_locale.as_deref().map(str::trim))
    .bind(input.default_input_locale.as_deref().map(str::trim))
    .bind(input.preferred_ui_locales.as_deref())
    .bind(input.preferred_input_locales.as_deref())
    .bind(input.willow_sync_enabled)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to update profile: {e}"))?;

    let updated = profile_from_row(&row)?;

    // Keep default persona aligned with primary profile edits
    let _ = sqlx::query(
        r#"
        UPDATE user_personas
        SET display_name = $2,
            handle = $3,
            bio = $4,
            avatar_asset_id = $5,
            avatar_url = $6,
            avatar_thumb_url = $7,
            willow_timestamp_us = $8,
            updated_at = NOW()
        WHERE persona_did = $1 AND owner_did = $1
        "#,
    )
    .bind(did_peer)
    .bind(&updated.display_name)
    .bind(&updated.pen_name)
    .bind(&updated.bio)
    .bind(&updated.avatar_asset_id)
    .bind(&updated.avatar_url)
    .bind(&updated.avatar_thumb_url)
    .bind(ts_us)
    .execute(pool)
    .await;

    Ok(updated)
}

pub async fn save_avatar_asset(
    pool: &PgPool,
    owner_did: &str,
    target_persona_did: Option<&str>,
    derivatives: Vec<ImageDerivative>,
) -> Result<AvatarUploadResult, String> {
    let asset_id = random_token("ast", 12);
    let ts_us = now_timestamp_us();
    let persona_did = target_persona_did
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(owner_did);

    let owns = sqlx::query(
        "SELECT is_default FROM user_personas WHERE persona_did = $1 AND owner_did = $2 LIMIT 1",
    )
    .bind(persona_did)
    .bind(owner_did)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let Some(persona_row) = owns else {
        return Err("Target persona does not belong to your account.".to_string());
    };
    let is_default: bool = persona_row.try_get("is_default").unwrap_or(false);

    for d in &derivatives {
        let willow_path = format!("/media/{asset_id}/{}", d.variant);
        sqlx::query(
            r#"
            INSERT INTO media_assets (
                asset_id, variant, owner_did, persona_did,
                mime_type, width, height, byte_length, payload_digest,
                willow_namespace, willow_subspace, willow_path, willow_timestamp_us,
                bytes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'exonovel', $3, $10, $11, $12)
            "#,
        )
        .bind(&asset_id)
        .bind(d.variant)
        .bind(owner_did)
        .bind(persona_did)
        .bind(d.mime_type)
        .bind(d.width)
        .bind(d.height)
        .bind(d.bytes.len() as i64)
        .bind(&d.payload_digest)
        .bind(&willow_path)
        .bind(ts_us)
        .bind(&d.bytes)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to store image variant: {e}"))?;
    }

    let thumb_url = format!("/api/media/{asset_id}/thumb");
    let medium_url = format!("/api/media/{asset_id}/medium");
    let large_url = format!("/api/media/{asset_id}/large");

    let _ = sqlx::query(
        r#"
        UPDATE user_personas
        SET avatar_asset_id = $2, avatar_url = $3, avatar_thumb_url = $4, willow_timestamp_us = $5, updated_at = NOW()
        WHERE persona_did = $1 AND owner_did = $6
        "#,
    )
    .bind(persona_did)
    .bind(&asset_id)
    .bind(&medium_url)
    .bind(&thumb_url)
    .bind(ts_us)
    .bind(owner_did)
    .execute(pool)
    .await;

    if is_default {
        let _ = sqlx::query(
            r#"
            UPDATE user_profiles
            SET avatar_asset_id = $2, avatar_url = $3, avatar_thumb_url = $4, willow_timestamp_us = $5, updated_at = NOW()
            WHERE did_peer = $1
            "#,
        )
        .bind(owner_did)
        .bind(&asset_id)
        .bind(&medium_url)
        .bind(&thumb_url)
        .bind(ts_us)
        .execute(pool)
        .await;
    }

    Ok(AvatarUploadResult {
        asset_id,
        thumb_url,
        medium_url,
        large_url,
    })
}

pub async fn load_media_variant(
    pool: &PgPool,
    asset_id: &str,
    variant: &str,
) -> Option<(String, String, Vec<u8>)> {
    if !["thumb", "medium", "large"].contains(&variant) {
        return None;
    }
    let row = sqlx::query(
        "SELECT mime_type, payload_digest, bytes FROM media_assets WHERE asset_id = $1 AND variant = $2 LIMIT 1",
    )
    .bind(asset_id)
    .bind(variant)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    Some((
        row.try_get("mime_type").ok()?,
        row.try_get("payload_digest").ok()?,
        row.try_get("bytes").ok()?,
    ))
}

pub async fn update_identity_account_role(
    pool: &PgPool,
    did_peer: &str,
    account_role: Option<&str>,
    account_status: Option<&str>,
) -> Result<(), String> {
    if let Some(role) = account_role {
        if !["user", "moderator", "admin"].contains(&role.trim()) {
            return Err("Role must be user, moderator, or admin.".to_string());
        }
    }
    if let Some(status) = account_status {
        if !["active", "restricted", "suspended"].contains(&status.trim()) {
            return Err("Account status must be active, restricted, or suspended.".to_string());
        }
    }
    let row = sqlx::query("SELECT email FROM user_identities WHERE did_peer = $1 LIMIT 1")
        .bind(did_peer)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(r) = row {
        let em: Option<String> = r.try_get("email").unwrap_or(None);
        if em.as_deref().is_some_and(|e| e.eq_ignore_ascii_case(&get_admin_email())) {
            return Ok(());
        }
    }

    sqlx::query(
        r#"
        UPDATE user_identities
        SET account_role = COALESCE($2, account_role),
            account_status = COALESCE($3, account_status),
            updated_at = NOW()
        WHERE did_peer = $1
        "#,
    )
    .bind(did_peer)
    .bind(account_role.map(str::trim))
    .bind(account_status.map(str::trim))
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn is_authorized_for_docs(
    pool: Option<&PgPool>,
    did_peer: Option<&str>,
    email: Option<&str>,
) -> (bool, bool) {
    let normalized_email = email.map(|e| e.trim().to_lowercase());
    if normalized_email.as_deref() == Some(get_admin_email().as_str()) {
        return (true, true);
    }

    let Some(db) = pool else {
        return (false, false);
    };

    let row = sqlx::query(
        r#"
        SELECT status, enabled, role
        FROM docs_access_allowlist
        WHERE ($1::text IS NOT NULL AND did_peer = $1)
           OR ($2::text IS NOT NULL AND LOWER(email) = LOWER($2))
        ORDER BY enabled DESC
        LIMIT 1
        "#,
    )
    .bind(did_peer)
    .bind(normalized_email)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    match row {
        Some(r) => {
            let status: String = r.try_get("status").unwrap_or_default();
            let active = ["admin", "moderator", "reader"].contains(&status.as_str());
            let is_admin = status == "admin";
            (active, is_admin)
        }
        None => (false, false),
    }
}

pub async fn list_identities(pool: &PgPool) -> Result<Vec<IdentityRecord>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            ui.did_peer,
            ui.public_key,
            ui.email,
            ui.auth_user_id,
            ui.provider,
            ui.account_role,
            ui.account_status,
            to_char(ui.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS ui_created_at,
            COALESCE(up.active_persona_did, ui.did_peer) AS active_persona_did,
            COALESCE(up.display_name, '') AS display_name,
            COALESCE(up.pen_name, '') AS pen_name,
            COALESCE(up.bio, '') AS bio,
            COALESCE(up.avatar_asset_id, '') AS avatar_asset_id,
            COALESCE(up.avatar_url, '') AS avatar_url,
            COALESCE(up.avatar_thumb_url, '') AS avatar_thumb_url,
            COALESCE(up.website, '') AS website,
            COALESCE(up.location, '') AS location,
            COALESCE(up.locale, '') AS locale,
            COALESCE(up.timezone, '') AS timezone,
            COALESCE(up.preferred_format, 'novel') AS preferred_format,
            COALESCE(up.appearance_mode, 'system') AS appearance_mode,
            COALESCE(up.ui_locale, 'en') AS ui_locale,
            COALESCE(up.default_input_locale, 'en') AS default_input_locale,
            COALESCE(up.preferred_ui_locales, ARRAY['en']::text[]) AS preferred_ui_locales,
            COALESCE(up.preferred_input_locales, ARRAY['en']::text[]) AS preferred_input_locales,
            COALESCE(up.sync_plan_tier, 'free') AS sync_plan_tier,
            COALESCE(up.willow_sync_enabled, FALSE) AS willow_sync_enabled,
            COALESCE(up.last_active_tab, 'workspace') AS last_active_tab,
            COALESCE(up.last_doc_path, '') AS last_doc_path,
            COALESCE(up.provider_given_name, '') AS provider_given_name,
            COALESCE(up.provider_family_name, '') AS provider_family_name,
            COALESCE(up.provider_account_id, '') AS provider_account_id,
            COALESCE(up.provider_name, ui.provider) AS provider_name,
            COALESCE(up.provider_scopes, '') AS provider_scopes,
            COALESCE(up.email_verified, FALSE) AS email_verified,
            to_char(up.auth_created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS auth_created_at,
            COALESCE(to_char(up.last_sign_in_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'), '') AS last_sign_in_at,
            COALESCE(up.last_sign_in_ip, '') AS last_sign_in_ip,
            COALESCE(up.last_user_agent, '') AS last_user_agent,
            COALESCE(up.provider_raw_json, '{}'::jsonb) AS provider_raw_json,
            COALESCE(up.willow_namespace, 'exonovel') AS willow_namespace,
            COALESCE(up.willow_subspace, ui.did_peer) AS willow_subspace,
            COALESCE(up.willow_path, '/profile') AS willow_path,
            COALESCE(up.willow_timestamp_us, 0) AS willow_timestamp_us,
            COALESCE(da.status, 'none') AS docs_status,
            COALESCE(da.enabled, FALSE) AS docs_allowed,
            da.role AS docs_role
        FROM user_identities ui
        LEFT JOIN user_profiles up ON up.did_peer = ui.did_peer
        LEFT JOIN LATERAL (
            SELECT a.status, a.enabled, a.role
            FROM docs_access_allowlist a
            WHERE a.did_peer = ui.did_peer
               OR (ui.email IS NOT NULL AND LOWER(a.email) = LOWER(ui.email))
            ORDER BY a.enabled DESC
            LIMIT 1
        ) da ON TRUE
        ORDER BY ui.created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|r| {
            let profile = profile_from_row(&r)?;
            Ok(IdentityRecord {
                did_peer: r.try_get("did_peer").map_err(|e| e.to_string())?,
                public_key: r.try_get("public_key").unwrap_or_default(),
                email: r.try_get("email").unwrap_or(None),
                auth_user_id: r.try_get("auth_user_id").map_err(|e| e.to_string())?,
                provider: r.try_get("provider").map_err(|e| e.to_string())?,
                account_role: r.try_get("account_role").unwrap_or_else(|_| "user".to_string()),
                account_status: r.try_get("account_status").unwrap_or_else(|_| "active".to_string()),
                created_at: r.try_get("ui_created_at").map_err(|e| e.to_string())?,
                profile,
                docs_status: r.try_get("docs_status").unwrap_or_else(|_| "none".to_string()),
                docs_allowed: r.try_get("docs_allowed").unwrap_or(false),
                docs_role: r.try_get("docs_role").unwrap_or(None),
            })
        })
        .collect()
}

pub async fn list_allowlist(pool: &PgPool) -> Result<Vec<AllowlistEntry>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            COALESCE(a.did_peer, ui.did_peer) AS did_peer,
            COALESCE(NULLIF(up.display_name, ''), NULLIF(SPLIT_PART(a.email, '@', 1), ''), COALESCE(a.did_peer, ui.did_peer, a.email)) AS display_name,
            a.email,
            a.status,
            a.enabled,
            a.role,
            a.note,
            a.added_by,
            to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        FROM docs_access_allowlist a
        LEFT JOIN user_identities ui
            ON (a.did_peer IS NOT NULL AND ui.did_peer = a.did_peer)
            OR (a.did_peer IS NULL AND ui.email IS NOT NULL AND LOWER(ui.email) = LOWER(a.email))
        LEFT JOIN user_profiles up
            ON up.did_peer = COALESCE(a.did_peer, ui.did_peer)
        ORDER BY CASE WHEN a.status = 'admin' THEN 0 ELSE 1 END, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|r| {
            Ok(AllowlistEntry {
                did_peer: r.try_get("did_peer").unwrap_or(None),
                display_name: r.try_get("display_name").unwrap_or_default(),
                email: r.try_get("email").map_err(|e| e.to_string())?,
                status: r.try_get("status").unwrap_or_else(|_| "reader".to_string()),
                enabled: r.try_get("enabled").map_err(|e| e.to_string())?,
                role: r.try_get("role").map_err(|e| e.to_string())?,
                note: r.try_get("note").map_err(|e| e.to_string())?,
                added_by: r.try_get("added_by").map_err(|e| e.to_string())?,
                updated_at: r.try_get("updated_at").map_err(|e| e.to_string())?,
            })
        })
        .collect()
}

pub async fn upsert_allowlist_entry(
    pool: &PgPool,
    did_peer: Option<&str>,
    email: Option<&str>,
    status: &str,
    note: &str,
    added_by: &str,
) -> Result<AllowlistEntry, String> {
    let clean_status = status.trim().to_lowercase();
    if !["admin", "moderator", "reader", "pending", "suspended"].contains(&clean_status.as_str()) {
        return Err("Status must be admin, moderator, reader, pending, or suspended.".to_string());
    }

    let clean_did = did_peer
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let clean_email = email
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());

    let (resolved_did, resolved_email) = match (clean_did, clean_email) {
        (Some(did), email_opt) => {
            let row = sqlx::query("SELECT email FROM user_identities WHERE did_peer = $1 LIMIT 1")
                .bind(&did)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;
            let found_email: Option<String> = row.and_then(|r| r.try_get("email").ok().flatten());
            let final_email = email_opt.or(found_email).unwrap_or_else(|| did.clone());
            (Some(did), final_email)
        }
        (None, Some(em)) => {
            let row = sqlx::query(
                "SELECT did_peer FROM user_identities WHERE LOWER(email) = LOWER($1) LIMIT 1",
            )
            .bind(&em)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
            let found_did: Option<String> = row.and_then(|r| r.try_get("did_peer").ok());
            (found_did, em)
        }
        (None, None) => return Err("Provide a did:peer ID or email address.".to_string()),
    };

    let admin = get_admin_email();
    let final_status = if resolved_email == admin {
        "admin".to_string()
    } else if clean_status == "admin" {
        "moderator".to_string()
    } else {
        clean_status
    };
    let final_role = match final_status.as_str() {
        "admin" => "admin",
        "moderator" => "moderator",
        _ => "reader",
    };
    let final_enabled = ["admin", "moderator", "reader"].contains(&final_status.as_str());

    let existing_entry = sqlx::query(
        r#"
        SELECT email, role
        FROM docs_access_allowlist
        WHERE ($1::text IS NOT NULL AND did_peer = $1)
           OR LOWER(email) = LOWER($2)
        LIMIT 1
        "#,
    )
    .bind(resolved_did.as_deref())
    .bind(&resolved_email)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = existing_entry {
        let existing_email: String = row.try_get("email").map_err(|e| e.to_string())?;
        let existing_role: String = row.try_get("role").unwrap_or_else(|_| final_role.to_string());
        let is_primary = existing_role == "admin" || resolved_email == admin;
        let saved_status = if is_primary { "admin" } else { final_status.as_str() };
        let saved_role = if is_primary { "admin" } else { final_role };
        let saved_enabled = if is_primary { true } else { final_enabled };

        sqlx::query(
            r#"
            UPDATE docs_access_allowlist
            SET did_peer = COALESCE($2, did_peer),
                status = $3,
                enabled = $4,
                role = $5,
                note = COALESCE(NULLIF($6, ''), note),
                updated_at = NOW()
            WHERE email = $1
            "#,
        )
        .bind(&existing_email)
        .bind(resolved_did.as_deref())
        .bind(saved_status)
        .bind(saved_enabled)
        .bind(saved_role)
        .bind(note.trim())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO docs_access_allowlist (email, did_peer, status, enabled, role, note, added_by, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            "#,
        )
        .bind(&resolved_email)
        .bind(resolved_did.as_deref())
        .bind(&final_status)
        .bind(final_enabled)
        .bind(final_role)
        .bind(note.trim())
        .bind(added_by)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    let entries = list_allowlist(pool).await?;
    entries
        .into_iter()
        .find(|e| (resolved_did.is_some() && e.did_peer == resolved_did) || e.email == resolved_email)
        .ok_or_else(|| "Allowlist entry saved but could not be reloaded.".to_string())
}

pub async fn delete_allowlist_entry(
    pool: &PgPool,
    did_peer: Option<&str>,
    email: Option<&str>,
) -> Result<bool, String> {
    let admin = get_admin_email();
    if let Some(em) = email {
        if em.trim().eq_ignore_ascii_case(&admin) {
            return Err("Cannot remove primary admin account.".to_string());
        }
    }
    if let Some(did) = did_peer {
        let admin_row = sqlx::query(
            "SELECT email FROM docs_access_allowlist WHERE did_peer = $1 LIMIT 1",
        )
        .bind(did.trim())
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        if let Some(r) = admin_row {
            let em: String = r.try_get("email").unwrap_or_default();
            if em.eq_ignore_ascii_case(&admin) {
                return Err("Cannot remove primary admin account.".to_string());
            }
        }
    }

    let res = if let Some(did) = did_peer.map(str::trim).filter(|s| !s.is_empty()) {
        sqlx::query("DELETE FROM docs_access_allowlist WHERE did_peer = $1")
            .bind(did)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?
    } else if let Some(em) = email.map(str::trim).filter(|s| !s.is_empty()) {
        sqlx::query("DELETE FROM docs_access_allowlist WHERE LOWER(email) = LOWER($1)")
            .bind(em)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        return Err("Missing did_peer or email parameter.".to_string());
    };
    Ok(res.rows_affected() > 0)
}

pub async fn create_group(
    pool: &PgPool,
    owner_did: &str,
    persona_did: &str,
    name: &str,
    description: &str,
) -> Result<GroupView, String> {
    let clean_name = name.trim();
    if clean_name.is_empty() {
        return Err("Group name is required.".to_string());
    }
    let group_id = random_token("grp", 12);
    let ts_us = now_timestamp_us();
    let willow_path = format!("/groups/{group_id}");

    sqlx::query(
        r#"
        INSERT INTO groups (
            group_id, owner_did, created_by_persona_did, name, description,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $2, $3, $4, $5, 'exonovel', $2, $6, $7)
        "#,
    )
    .bind(&group_id)
    .bind(owner_did)
    .bind(persona_did)
    .bind(clean_name)
    .bind(description.trim())
    .bind(&willow_path)
    .bind(ts_us)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create group: {e}"))?;

    let member_path = format!("/groups/{group_id}/members/{owner_did}");
    sqlx::query(
        r#"
        INSERT INTO group_members (
            group_id, member_owner_did, member_persona_did, role, invited_by_did,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $2, $3, 'manager', $2, 'exonovel', $2, $4, $5)
        "#,
    )
    .bind(&group_id)
    .bind(owner_did)
    .bind(persona_did)
    .bind(&member_path)
    .bind(ts_us)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to add creator as group manager: {e}"))?;

    let groups = list_groups_for_user(pool, owner_did).await?;
    groups
        .into_iter()
        .find(|g| g.group_id == group_id)
        .ok_or_else(|| "Group created but could not be reloaded.".to_string())
}

pub async fn list_groups_for_user(pool: &PgPool, owner_did: &str) -> Result<Vec<GroupView>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            g.group_id, g.owner_did, g.created_by_persona_did, g.name, g.description,
            g.willow_path, g.willow_timestamp_us,
            to_char(g.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            COALESCE(my_m.role, CASE WHEN g.owner_did = $1 THEN 'manager' ELSE 'member' END) AS my_role
        FROM groups g
        LEFT JOIN group_members my_m
            ON my_m.group_id = g.group_id AND my_m.member_owner_did = $1
        WHERE g.owner_did = $1 OR my_m.member_owner_did IS NOT NULL
        ORDER BY g.created_at DESC
        "#,
    )
    .bind(owner_did)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let group_id: String = r.try_get("group_id").map_err(|e| e.to_string())?;
        let member_rows = sqlx::query(
            r#"
            SELECT
                gm.member_owner_did,
                gm.member_persona_did,
                COALESCE(NULLIF(up_p.display_name, ''), NULLIF(up.display_name, ''), gm.member_owner_did) AS display_name,
                COALESCE(NULLIF(up_p.avatar_thumb_url, ''), COALESCE(up.avatar_thumb_url, '')) AS avatar_thumb_url,
                gm.role,
                to_char(gm.joined_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS joined_at
            FROM group_members gm
            LEFT JOIN user_personas up_p ON up_p.persona_did = gm.member_persona_did
            LEFT JOIN user_profiles up ON up.did_peer = gm.member_owner_did
            WHERE gm.group_id = $1
            ORDER BY CASE WHEN gm.role = 'manager' THEN 0 ELSE 1 END, gm.joined_at ASC
            "#,
        )
        .bind(&group_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let members = member_rows
            .into_iter()
            .map(|mr| GroupMemberView {
                member_owner_did: mr.try_get("member_owner_did").unwrap_or_default(),
                member_persona_did: mr.try_get("member_persona_did").unwrap_or_default(),
                display_name: mr.try_get("display_name").unwrap_or_default(),
                avatar_thumb_url: mr.try_get("avatar_thumb_url").unwrap_or_default(),
                role: mr.try_get("role").unwrap_or_else(|_| "member".to_string()),
                joined_at: mr.try_get("joined_at").unwrap_or_default(),
            })
            .collect();

        out.push(GroupView {
            group_id,
            owner_did: r.try_get("owner_did").unwrap_or_default(),
            created_by_persona_did: r.try_get("created_by_persona_did").unwrap_or_default(),
            name: r.try_get("name").unwrap_or_default(),
            description: r.try_get("description").unwrap_or_default(),
            my_role: r.try_get("my_role").unwrap_or_else(|_| "member".to_string()),
            willow_path: r.try_get("willow_path").unwrap_or_default(),
            willow_timestamp_us: r.try_get("willow_timestamp_us").unwrap_or(0),
            created_at: r.try_get("created_at").unwrap_or_default(),
            members,
        });
    }

    Ok(out)
}

pub async fn set_group_member_role(
    pool: &PgPool,
    actor_owner_did: &str,
    group_id: &str,
    target_member_did: &str,
    new_role: &str,
) -> Result<(), String> {
    let clean_role = new_role.trim();
    if !["manager", "member"].contains(&clean_role) {
        return Err("Group role must be manager or member.".to_string());
    }

    let auth_row = sqlx::query(
        r#"
        SELECT g.owner_did, gm.role AS actor_role
        FROM groups g
        LEFT JOIN group_members gm
            ON gm.group_id = g.group_id AND gm.member_owner_did = $2
        WHERE g.group_id = $1
        LIMIT 1
        "#,
    )
    .bind(group_id)
    .bind(actor_owner_did)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let Some(r) = auth_row else {
        return Err("Group not found.".to_string());
    };
    let group_owner: String = r.try_get("owner_did").unwrap_or_default();
    let actor_role: Option<String> = r.try_get("actor_role").unwrap_or(None);
    if group_owner != actor_owner_did && actor_role.as_deref() != Some("manager") {
        return Err("Only a group manager can assign group roles.".to_string());
    }

    if target_member_did == group_owner && clean_role != "manager" {
        return Err("The group creator remains a manager.".to_string());
    }

    sqlx::query(
        "UPDATE group_members SET role = $3, willow_timestamp_us = $4 WHERE group_id = $1 AND member_owner_did = $2",
    )
    .bind(group_id)
    .bind(target_member_did)
    .bind(clean_role)
    .bind(now_timestamp_us())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn create_invite(
    pool: &PgPool,
    inviter_owner_did: &str,
    inviter_persona_did: &str,
    group_id: Option<&str>,
    target_group_role: &str,
    note: &str,
) -> Result<InviteView, String> {
    let clean_group_id = group_id.map(str::trim).filter(|s| !s.is_empty());
    let clean_role = if target_group_role.trim() == "manager" {
        "manager"
    } else {
        "member"
    };

    if let Some(gid) = clean_group_id {
        let check = sqlx::query(
            r#"
            SELECT g.owner_did, gm.role
            FROM groups g
            LEFT JOIN group_members gm ON gm.group_id = g.group_id AND gm.member_owner_did = $2
            WHERE g.group_id = $1
            LIMIT 1
            "#,
        )
        .bind(gid)
        .bind(inviter_owner_did)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        let Some(r) = check else {
            return Err("Group not found.".to_string());
        };
        let owner: String = r.try_get("owner_did").unwrap_or_default();
        let member_role: Option<String> = r.try_get("role").unwrap_or(None);
        if owner != inviter_owner_did && member_role.is_none() {
            return Err("You must belong to a group to create invites for it.".to_string());
        }
    }

    let code = random_token("inv", 14);
    let ts_us = now_timestamp_us();
    let willow_path = format!("/invites/{code}");

    sqlx::query(
        r#"
        INSERT INTO invites (
            code, inviter_owner_did, inviter_persona_did, group_id,
            target_group_role, note, willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'exonovel', $2, $7, $8)
        "#,
    )
    .bind(&code)
    .bind(inviter_owner_did)
    .bind(inviter_persona_did)
    .bind(clean_group_id)
    .bind(clean_role)
    .bind(note.trim())
    .bind(&willow_path)
    .bind(ts_us)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create invite: {e}"))?;

    let (invites, _, _) = list_invites_and_referrals(pool, inviter_owner_did).await?;
    invites
        .into_iter()
        .find(|i| i.code == code)
        .ok_or_else(|| "Invite created but could not be reloaded.".to_string())
}

pub async fn preview_invite(pool: &PgPool, code: &str) -> Result<InvitePreview, String> {
    let clean = code.trim();
    if clean.is_empty() {
        return Err("Missing invite code.".to_string());
    }
    let row = sqlx::query(
        r#"
        SELECT
            i.code,
            i.active,
            COALESCE(NULLIF(p.display_name, ''), NULLIF(up.display_name, ''), 'An ExoNovel Writer') AS inviter_display_name,
            COALESCE(NULLIF(p.avatar_thumb_url, ''), COALESCE(up.avatar_thumb_url, '')) AS inviter_avatar_thumb_url,
            g.name AS group_name
        FROM invites i
        LEFT JOIN user_personas p ON p.persona_did = i.inviter_persona_did
        LEFT JOIN user_profiles up ON up.did_peer = i.inviter_owner_did
        LEFT JOIN groups g ON g.group_id = i.group_id
        WHERE i.code = $1
        LIMIT 1
        "#,
    )
    .bind(clean)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => Ok(InvitePreview {
            code: r.try_get("code").unwrap_or_default(),
            valid: r.try_get("active").unwrap_or(false),
            inviter_display_name: r.try_get("inviter_display_name").unwrap_or_default(),
            inviter_avatar_thumb_url: r.try_get("inviter_avatar_thumb_url").unwrap_or_default(),
            group_name: r.try_get("group_name").unwrap_or(None),
        }),
        None => Ok(InvitePreview {
            code: clean.to_string(),
            valid: false,
            inviter_display_name: String::new(),
            inviter_avatar_thumb_url: String::new(),
            group_name: None,
        }),
    }
}

pub async fn claim_invite(
    pool: &PgPool,
    claimer_owner_did: &str,
    claimer_persona_did: &str,
    code: &str,
) -> Result<bool, String> {
    let clean = code.trim();
    if clean.is_empty() {
        return Ok(false);
    }

    let inv_row = sqlx::query(
        r#"
        SELECT inviter_owner_did, inviter_persona_did, group_id, target_group_role, active
        FROM invites
        WHERE code = $1
        LIMIT 1
        "#,
    )
    .bind(clean)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let Some(r) = inv_row else {
        return Err("Invite link not found.".to_string());
    };
    let active: bool = r.try_get("active").unwrap_or(false);
    if !active {
        return Err("This invite link is no longer active.".to_string());
    }

    let inviter_owner_did: String = r.try_get("inviter_owner_did").map_err(|e| e.to_string())?;
    let inviter_persona_did: String = r.try_get("inviter_persona_did").unwrap_or_default();
    let group_id: Option<String> = r.try_get("group_id").unwrap_or(None);
    let target_role: String = r.try_get("target_group_role").unwrap_or_else(|_| "member".to_string());

    if inviter_owner_did == claimer_owner_did {
        return Ok(false);
    }

    let ts_us = now_timestamp_us();
    let ref_path = format!("/referrals/{clean}/{claimer_owner_did}");
    let inserted = sqlx::query(
        r#"
        INSERT INTO referrals (
            referrer_owner_did, referrer_persona_did,
            referred_owner_did, referred_persona_did,
            invite_code, group_id,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'exonovel', $1, $7, $8)
        ON CONFLICT (invite_code, referred_owner_did) DO NOTHING
        "#,
    )
    .bind(&inviter_owner_did)
    .bind(&inviter_persona_did)
    .bind(claimer_owner_did)
    .bind(claimer_persona_did)
    .bind(clean)
    .bind(group_id.as_deref())
    .bind(&ref_path)
    .bind(ts_us)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    if inserted.rows_affected() > 0 {
        let _ = sqlx::query("UPDATE invites SET use_count = use_count + 1 WHERE code = $1")
            .bind(clean)
            .execute(pool)
            .await;
    }

    if let Some(gid) = group_id {
        let member_path = format!("/groups/{gid}/members/{claimer_owner_did}");
        let role_to_grant = if target_role == "manager" { "manager" } else { "member" };
        let _ = sqlx::query(
            r#"
            INSERT INTO group_members (
                group_id, member_owner_did, member_persona_did, role, invited_by_did,
                willow_namespace, willow_subspace, willow_path, willow_timestamp_us
            )
            VALUES ($1, $2, $3, $4, $5, 'exonovel', $2, $6, $7)
            ON CONFLICT (group_id, member_owner_did) DO NOTHING
            "#,
        )
        .bind(&gid)
        .bind(claimer_owner_did)
        .bind(claimer_persona_did)
        .bind(role_to_grant)
        .bind(&inviter_owner_did)
        .bind(&member_path)
        .bind(ts_us)
        .execute(pool)
        .await;
    }

    Ok(true)
}

pub async fn list_invites_and_referrals(
    pool: &PgPool,
    owner_did: &str,
) -> Result<(Vec<InviteView>, Vec<ReferralView>, Option<String>), String> {
    let inv_rows = sqlx::query(
        r#"
        SELECT
            i.code, i.group_id, g.name AS group_name,
            i.target_group_role, i.note, i.use_count, i.active,
            to_char(i.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM invites i
        LEFT JOIN groups g ON g.group_id = i.group_id
        WHERE i.inviter_owner_did = $1
        ORDER BY i.created_at DESC
        "#,
    )
    .bind(owner_did)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let invites = inv_rows
        .into_iter()
        .map(|r| InviteView {
            code: r.try_get("code").unwrap_or_default(),
            group_id: r.try_get("group_id").unwrap_or(None),
            group_name: r.try_get("group_name").unwrap_or(None),
            target_group_role: r.try_get("target_group_role").unwrap_or_else(|_| "member".to_string()),
            note: r.try_get("note").unwrap_or_default(),
            use_count: r.try_get("use_count").unwrap_or(0),
            active: r.try_get("active").unwrap_or(true),
            created_at: r.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let ref_rows = sqlx::query(
        r#"
        SELECT
            rf.referred_owner_did,
            rf.referred_persona_did,
            COALESCE(NULLIF(p.display_name, ''), NULLIF(up.display_name, ''), rf.referred_owner_did) AS display_name,
            COALESCE(NULLIF(p.avatar_thumb_url, ''), COALESCE(up.avatar_thumb_url, '')) AS avatar_thumb_url,
            rf.invite_code,
            g.name AS group_name,
            to_char(rf.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM referrals rf
        LEFT JOIN user_personas p ON p.persona_did = rf.referred_persona_did
        LEFT JOIN user_profiles up ON up.did_peer = rf.referred_owner_did
        LEFT JOIN groups g ON g.group_id = rf.group_id
        WHERE rf.referrer_owner_did = $1
        ORDER BY rf.created_at DESC
        "#,
    )
    .bind(owner_did)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let referrals = ref_rows
        .into_iter()
        .map(|r| ReferralView {
            referred_owner_did: r.try_get("referred_owner_did").unwrap_or_default(),
            referred_persona_did: r.try_get("referred_persona_did").unwrap_or_default(),
            display_name: r.try_get("display_name").unwrap_or_default(),
            avatar_thumb_url: r.try_get("avatar_thumb_url").unwrap_or_default(),
            invite_code: r.try_get("invite_code").unwrap_or_default(),
            group_name: r.try_get("group_name").unwrap_or(None),
            created_at: r.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let referred_by_row = sqlx::query(
        r#"
        SELECT COALESCE(NULLIF(p.display_name, ''), NULLIF(up.display_name, ''), rf.referrer_owner_did) AS ref_name
        FROM referrals rf
        LEFT JOIN user_personas p ON p.persona_did = rf.referrer_persona_did
        LEFT JOIN user_profiles up ON up.did_peer = rf.referrer_owner_did
        WHERE rf.referred_owner_did = $1
        ORDER BY rf.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(owner_did)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let referred_by = referred_by_row.and_then(|r| r.try_get("ref_name").ok());

    Ok((invites, referrals, referred_by))
}

#[cfg(test)]
mod tests {
    use super::mint_did_peer_keypair;

    #[test]
    fn mints_valid_did_peer_zero() {
        let (did, pub_b58, sealed) = mint_did_peer_keypair().expect("mint should succeed");
        assert!(did.starts_with("did:peer:0z"));
        assert!(pub_b58.starts_with('z'));
        assert_eq!(sealed.split('.').count(), 3);
    }
}
