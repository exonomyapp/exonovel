use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{
    pkcs8::{spki::der::pem::LineEnding, EncodePrivateKey},
    SigningKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityRecord {
    pub did_peer: String,
    pub email: Option<String>,
    pub auth_user_id: String,
    pub provider: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowlistEntry {
    pub email: String,
    pub enabled: bool,
    pub role: String,
    pub note: String,
    pub added_by: String,
    pub updated_at: String,
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
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS docs_access_allowlist (
            email TEXT PRIMARY KEY,
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

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS docs_storage_snapshots (
            id SERIAL PRIMARY KEY,
            doc_path TEXT NOT NULL,
            object_key TEXT NOT NULL UNIQUE,
            size_bytes INTEGER NOT NULL DEFAULT 0,
            uploaded_by_did TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    let admin_email = get_admin_email();
    sqlx::query(
        r#"
        INSERT INTO docs_access_allowlist (email, enabled, role, note, added_by)
        VALUES ($1, TRUE, 'admin', 'Primary ExoNovel documentation owner', 'system')
        ON CONFLICT (email) DO UPDATE
        SET enabled = TRUE, role = 'admin', updated_at = NOW();
        "#,
    )
    .bind(admin_email)
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

pub async fn resolve_or_mint_did_peer(
    pool: &PgPool,
    auth_user_id: &str,
    email: Option<&str>,
    provider: &str,
) -> Result<IdentityRecord, String> {
    if let Some(row) = sqlx::query(
        r#"
        SELECT did_peer, email, auth_user_id, provider,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM user_identities
        WHERE auth_user_id = $1
        LIMIT 1
        "#,
    )
    .bind(auth_user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("DB lookup failed: {e}"))?
    {
        let existing_email: Option<String> = row.try_get("email").unwrap_or(None);
        if let Some(new_email) = email {
            if existing_email.as_deref() != Some(new_email) {
                let _ = sqlx::query(
                    "UPDATE user_identities SET email = $1, updated_at = NOW() WHERE auth_user_id = $2",
                )
                .bind(new_email)
                .bind(auth_user_id)
                .execute(pool)
                .await;
            }
        }
        return Ok(IdentityRecord {
            did_peer: row.try_get("did_peer").map_err(|e| e.to_string())?,
            email: email.map(str::to_string).or(existing_email),
            auth_user_id: row.try_get("auth_user_id").map_err(|e| e.to_string())?,
            provider: row.try_get("provider").map_err(|e| e.to_string())?,
            created_at: row.try_get("created_at").map_err(|e| e.to_string())?,
        });
    }

    let (did_peer, pub_b58, encrypted_priv) = mint_did_peer_keypair()?;
    let row = sqlx::query(
        r#"
        INSERT INTO user_identities (did_peer, public_key, encrypted_private_key, auth_user_id, email, provider)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (auth_user_id) DO UPDATE
        SET email = COALESCE(EXCLUDED.email, user_identities.email), updated_at = NOW()
        RETURNING did_peer, email, auth_user_id, provider,
                  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        "#,
    )
    .bind(did_peer)
    .bind(pub_b58)
    .bind(encrypted_priv)
    .bind(auth_user_id)
    .bind(email)
    .bind(provider)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("DB insert failed: {e}"))?;

    Ok(IdentityRecord {
        did_peer: row.try_get("did_peer").map_err(|e| e.to_string())?,
        email: row.try_get("email").unwrap_or(None),
        auth_user_id: row.try_get("auth_user_id").map_err(|e| e.to_string())?,
        provider: row.try_get("provider").map_err(|e| e.to_string())?,
        created_at: row.try_get("created_at").map_err(|e| e.to_string())?,
    })
}

pub async fn is_email_authorized_for_docs(
    pool: Option<&PgPool>,
    email: Option<&str>,
) -> (bool, bool) {
    let Some(raw_email) = email else {
        return (false, false);
    };
    let normalized = raw_email.trim().to_lowercase();
    if normalized == get_admin_email() {
        return (true, true);
    }

    let Some(db) = pool else {
        return (false, false);
    };

    let row = sqlx::query(
        "SELECT enabled, role FROM docs_access_allowlist WHERE email = $1 LIMIT 1",
    )
    .bind(normalized)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    match row {
        Some(r) => {
            let enabled: bool = r.try_get("enabled").unwrap_or(false);
            let role: String = r.try_get("role").unwrap_or_default();
            (enabled, enabled && role == "admin")
        }
        None => (false, false),
    }
}

pub async fn list_identities(pool: &PgPool) -> Result<Vec<IdentityRecord>, String> {
    let rows = sqlx::query(
        r#"
        SELECT did_peer, email, auth_user_id, provider,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM user_identities
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|r| {
            Ok(IdentityRecord {
                did_peer: r.try_get("did_peer").map_err(|e| e.to_string())?,
                email: r.try_get("email").unwrap_or(None),
                auth_user_id: r.try_get("auth_user_id").map_err(|e| e.to_string())?,
                provider: r.try_get("provider").map_err(|e| e.to_string())?,
                created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
            })
        })
        .collect()
}

pub async fn list_allowlist(pool: &PgPool) -> Result<Vec<AllowlistEntry>, String> {
    let rows = sqlx::query(
        r#"
        SELECT email, enabled, role, note, added_by,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        FROM docs_access_allowlist
        ORDER BY CASE WHEN role = 'admin' THEN 0 ELSE 1 END, email ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|r| {
            Ok(AllowlistEntry {
                email: r.try_get("email").map_err(|e| e.to_string())?,
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
    email: &str,
    enabled: bool,
    note: &str,
    added_by: &str,
) -> Result<AllowlistEntry, String> {
    let normalized = email.trim().to_lowercase();
    let admin = get_admin_email();
    let role = if normalized == admin { "admin" } else { "reader" };
    let final_enabled = if normalized == admin { true } else { enabled };

    let row = sqlx::query(
        r#"
        INSERT INTO docs_access_allowlist (email, enabled, role, note, added_by, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (email) DO UPDATE
        SET enabled = $2,
            note = COALESCE(NULLIF($4, ''), docs_access_allowlist.note),
            updated_at = NOW()
        RETURNING email, enabled, role, note, added_by,
                  to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        "#,
    )
    .bind(normalized)
    .bind(final_enabled)
    .bind(role)
    .bind(note.trim())
    .bind(added_by)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(AllowlistEntry {
        email: row.try_get("email").map_err(|e| e.to_string())?,
        enabled: row.try_get("enabled").map_err(|e| e.to_string())?,
        role: row.try_get("role").map_err(|e| e.to_string())?,
        note: row.try_get("note").map_err(|e| e.to_string())?,
        added_by: row.try_get("added_by").map_err(|e| e.to_string())?,
        updated_at: row.try_get("updated_at").map_err(|e| e.to_string())?,
    })
}

pub async fn delete_allowlist_entry(pool: &PgPool, email: &str) -> Result<bool, String> {
    let normalized = email.trim().to_lowercase();
    if normalized == get_admin_email() {
        return Err("Cannot remove primary admin account.".to_string());
    }
    let res = sqlx::query("DELETE FROM docs_access_allowlist WHERE email = $1")
        .bind(normalized)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.rows_affected() > 0)
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
