use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::{
    auth::AuthUser,
    db::now_timestamp_us,
    live::{emit_live_patch, LiveBus},
    AppState,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferences {
    pub did_peer: String,
    pub in_app_enabled: bool,
    pub web_push_enabled: bool,
    pub native_push_enabled: bool,
    pub email_enabled: bool,
    pub email_digest_mode: String,
    pub notify_author_earnings: bool,
    pub notify_reader_purchases: bool,
    pub notify_group_invites: bool,
    pub notify_referral_signups: bool,
    pub notify_moderator_comments: bool,
    pub notify_collaborator_comments: bool,
    pub notify_mentions_and_forks: bool,
    pub notify_security_signins: bool,
    pub notify_maturity_releases: bool,
    pub notify_maturity_unblocked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDevice {
    pub device_id: String,
    pub platform: String,
    pub transport: String,
    pub endpoint_or_token: String,
    pub device_label: String,
    pub last_seen_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    pub id: String,
    pub recipient_did: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub action_url: String,
    pub is_read: bool,
    pub email_sent: bool,
    pub willow_namespace: String,
    pub willow_subspace: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingLedgerItem {
    pub id: String,
    pub kind: String,
    pub currency: String,
    pub amount_cents: i64,
    pub billing_cycle: String,
    pub status: String,
    pub counterparty_did: String,
    pub note: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
    pub created_at: String,
}

pub async fn ensure_notify_and_billing_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notification_preferences (
            did_peer TEXT PRIMARY KEY REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            in_app_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            web_push_enabled BOOLEAN NOT NULL DEFAULT FALSE,
            native_push_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            email_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            email_digest_mode TEXT NOT NULL DEFAULT 'instant',
            notify_author_earnings BOOLEAN NOT NULL DEFAULT TRUE,
            notify_reader_purchases BOOLEAN NOT NULL DEFAULT TRUE,
            notify_group_invites BOOLEAN NOT NULL DEFAULT TRUE,
            notify_referral_signups BOOLEAN NOT NULL DEFAULT TRUE,
            notify_moderator_comments BOOLEAN NOT NULL DEFAULT TRUE,
            notify_collaborator_comments BOOLEAN NOT NULL DEFAULT TRUE,
            notify_mentions_and_forks BOOLEAN NOT NULL DEFAULT TRUE,
            notify_security_signins BOOLEAN NOT NULL DEFAULT TRUE,
            notify_maturity_releases BOOLEAN NOT NULL DEFAULT TRUE,
            notify_maturity_unblocked BOOLEAN NOT NULL DEFAULT TRUE,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notification_devices (
            device_id TEXT PRIMARY KEY,
            did_peer TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            platform TEXT NOT NULL,
            transport TEXT NOT NULL,
            endpoint_or_token TEXT NOT NULL,
            p256dh TEXT NOT NULL DEFAULT '',
            auth_secret TEXT NOT NULL DEFAULT '',
            device_label TEXT NOT NULL DEFAULT '',
            last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notifications (
            id TEXT PRIMARY KEY,
            recipient_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            kind TEXT NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            action_url TEXT NOT NULL DEFAULT '/',
            is_read BOOLEAN NOT NULL DEFAULT FALSE,
            email_sent BOOLEAN NOT NULL DEFAULT FALSE,
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
        "CREATE INDEX IF NOT EXISTS idx_notifications_recipient ON notifications(recipient_did, created_at DESC);",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS billing_ledger (
            id TEXT PRIMARY KEY,
            owner_did TEXT NOT NULL REFERENCES user_identities(did_peer) ON DELETE CASCADE,
            kind TEXT NOT NULL,
            currency TEXT NOT NULL DEFAULT 'USD',
            amount_cents BIGINT NOT NULL DEFAULT 0,
            billing_cycle TEXT NOT NULL DEFAULT 'one-time',
            status TEXT NOT NULL DEFAULT 'active',
            counterparty_did TEXT NOT NULL DEFAULT '',
            note TEXT NOT NULL DEFAULT '',
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

    Ok(())
}

pub async fn get_or_init_preferences(
    pool: &PgPool,
    did_peer: &str,
) -> Result<NotificationPreferences, String> {
    let _ = sqlx::query(
        "INSERT INTO notification_preferences (did_peer) VALUES ($1) ON CONFLICT (did_peer) DO NOTHING",
    )
    .bind(did_peer)
    .execute(pool)
    .await;

    let r = sqlx::query(
        r#"
        SELECT
            did_peer, in_app_enabled, web_push_enabled, native_push_enabled,
            email_enabled, email_digest_mode,
            notify_author_earnings, notify_reader_purchases,
            notify_group_invites, notify_referral_signups,
            notify_moderator_comments, notify_collaborator_comments,
            notify_mentions_and_forks, notify_security_signins,
            notify_maturity_releases, notify_maturity_unblocked
        FROM notification_preferences
        WHERE did_peer = $1
        LIMIT 1
        "#,
    )
    .bind(did_peer)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(NotificationPreferences {
        did_peer: r.try_get("did_peer").unwrap_or_default(),
        in_app_enabled: r.try_get("in_app_enabled").unwrap_or(true),
        web_push_enabled: r.try_get("web_push_enabled").unwrap_or(false),
        native_push_enabled: r.try_get("native_push_enabled").unwrap_or(true),
        email_enabled: r.try_get("email_enabled").unwrap_or(true),
        email_digest_mode: r
            .try_get("email_digest_mode")
            .unwrap_or_else(|_| "instant".to_string()),
        notify_author_earnings: r.try_get("notify_author_earnings").unwrap_or(true),
        notify_reader_purchases: r.try_get("notify_reader_purchases").unwrap_or(true),
        notify_group_invites: r.try_get("notify_group_invites").unwrap_or(true),
        notify_referral_signups: r.try_get("notify_referral_signups").unwrap_or(true),
        notify_moderator_comments: r.try_get("notify_moderator_comments").unwrap_or(true),
        notify_collaborator_comments: r.try_get("notify_collaborator_comments").unwrap_or(true),
        notify_mentions_and_forks: r.try_get("notify_mentions_and_forks").unwrap_or(true),
        notify_security_signins: r.try_get("notify_security_signins").unwrap_or(true),
        notify_maturity_releases: r.try_get("notify_maturity_releases").unwrap_or(true),
        notify_maturity_unblocked: r.try_get("notify_maturity_unblocked").unwrap_or(true),
    })
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrevoSenderRole {
    /// Notification-domain emails ("Notes from ExoNovel" <notes@exosystems.net>)
    Notes,
    /// General automated emails outside notifications ("Exosystems" <noreply@exosystems.net>)
    Noreply,
    /// User support issue threads ("ExoSupport" <support@exosystems.net>)
    Support,
}

impl BrevoSenderRole {
    pub fn resolve(self) -> (String, String) {
        let env_or = |key: &str| {
            std::env::var(key)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };
        match self {
            BrevoSenderRole::Notes => (
                env_or("BREVO_SENDER_NOTES_EMAIL")
                    .or_else(|| env_or("BREVO_SENDER_EMAIL"))
                    .unwrap_or_else(|| "notes@exosystems.net".to_string()),
                env_or("BREVO_SENDER_NOTES_NAME")
                    .or_else(|| env_or("BREVO_SENDER_NAME"))
                    .unwrap_or_else(|| "Notes from ExoNovel".to_string()),
            ),
            BrevoSenderRole::Noreply => (
                env_or("BREVO_SENDER_NOREPLY_EMAIL")
                    .unwrap_or_else(|| "noreply@exosystems.net".to_string()),
                env_or("BREVO_SENDER_NOREPLY_NAME")
                    .unwrap_or_else(|| "Exosystems".to_string()),
            ),
            BrevoSenderRole::Support => (
                env_or("BREVO_SENDER_SUPPORT_EMAIL")
                    .unwrap_or_else(|| "support@exosystems.net".to_string()),
                env_or("BREVO_SENDER_SUPPORT_NAME")
                    .unwrap_or_else(|| "ExoSupport".to_string()),
            ),
        }
    }
}

pub async fn send_brevo_email(
    to_email: &str,
    to_name: &str,
    subject: &str,
    html_content: &str,
) -> Result<bool, String> {
    send_brevo_email_as(BrevoSenderRole::Notes, to_email, to_name, subject, html_content).await
}

pub async fn send_brevo_email_as(
    role: BrevoSenderRole,
    to_email: &str,
    to_name: &str,
    subject: &str,
    html_content: &str,
) -> Result<bool, String> {
    let api_key = std::env::var("BREVO_API_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let Some(key) = api_key else {
        return Ok(false);
    };

    let (sender_email, sender_name) = role.resolve();

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "sender": {
            "name": sender_name,
            "email": sender_email
        },
        "to": [{
            "email": to_email,
            "name": if to_name.trim().is_empty() { "ExoNovel Member" } else { to_name.trim() }
        }],
        "subject": subject,
        "htmlContent": html_content
    });

    let resp = client
        .post("https://api.brevo.com/v3/smtp/email")
        .header("api-key", key)
        .header("accept", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Email delivery request failed: {e}"))?;

    if !resp.status().is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!("Email delivery error: {txt}"));
    }
    Ok(true)
}

pub async fn dispatch_notification(
    pool: &PgPool,
    bus: &LiveBus,
    recipient_did: &str,
    kind: &str,
    title: &str,
    body: &str,
    action_url: &str,
) -> Result<NotificationItem, String> {
    let prefs = get_or_init_preferences(pool, recipient_did).await?;
    let id = format!("ntf_{}", now_timestamp_us());
    let ts_us = now_timestamp_us();
    let willow_path = format!("/notifications/{id}");

    let mut email_sent = false;
    let should_email = prefs.email_enabled && prefs.email_digest_mode == "instant";
    if should_email {
        if let Ok(Some(row)) = sqlx::query(
            r#"
            SELECT ui.email, COALESCE(up.display_name, '') AS display_name
            FROM user_identities ui
            LEFT JOIN user_profiles up ON up.did_peer = ui.did_peer
            WHERE ui.did_peer = $1
            LIMIT 1
            "#,
        )
        .bind(recipient_did)
        .fetch_optional(pool)
        .await
        {
            let email: Option<String> = row.try_get("email").unwrap_or(None);
            let name: String = row.try_get("display_name").unwrap_or_default();
            if let Some(to_addr) = email.filter(|s| s.contains('@')) {
                let html = format!(
                    "<div style=\"font-family:sans-serif;max-width:560px;margin:0 auto;padding:20px;\">\
                     <h2 style=\"margin:0 0 12px;\">{}</h2>\
                     <p style=\"line-height:1.6;color:#222;\">{}</p>\
                     <p style=\"margin-top:20px;font-size:13px;color:#666;\">ExoNovel · ExoSystems</p>\
                     </div>",
                    title.replace('<', "&lt;").replace('>', "&gt;"),
                    body.replace('<', "&lt;").replace('>', "&gt;"),
                );
                if let Ok(sent) = send_brevo_email(&to_addr, &name, title, &html).await {
                    email_sent = sent;
                }
            }
        }
    }

    let row = sqlx::query(
        r#"
        INSERT INTO notifications (
            id, recipient_did, kind, title, body, action_url,
            is_read, email_sent, willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $2, $3, $4, $5, $6, FALSE, $7, 'exonovel', $2, $8, $9)
        RETURNING
            id, recipient_did, kind, title, body, action_url, is_read, email_sent,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        "#,
    )
    .bind(&id)
    .bind(recipient_did)
    .bind(kind)
    .bind(title)
    .bind(body)
    .bind(action_url)
    .bind(email_sent)
    .bind(&willow_path)
    .bind(ts_us)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let item = NotificationItem {
        id: row.try_get("id").unwrap_or_default(),
        recipient_did: row.try_get("recipient_did").unwrap_or_default(),
        kind: row.try_get("kind").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        body: row.try_get("body").unwrap_or_default(),
        action_url: row.try_get("action_url").unwrap_or_default(),
        is_read: row.try_get("is_read").unwrap_or(false),
        email_sent: row.try_get("email_sent").unwrap_or(false),
        willow_namespace: row
            .try_get("willow_namespace")
            .unwrap_or_else(|_| "exonovel".to_string()),
        willow_subspace: row.try_get("willow_subspace").unwrap_or_default(),
        willow_path: row.try_get("willow_path").unwrap_or_default(),
        willow_timestamp_us: row.try_get("willow_timestamp_us").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or_default(),
    };

    emit_live_patch(
        Some(pool),
        bus,
        format!("user:{recipient_did}"),
        "notification",
        &item.id,
        "insert",
        vec!["id".to_string(), "title".to_string(), "isRead".to_string()],
        serde_json::json!(item),
    )
    .await;

    Ok(item)
}

pub async fn api_list_notifications(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let Some(did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };

    let rows = sqlx::query(
        r#"
        SELECT
            id, recipient_did, kind, title, body, action_url, is_read, email_sent,
            willow_namespace, willow_subspace, willow_path, willow_timestamp_us,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM notifications
        WHERE recipient_did = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(did)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let items: Vec<NotificationItem> = rows
        .into_iter()
        .map(|row| NotificationItem {
            id: row.try_get("id").unwrap_or_default(),
            recipient_did: row.try_get("recipient_did").unwrap_or_default(),
            kind: row.try_get("kind").unwrap_or_default(),
            title: row.try_get("title").unwrap_or_default(),
            body: row.try_get("body").unwrap_or_default(),
            action_url: row.try_get("action_url").unwrap_or_default(),
            is_read: row.try_get("is_read").unwrap_or(false),
            email_sent: row.try_get("email_sent").unwrap_or(false),
            willow_namespace: row
                .try_get("willow_namespace")
                .unwrap_or_else(|_| "exonovel".to_string()),
            willow_subspace: row.try_get("willow_subspace").unwrap_or_default(),
            willow_path: row.try_get("willow_path").unwrap_or_default(),
            willow_timestamp_us: row.try_get("willow_timestamp_us").unwrap_or(0),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let unread_count = items.iter().filter(|n| !n.is_read).count();
    Ok(Json(serde_json::json!({
        "notifications": items,
        "unreadCount": unread_count,
    })))
}

pub async fn api_mark_notification_read(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let Some(did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };

    if id == "all" {
        let _ = sqlx::query("UPDATE notifications SET is_read = TRUE WHERE recipient_did = $1")
            .bind(did)
            .execute(pool)
            .await;
    } else {
        let _ = sqlx::query(
            "UPDATE notifications SET is_read = TRUE WHERE id = $1 AND recipient_did = $2",
        )
        .bind(&id)
        .bind(did)
        .execute(pool)
        .await;
    }

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{did}"),
        "notification",
        &id,
        "update",
        vec!["isRead".to_string()],
        serde_json::json!({ "id": id, "isRead": true }),
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn api_get_notification_prefs(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let Some(did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let prefs = get_or_init_preferences(pool, did).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    let devices_rows = sqlx::query(
        r#"
        SELECT device_id, platform, transport, endpoint_or_token, device_label,
               to_char(last_seen_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_seen_at,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM notification_devices
        WHERE did_peer = $1
        ORDER BY last_seen_at DESC
        "#,
    )
    .bind(did)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let devices: Vec<NotificationDevice> = devices_rows
        .into_iter()
        .map(|r| NotificationDevice {
            device_id: r.try_get("device_id").unwrap_or_default(),
            platform: r.try_get("platform").unwrap_or_default(),
            transport: r.try_get("transport").unwrap_or_default(),
            endpoint_or_token: r.try_get("endpoint_or_token").unwrap_or_default(),
            device_label: r.try_get("device_label").unwrap_or_default(),
            last_seen_at: r.try_get("last_seen_at").unwrap_or_default(),
            created_at: r.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let email_configured = std::env::var("BREVO_API_KEY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    Ok(Json(serde_json::json!({
        "preferences": prefs,
        "devices": devices,
        "emailConfigured": email_configured,
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotificationPrefsBody {
    pub in_app_enabled: Option<bool>,
    pub web_push_enabled: Option<bool>,
    pub native_push_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub email_digest_mode: Option<String>,
    pub notify_author_earnings: Option<bool>,
    pub notify_reader_purchases: Option<bool>,
    pub notify_group_invites: Option<bool>,
    pub notify_referral_signups: Option<bool>,
    pub notify_moderator_comments: Option<bool>,
    pub notify_collaborator_comments: Option<bool>,
    pub notify_mentions_and_forks: Option<bool>,
    pub notify_security_signins: Option<bool>,
    pub notify_maturity_releases: Option<bool>,
    pub notify_maturity_unblocked: Option<bool>,
}

pub async fn api_update_notification_prefs(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<UpdateNotificationPrefsBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let Some(did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };
    let _ = get_or_init_preferences(pool, did).await;
    if let Some(m) = &body.email_digest_mode {
        if !["instant", "daily", "off"].contains(&m.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Digest mode must be instant, daily, or off." })),
            ));
        }
    }

    sqlx::query(
        r#"
        UPDATE notification_preferences
        SET in_app_enabled = COALESCE($2, in_app_enabled),
            web_push_enabled = COALESCE($3, web_push_enabled),
            native_push_enabled = COALESCE($4, native_push_enabled),
            email_enabled = COALESCE($5, email_enabled),
            email_digest_mode = COALESCE($6, email_digest_mode),
            notify_author_earnings = COALESCE($7, notify_author_earnings),
            notify_reader_purchases = COALESCE($8, notify_reader_purchases),
            notify_group_invites = COALESCE($9, notify_group_invites),
            notify_referral_signups = COALESCE($10, notify_referral_signups),
            notify_moderator_comments = COALESCE($11, notify_moderator_comments),
            notify_collaborator_comments = COALESCE($12, notify_collaborator_comments),
            notify_mentions_and_forks = COALESCE($13, notify_mentions_and_forks),
            notify_security_signins = COALESCE($14, notify_security_signins),
            notify_maturity_releases = COALESCE($15, notify_maturity_releases),
            notify_maturity_unblocked = COALESCE($16, notify_maturity_unblocked),
            updated_at = NOW()
        WHERE did_peer = $1
        "#,
    )
    .bind(did)
    .bind(body.in_app_enabled)
    .bind(body.web_push_enabled)
    .bind(body.native_push_enabled)
    .bind(body.email_enabled)
    .bind(body.email_digest_mode.as_deref())
    .bind(body.notify_author_earnings)
    .bind(body.notify_reader_purchases)
    .bind(body.notify_group_invites)
    .bind(body.notify_referral_signups)
    .bind(body.notify_moderator_comments)
    .bind(body.notify_collaborator_comments)
    .bind(body.notify_mentions_and_forks)
    .bind(body.notify_security_signins)
    .bind(body.notify_maturity_releases)
    .bind(body.notify_maturity_unblocked)
    .execute(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let updated = get_or_init_preferences(pool, did).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    emit_live_patch(
        Some(pool),
        &state.live_bus,
        format!("user:{did}"),
        "notification_prefs",
        did,
        "update",
        vec!["preferences".to_string()],
        serde_json::json!(updated),
    )
    .await;

    Ok(Json(serde_json::json!({ "preferences": updated })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceBody {
    pub platform: String,
    pub transport: String,
    pub endpoint_or_token: String,
    pub p256dh: Option<String>,
    pub auth_secret: Option<String>,
    pub device_label: Option<String>,
}

pub async fn api_register_notification_device(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<RegisterDeviceBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let Some(did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };

    if !["web", "linux", "macos", "windows", "android", "ios"].contains(&body.platform.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unsupported platform." })),
        ));
    }
    if !["web_push", "unifiedpush", "fcm", "apns", "wns"].contains(&body.transport.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unsupported push transport." })),
        ));
    }
    if body.endpoint_or_token.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Device token or endpoint URL required." })),
        ));
    }

    let device_id = format!("dev_{}", now_timestamp_us());
    sqlx::query(
        r#"
        INSERT INTO notification_devices (
            device_id, did_peer, platform, transport, endpoint_or_token, p256dh, auth_secret, device_label
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(&device_id)
    .bind(did)
    .bind(&body.platform)
    .bind(&body.transport)
    .bind(body.endpoint_or_token.trim())
    .bind(body.p256dh.as_deref().unwrap_or(""))
    .bind(body.auth_secret.as_deref().unwrap_or(""))
    .bind(body.device_label.as_deref().unwrap_or("Registered Device"))
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
        format!("user:{did}"),
        "notification_device",
        &device_id,
        "insert",
        vec!["platform".to_string(), "transport".to_string()],
        serde_json::json!({
            "deviceId": device_id,
            "platform": body.platform,
            "transport": body.transport,
            "deviceLabel": body.device_label.unwrap_or_else(|| "Registered Device".to_string()),
        }),
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true, "deviceId": device_id })))
}

pub async fn api_test_notification(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let Some(did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };

    let item = dispatch_notification(
        pool,
        &state.live_bus,
        did,
        "security_signin",
        "ExoNovel Live Notification Test",
        "Real-time live stream delivery and transactional email routing verified.",
        "/",
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    Ok(Json(serde_json::json!({ "notification": item })))
}

// =========================================================================
// Billing, Wallets, Paywalls, Reader Purchases & Personal Sync Subscription
// Strict architectural rule (ADR 004 / 008): Owned exclusively by Profile page
// =========================================================================

pub async fn api_list_billing(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let Some(did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };

    let rows = sqlx::query(
        r#"
        SELECT id, kind, currency, amount_cents, billing_cycle, status, counterparty_did,
               note, willow_path, willow_timestamp_us,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM billing_ledger
        WHERE owner_did = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .bind(did)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let items: Vec<BillingLedgerItem> = rows
        .into_iter()
        .map(|r| BillingLedgerItem {
            id: r.try_get("id").unwrap_or_default(),
            kind: r.try_get("kind").unwrap_or_default(),
            currency: r.try_get("currency").unwrap_or_else(|_| "USD".to_string()),
            amount_cents: r.try_get("amount_cents").unwrap_or(0),
            billing_cycle: r
                .try_get("billing_cycle")
                .unwrap_or_else(|_| "one-time".to_string()),
            status: r.try_get("status").unwrap_or_else(|_| "active".to_string()),
            counterparty_did: r.try_get("counterparty_did").unwrap_or_default(),
            note: r.try_get("note").unwrap_or_default(),
            willow_path: r.try_get("willow_path").unwrap_or_default(),
            willow_timestamp_us: r.try_get("willow_timestamp_us").unwrap_or(0),
            created_at: r.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    Ok(Json(serde_json::json!({
        "entries": items,
        "exonovelSyncPeerDid": "did:peer:0zExoNovelRelaySyncService",
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivateSyncPlanBody {
    pub plan_tier: String,
    pub currency: String,
    pub local_amount_cents: Option<i64>,
    pub payment_rail: String,
}

pub async fn api_activate_sync_plan(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<ActivateSyncPlanBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(pool) = state.db.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Database unavailable." })),
        ));
    };
    let Some(did) = user.did_peer.as_deref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "did:peer identity unavailable." })),
        ));
    };

    let (cycle, default_usd_cents, willow_enabled) = match body.plan_tier.as_str() {
        "sync-monthly" => ("monthly", 900_i64, true),
        "sync-annual" => ("annual", 9000_i64, true),
        "free" => ("free", 0_i64, false),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Plan tier must be free, sync-monthly, or sync-annual." })),
            ))
        }
    };

    let amount_cents = body.local_amount_cents.unwrap_or(default_usd_cents);
    let currency = body.currency.trim().to_uppercase();
    let currency = if currency.is_empty() {
        "USD".to_string()
    } else {
        currency.chars().take(8).collect()
    };

    let ts_us = now_timestamp_us();
    sqlx::query(
        r#"
        UPDATE user_profiles
        SET sync_plan_tier = $2,
            willow_sync_enabled = $3,
            willow_timestamp_us = $4,
            updated_at = NOW()
        WHERE did_peer = $1
        "#,
    )
    .bind(did)
    .bind(&body.plan_tier)
    .bind(willow_enabled)
    .bind(ts_us)
    .execute(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let entry_id = format!("bil_{ts_us}");
    let note = format!(
        "ExoNovel Personal Web-Device Sync ({}) via {}",
        body.plan_tier,
        body.payment_rail.trim()
    );
    let willow_path = format!("/billing/{entry_id}");
    sqlx::query(
        r#"
        INSERT INTO billing_ledger (
            id, owner_did, kind, currency, amount_cents, billing_cycle, status,
            counterparty_did, note, willow_namespace, willow_subspace, willow_path, willow_timestamp_us
        )
        VALUES ($1, $2, 'sync_subscription', $3, $4, $5, 'active', 'did:peer:0zExoNovelRelaySyncService', $6, 'exonovel', $2, $7, $8)
        "#,
    )
    .bind(&entry_id)
    .bind(did)
    .bind(&currency)
    .bind(amount_cents)
    .bind(cycle)
    .bind(&note)
    .bind(&willow_path)
    .bind(ts_us)
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
        format!("user:{did}"),
        "billing",
        &entry_id,
        "insert",
        vec!["planTier".to_string(), "willowSyncEnabled".to_string()],
        serde_json::json!({
            "id": entry_id,
            "syncPlanTier": body.plan_tier,
            "willowSyncEnabled": willow_enabled,
        }),
    )
    .await;

    let _ = dispatch_notification(
        pool,
        &state.live_bus,
        did,
        "author_earnings",
        "ExoNovel Sync Subscription Updated",
        &format!(
            "Personal Sync plan '{}' recorded ({} {:.2}). Active under Profile -> Plans.",
            body.plan_tier,
            currency,
            amount_cents as f64 / 100.0
        ),
        "/?tab=profile",
    )
    .await;

    Ok(Json(serde_json::json!({
        "ok": true,
        "syncPlanTier": body.plan_tier,
        "willowSyncEnabled": willow_enabled,
    })))
}
