use std::collections::HashMap;

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::Deserialize;

use crate::models::{sleep_ms, AuthConfigResponse, LivePatchEvent, resolve_fresh_jwt};

/// Global App-Wide Live UI Subscription & Arrival Motion Registry (ADR 009).
/// Supports store-agnostic "neon" (SSE/PgListener) and "willow" (local store) origins.
#[derive(Clone, Copy)]
pub struct LiveSyncState {
    pub connected: RwSignal<bool>,
    pub last_event: RwSignal<Option<LivePatchEvent>>,
    pub last_origin: RwSignal<String>,
    pub pulse_seq: RwSignal<u64>,
    pub active_animations: RwSignal<HashMap<String, (&'static str, u64)>>,
}

impl LiveSyncState {
    pub fn new() -> Self {
        Self {
            connected: RwSignal::new(false),
            last_event: RwSignal::new(None),
            last_origin: RwSignal::new("neon".to_string()),
            pulse_seq: RwSignal::new(0),
            active_animations: RwSignal::new(HashMap::new()),
        }
    }

    pub fn record_event(&self, ev: LivePatchEvent) {
        let key = format!("{}:{}", ev.entity_kind, ev.entity_id);
        let cls = if ev.op == "insert" {
            "exo-live-insert"
        } else {
            "exo-live-update"
        };
        let next_seq = self.pulse_seq.get_untracked() + 1;
        self.pulse_seq.set(next_seq);
        self.last_origin.set(ev.origin.clone());
        self.last_event.set(Some(ev));

        self.active_animations.update(|map| {
            map.insert(key.clone(), (cls, next_seq));
        });

        let anim_map = self.active_animations;
        leptos::task::spawn_local(async move {
            sleep_ms(1100).await;
            anim_map.update(|map| {
                if let Some((_, seq)) = map.get(&key) {
                    if *seq == next_seq {
                        map.remove(&key);
                    }
                }
            });
        });
    }

    pub fn anim_class(&self, entity_kind: &str, entity_id: &str) -> &'static str {
        let key = format!("{entity_kind}:{entity_id}");
        self.active_animations
            .get()
            .get(&key)
            .map(|(cls, _)| *cls)
            .unwrap_or("")
    }

    pub fn matches_kind(&self, kinds: &[&str]) -> bool {
        self.last_event
            .get()
            .is_some_and(|ev| kinds.contains(&ev.entity_kind.as_str()))
    }
}

pub fn use_live_sync() -> LiveSyncState {
    expect_context::<LiveSyncState>()
}

/// Start authenticated EventSource subscription using one-time /api/events/ticket exchange.
#[cfg(not(feature = "ssr"))]
pub fn spawn_live_sse_connector(
    live: LiveSyncState,
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
    active: ReadSignal<bool>,
) {
    use wasm_bindgen::{closure::Closure, JsCast};
    use web_sys::{EventSource, MessageEvent};

    #[derive(Deserialize)]
    struct TicketResponse {
        ticket: String,
    }

    leptos::task::spawn_local(async move {
        let mut backoff_ms = 1500_i32;
        while active.get_untracked() {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                live.connected.set(false);
                sleep_ms(backoff_ms).await;
                backoff_ms = (backoff_ms * 2).min(15_000);
                continue;
            };

            let ticket_opt = match Request::post("/api/events/ticket")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                Ok(resp) if resp.ok() => resp.json::<TicketResponse>().await.ok().map(|t| t.ticket),
                _ => None,
            };

            let Some(ticket) = ticket_opt else {
                live.connected.set(false);
                sleep_ms(backoff_ms).await;
                backoff_ms = (backoff_ms * 2).min(15_000);
                continue;
            };

            let stream_url = format!("/api/events/stream?ticket={ticket}");
            let Ok(es) = EventSource::new(&stream_url) else {
                live.connected.set(false);
                sleep_ms(backoff_ms).await;
                continue;
            };

            let (done_tx, done_rx) = futures_channel_oneshot();
            let done_signal = std::rc::Rc::new(std::cell::RefCell::new(Some(done_tx)));

            let live_open = live;
            let on_open = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                live_open.connected.set(true);
            }) as Box<dyn FnMut(web_sys::Event)>);
            let _ = es.add_event_listener_with_callback("open", on_open.as_ref().unchecked_ref());

            let live_patch = live;
            let on_patch = Closure::wrap(Box::new(move |ev: MessageEvent| {
                if let Some(raw) = ev.data().as_string() {
                    if let Ok(parsed) = serde_json::from_str::<LivePatchEvent>(&raw) {
                        live_patch.record_event(parsed);
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            let _ = es.add_event_listener_with_callback(
                "live_patch",
                on_patch.as_ref().unchecked_ref(),
            );

            let live_err = live;
            let es_err_close = es.clone();
            let done_err = done_signal.clone();
            let on_err = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                live_err.connected.set(false);
                es_err_close.close();
                if let Some(notify) = done_err.borrow_mut().take() {
                    notify();
                }
            }) as Box<dyn FnMut(web_sys::Event)>);
            let _ = es.add_event_listener_with_callback("error", on_err.as_ref().unchecked_ref());

            // Wait until connection drops or session signs out
            done_rx.await;
            es.close();
            drop(on_open);
            drop(on_patch);
            drop(on_err);
            live.connected.set(false);

            if !active.get_untracked() {
                break;
            }
            sleep_ms(backoff_ms).await;
            backoff_ms = (backoff_ms * 2).min(15_000);
        }
    });
}

#[cfg(feature = "ssr")]
pub fn spawn_live_sse_connector(
    _live: LiveSyncState,
    _auth_config: ReadSignal<Option<AuthConfigResponse>>,
    _initial_jwt: ReadSignal<Option<String>>,
    _active: ReadSignal<bool>,
) {
}

#[cfg(not(feature = "ssr"))]
fn futures_channel_oneshot() -> (Box<dyn FnOnce()>, std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>) {
    let resolved = std::rc::Rc::new(std::cell::Cell::new(false));
    let waker: std::rc::Rc<std::cell::RefCell<Option<std::task::Waker>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    struct SimpleOneshot {
        resolved: std::rc::Rc<std::cell::Cell<bool>>,
        waker: std::rc::Rc<std::cell::RefCell<Option<std::task::Waker>>>,
    }
    impl std::future::Future for SimpleOneshot {
        type Output = ();
        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<()> {
            if self.resolved.get() {
                std::task::Poll::Ready(())
            } else {
                *self.waker.borrow_mut() = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        }
    }

    let r_trig = resolved.clone();
    let w_trig = waker.clone();
    let trigger = Box::new(move || {
        r_trig.set(true);
        if let Some(w) = w_trig.borrow_mut().take() {
            w.wake();
        }
    });

    (
        trigger,
        Box::pin(SimpleOneshot { resolved, waker }),
    )
}
