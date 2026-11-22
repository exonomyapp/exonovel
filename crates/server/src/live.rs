use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use aes_gcm::aead::OsRng;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures_util::stream::{Stream, StreamExt};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgListener, PgPool};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use crate::{auth::AuthUser, AppState};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePatchEvent {
    pub seq: u64,
    /// Scope filter: "user:<did_peer>", "authorized", "admin", or "*"
    pub scope_did: String,
    /// Entity bucket: "profile", "persona", "group", "invite", "allowlist", "identity",
    /// "notification", "notification_prefs", "notification_device", "adr", "tooling", "maturity_note", "billing"
    pub entity_kind: String,
    pub entity_id: String,
    /// "insert" | "update" | "delete" | "reload"
    pub op: String,
    pub changed_fields: Vec<String>,
    pub payload: serde_json::Value,
    /// "neon" or "willow" (ADR 009 store-agnostic parity)
    pub origin: String,
    pub emitted_at_ms: i64,
}

#[derive(Clone)]
pub struct SseTicketPrincipal {
    pub did_peer: String,
    pub authorized: bool,
    pub is_admin: bool,
    pub expires_at: Instant,
}

#[derive(Clone)]
pub struct LiveBus {
    tx: broadcast::Sender<LivePatchEvent>,
    seq: Arc<AtomicU64>,
    tickets: Arc<RwLock<HashMap<String, SseTicketPrincipal>>>,
}

impl LiveBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(512);
        Self {
            tx,
            seq: Arc::new(AtomicU64::new(1)),
            tickets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn next_seq(&self) -> u64 {
        self.seq.fetch_add(1, Ordering::Relaxed)
    }

    pub fn publish_event(&self, mut event: LivePatchEvent) {
        if event.seq == 0 {
            event.seq = self.next_seq();
        }
        if event.emitted_at_ms == 0 {
            event.emitted_at_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
        }
        if event.origin.is_empty() {
            event.origin = "neon".to_string();
        }
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LivePatchEvent> {
        self.tx.subscribe()
    }

    pub async fn issue_ticket(
        &self,
        did_peer: String,
        authorized: bool,
        is_admin: bool,
    ) -> String {
        let mut raw = [0_u8; 24];
        OsRng.fill_bytes(&mut raw);
        let token = bs58::encode(&raw).into_string();
        let now = Instant::now();
        let mut guard = self.tickets.write().await;
        guard.retain(|_, v| v.expires_at > now);
        guard.insert(
            token.clone(),
            SseTicketPrincipal {
                did_peer,
                authorized,
                is_admin,
                expires_at: now + Duration::from_secs(90),
            },
        );
        token
    }

    pub async fn consume_ticket(&self, token: &str) -> Option<SseTicketPrincipal> {
        let now = Instant::now();
        let mut guard = self.tickets.write().await;
        guard.retain(|_, v| v.expires_at > now);
        guard.remove(token)
    }
}

pub async fn emit_live_patch(
    pool: Option<&PgPool>,
    bus: &LiveBus,
    scope_did: impl Into<String>,
    entity_kind: impl Into<String>,
    entity_id: impl Into<String>,
    op: impl Into<String>,
    changed_fields: Vec<String>,
    payload: serde_json::Value,
) {
    let event = LivePatchEvent {
        seq: bus.next_seq(),
        scope_did: scope_did.into(),
        entity_kind: entity_kind.into(),
        entity_id: entity_id.into(),
        op: op.into(),
        changed_fields,
        payload,
        origin: "neon".to_string(),
        emitted_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0),
    };

    // Immediate in-process broadcast (low latency, works even on PgBouncer-pooled DATABASE_URL)
    bus.publish_event(event.clone());

    // Multi-instance PG NOTIFY broadcast (received via DATABASE_URL_UNPOOLED PgListener)
    if let Some(db) = pool {
        if let Ok(json_str) = serde_json::to_string(&event) {
            if json_str.len() < 7500 {
                let _ = sqlx::query("SELECT pg_notify('exonovel_live', $1)")
                    .bind(&json_str)
                    .execute(db)
                    .await;
            } else {
                let compact = LivePatchEvent {
                    payload: serde_json::json!({ "compact": true }),
                    ..event
                };
                if let Ok(compact_str) = serde_json::to_string(&compact) {
                    let _ = sqlx::query("SELECT pg_notify('exonovel_live', $1)")
                        .bind(&compact_str)
                        .execute(db)
                        .await;
                }
            }
        }
    }
}

pub fn spawn_pg_listener(bus: LiveBus) {
    let listen_url = std::env::var("DATABASE_URL_UNPOOLED")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::env::var("DATABASE_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });

    let Some(url) = listen_url else {
        return;
    };

    tokio::spawn(async move {
        let mut backoff_secs = 2_u64;
        loop {
            match PgListener::connect(&url).await {
                Ok(mut listener) => {
                    if let Err(err) = listener.listen("exonovel_live").await {
                        eprintln!("PgListener listen warning: {err}");
                        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                        backoff_secs = (backoff_secs * 2).min(60);
                        continue;
                    }
                    backoff_secs = 2;
                    loop {
                        match listener.recv().await {
                            Ok(notification) => {
                                if let Ok(ev) =
                                    serde_json::from_str::<LivePatchEvent>(notification.payload())
                                {
                                    bus.publish_event(ev);
                                }
                            }
                            Err(err) => {
                                eprintln!("PgListener reconnect needed: {err}");
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("PgListener connect warning: {err}");
                }
            }
            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs * 2).min(60);
        }
    });
}

pub async fn api_issue_sse_ticket(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let did = user.did_peer.clone().unwrap_or(user.auth_user_id);
    let ticket = state
        .live_bus
        .issue_ticket(did, user.authorized, user.is_admin)
        .await;
    Ok(Json(serde_json::json!({ "ticket": ticket })))
}

#[derive(Deserialize)]
pub struct StreamQuery {
    pub ticket: String,
}

fn event_matches_principal(ev: &LivePatchEvent, p: &SseTicketPrincipal) -> bool {
    let scope = ev.scope_did.as_str();
    if scope == "*" {
        return true;
    }
    if scope == "admin" {
        return p.is_admin;
    }
    if scope == "authorized" {
        return p.authorized || p.is_admin;
    }
    if let Some(target_did) = scope.strip_prefix("user:") {
        return target_did == p.did_peer;
    }
    scope == p.did_peer
}

pub async fn api_events_stream(
    State(state): State<AppState>,
    Query(query): Query<StreamQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, StatusCode> {
    let Some(principal) = state.live_bus.consume_ticket(query.ticket.trim()).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let rx = state.live_bus.subscribe();
    let stream = BroadcastStream::new(rx);

    let sse_stream = stream.filter_map(move |item| {
        let p = principal.clone();
        async move {
            match item {
                Ok(ev) if event_matches_principal(&ev, &p) => {
                    let event = Event::default()
                        .id(ev.seq.to_string())
                        .event("live_patch")
                        .json_data(&ev)
                        .unwrap_or_else(|_| Event::default().comment("encode_error"));
                    Some(Ok(event))
                }
                Ok(_) => None,
                Err(BroadcastStreamRecvError::Lagged(_)) => {
                    let sync_ev = LivePatchEvent {
                        seq: 0,
                        scope_did: format!("user:{}", p.did_peer),
                        entity_kind: "system".to_string(),
                        entity_id: "resync".to_string(),
                        op: "reload".to_string(),
                        changed_fields: Vec::new(),
                        payload: serde_json::json!({ "reason": "lagged" }),
                        origin: "neon".to_string(),
                        emitted_at_ms: 0,
                    };
                    Event::default()
                        .event("live_patch")
                        .json_data(&sync_ev)
                        .ok()
                        .map(Ok)
                }
            }
        }
    });

    Ok(Sse::new(sse_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(20))
            .text("keep-alive"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principal_scope_filtering_enforces_boundaries() {
        let user_p = SseTicketPrincipal {
            did_peer: "did:peer:0zUserA".to_string(),
            authorized: false,
            is_admin: false,
            expires_at: Instant::now() + Duration::from_secs(30),
        };
        let other_ev = LivePatchEvent {
            seq: 1,
            scope_did: "user:did:peer:0zUserB".to_string(),
            entity_kind: "notification".to_string(),
            entity_id: "1".to_string(),
            op: "insert".to_string(),
            changed_fields: vec![],
            payload: serde_json::json!({}),
            origin: "neon".to_string(),
            emitted_at_ms: 1,
        };
        assert!(!event_matches_principal(&other_ev, &user_p));
        let admin_ev = LivePatchEvent {
            scope_did: "admin".to_string(),
            ..other_ev.clone()
        };
        assert!(!event_matches_principal(&admin_ev, &user_p));
        let own_ev = LivePatchEvent {
            scope_did: "user:did:peer:0zUserA".to_string(),
            ..other_ev
        };
        assert!(event_matches_principal(&own_ev, &user_p));
    }
}
