mod cms;
mod i18n;
mod live;
mod models;
mod profile;
mod settings;

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use serde::Deserialize;
use wasm_bindgen::JsCast;
use web_sys::RequestCredentials;

use crate::{
    cms::DocsCmsView,
    i18n::{fetch_locale_catalog, fetch_locale_pack, use_i18n, I18nState},
    live::{spawn_live_sse_connector, use_live_sync, LiveSyncState},
    models::{
        active_persona_for_user, copy_text_to_clipboard, detect_browser_timezone,
        display_name_for_user, format_account_role, is_safe_avatar_url, load_authenticated_user,
        read_session_storage, remove_session_storage, resolve_fresh_jwt, save_profile_patch,
        sleep_ms, write_session_storage, ActiveTab, AppearanceMode, AuthConfigResponse, BellIcon,
        CopyIcon, GitHubLogoIcon, GoogleLogoIcon, InvitePreview, MonitorIcon, MoonIcon,
        NotificationItem, NotificationsListResponse, ProfilePatchResponse, SunIcon, VerifiedUser,
    },
    profile::ProfileView,
    settings::SettingsView,
};

#[cfg(not(feature = "ssr"))]
fn fire_browser_push_ping(title: &str, body: &str) {
    if web_sys::Notification::permission() == web_sys::NotificationPermission::Granted {
        let msg = format!("{title}: {body}");
        let _ = web_sys::Notification::new(&msg);
    }
}

#[cfg(feature = "ssr")]
fn fire_browser_push_ping(_title: &str, _body: &str) {}

async fn claim_pending_invite_if_any(jwt: &str) {
    let code = read_session_storage("exo_pending_invite")
        .or_else(|| {
            web_sys::window()
                .and_then(|w| w.location().search().ok())
                .and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
                .and_then(|p| p.get("invite"))
        })
        .unwrap_or_default();

    let clean = code.trim().to_string();
    if clean.is_empty() {
        return;
    }
    if let Ok(req) = Request::post("/api/invites/claim")
        .header("Authorization", &format!("Bearer {jwt}"))
        .json(&serde_json::json!({ "code": clean }))
    {
        let _ = req.send().await;
    }
    remove_session_storage("exo_pending_invite");
    if let Some(win) = web_sys::window() {
        if let Ok(search) = win.location().search() {
            if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
                if params.has("invite") {
                    params.delete("invite");
                    let clean_query = params.to_string().as_string().unwrap_or_default();
                    let pathname = win.location().pathname().unwrap_or_else(|_| "/".to_string());
                    let next_url = if clean_query.is_empty() {
                        pathname
                    } else {
                        format!("{pathname}?{clean_query}")
                    };
                    if let Ok(history) = win.history() {
                        let _ = history.replace_state_with_url(
                            &wasm_bindgen::JsValue::NULL,
                            "",
                            Some(&next_url),
                        );
                    }
                }
            }
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="ExoNovel · ExoSystems" />
        <Router>
            <Routes fallback=|| view! { <HomeView /> }>
                <Route path=path!("/") view=HomeView />
                <Route path=path!("/privacy") view=PrivacyPolicyView />
                <Route path=path!("/terms") view=TermsOfServiceView />
            </Routes>
        </Router>
    }
}

#[component]
fn HomeView() -> impl IntoView {
    let live = LiveSyncState::new();
    let i18n = I18nState::new();
    provide_context(live);
    provide_context(i18n);

    let (auth_config, set_auth_config) = signal::<Option<AuthConfigResponse>>(None);
    let (loading_session, set_loading_session) = signal(true);
    let (current_user, set_current_user) = signal::<Option<VerifiedUser>>(None);
    let (session_jwt, set_session_jwt) = signal::<Option<String>>(None);
    let (active_tab, set_active_tab) = signal(ActiveTab::Workspace);
    let (appearance, set_appearance) = signal(AppearanceMode::System);
    let (active_doc_path, set_active_doc_path) = signal(String::new());
    let (return_tab, set_return_tab) = signal(ActiveTab::Workspace);
    let (chip_open, set_chip_open) = signal(false);
    let (bell_open, set_bell_open) = signal(false);
    let (notifications, set_notifications) = signal::<Vec<NotificationItem>>(Vec::new());
    let (unread_count, set_unread_count) = signal(0_usize);
    let (toast_msg, set_toast_msg) = signal::<Option<String>>(None);
    let (toast_seq, set_toast_seq) = signal(0_u64);
    let (pending_invite, set_pending_invite) = signal::<Option<String>>(None);
    let (sse_started, set_sse_started) = signal(false);

    let is_authenticated = Memo::new(move |_| current_user.get().is_some());

    let show_toast = move |msg: String| {
        set_toast_msg.set(Some(msg));
        let next_seq = toast_seq.get_untracked() + 1;
        set_toast_seq.set(next_seq);
        leptos::task::spawn_local(async move {
            sleep_ms(3000).await;
            if toast_seq.get_untracked() == next_seq {
                set_toast_msg.set(None);
            }
        });
    };

    let show_did_copied_toast = move |did: String| {
        copy_text_to_clipboard(&did);
        show_toast("did:peer ID copied".to_string());
    };

    let reload_notifications = move || {
        if current_user.get_untracked().is_none() {
            return;
        }
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, session_jwt).await else {
                return;
            };
            if let Ok(resp) = Request::get("/api/notifications")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<NotificationsListResponse>().await {
                        set_notifications.set(data.notifications);
                        set_unread_count.set(data.unread_count);
                    }
                }
            }
        });
    };

    let apply_user_profile_state = move |user: &VerifiedUser| {
        if let Some(prof) = &user.profile {
            if !prof.appearance_mode.is_empty() {
                set_appearance.set(AppearanceMode::from_str(&prof.appearance_mode));
            }
            if !prof.ui_locale.is_empty() {
                i18n.ui_locale.set(prof.ui_locale.clone());
            }
            if !prof.default_input_locale.is_empty() {
                i18n.input_locale.set(prof.default_input_locale.clone());
            }
            if !prof.last_doc_path.is_empty() && active_doc_path.get_untracked().is_empty() {
                set_active_doc_path.set(prof.last_doc_path.clone());
            }
            if prof.last_active_tab == "docs" && user.authorized {
                set_return_tab.set(ActiveTab::Docs);
            } else {
                set_return_tab.set(ActiveTab::Workspace);
            }
        }
    };

    // Fetch 29-language catalog + URL params (invite/doc) once on mount
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            fetch_locale_catalog(i18n).await;
        });
        if let Some(win) = web_sys::window() {
            if let Ok(search) = win.location().search() {
                if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
                    if let Some(doc_param) = params.get("doc").filter(|d| !d.trim().is_empty()) {
                        set_active_doc_path.set(doc_param);
                        set_active_tab.set(ActiveTab::Docs);
                    }
                    if let Some(inv_param) = params.get("invite").filter(|i| !i.trim().is_empty()) {
                        write_session_storage("exo_pending_invite", &inv_param);
                        set_pending_invite.set(Some(inv_param));
                    }
                }
            }
        }
        if pending_invite.get_untracked().is_none() {
            if let Some(saved_inv) = read_session_storage("exo_pending_invite") {
                if !saved_inv.trim().is_empty() {
                    set_pending_invite.set(Some(saved_inv));
                }
            }
        }
    });

    // Apply Theme Appearance cleanly to <html> WITHOUT touching localStorage
    Effect::new(move |_| {
        let mode = appearance.get();
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let _ = root.set_attribute("data-appearance", mode.as_str());
        }
    });

    // Apply Output UI Locale (<html lang dir="ltr|rtl">) and download .ftl pack
    Effect::new(move |_| {
        let code = i18n.ui_locale.get();
        let dir = if code == "ar" || code == "he" { "rtl" } else { "ltr" };
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let _ = root.set_attribute("lang", &code);
            let _ = root.set_attribute("dir", dir);
        }
        leptos::task::spawn_local(async move {
            fetch_locale_pack(i18n, &code).await;
        });
    });

    let persist_profile_patch = move |patch: serde_json::Value| {
        if current_user.get_untracked().is_none() {
            return;
        }
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        let jwt = session_jwt.get_untracked();
        leptos::task::spawn_local(async move {
            if let Ok(updated) =
                save_profile_patch(auth_url.as_deref(), jwt.as_deref(), patch).await
            {
                set_current_user.update(|u_opt| {
                    if let Some(u) = u_opt {
                        u.profile = Some(updated.profile);
                        u.personas = updated.personas;
                        u.active_persona = updated.active_persona;
                    }
                });
            }
        });
    };

    // Start global Neon SSE listener once signed in & react to notification events
    Effect::new(move |_| {
        if is_authenticated.get() && !sse_started.get_untracked() {
            set_sse_started.set(true);
            let (sse_active, set_sse_active) = signal(true);
            Effect::new(move |_| {
                set_sse_active.set(is_authenticated.get());
            });
            spawn_live_sse_connector(live, auth_config, session_jwt, sse_active);
            reload_notifications();
        }
    });

    Effect::new(move |_| {
        let _seq = live.pulse_seq.get();
        if let Some(ev) = live.last_event.get() {
            if ev.entity_kind == "notification" {
                reload_notifications();
                if ev.op == "insert" {
                    if let Some(t) = ev.payload.get("title").and_then(|v| v.as_str()) {
                        let b = ev.payload.get("body").and_then(|v| v.as_str()).unwrap_or("");
                        fire_browser_push_ping(t, b);
                    }
                }
            }
        }
    });

    let navigate_to_subpage = move |dest: ActiveTab| {
        let current = active_tab.get_untracked();
        if current != ActiveTab::Settings && current != ActiveTab::Profile {
            let tab_to_save = match current {
                ActiveTab::Docs => ActiveTab::Docs,
                _ => ActiveTab::Workspace,
            };
            set_return_tab.set(tab_to_save);
            let tab_str = if tab_to_save == ActiveTab::Docs {
                "docs"
            } else {
                "workspace"
            };
            let doc_str = active_doc_path.get_untracked();
            persist_profile_patch(serde_json::json!({
                "lastActiveTab": tab_str,
                "lastDocPath": doc_str,
            }));
        }
        set_chip_open.set(false);
        set_bell_open.set(false);
        set_active_tab.set(dest);
    };

    let mark_notification_read = move |id: String| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, session_jwt).await else {
                return;
            };
            let url = format!("/api/notifications/{id}/read");
            if let Ok(resp) = Request::post(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    reload_notifications();
                }
            }
        });
    };

    let refresh_session = move || {
        let Some(cfg) = auth_config.get_untracked() else {
            return;
        };
        let Some(auth_url) = cfg.auth_url else {
            set_loading_session.set(false);
            return;
        };
        set_loading_session.set(true);
        leptos::task::spawn_local(async move {
            let result = load_authenticated_user(&auth_url, None).await;
            match result {
                Some((user, jwt)) => {
                    claim_pending_invite_if_any(&jwt).await;
                    set_pending_invite.set(None);
                    apply_user_profile_state(&user);
                    let tz = detect_browser_timezone();
                    if !tz.is_empty() && user.profile.as_ref().is_some_and(|p| p.timezone.is_empty())
                    {
                        let _ = save_profile_patch(
                            Some(&auth_url),
                            Some(&jwt),
                            serde_json::json!({ "timezone": tz }),
                        )
                        .await;
                    }
                    set_current_user.set(Some(user));
                    set_session_jwt.set(Some(jwt));
                }
                None => {
                    set_current_user.set(None);
                    set_session_jwt.set(None);
                }
            }
            set_loading_session.set(false);
        });
    };

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            let cfg_res = Request::get("/api/auth-config").send().await;
            let cfg = match cfg_res {
                Ok(r) if r.ok() => r.json::<AuthConfigResponse>().await.ok(),
                _ => None,
            };
            set_auth_config.set(cfg.clone());

            let verifier = web_sys::window().and_then(|w| {
                let search = w.location().search().ok()?;
                let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
                let val = params.get("neon_auth_session_verifier");
                if val.is_some() {
                    params.delete("neon_auth_session_verifier");
                    let clean_query = params.to_string().as_string().unwrap_or_default();
                    let pathname = w.location().pathname().unwrap_or_else(|_| "/".to_string());
                    let new_url = if clean_query.is_empty() {
                        pathname
                    } else {
                        format!("{pathname}?{clean_query}")
                    };
                    if let Ok(history) = w.history() {
                        let _ = history.replace_state_with_url(
                            &wasm_bindgen::JsValue::NULL,
                            "",
                            Some(&new_url),
                        );
                    }
                }
                val
            });

            if let Some(AuthConfigResponse {
                auth_configured: true,
                auth_url: Some(url),
                ..
            }) = cfg
            {
                if let Some((user, jwt)) = load_authenticated_user(&url, verifier.as_deref()).await {
                    claim_pending_invite_if_any(&jwt).await;
                    set_pending_invite.set(None);
                    apply_user_profile_state(&user);
                    let tz = detect_browser_timezone();
                    if !tz.is_empty() && user.profile.as_ref().is_some_and(|p| p.timezone.is_empty())
                    {
                        let _ = save_profile_patch(
                            Some(&url),
                            Some(&jwt),
                            serde_json::json!({ "timezone": tz }),
                        )
                        .await;
                    }
                    set_current_user.set(Some(user));
                    set_session_jwt.set(Some(jwt));
                }
            }
            set_loading_session.set(false);
        });
    });

    let sign_out = move || {
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        set_chip_open.set(false);
        set_bell_open.set(false);
        leptos::task::spawn_local(async move {
            if let Some(url) = auth_url {
                let endpoint = format!("{}/sign-out", url.trim_end_matches('/'));
                if let Ok(req) = Request::post(&endpoint)
                    .credentials(RequestCredentials::Include)
                    .json(&serde_json::json!({}))
                {
                    let _ = req.send().await;
                }
            }
            set_current_user.set(None);
            set_session_jwt.set(None);
            set_sse_started.set(false);
            set_active_tab.set(ActiveTab::Workspace);
        });
    };

    view! {
        <div class="exo-shell">
            <nav class="navbar-solid">
                <a href="/" title="ExoNovel" class="navbar-brand-link">
                    <img src="/exonovel-logo.png" alt="ExoNovel" class="navbar-logo" />
                </a>

                <div class="navbar-menu">
                    <a href="https://exosystems.net/" class="nav-link">
                        {move || i18n.tr("nav_home", "Home")}
                    </a>

                    <button
                        type="button"
                        class=move || if active_tab.get() == ActiveTab::Workspace { "nav-link nav-link-active" } else { "nav-link" }
                        on:click=move |_| {
                            set_chip_open.set(false);
                            set_bell_open.set(false);
                            set_active_tab.set(ActiveTab::Workspace);
                            persist_profile_patch(serde_json::json!({ "lastActiveTab": "workspace" }));
                        }
                    >
                        {move || i18n.tr("nav_workspace", "ExoNovel")}
                    </button>

                    {move || match current_user.get() {
                        Some(user) => {
                            let can_read_docs = user.authorized;
                            let active_p = active_persona_for_user(&user);
                            let chip_name = display_name_for_user(&user);
                            let chip_avatar = if !active_p.avatar_thumb_url.is_empty() {
                                active_p.avatar_thumb_url.clone()
                            } else {
                                active_p.avatar_url.clone()
                            };
                            let chip_initial = chip_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                            let active_did_cmp = active_p.persona_did.clone();
                            let personas_list = user.personas.clone();
                            let pinned_ui = user
                                .profile
                                .as_ref()
                                .map(|p| p.preferred_ui_locales.clone())
                                .unwrap_or_default();

                            view! {
                                <>
                                    {if can_read_docs {
                                        view! {
                                            <button
                                                type="button"
                                                class=move || if active_tab.get() == ActiveTab::Docs { "nav-link nav-link-active" } else { "nav-link" }
                                                on:click=move |_| {
                                                    set_chip_open.set(false);
                                                    set_bell_open.set(false);
                                                    set_active_tab.set(ActiveTab::Docs);
                                                    persist_profile_patch(serde_json::json!({ "lastActiveTab": "docs" }));
                                                }
                                            >
                                                {move || i18n.tr("nav_docs", "Docs & ADRs")}
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span /> }.into_any()
                                    }}

                                    <div class="exo-live-indicator" title="Global Neon SSE Real-Time Connection">
                                        <span class=move || if live.connected.get() { "exo-live-dot exo-live-dot-on" } else { "exo-live-dot" } />
                                        <span>{move || i18n.tr("live_sync_label", "Live")}</span>
                                    </div>

                                    <select
                                        class="exo-nav-select"
                                        aria-label="App Output Language"
                                        prop:value=move || i18n.ui_locale.get()
                                        on:change=move |ev| {
                                            let next = event_target_value(&ev);
                                            i18n.ui_locale.set(next.clone());
                                            persist_profile_patch(serde_json::json!({ "uiLocale": next }));
                                        }
                                    >
                                        {move || {
                                            let catalog = i18n.catalog.get();
                                            let list = if pinned_ui.is_empty() {
                                                catalog
                                            } else {
                                                let filtered: Vec<_> = catalog
                                                    .iter()
                                                    .filter(|l| pinned_ui.contains(&l.code) || l.code == i18n.ui_locale.get_untracked())
                                                    .cloned()
                                                    .collect();
                                                if filtered.is_empty() { catalog } else { filtered }
                                            };
                                            list.into_iter().map(|l| {
                                                view! { <option value=l.code>{l.native_name}</option> }
                                            }).collect_view()
                                        }}
                                    </select>

                                    <div class="exo-bell-wrap">
                                        <button
                                            type="button"
                                            class="ctl"
                                            aria-label="Notifications"
                                            on:click=move |_| {
                                                set_chip_open.set(false);
                                                set_bell_open.update(|v| *v = !*v);
                                            }
                                        >
                                            <BellIcon />
                                            <Show when=move || { unread_count.get() > 0 }>
                                                <span class="exo-bell-badge tabular-nums">{move || unread_count.get()}</span>
                                            </Show>
                                        </button>
                                        <Show when=move || bell_open.get()>
                                            <div class="exo-bell-menu">
                                                <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px;">
                                                    <strong>{move || i18n.tr("notifications_title", "Notifications")}</strong>
                                                    <button
                                                        type="button"
                                                        class="exo-btn"
                                                        style="min-height: 30px; padding: 0 8px; font-size: 13px;"
                                                        on:click=move |_| mark_notification_read("all".to_string())
                                                    >
                                                        "Mark all read"
                                                    </button>
                                                </div>
                                                {move || {
                                                    let list = notifications.get();
                                                    if list.is_empty() {
                                                        view! { <p class="exo-sub">"No notifications yet."</p> }.into_any()
                                                    } else {
                                                        list.into_iter().take(12).map(|n| {
                                                            let nid = n.id.clone();
                                                            let nid_anim = n.id.clone();
                                                            let base_cls = if n.is_read { "exo-notif-row" } else { "exo-notif-row exo-notif-row-unread" };
                                                            view! {
                                                                <div class=move || format!("{base_cls} {}", live.anim_class("notification", &nid_anim))>
                                                                    <div style="font-weight: 600; font-size: 15px;">{n.title}</div>
                                                                    <div class="exo-sub" style="font-size: 14px; margin-top: 2px;">{n.body}</div>
                                                                    {(!n.is_read).then(|| view! {
                                                                        <button
                                                                            type="button"
                                                                            class="exo-btn"
                                                                            style="min-height: 28px; padding: 0 8px; font-size: 13px; margin-top: 6px;"
                                                                            on:click=move |_| mark_notification_read(nid.clone())
                                                                        >
                                                                            "Mark read"
                                                                        </button>
                                                                    })}
                                                                </div>
                                                            }
                                                        }).collect_view().into_any()
                                                    }
                                                }}
                                            </div>
                                        </Show>
                                    </div>

                                    <div class="exo-user-chip-wrap">
                                        <button
                                            type="button"
                                            class="exo-user-chip"
                                            on:click=move |_| {
                                                set_bell_open.set(false);
                                                set_chip_open.update(|v| *v = !*v);
                                            }
                                        >
                                            {if is_safe_avatar_url(&chip_avatar) {
                                                view! { <img src=chip_avatar alt=chip_name.clone() class="exo-avatar-sm" /> }.into_any()
                                            } else {
                                                view! { <div class="exo-avatar-sm">{chip_initial}</div> }.into_any()
                                            }}
                                            <span>{chip_name}</span>
                                            <span style="font-size: 14px; opacity: 0.8;">"▾"</span>
                                        </button>

                                        <Show when=move || chip_open.get()>
                                            <div class="exo-chip-menu" role="menu">
                                                <button
                                                    type="button"
                                                    class=move || if active_tab.get() == ActiveTab::Profile { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                                    on:click=move |_| navigate_to_subpage(ActiveTab::Profile)
                                                >
                                                    <span>{move || i18n.tr("nav_profile", "Profile & Billing")}</span>
                                                </button>

                                                <div class="exo-chip-menu-divider" />
                                                <div class="exo-chip-menu-label">"Switch Persona"</div>
                                                {personas_list.clone().into_iter().map(|p| {
                                                    let p_did = p.persona_did.clone();
                                                    let is_curr = p_did == active_did_cmp;
                                                    let p_name = if p.display_name.is_empty() { "Default Persona".to_string() } else { p.display_name.clone() };
                                                    let p_avatar = if !p.avatar_thumb_url.is_empty() { p.avatar_thumb_url.clone() } else { p.avatar_url.clone() };
                                                    let p_init = p_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                                                    let p_name_toast = p_name.clone();
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class=if is_curr { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                                            on:click=move |_| {
                                                                set_chip_open.set(false);
                                                                persist_profile_patch(serde_json::json!({ "activePersonaDid": p_did.clone() }));
                                                                show_toast(format!("Switched to {p_name_toast}"));
                                                            }
                                                        >
                                                            <span style="display: inline-flex; align-items: center; gap: 8px; min-width: 0;">
                                                                {if is_safe_avatar_url(&p_avatar) {
                                                                    view! { <img src=p_avatar alt=p_name.clone() class="exo-avatar-sm" /> }.into_any()
                                                                } else {
                                                                    view! { <div class="exo-avatar-sm">{p_init}</div> }.into_any()
                                                                }}
                                                                <span style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{p_name}</span>
                                                            </span>
                                                            {if is_curr {
                                                                view! { <span class="exo-tag exo-tag-accent">"Active"</span> }.into_any()
                                                            } else {
                                                                view! { <span /> }.into_any()
                                                            }}
                                                        </button>
                                                    }
                                                }).collect_view()}

                                                <div class="exo-chip-menu-divider" />
                                                <button
                                                    type="button"
                                                    class=move || if active_tab.get() == ActiveTab::Settings { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                                    on:click=move |_| navigate_to_subpage(ActiveTab::Settings)
                                                >
                                                    <span>{move || i18n.tr("nav_settings", "Settings")}</span>
                                                </button>
                                                <button
                                                    type="button"
                                                    class="exo-chip-menu-item"
                                                    on:click=move |_| sign_out()
                                                >
                                                    <span>{move || i18n.tr("nav_sign_out", "Sign out")}</span>
                                                </button>
                                            </div>
                                        </Show>
                                    </div>
                                </>
                            }.into_any()
                        }
                        None => view! {
                            <span class="nav-link nav-link-active">"Sign in"</span>
                        }.into_any(),
                    }}

                    <span class="gadget-row">
                        <button
                            type="button"
                            class="ctl"
                            title=move || appearance.get().label()
                            aria-label=move || appearance.get().label()
                            on:click=move |_| {
                                let next = appearance.get_untracked().next();
                                set_appearance.set(next);
                                persist_profile_patch(serde_json::json!({ "appearanceMode": next.as_str() }));
                            }
                        >
                            {move || match appearance.get() {
                                AppearanceMode::Light => view! { <SunIcon /> }.into_any(),
                                AppearanceMode::Dark => view! { <MoonIcon /> }.into_any(),
                                AppearanceMode::System => view! { <MonitorIcon /> }.into_any(),
                            }}
                        </button>
                    </span>
                </div>
            </nav>

            <Show
                when=move || !loading_session.get()
                fallback=|| view! {
                    <main class="page-main">
                        <div class="exo-auth-wrap">
                            <div class="exo-card">
                                <p class="exo-sub">"Loading ExoNovel session..."</p>
                            </div>
                        </div>
                    </main>
                }
            >
                {
                    let session_view_key = Memo::new(move |_| {
                        current_user
                            .get()
                            .map(|u| (u.auth_user_id, u.authorized, u.is_admin))
                    });
                    move || match session_view_key.get() {
                        None => view! {
                            <SignInView
                                auth_config=auth_config
                                pending_invite=pending_invite
                                on_signed_in=refresh_session
                            />
                        }.into_any(),
                        Some((_uid, authorized, _is_admin)) => {
                            let tab = active_tab.get();
                            match tab {
                                ActiveTab::Docs if authorized => view! {
                                    <DocsCmsView
                                        auth_config=auth_config
                                        initial_jwt=session_jwt
                                        current_user=current_user
                                        active_doc_path=active_doc_path
                                        set_active_doc_path=set_active_doc_path
                                        on_toast=show_toast
                                    />
                                }.into_any(),
                                ActiveTab::Profile => view! {
                                    <ProfileView
                                        auth_config=auth_config
                                        initial_jwt=session_jwt
                                        current_user=current_user
                                        set_current_user=set_current_user
                                        return_tab=return_tab
                                        on_copy_did=show_did_copied_toast
                                        on_toast=show_toast
                                        on_back=move || {
                                            let dest = return_tab.get_untracked();
                                            let is_auth = current_user.get_untracked().is_some_and(|u| u.authorized);
                                            if dest == ActiveTab::Docs && is_auth {
                                                set_active_tab.set(ActiveTab::Docs);
                                            } else {
                                                set_active_tab.set(ActiveTab::Workspace);
                                            }
                                        }
                                    />
                                }.into_any(),
                                ActiveTab::Settings => {
                                    if let Some(user) = current_user.get_untracked() {
                                        view! {
                                            <SettingsView
                                                auth_config=auth_config
                                                initial_jwt=session_jwt
                                                user=user
                                                return_tab=return_tab
                                                appearance=appearance
                                                set_appearance=set_appearance
                                                on_copy_did=show_did_copied_toast
                                                on_toast=show_toast
                                                on_open_profile_billing=move || navigate_to_subpage(ActiveTab::Profile)
                                                on_profile_updated=move |updated: ProfilePatchResponse| {
                                                    set_current_user.update(|u_opt| {
                                                        if let Some(u) = u_opt {
                                                            u.profile = Some(updated.profile);
                                                            u.personas = updated.personas;
                                                            u.active_persona = updated.active_persona;
                                                        }
                                                    });
                                                }
                                                on_back=move || {
                                                    let dest = return_tab.get_untracked();
                                                    let is_auth = current_user.get_untracked().is_some_and(|u| u.authorized);
                                                    if dest == ActiveTab::Docs && is_auth {
                                                        set_active_tab.set(ActiveTab::Docs);
                                                    } else {
                                                        set_active_tab.set(ActiveTab::Workspace);
                                                    }
                                                }
                                            />
                                        }.into_any()
                                    } else {
                                        view! { <div /> }.into_any()
                                    }
                                }
                                _ => view! {
                                    <WorkspaceHomeView
                                        current_user=current_user
                                        on_open_docs=move || {
                                            set_active_tab.set(ActiveTab::Docs);
                                            persist_profile_patch(serde_json::json!({ "lastActiveTab": "docs" }));
                                        }
                                        on_open_profile=move || navigate_to_subpage(ActiveTab::Profile)
                                        on_open_settings=move || navigate_to_subpage(ActiveTab::Settings)
                                        on_copy_did=show_did_copied_toast
                                    />
                                }.into_any(),
                            }
                        }
                    }
                }
            </Show>

            <Show when=move || toast_msg.get().is_some()>
                <div class="exo-toast" role="status" aria-live="polite">
                    {move || toast_msg.get().unwrap_or_default()}
                </div>
            </Show>

            <PublicFooter />
        </div>
    }
}

#[component]
fn WorkspaceHomeView(
    current_user: ReadSignal<Option<VerifiedUser>>,
    on_open_docs: impl Fn() + Copy + Send + Sync + 'static,
    on_open_profile: impl Fn() + Copy + Send + Sync + 'static,
    on_open_settings: impl Fn() + Copy + Send + Sync + 'static,
    on_copy_did: impl Fn(String) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let i18n = use_i18n();
    let live = use_live_sync();

    view! {
        <main class="page-main">
            {move || {
                let Some(user) = current_user.get() else {
                    return view! { <div /> }.into_any();
                };
                let persona = active_persona_for_user(&user);
                let display_name = display_name_for_user(&user);
                let email_label = user.email.clone().unwrap_or_default();
                let persona_did = if persona.persona_did.is_empty() {
                    user.did_peer.clone().unwrap_or_else(|| "Pending did:peer".to_string())
                } else {
                    persona.persona_did.clone()
                };
                let did_copy = persona_did.clone();
                let did_anim = persona_did.clone();
                let avatar_url = persona.avatar_url.clone();
                let handle = persona.handle.clone();
                let initial = display_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                let can_read_docs = user.authorized;
                let role_label = format_account_role(&user.account_role);
                let sync_tier = user.profile.as_ref().map(|p| p.sync_plan_tier.clone()).unwrap_or_else(|| "free".to_string());

                view! {
                    <>
                        <div class=move || format!("exo-card {}", live.anim_class("profile", &did_anim))>
                            <div class="exo-home-hero">
                                {if is_safe_avatar_url(&avatar_url) {
                                    view! { <img src=avatar_url alt=display_name.clone() class="exo-avatar" /> }.into_any()
                                } else {
                                    view! { <div class="exo-avatar">{initial}</div> }.into_any()
                                }}
                                <div>
                                    <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
                                        <h1 class="exo-h1">
                                            {i18n.tr("welcome_prefix", "Welcome,")} " " {display_name} "."
                                        </h1>
                                        <span class="exo-tag exo-tag-accent">{role_label}</span>
                                        <span class="exo-tag tabular-nums">{"Plan: "}{sync_tier}</span>
                                    </div>
                                    <p class="exo-sub">
                                        {if !handle.is_empty() {
                                            format!("Pen name: {handle} · {email_label}")
                                        } else {
                                            email_label
                                        }}
                                    </p>
                                </div>
                            </div>

                            <p class="exo-sub" style="margin-top: 16px;">
                                "Your collaborative creative writing space anchored to your sovereign did:peer identity across Web Mode 1, Free Device Mode 2, and Managed Blind Sync Mode 3."
                            </p>

                            <div style="margin-top: 20px; display: flex; gap: 12px; flex-wrap: wrap;">
                                <button type="button" class="exo-btn exo-btn-primary" on:click=move |_| on_open_profile()>
                                    "Profile, Personas & Billing"
                                </button>
                                <button type="button" class="exo-btn" on:click=move |_| on_open_settings()>
                                    {i18n.tr("nav_settings", "Settings")}
                                </button>
                                {if can_read_docs {
                                    view! {
                                        <button type="button" class="exo-btn" on:click=move |_| on_open_docs()>
                                            "Open MADR Docs CMS"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! { <span /> }.into_any()
                                }}
                            </div>
                        </div>

                        <div class="exo-card">
                            <div class="exo-label">"Active Persona Workspace Key (did:peer)"</div>
                            <div class="exo-id-cell">
                                <button type="button" class="exo-copy-btn" on:click=move |_| on_copy_did(did_copy.clone())>
                                    <CopyIcon />
                                </button>
                                <span class="exo-mono">{persona_did}</span>
                            </div>
                        </div>
                    </>
                }.into_any()
            }}
        </main>
    }
}

#[component]
fn SignInView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    pending_invite: ReadSignal<Option<String>>,
    on_signed_in: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let (is_signup, set_is_signup) = signal(false);
    let (name, set_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);
    let (invite_preview, set_invite_preview) = signal::<Option<InvitePreview>>(None);

    Effect::new(move |_| {
        if let Some(code) = pending_invite.get() {
            leptos::task::spawn_local(async move {
                let encoded = js_sys::encode_uri_component(&code).as_string().unwrap_or(code);
                let url = format!("/api/invites/preview?code={encoded}");
                if let Ok(resp) = Request::get(&url).send().await {
                    if resp.ok() {
                        if let Ok(preview) = resp.json::<InvitePreview>().await {
                            if preview.valid {
                                set_invite_preview.set(Some(preview));
                            }
                        }
                    }
                }
            });
        }
    });

    let start_social = move |provider: &'static str| {
        let Some(auth_url) = auth_config.get_untracked().and_then(|c| c.auth_url) else {
            return;
        };
        set_error_msg.set(None);
        set_submitting.set(true);
        let inv = pending_invite.get_untracked();
        leptos::task::spawn_local(async move {
            let origin = web_sys::window().and_then(|w| w.location().origin().ok()).unwrap_or_default();
            let callback_url = match inv {
                Some(code) if !origin.is_empty() => {
                    let encoded = js_sys::encode_uri_component(&code).as_string().unwrap_or(code);
                    format!("{origin}/?invite={encoded}")
                }
                _ if !origin.is_empty() => format!("{origin}/"),
                _ => "/".to_string(),
            };
            let endpoint = format!("{}/sign-in/social", auth_url.trim_end_matches('/'));
            if let Ok(builder) = Request::post(&endpoint)
                .credentials(RequestCredentials::Include)
                .json(&serde_json::json!({ "provider": provider, "callbackURL": callback_url }))
            {
                if let Ok(resp) = builder.send().await {
                    if resp.ok() {
                        #[derive(Deserialize)]
                        struct SocialRes {
                            url: Option<String>,
                        }
                        if let Ok(body) = resp.json::<SocialRes>().await {
                            if let Some(redirect_url) = body.url {
                                if let Some(win) = web_sys::window() {
                                    let _ = win.location().assign(&redirect_url);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
            set_submitting.set(false);
        });
    };

    let submit_email = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(auth_url) = auth_config.get_untracked().and_then(|c| c.auth_url) else {
            return;
        };
        set_error_msg.set(None);
        set_submitting.set(true);
        let signup = is_signup.get_untracked();
        let user_email = email.get_untracked().trim().to_string();
        let user_pass = password.get_untracked();
        let user_name = name.get_untracked().trim().to_string();

        leptos::task::spawn_local(async move {
            let base = auth_url.trim_end_matches('/');
            let (endpoint, payload) = if signup {
                let fallback = user_email.split('@').next().unwrap_or("Author").to_string();
                (
                    format!("{base}/sign-up/email"),
                    serde_json::json!({
                        "email": user_email,
                        "password": user_pass,
                        "name": if user_name.is_empty() { fallback } else { user_name },
                    }),
                )
            } else {
                (
                    format!("{base}/sign-in/email"),
                    serde_json::json!({ "email": user_email, "password": user_pass }),
                )
            };
            if let Ok(req) = Request::post(&endpoint).credentials(RequestCredentials::Include).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        on_signed_in();
                        return;
                    }
                    set_error_msg.set(Some(resp.text().await.unwrap_or_else(|_| "Authentication failed.".to_string())));
                }
            }
            set_submitting.set(false);
        });
    };

    view! {
        <main class="page-main">
            <header class="page-head">
                <h1>"Sign in."</h1>
                <p>"Resolve your authoritative did:peer identity and open ExoNovel."</p>
            </header>

            <div class="exo-auth-wrap">
                {move || invite_preview.get().map(|inv| {
                    let av = inv.inviter_avatar_thumb_url.clone();
                    let init = inv.inviter_display_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                    let detail = match inv.group_name {
                        Some(g) => format!("{} invited you to join \"{g}\" on ExoNovel.", inv.inviter_display_name),
                        None => format!("{} invited you to join ExoNovel.", inv.inviter_display_name),
                    };
                    view! {
                        <div class="exo-invite-banner">
                            {if is_safe_avatar_url(&av) {
                                view! { <img src=av alt="Inviter" class="exo-avatar-sm" /> }.into_any()
                            } else {
                                view! { <div class="exo-avatar-sm">{init}</div> }.into_any()
                            }}
                            <div>
                                <div style="font-weight: 700;">"Personal Invitation"</div>
                                <div class="exo-sub" style="margin: 2px 0 0; color: var(--text);">{detail}</div>
                            </div>
                        </div>
                    }
                })}

                <div class="exo-card">
                    <h2 class="exo-h2">{move || if is_signup.get() { "Create account" } else { "Continue to ExoNovel" }}</h2>
                    <div class="exo-social-grid">
                        <button type="button" class="exo-btn exo-btn-google" disabled=move || submitting.get() on:click=move |_| start_social("google")>
                            <GoogleLogoIcon />
                            <span>"Google"</span>
                        </button>
                        <button type="button" class="exo-btn exo-btn-github" disabled=move || submitting.get() on:click=move |_| start_social("github")>
                            <GitHubLogoIcon />
                            <span>"GitHub"</span>
                        </button>
                    </div>

                    <div class="exo-divider">"or email"</div>
                    {move || error_msg.get().map(|msg| view! { <div class="exo-alert-error">{msg}</div> })}

                    <form on:submit=submit_email>
                        <Show when=move || is_signup.get()>
                            <div class="exo-field">
                                <label class="exo-label">"Name"</label>
                                <input type="text" class="exo-input" prop:value=move || name.get() on:input=move |ev| set_name.set(event_target_value(&ev)) />
                            </div>
                        </Show>
                        <div class="exo-field">
                            <label class="exo-label">"Email"</label>
                            <input type="email" class="exo-input" required=true prop:value=move || email.get() on:input=move |ev| set_email.set(event_target_value(&ev)) />
                        </div>
                        <div class="exo-field">
                            <label class="exo-label">"Password"</label>
                            <input type="password" class="exo-input" required=true minlength="8" prop:value=move || password.get() on:input=move |ev| set_password.set(event_target_value(&ev)) />
                        </div>
                        <button type="submit" class="exo-btn exo-btn-primary" style="width: 100%;" disabled=move || submitting.get()>
                            {move || if submitting.get() { "Authenticating..." } else if is_signup.get() { "Create account" } else { "Sign in" }}
                        </button>
                    </form>

                    <div style="margin-top: 16px; text-align: center;">
                        <button
                            type="button"
                            class="exo-btn"
                            style="border: 0; background: transparent; color: var(--accent);"
                            on:click=move |_| {
                                set_error_msg.set(None);
                                set_is_signup.update(|v| *v = !*v);
                            }
                        >
                            {move || if is_signup.get() { "Already have an account? Sign in" } else { "Need an account? Create one" }}
                        </button>
                    </div>
                </div>
            </div>
        </main>
    }
}

#[component]
fn PublicFooter() -> impl IntoView {
    view! {
        <footer class="exo-footer">
            <span>"ExoNovel · Operated by Exosystems (exosystems.net)"</span>
            <div class="exo-footer-links">
                <a href="/">"Home"</a>
                <a href="/privacy">"Privacy Policy"</a>
                <a href="/terms">"Terms of Service"</a>
                <a href="mailto:support@exosystems.net">"support@exosystems.net"</a>
            </div>
        </footer>
    }
}

#[component]
fn PrivacyPolicyView() -> impl IntoView {
    view! {
        <div class="exo-shell">
            <Title text="Privacy Policy · ExoNovel" />
            <nav class="navbar-solid">
                <a href="/" title="ExoNovel" class="navbar-brand-link">
                    <img src="/exonovel-logo.png" alt="ExoNovel" class="navbar-logo" />
                </a>
                <div class="navbar-menu">
                    <a href="/" class="nav-link">"Home"</a>
                    <a href="/privacy" class="nav-link nav-link-active">"Privacy Policy"</a>
                    <a href="/terms" class="nav-link">"Terms of Service"</a>
                </div>
            </nav>

            <main class="page-main" style="max-width: 860px; margin: 0 auto;">
                <div class="exo-card">
                    <div class="exo-eyebrow">"Last Updated: November 21, 2026"</div>
                    <h1 class="exo-h1" style="margin-top: 6px;">"ExoNovel Privacy Policy"</h1>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "ExoNovel (https://exonovel.exosystems.net) is a collaborative creative writing and local-first publishing platform operated by Exosystems (https://exosystems.net). This Privacy Policy explains what information ExoNovel collects, how Google user data and account credentials are used, how we protect your information, and how you can export or delete your data."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"1. Information We Collect & Google User Data"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "When you sign in to ExoNovel using Google OAuth (scopes: openid, email, and profile), we receive and store only your verified email address, display name, and profile picture URL. If you sign in with GitHub or email/password, or create author personas and creative works, we store your display name, pen name/handle, avatar image variants, language and notification preferences, and the cryptographic public identity (did:peer) associated with your workspace."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"2. How We Use Your Information"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "ExoNovel uses your account information strictly to authenticate your session, display your author or reader profile and personas, enable collaborative story groups and invitations you choose to join, and deliver transactional or notification emails you enable (sent via Brevo from notes@exosystems.net, noreply@exosystems.net, or support@exosystems.net). We do not use your Google profile data or creative writing content for advertising, behavioral profiling, or training third-party AI models."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"3. Google API Services User Data Policy (Limited Use)"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "ExoNovel's use and transfer to any other app of information received from Google APIs will adhere to the Google API Services User Data Policy, including the Limited Use requirements. We do not sell, rent, or transfer Google user data to data brokers or third parties for marketing."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"4. Data Storage, Infrastructure & Cryptographic Blindness"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "All network traffic to ExoNovel is encrypted via HTTPS/TLS. Hosted account records and Web Mode workspaces are persisted in Neon PostgreSQL and Neon Managed Auth. On device-native installs using peer-to-peer Willow and iroh relay synchronization, payloads are end-to-end encrypted on your device so ExoNovel's relay infrastructure remains cryptographically blind to relayed content."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"5. Data Retention, Export & Account Deletion"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "We retain your account information only while your account remains active or as required to resolve support requests and legal obligations. You may request full deletion or export of your account, Google OAuth profile metadata, and associated workspace records at any time from your ExoNovel account settings or by emailing support@exosystems.net (or exonomyapp@gmail.com). Verified deletion requests are processed promptly and remove active personal records from our hosted database."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"6. Contact Us"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "If you have privacy questions or data deletion requests, contact Exosystems Support at support@exosystems.net or exonomyapp@gmail.com."
                    </p>
                </div>
            </main>

            <PublicFooter />
        </div>
    }
}

#[component]
fn TermsOfServiceView() -> impl IntoView {
    view! {
        <div class="exo-shell">
            <Title text="Terms of Service · ExoNovel" />
            <nav class="navbar-solid">
                <a href="/" title="ExoNovel" class="navbar-brand-link">
                    <img src="/exonovel-logo.png" alt="ExoNovel" class="navbar-logo" />
                </a>
                <div class="navbar-menu">
                    <a href="/" class="nav-link">"Home"</a>
                    <a href="/privacy" class="nav-link">"Privacy Policy"</a>
                    <a href="/terms" class="nav-link nav-link-active">"Terms of Service"</a>
                </div>
            </nav>

            <main class="page-main" style="max-width: 860px; margin: 0 auto;">
                <div class="exo-card">
                    <div class="exo-eyebrow">"Last Updated: November 21, 2026"</div>
                    <h1 class="exo-h1" style="margin-top: 6px;">"ExoNovel Terms of Service"</h1>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "By accessing or using ExoNovel (https://exonovel.exosystems.net), operated by Exosystems, you agree to these Terms of Service."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"1. Ownership of Your Creative Works"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "You retain full ownership and copyright of all manuscripts, stories, characters, annotations, and creative assets you write or publish on ExoNovel. You grant ExoNovel only the limited technical license needed to store, display, synchronize, and transmit your content as directed by your workspace and sharing settings."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"2. Accounts, Keys & Acceptable Use"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "You are responsible for safeguarding your login credentials and sovereign did:peer device keys. You agree not to misuse ExoNovel for unlawful content, automated abuse, spam, or unauthorized access to other users' workspaces."
                    </p>

                    <h2 class="exo-h2" style="margin-top: 22px;">"3. Service Modes & Availability"</h2>
                    <p class="exo-sub" style="color: var(--text); line-height: 1.65;">
                        "ExoNovel provides hosted web collaboration, free local device storage, and optional paid cloud-relay backup plans. Features are provided on an 'as available' basis while the platform continues active development. For support or billing inquiries, contact support@exosystems.net."
                    </p>
                </div>
            </main>

            <PublicFooter />
        </div>
    }
}
