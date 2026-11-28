#![recursion_limit = "512"]

mod cms;
mod connected_apps_ui;
mod i18n;
mod live;
mod models;
mod preferences;
mod profile;
mod ui;

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use serde::Deserialize;
use web_sys::RequestCredentials;

use crate::{
    cms::DocsCmsView,
    i18n::{fetch_locale_catalog, fetch_locale_pack, use_i18n, I18nState, InputLocaleBar},
    live::{spawn_live_sse_connector, use_live_sync, LiveSyncState},
    models::{
        active_persona_for_user, copy_text_to_clipboard, detect_browser_timezone,
        display_name_for_user, is_safe_avatar_url, load_authenticated_user, read_session_storage,
        remove_session_storage, resolve_fresh_jwt, save_profile_patch, short_did_preview,
        sleep_ms, write_session_storage, ActiveTab, AppearanceMode, AuthConfigResponse, BellIcon,
        CopyIcon, GitHubLogoIcon, GoogleLogoIcon, GuideChapterDetail, GuideChapterSummary,
        GuidesListResponse, InvitePreview, LocaleFlag, MonitorIcon, MoonIcon, NotificationItem,
        NotificationsListResponse, PreferencesSection, ProfilePatchResponse, ProfileTab, SunIcon,
        VerifiedUser,
    },
    preferences::PreferencesView,
    profile::ProfileView,
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
    let i18n = I18nState::new();
    provide_context(i18n);

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            fetch_locale_catalog(i18n).await;
        });
    });

    // Apply Output UI Locale (<html lang dir="ltr|rtl">) and download .ftl pack
    Effect::new(move |_| {
        let code = i18n.ui_locale.get();
        let dir = i18n.ui_dir();
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

    view! {
        <Title text="ExoNovel — Collaborative Creative Writing & Sovereign Publishing" />
        <Router>
            <Routes fallback=|| view! { <HomeView initial_show_signin=false /> }>
                <Route path=path!("/") view=|| view! { <HomeView initial_show_signin=false /> } />
                <Route path=path!("/dear-world") view=|| view! { <HomeView initial_show_signin=false initial_guides=true /> } />
                <Route path=path!("/signin") view=|| view! { <HomeView initial_show_signin=true /> } />
                <Route path=path!("/privacy") view=PrivacyPolicyView />
                <Route path=path!("/terms") view=TermsOfServiceView />
            </Routes>
        </Router>
    }
}

#[component]
fn HomeView(
    #[prop(default = false)] initial_show_signin: bool,
    #[prop(default = false)] initial_guides: bool,
) -> impl IntoView {
    let live = LiveSyncState::new();
    provide_context(live);
    let i18n = use_i18n();

    let (auth_config, set_auth_config) = signal::<Option<AuthConfigResponse>>(None);
    let (loading_session, set_loading_session) = signal(true);
    let (show_signin, set_show_signin) = signal(initial_show_signin);
    let (show_guides, set_show_guides) = signal(initial_guides);
    let (current_user, set_current_user) = signal::<Option<VerifiedUser>>(None);
    let (session_jwt, set_session_jwt) = signal::<Option<String>>(None);
    let (active_tab, set_active_tab) = signal(if initial_guides {
        ActiveTab::Guides
    } else {
        ActiveTab::Workspace
    });
    let (appearance, set_appearance) = signal(AppearanceMode::System);
    let (active_doc_path, set_active_doc_path) = signal(String::new());
    let (return_tab, set_return_tab) = signal(if initial_guides {
        ActiveTab::Guides
    } else {
        ActiveTab::Workspace
    });
    let (chip_open, set_chip_open) = signal(false);
    let (bell_open, set_bell_open) = signal(false);
    let (notifications, set_notifications) = signal::<Vec<NotificationItem>>(Vec::new());
    let (unread_count, set_unread_count) = signal(0_usize);
    let (toast_msg, set_toast_msg) = signal::<Option<String>>(None);
    let (toast_seq, set_toast_seq) = signal(0_u64);
    let (pending_invite, set_pending_invite) = signal::<Option<String>>(None);
    let (sse_started, set_sse_started) = signal(false);
    let preferences_initial_section = RwSignal::new(None::<PreferencesSection>);
    let profile_initial_tab = RwSignal::new(None::<ProfileTab>);
    let (oauth_banner, set_oauth_banner) = signal::<Option<(bool, String)>>(None);

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
        show_toast(i18n.tr_untracked("toast_did_copied", "did:peer ID copied"));
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
                i18n.pinned_ui_locales
                    .set(updated.profile.preferred_ui_locales.clone());
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

    let apply_user_profile_state = move |user: &VerifiedUser| {
        i18n.persist_ui_locale.set(Some(Callback::new(move |next_ui: String| {
            persist_profile_patch(serde_json::json!({ "uiLocale": next_ui }));
        })));
        if let Some(prof) = &user.profile {
            if !prof.appearance_mode.is_empty() {
                set_appearance.set(AppearanceMode::from_str(&prof.appearance_mode));
            }
            if !prof.ui_locale.is_empty() {
                i18n.ui_locale.set(prof.ui_locale.clone());
            }
            i18n.pinned_ui_locales.set(prof.preferred_ui_locales.clone());
            if !prof.default_input_locale.is_empty() {
                i18n.input_locale.set(prof.default_input_locale.clone());
            }
            if !prof.last_doc_path.is_empty() && active_doc_path.get_untracked().is_empty() {
                set_active_doc_path.set(prof.last_doc_path.clone());
            }
            if !initial_guides {
                if prof.last_active_tab == "docs" && user.authorized {
                    set_return_tab.set(ActiveTab::Docs);
                } else if prof.last_active_tab == "guides" {
                    set_return_tab.set(ActiveTab::Guides);
                } else {
                    set_return_tab.set(ActiveTab::Workspace);
                }
            }
        }
    };

    // Parse URL params (invite/doc/tab/connected) once on mount
    Effect::new(move |_| {
        if let Some(win) = web_sys::window() {
            if let Ok(search) = win.location().search() {
                if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
                    let mut should_clean = false;
                    if let Some(doc_param) = params.get("doc").filter(|d| !d.trim().is_empty()) {
                        set_active_doc_path.set(doc_param);
                        set_active_tab.set(ActiveTab::Docs);
                    }
                    if let Some(tab_param) = params.get("tab") {
                        match tab_param.as_str() {
                            "preferences" => {
                                set_active_tab.set(ActiveTab::Preferences);
                                should_clean = true;
                            }
                            "docs" => {
                                set_active_tab.set(ActiveTab::Docs);
                                should_clean = true;
                            }
                            "guides" | "dear-world" => {
                                set_active_tab.set(ActiveTab::Guides);
                                set_show_guides.set(true);
                                should_clean = true;
                            }
                            _ => {}
                        }
                        params.delete("tab");
                    }
                    if let Some(sec_param) = params.get("section") {
                        if sec_param == "connected-apps" {
                            preferences_initial_section.set(Some(PreferencesSection::ConnectedApps));
                            set_active_tab.set(ActiveTab::Preferences);
                        }
                        params.delete("section");
                        should_clean = true;
                    }
                    if let Some(conn) = params.get("connected") {
                        if conn == "google" {
                            set_oauth_banner.set(Some((
                                false,
                                i18n.tr_untracked(
                                    "oauth_connected_google",
                                    "Google Drive & Docs connected successfully.",
                                ),
                            )));
                        }
                        params.delete("connected");
                        should_clean = true;
                    }
                    if let Some(err_param) = params.get("oauth_error") {
                        set_oauth_banner.set(Some((true, err_param)));
                        params.delete("oauth_error");
                        should_clean = true;
                    }
                    if should_clean {
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
                    if let Some(inv_param) = params.get("invite").filter(|i| !i.trim().is_empty()) {
                        write_session_storage("exo_pending_invite", &inv_param);
                        set_pending_invite.set(Some(inv_param));
                        set_show_signin.set(true);
                    }
                }
            }
        }
        if pending_invite.get_untracked().is_none() {
            if let Some(saved_inv) = read_session_storage("exo_pending_invite") {
                if !saved_inv.trim().is_empty() {
                    set_pending_invite.set(Some(saved_inv));
                    set_show_signin.set(true);
                }
            }
        }
    });

    // Apply selected theme cleanly to <html>
    Effect::new(move |_| {
        let mode = appearance.get();
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let _ = root.set_attribute("data-appearance", mode.as_str());
        }
    });

    // Start global real-time stream listener once signed in & react to notification events
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
        if current != ActiveTab::Preferences && current != ActiveTab::Profile {
            let tab_to_save = match current {
                ActiveTab::Docs => ActiveTab::Docs,
                ActiveTab::Guides => ActiveTab::Guides,
                _ => ActiveTab::Workspace,
            };
            set_return_tab.set(tab_to_save);
            let tab_str = match tab_to_save {
                ActiveTab::Docs => "docs",
                ActiveTab::Guides => "guides",
                _ => "workspace",
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
            i18n.persist_ui_locale.set(None);
            i18n.pinned_ui_locales.set(Vec::new());
            set_current_user.set(None);
            set_session_jwt.set(None);
            set_sse_started.set(false);
            set_show_signin.set(false);
            set_show_guides.set(false);
            set_active_tab.set(ActiveTab::Workspace);
        });
    };

    view! {
        <div class="exo-shell">
            <Show when=move || chip_open.get() || bell_open.get()>
                <div
                    class="exo-nav-backdrop"
                    on:click=move |_| {
                        set_chip_open.set(false);
                        set_bell_open.set(false);
                    }
                />
            </Show>

            <nav class="navbar-solid">
                <a
                    href="/"
                    title="ExoNovel"
                    class="navbar-brand-link"
                    on:click=move |_| {
                        set_show_signin.set(false);
                        set_show_guides.set(false);
                        set_chip_open.set(false);
                        set_bell_open.set(false);
                        set_active_tab.set(ActiveTab::Workspace);
                    }
                >
                    <img src="/exonovel-logo.png" alt="ExoNovel" class="navbar-logo navbar-logo-light" />
                    <img src="/exonovel-logo-dark.png" alt="ExoNovel" class="navbar-logo navbar-logo-dark" />
                </a>

                <div class="navbar-menu">
                    {move || match current_user.get() {
                        Some(user) => {
                            let can_read_docs = user.authorized;
                            let active_p = active_persona_for_user(&user);
                            let chip_name = display_name_for_user(&user);
                            let chip_aria = chip_name.clone();
                            let chip_alt = chip_name.clone();
                            let chip_avatar = if !active_p.avatar_thumb_url.is_empty() {
                                active_p.avatar_thumb_url.clone()
                            } else {
                                active_p.avatar_url.clone()
                            };
                            let chip_initial = chip_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                            let active_did_cmp = active_p.persona_did.clone();
                            let other_personas: Vec<_> = user
                                .personas
                                .clone()
                                .into_iter()
                                .filter(|p| p.persona_did != active_did_cmp)
                                .collect();

                            view! {
                                <>
                                    <div class="exo-desktop-nav-links">
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
                                            {move || i18n.tr("nav_desk", "Desk")}
                                        </button>

                                        <button
                                            type="button"
                                            class=move || if active_tab.get() == ActiveTab::Guides { "nav-link nav-link-active" } else { "nav-link" }
                                            on:click=move |_| {
                                                set_chip_open.set(false);
                                                set_bell_open.set(false);
                                                set_active_tab.set(ActiveTab::Guides);
                                                persist_profile_patch(serde_json::json!({ "lastActiveTab": "guides" }));
                                            }
                                        >
                                            {move || i18n.tr("nav_dear_world", "Dear World")}
                                        </button>
                                    </div>

                                    <div class="exo-bell-wrap">
                                        <button
                                            type="button"
                                            class="ctl"
                                            aria-label=move || i18n.tr("notifications_aria", "Notifications")
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
                                                        {move || i18n.tr("mark_all_read", "Mark all read")}
                                                    </button>
                                                </div>
                                                {move || {
                                                    let list = notifications.get();
                                                    if list.is_empty() {
                                                        view! { <p class="exo-sub">{move || i18n.tr("no_notifications", "No notifications yet.")}</p> }.into_any()
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
                                                                            {move || i18n.tr("mark_read", "Mark read")}
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

                                    <span class="gadget-row">
                                        <button
                                            type="button"
                                            class="ctl"
                                            title=move || {
                                                let m = appearance.get();
                                                i18n.tr(m.label_key(), m.label())
                                            }
                                            aria-label=move || {
                                                let m = appearance.get();
                                                i18n.tr(m.label_key(), m.label())
                                            }
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

                                    <div class="exo-user-chip-wrap">
                                        <button
                                            type="button"
                                            class="exo-user-chip"
                                            aria-label=chip_aria
                                            aria-expanded=move || if chip_open.get() { "true" } else { "false" }
                                            on:click=move |_| {
                                                set_bell_open.set(false);
                                                set_chip_open.update(|v| *v = !*v);
                                            }
                                        >
                                            {if is_safe_avatar_url(&chip_avatar) {
                                                view! { <img src=chip_avatar alt=chip_alt class="exo-avatar-sm" /> }.into_any()
                                            } else {
                                                view! { <div class="exo-avatar-sm">{chip_initial}</div> }.into_any()
                                            }}
                                            <span class="exo-user-chip-name">{chip_name}</span>
                                            <span class="exo-user-chip-caret">"▾"</span>
                                        </button>

                                        <Show when=move || chip_open.get()>
                                            <div class="exo-chip-menu" role="menu">
                                                <div class="exo-mobile-nav-section">
                                                    <div class="exo-chip-menu-row">
                                                        <span class="exo-live-indicator">
                                                            <span class=move || if live.connected.get() { "exo-live-dot exo-live-dot-on" } else { "exo-live-dot" } />
                                                            <span>{move || i18n.tr("live_sync_label", "Live")}</span>
                                                        </span>
                                                    </div>
                                                    <button
                                                        type="button"
                                                        class=move || if active_tab.get() == ActiveTab::Workspace { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                                        on:click=move |_| {
                                                            set_chip_open.set(false);
                                                            set_active_tab.set(ActiveTab::Workspace);
                                                            persist_profile_patch(serde_json::json!({ "lastActiveTab": "workspace" }));
                                                        }
                                                    >
                                                        <span>{move || i18n.tr("nav_desk", "Desk")}</span>
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class=move || if active_tab.get() == ActiveTab::Guides { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                                        on:click=move |_| {
                                                            set_chip_open.set(false);
                                                            set_active_tab.set(ActiveTab::Guides);
                                                            persist_profile_patch(serde_json::json!({ "lastActiveTab": "guides" }));
                                                        }
                                                    >
                                                        <span>{move || i18n.tr("nav_dear_world", "Dear World")}</span>
                                                    </button>
                                                    <div class="exo-chip-menu-divider" />
                                                </div>

                                                <button
                                                    type="button"
                                                    class=move || if active_tab.get() == ActiveTab::Profile { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                                    on:click=move |_| {
                                                        profile_initial_tab.set(Some(ProfileTab::Overview));
                                                        navigate_to_subpage(ActiveTab::Profile);
                                                    }
                                                >
                                                    <span>{move || i18n.tr("nav_profile", "Profile")}</span>
                                                </button>

                                                {if can_read_docs {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class=move || if active_tab.get() == ActiveTab::Docs { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                                            on:click=move |_| {
                                                                set_chip_open.set(false);
                                                                set_active_tab.set(ActiveTab::Docs);
                                                                persist_profile_patch(serde_json::json!({ "lastActiveTab": "docs" }));
                                                            }
                                                        >
                                                            <span>{move || i18n.tr("nav_docs", "Docs & ADRs")}</span>
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! { <span /> }.into_any()
                                                }}

                                                <div class="exo-chip-menu-divider" />
                                                {if other_personas.is_empty() {
                                                    view! {
                                                        <div class="exo-chip-menu-row">
                                                            <span class="exo-chip-menu-row-text">
                                                                {move || i18n.tr("create_personas_prompt", "Create Personas in your Profile")}
                                                            </span>
                                                            <button
                                                                type="button"
                                                                class="exo-btn exo-chip-menu-row-btn"
                                                                on:click=move |_| {
                                                                    profile_initial_tab.set(Some(ProfileTab::Personas));
                                                                    navigate_to_subpage(ActiveTab::Profile);
                                                                }
                                                            >
                                                                {move || i18n.tr("tab_personas", "Personas")}
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <>
                                                            <div class="exo-chip-menu-label">{move || i18n.tr("switch_persona", "Switch Persona")}</div>
                                                            {other_personas.clone().into_iter().map(|p| {
                                                                let p_did = p.persona_did.clone();
                                                                let p_empty = p.display_name.is_empty();
                                                                let p_raw = p.display_name.clone();
                                                                let p_name = if p_empty {
                                                                    i18n.tr_untracked("default_persona_label", "Default Persona")
                                                                } else {
                                                                    p.display_name.clone()
                                                                };
                                                                let p_avatar = if !p.avatar_thumb_url.is_empty() { p.avatar_thumb_url.clone() } else { p.avatar_url.clone() };
                                                                let p_init = p_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                                                                let p_name_toast = p_name.clone();
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        class="exo-chip-menu-item"
                                                                        on:click=move |_| {
                                                                            set_chip_open.set(false);
                                                                            persist_profile_patch(serde_json::json!({ "activePersonaDid": p_did.clone() }));
                                                                            let prefix = i18n.tr_untracked("toast_switched_persona", "Switched persona");
                                                                            show_toast(format!("{prefix}: {p_name_toast}"));
                                                                        }
                                                                    >
                                                                        <span style="display: inline-flex; align-items: center; gap: 8px; min-width: 0;">
                                                                            {if is_safe_avatar_url(&p_avatar) {
                                                                                view! { <img src=p_avatar alt=p_name.clone() class="exo-avatar-sm" /> }.into_any()
                                                                            } else {
                                                                                view! { <div class="exo-avatar-sm">{p_init}</div> }.into_any()
                                                                            }}
                                                                            <span style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                                                                                {move || if p_empty {
                                                                                    i18n.tr("default_persona_label", "Default Persona")
                                                                                } else {
                                                                                    p_raw.clone()
                                                                                }}
                                                                            </span>
                                                                        </span>
                                                                    </button>
                                                                }
                                                            }).collect_view()}
                                                        </>
                                                    }.into_any()
                                                }}

                                                <div class="exo-chip-menu-divider" />
                                                <button
                                                    type="button"
                                                    class=move || if active_tab.get() == ActiveTab::Preferences { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                                    on:click=move |_| navigate_to_subpage(ActiveTab::Preferences)
                                                >
                                                    <span>{move || i18n.tr("nav_preferences", "Preferences")}</span>
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

                                    <div class="exo-live-indicator exo-desktop-live" title=move || i18n.tr("live_connection_title", "Global Real-Time Stream Connection")>
                                        <span class=move || if live.connected.get() { "exo-live-dot exo-live-dot-on" } else { "exo-live-dot" } />
                                        <span>{move || i18n.tr("live_sync_label", "Live")}</span>
                                    </div>
                                </>
                            }.into_any()
                        }
                        None => view! {
                            <>
                                <button
                                    type="button"
                                    class=move || if show_guides.get() && !show_signin.get() { "nav-link nav-link-active" } else { "nav-link" }
                                    on:click=move |_| {
                                        set_show_signin.set(false);
                                        set_show_guides.set(true);
                                    }
                                >
                                    {move || i18n.tr("nav_dear_world", "Dear World")}
                                </button>
                                <button
                                    type="button"
                                    class="exo-btn exo-btn-primary"
                                    on:click=move |_| {
                                        set_show_guides.set(false);
                                        set_show_signin.set(true);
                                    }
                                >
                                    {move || i18n.tr("nav_sign_in", "Sign in")}
                                </button>
                                <span class="gadget-row">
                                    <button
                                        type="button"
                                        class="ctl"
                                        title=move || {
                                            let m = appearance.get();
                                            i18n.tr(m.label_key(), m.label())
                                        }
                                        aria-label=move || {
                                            let m = appearance.get();
                                            i18n.tr(m.label_key(), m.label())
                                        }
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
                            </>
                        }.into_any(),
                    }}
                </div>
            </nav>

            {move || oauth_banner.get().map(|(is_err, msg)| {
                let cls = if is_err { "exo-alert-error" } else { "exo-invite-banner" };
                view! {
                    <div class="page-main" style="padding-bottom: 0; margin-bottom: 0;">
                        <div class=cls style="display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap;">
                            <span>{msg}</span>
                            <button type="button" class="exo-btn" style="min-height: 30px; font-size: 13px;" on:click=move |_| set_oauth_banner.set(None)>
                                {move || i18n.tr("dismiss_label", "Dismiss")}
                            </button>
                        </div>
                    </div>
                }
            })}

            {
                let session_view_key = Memo::new(move |_| {
                    current_user
                        .get()
                        .map(|u| (u.auth_user_id, u.authorized, u.is_admin))
                });
                move || match session_view_key.get() {
                    None => {
                        if show_signin.get() {
                            view! {
                                <SignInView
                                    auth_config=auth_config
                                    pending_invite=pending_invite
                                    loading_session=loading_session
                                    on_back_home=move || set_show_signin.set(false)
                                    on_signed_in=refresh_session
                                />
                            }.into_any()
                        } else if show_guides.get() {
                            view! {
                                <DearWorldGuidesView
                                    is_authenticated=false
                                    on_open_desk=move || set_show_guides.set(false)
                                    on_open_signin=move || {
                                        set_show_guides.set(false);
                                        set_show_signin.set(true);
                                    }
                                />
                            }.into_any()
                        } else {
                            view! {
                                <PublicLandingView
                                    on_open_signin=move || set_show_signin.set(true)
                                />
                            }.into_any()
                        }
                    }
                    Some((_uid, authorized, _is_admin)) => {
                        let tab = active_tab.get();
                        match tab {
                            ActiveTab::Guides => view! {
                                <DearWorldGuidesView
                                    is_authenticated=true
                                    on_open_desk=move || {
                                        set_active_tab.set(ActiveTab::Workspace);
                                        persist_profile_patch(serde_json::json!({ "lastActiveTab": "workspace" }));
                                    }
                                    on_open_signin=move || {}
                                />
                            }.into_any(),
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
                                    initial_tab=profile_initial_tab
                                    on_back=move || {
                                        let dest = return_tab.get_untracked();
                                        let is_auth = current_user.get_untracked().is_some_and(|u| u.authorized);
                                        if dest == ActiveTab::Docs && is_auth {
                                            set_active_tab.set(ActiveTab::Docs);
                                        } else if dest == ActiveTab::Guides {
                                            set_active_tab.set(ActiveTab::Guides);
                                        } else {
                                            set_active_tab.set(ActiveTab::Workspace);
                                        }
                                    }
                                />
                            }.into_any(),
                            ActiveTab::Preferences => {
                                if let Some(user) = current_user.get_untracked() {
                                    view! {
                                        <PreferencesView
                                            auth_config=auth_config
                                            initial_jwt=session_jwt
                                            user=user
                                            return_tab=return_tab
                                            on_copy_did=show_did_copied_toast
                                            on_toast=show_toast
                                            on_open_profile_plans=move || {
                                                profile_initial_tab.set(Some(ProfileTab::Plans));
                                                navigate_to_subpage(ActiveTab::Profile);
                                            }
                                            on_profile_updated=move |updated: ProfilePatchResponse| {
                                                i18n.pinned_ui_locales.set(updated.profile.preferred_ui_locales.clone());
                                                set_current_user.update(|u_opt| {
                                                    if let Some(u) = u_opt {
                                                        u.profile = Some(updated.profile);
                                                        u.personas = updated.personas;
                                                        u.active_persona = updated.active_persona;
                                                    }
                                                });
                                            }
                                            initial_section=preferences_initial_section
                                            on_back=move || {
                                                let dest = return_tab.get_untracked();
                                                let is_auth = current_user.get_untracked().is_some_and(|u| u.authorized);
                                                if dest == ActiveTab::Docs && is_auth {
                                                    set_active_tab.set(ActiveTab::Docs);
                                                } else if dest == ActiveTab::Guides {
                                                    set_active_tab.set(ActiveTab::Guides);
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
                                    on_copy_did=show_did_copied_toast
                                />
                            }.into_any(),
                        }
                    }
                }
            }

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
    on_copy_did: impl Fn(String) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let i18n = use_i18n();
    let live = use_live_sync();
    let (selected_format, set_selected_format) = signal("novel".to_string());
    let (scratch_title, set_scratch_title) = signal(String::new());
    let (scratch_body, set_scratch_body) = signal(String::new());
    let (scratch_saved, set_scratch_saved) = signal(false);

    view! {
        <main class="page-main">
            {move || {
                let Some(user) = current_user.get() else {
                    return view! { <div /> }.into_any();
                };
                let persona = active_persona_for_user(&user);
                let persona_did = if persona.persona_did.is_empty() {
                    user.did_peer
                        .clone()
                        .unwrap_or_else(|| i18n.tr("pending_did_label", "Pending did:peer"))
                } else {
                    persona.persona_did.clone()
                };
                let did_copy = persona_did.clone();
                let did_title = persona_did.clone();
                let did_preview = short_did_preview(&persona_did);
                let did_anim = persona_did.clone();
                let pinned_input = user
                    .profile
                    .as_ref()
                    .map(|p| p.preferred_input_locales.clone())
                    .unwrap_or_default();

                let starter_formats = [
                    ("novel", "format_novel_title", "Novels", "format_novel_desc", "Build entire worlds from scratch. Organize characters, plot twists, and chapters in one place so big stories never get messy.", "desk_starter_novel_hint", "Ready on your shelf: Story Outline · Character Cards · Chapter 1 Draft · World Timeline."),
                    ("article", "format_article_title", "Articles & Nonfiction", "format_article_desc", "Turn ideas and research into sharp essays, blog posts, or school papers with clear outlines and neat sources.", "desk_starter_article_hint", "Ready on your shelf: Thesis Hook · Section Outline · Source Notes · Final Essay Draft."),
                    ("short", "format_short_title", "Short Stories", "format_short_desc", "Nail the hook, the twist, and the ending—great for flash fiction, standalone stories, or your own story collection.", "desk_starter_short_hint", "Ready on your shelf: Opening Hook · Turning Point · Character Beat · Scene Draft."),
                    ("screenplay", "format_screenplay_title", "Screenplays & Scripts", "format_screenplay_desc", "Write scenes and dialogue that look like real movie or video scripts, formatted cleanly as you type.", "desk_starter_script_hint", "Ready on your shelf: Scene Heading · Cast & Voices · Beat Sheet · Script Pages."),
                    ("poetry", "format_poetry_title", "Poetry", "format_poetry_desc", "Shape rhythm, stanzas, and line breaks that stay right where you put them—from a single poem to a full book.", "desk_starter_poetry_hint", "Ready on your shelf: Stanza Draft · Line Rhythm Notes · Collection Order · Author Dedication."),
                ];

                view! {
                    <div class=move || format!("exo-card exo-desk-card {}", live.anim_class("profile", &did_anim))>
                        <div class="exo-desk-header">
                            <div style="min-width: 0; flex: 1;">
                                <h1 class="exo-h1">{move || i18n.tr("nav_desk", "Desk")}</h1>
                                <p class="exo-sub" style="margin-top: 10px; max-width: 72ch;">
                                    {move || i18n.tr(
                                        "workspace_desc",
                                        "Your writing desk keeps everything you're working on close at hand—whether you're outlining your very first story for class, shaping characters with friends, or drafting poems on your own. Every character, timeline, scene, and comment you add stays credited to your pen name, so you can share ideas freely, learn together, and know your work always remains yours.",
                                    )}
                                </p>
                            </div>
                            <div class="exo-desk-did-chip">
                                <button
                                    type="button"
                                    class="exo-copy-btn"
                                    title=move || i18n.tr("copy_text_label", "Copy Text")
                                    aria-label=move || i18n.tr("copy_text_label", "Copy Text")
                                    on:click=move |_| on_copy_did(did_copy.clone())
                                >
                                    <CopyIcon />
                                </button>
                                <span class="exo-mono" title=did_title>{did_preview}</span>
                            </div>
                        </div>

                        <div class="exo-desk-sections">
                            <section class="exo-desk-section">
                                <div class="exo-eyebrow">{move || i18n.tr("desk_start_eyebrow", "Start a New Project")}</div>
                                <h2 class="exo-h2">{move || i18n.tr("desk_start_title", "What would you like to write today?")}</h2>
                                <div class="exo-desk-starter-grid">
                                    {starter_formats.into_iter().map(|(id, t_key, t_fb, d_key, d_fb, _, _)| {
                                        let id_str = id.to_string();
                                        let id_cmp = id_str.clone();
                                        let id_click = id_str.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class=move || if selected_format.get() == id_cmp {
                                                    "exo-desk-starter-btn exo-desk-starter-btn-active"
                                                } else {
                                                    "exo-desk-starter-btn"
                                                }
                                                on:click=move |_| set_selected_format.set(id_click.clone())
                                            >
                                                <strong>{move || i18n.tr(t_key, t_fb)}</strong>
                                                <span>{move || i18n.tr(d_key, d_fb)}</span>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                                <div class="exo-dep-card" style="margin-top: 12px;">
                                    {move || {
                                        let cur = selected_format.get();
                                        let (_, t_key, t_fb, _, _, h_key, h_fb) = starter_formats
                                            .iter()
                                            .find(|(id, ..)| *id == cur)
                                            .copied()
                                            .unwrap_or(starter_formats[0]);
                                        format!("{} — {}", i18n.tr(t_key, t_fb), i18n.tr(h_key, h_fb))
                                    }}
                                </div>
                            </section>

                            <section class="exo-desk-section">
                                <div class="exo-eyebrow">{move || i18n.tr("desk_blocks_eyebrow", "Story Building Blocks")}</div>
                                <h2 class="exo-h2">{move || i18n.tr("desk_blocks_title", "Your Shelf of Story Pieces")}</h2>
                                <div class="exo-grid-2">
                                    <div class="exo-format-tile">
                                        <h3>{move || i18n.tr("desk_block_chars_title", "Characters")}</h3>
                                        <p>{move || i18n.tr("desk_block_chars_desc", "Keep track of heroes, rivals, sidekicks, and their backstories—even bring a friend's character into a scene with clear credit.")}</p>
                                    </div>
                                    <div class="exo-format-tile">
                                        <h3>{move || i18n.tr("desk_block_plots_title", "Plots")}</h3>
                                        <p>{move || i18n.tr("desk_block_plots_desc", "Sketch turning points, chapter beats, and surprises so your story moves smoothly from beginning to ending.")}</p>
                                    </div>
                                    <div class="exo-format-tile">
                                        <h3>{move || i18n.tr("desk_block_timelines_title", "Timelines")}</h3>
                                        <p>{move || i18n.tr("desk_block_timelines_desc", "Line up what happened first, next, and years before—helpful when writing mysteries, trilogies, or class projects together.")}</p>
                                    </div>
                                    <div class="exo-format-tile">
                                        <h3>{move || i18n.tr("desk_block_themes_title", "Themes")}</h3>
                                        <p>{move || i18n.tr("desk_block_themes_desc", "Pin the big questions, moods, and motifs at the heart of your writing so every co-writer stays on the same page.")}</p>
                                    </div>
                                </div>
                            </section>

                            <section class="exo-desk-section exo-desk-scratchpad">
                                <div class="exo-eyebrow">{move || i18n.tr("desk_scratchpad_eyebrow", "Scratchpad")}</div>
                                <h2 class="exo-h2">{move || i18n.tr("desk_scratchpad_title", "Jot Down a Quick Idea")}</h2>
                                <InputLocaleBar
                                    context_label=i18n.tr("desk_scratchpad_label", "Quick Idea Scratchpad")
                                    pinned_codes=pinned_input
                                />
                                <div class="exo-field">
                                    <label class="exo-label">{move || i18n.tr("desk_note_title_label", "Working title or prompt")}</label>
                                    <input
                                        type="text"
                                        class="exo-input"
                                        dir=move || i18n.input_dir()
                                        placeholder=move || i18n.tr("desk_note_title_ph", "e.g. Rain over the library steps...")
                                        prop:value=move || scratch_title.get()
                                        on:input=move |ev| {
                                            set_scratch_saved.set(false);
                                            set_scratch_title.set(event_target_value(&ev));
                                        }
                                    />
                                </div>
                                <div class="exo-field">
                                    <label class="exo-label">{move || i18n.tr("desk_note_body_label", "Opening lines or scene sketch")}</label>
                                    <textarea
                                        class="exo-input exo-textarea"
                                        dir=move || i18n.input_dir()
                                        placeholder=move || i18n.tr("desk_note_body_ph", "Write a few sentences, test dialogue, or switch your writing language above...")
                                        prop:value=move || scratch_body.get()
                                        on:input=move |ev| {
                                            set_scratch_saved.set(false);
                                            set_scratch_body.set(event_target_value(&ev));
                                        }
                                    />
                                </div>
                                <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
                                    <button
                                        type="button"
                                        class="exo-btn exo-btn-primary"
                                        on:click=move |_| set_scratch_saved.set(true)
                                    >
                                        {move || i18n.tr("desk_note_keep_btn", "Pin Idea on Desk")}
                                    </button>
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        on:click=move |_| {
                                            set_scratch_title.set(String::new());
                                            set_scratch_body.set(String::new());
                                            set_scratch_saved.set(false);
                                        }
                                    >
                                        {move || i18n.tr("desk_note_clear_btn", "Clear")}
                                    </button>
                                    <Show when=move || scratch_saved.get()>
                                        <span class="exo-tag exo-tag-ok">
                                            {move || i18n.tr("desk_note_pinned_msg", "Idea pinned on your desk for this session")}
                                        </span>
                                    </Show>
                                </div>
                            </section>
                        </div>
                    </div>
                }.into_any()
            }}
        </main>
    }
}

#[component]
fn DearWorldGuidesView(
    is_authenticated: bool,
    on_open_desk: impl Fn() + Copy + Send + Sync + 'static,
    on_open_signin: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let i18n = use_i18n();
    let (guides, set_guides) = signal::<Vec<GuideChapterSummary>>(Vec::new());
    let (selected_slug, set_selected_slug) = signal("01-dear-world".to_string());
    let (active_guide, set_active_guide) = signal::<Option<GuideChapterDetail>>(None);
    let (loading, set_loading) = signal(true);

    Effect::new(move |_| {
        let loc = i18n.ui_locale.get();
        let slug = selected_slug.get();
        set_loading.set(true);
        leptos::task::spawn_local(async move {
            let mut target_slug = slug.clone();
            let list_url = format!("/api/guides?locale={loc}");
            if let Ok(resp) = Request::get(&list_url).send().await {
                if resp.ok() {
                    if let Ok(data) = resp.json::<GuidesListResponse>().await {
                        if !data.guides.is_empty()
                            && !data.guides.iter().any(|g| g.slug == target_slug)
                        {
                            target_slug = data.guides[0].slug.clone();
                            set_selected_slug.set(target_slug.clone());
                        }
                        set_guides.set(data.guides);
                    }
                }
            }
            let detail_url = format!("/api/guides/{target_slug}?locale={loc}");
            if let Ok(resp) = Request::get(&detail_url).send().await {
                if resp.ok() {
                    if let Ok(detail) = resp.json::<GuideChapterDetail>().await {
                        set_active_guide.set(Some(detail));
                    }
                }
            }
            set_loading.set(false);
        });
    });

    view! {
        <main class="page-main">
            <div class="exo-guides-layout">
                <aside class="exo-guides-sidebar">
                    <div class="exo-eyebrow">{move || i18n.tr("guides_eyebrow", "Dear World · User Guides")}</div>
                    <h2 class="exo-h2" style="margin-bottom: 10px;">{move || i18n.tr("guides_sidebar_title", "Getting Started")}</h2>
                    <p class="exo-sub" style="margin-top: 0; margin-bottom: 14px; font-size: 15px;">
                        {move || i18n.tr("guides_sidebar_sub", "Step-by-step guides written for High School and University writers exploring collaborative storytelling on ExoNovel.")}
                    </p>
                    <div style="display: flex; flex-direction: column; gap: 6px;">
                        {move || {
                            let cur = selected_slug.get();
                            guides.get().into_iter().map(|g| {
                                let is_active = cur == g.slug;
                                let slug_click = g.slug.clone();
                                let mins_suffix = i18n.tr("cms_min_read_suffix", "min read");
                                view! {
                                    <button
                                        type="button"
                                        class=if is_active {
                                            "exo-guides-chapter-btn exo-guides-chapter-active"
                                        } else {
                                            "exo-guides-chapter-btn"
                                        }
                                        on:click=move |_| set_selected_slug.set(slug_click.clone())
                                    >
                                        <div style="font-weight: 600;">{format!("{}. {}", g.chapter_order, g.title)}</div>
                                        <div class="exo-eyebrow" style="margin: 2px 0 0; font-size: 13px;">
                                            {format!("{} {}", g.reading_minutes, mins_suffix)}
                                        </div>
                                    </button>
                                }
                            }).collect_view()
                        }}
                    </div>

                    <div style="margin-top: 18px; padding-top: 14px; border-top: 1px solid var(--border);">
                        {if is_authenticated {
                            view! {
                                <button
                                    type="button"
                                    class="exo-btn exo-btn-primary"
                                    style="width: 100%;"
                                    on:click=move |_| on_open_desk()
                                >
                                    {move || i18n.tr("guides_open_desk_btn", "Go to Your Desk")}
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    type="button"
                                    class="exo-btn exo-btn-primary"
                                    style="width: 100%;"
                                    on:click=move |_| on_open_signin()
                                >
                                    {move || i18n.tr("guides_signin_btn", "Sign in to Start Writing")}
                                </button>
                            }.into_any()
                        }}
                    </div>
                </aside>

                <article class="exo-card exo-guides-article">
                    {move || {
                        if loading.get() && active_guide.get().is_none() {
                            return view! {
                                <p class="exo-sub">{i18n.tr("guides_loading", "Opening your Dear World guide...")}</p>
                            }.into_any();
                        }
                        let Some(ch) = active_guide.get() else {
                            return view! {
                                <p class="exo-sub">{i18n.tr("guides_empty", "Select a guide chapter from the list to begin.")}</p>
                            }.into_any();
                        };
                        let all = guides.get();
                        let pos = all.iter().position(|g| g.slug == ch.slug);
                        let prev_item = pos.and_then(|i| if i > 0 { all.get(i - 1).cloned() } else { None });
                        let next_item = pos.and_then(|i| all.get(i + 1).cloned());

                        view! {
                            <>
                                <div style="display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap; margin-bottom: 14px;">
                                    <span class="exo-eyebrow" style="margin: 0;">
                                        {format!(
                                            "{} · {} {} · {} {}",
                                            ch.audience,
                                            ch.word_count,
                                            i18n.tr("cms_words_suffix", "words"),
                                            ch.reading_minutes,
                                            i18n.tr("cms_min_read_suffix", "min read")
                                        )}
                                    </span>
                                    {(ch.translation_status == "en-fallback").then(|| view! {
                                        <span class="exo-trans-badge exo-trans-fallback">
                                            {i18n.tr("cms_trans_fallback_en", "English Fallback")}
                                        </span>
                                    })}
                                </div>

                                <div class="exo-markdown" inner_html=ch.html />

                                <div class="exo-prev-next">
                                    <div>
                                        {prev_item.map(|p| {
                                            let s = p.slug.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class="exo-btn"
                                                    style="width: 100%; justify-content: flex-start;"
                                                    on:click=move |_| set_selected_slug.set(s.clone())
                                                >
                                                    {format!("← {}", p.title)}
                                                </button>
                                            }
                                        })}
                                    </div>
                                    <div>
                                        {next_item.map(|n| {
                                            let s = n.slug.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class="exo-btn exo-btn-primary"
                                                    style="width: 100%; justify-content: flex-end;"
                                                    on:click=move |_| set_selected_slug.set(s.clone())
                                                >
                                                    {format!("{} →", n.title)}
                                                </button>
                                            }
                                        })}
                                    </div>
                                </div>
                            </>
                        }.into_any()
                    }}
                </article>
            </div>
        </main>
    }
}

#[component]
fn PublicLandingView(
    on_open_signin: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let i18n = use_i18n();
    let (slide_idx, set_slide_idx) = signal(0_usize);
    let (card_menu_open, set_card_menu_open) = signal(false);
    let slides = [
        (
            "slide_1_tab",
            "Your Writing, Your Rights",
            "slide_1_title",
            "Create Together, But Remain the Owner of Each of Your Ideas",
            "slide_1_p1",
            "ExoNovel is a creative writing space where you can build stories on your own or team up with friends, co-writers, and writing groups in real time. Instead of losing track of who wrote what inside messy shared documents, every character profile, plot outline, timeline, theme, and scene remains clearly and securely credited to its creator, including the comments you make on the creations of others.",
            "slide_1_p2",
            "That means collaborating never costs you your work. You decide what remains private, what your group can edit together, and what other writers can borrow and remix into their own stories—with your entirely independent decentralized identity permanently attached to what you made.",
        ),
        (
            "slide_2_tab",
            "One Producer, Many Personas",
            "slide_2_title",
            "One Identity, as Many Personas as You Want",
            "slide_2_p1",
            "Writing a sci-fi story with friends, sharing poetry under a pen name, and drafting nonfiction under your real name shouldn't force you to juggle three separate accounts. ExoNovel supports creating many Personas—each with its own display name/handle, bio, and avatar, operating also as a content proxy (meaning that you can publish as your Persona), and with its own access to your community of readers and collaborators, so that you can even chat as your Persona about your own work and the work of others. ExoNovel creators need not disclose their true identity to the world, but may do so if they wish—all under one entirely independent login.",
            "slide_2_p2",
            "Switch personas instantly to switch to another Persona workspace, with its own projects, group chats, or genres. Collaborators and readers see the exact Persona display name you chose for each space, while everything you write across all of your Persona spaces remains safely organized and permanently legitimately yours, thanks to your Identity's securely decentralized P2P ID.",
        ),
        (
            "slide_3_tab",
            "Write, Wherever You Are",
            "slide_3_title",
            "Online and Offline, Collaborate Across Any Screen",
            "slide_3_p1",
            "Create right in your browser whenever you're online, or continue writing without the web using our free Linux, macOS, Windows, Android, and iOS apps. Whenever you collaborate with fellow writers, everyone stays synced automatically—even if one co-writer is in a web browser and another is working in a device app.",
            "slide_3_p2",
            "Your own personal sync between your web account and your personal offline devices is always available manually on demand, or automatically for a small monthly fee (or even smaller annual fee) in Profile -> Plans.",
        ),
        (
            "slide_4_tab",
            "Connect Your Way",
            "slide_4_title",
            "Connect Your Tools, Keep Total Freedom",
            "slide_4_p1",
            "Your writing shouldn't be trapped inside a single platform. Sign in the way you prefer, write in the formats that fit your project, and link external cloud storage, tools, and workflows for backups, quick exports, publishing, cross-platform projects, or whatever else your creative process needs.",
            "slide_4_p2",
            "Everything in ExoNovel works together without locking you in. Add connections whenever they help, combine tools as your projects grow, and disconnect any service anytime while enjoying full access to your account, personas, and creations.",
        ),
    ];

    view! {
        <main class="page-main">
            <div class="exo-landing-stack">
                <section id="overview" class="exo-landing-hero">
                    <div class="exo-eyebrow">
                        {move || i18n.tr("landing_eyebrow", "ExoNovel · Web, Desktop & Mobile")}
                    </div>
                    <h1 class="exo-landing-title">
                        <span class="exo-landing-title-sub">
                            {move || i18n.tr("landing_title_row1", "Collaborative Creative Writing")}
                        </span>
                        <span class="exo-landing-title-anchor">
                            <span class="exo-landing-title-sub">
                                {move || i18n.tr("landing_title_row2", "Where Authors Own Their Work")}
                            </span>
                            <span class="exo-landing-title-punch">
                                {move || i18n.tr("landing_title_row3", "... by Design!")}
                            </span>
                        </span>
                    </h1>
                    <p class="exo-landing-lead">
                        {move || i18n.tr(
                            "landing_lead",
                            "ExoNovel is a collaborative creative writing and local-first publishing platform built for authors, co-writers, and literary teams. Draft novels, essays, short stories, screenplays, and poetry together while keeping cryptographic ownership of your manuscripts."
                        )}
                    </p>
                    <div class="exo-landing-actions">
                        <button
                            type="button"
                            class="exo-btn exo-btn-primary"
                            on:click=move |_| on_open_signin()
                        >
                            {move || i18n.tr("landing_cta", "Sign in to ExoNovel")}
                        </button>
                    </div>
                </section>

                <section class="exo-carousel" aria-label=move || i18n.tr("landing_cap_aria", "Platform capabilities")>
                    <div class="exo-carousel-head">
                        <div class="exo-carousel-tabs" role="tablist">
                            {slides.into_iter().enumerate().map(|(idx, (tab_key, tab_fb, _, _, _, _, _, _))| {
                                view! {
                                    <button
                                        type="button"
                                        role="tab"
                                        aria-selected=move || if slide_idx.get() == idx { "true" } else { "false" }
                                        class=move || if slide_idx.get() == idx {
                                            "exo-carousel-tab exo-carousel-tab-active"
                                        } else {
                                            "exo-carousel-tab"
                                        }
                                        on:click=move |_| {
                                            set_card_menu_open.set(false);
                                            set_slide_idx.set(idx);
                                        }
                                    >
                                        <span class="exo-carousel-tab-title">{move || i18n.tr(tab_key, tab_fb)}</span>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <div class="exo-carousel-dropdown-wrap">
                            <Show when=move || card_menu_open.get()>
                                <div
                                    class="exo-nav-backdrop"
                                    on:click=move |_| set_card_menu_open.set(false)
                                />
                            </Show>
                            <button
                                type="button"
                                class="exo-carousel-dropdown-trigger"
                                aria-haspopup="menu"
                                aria-expanded=move || if card_menu_open.get() { "true" } else { "false" }
                                aria-label=move || i18n.tr("landing_cap_aria", "Platform capabilities")
                                on:click=move |_| set_card_menu_open.update(|v| *v = !*v)
                            >
                                <span class="exo-carousel-dropdown-label">
                                    {move || {
                                        let (tab_key, tab_fb, _, _, _, _, _, _) = slides[slide_idx.get()];
                                        i18n.tr(tab_key, tab_fb)
                                    }}
                                </span>
                                <span class="exo-carousel-dropdown-caret" aria-hidden="true">"▾"</span>
                            </button>
                            <Show when=move || card_menu_open.get()>
                                <div class="exo-carousel-dropdown-menu" role="menu">
                                    {slides.into_iter().enumerate().map(|(idx, (tab_key, tab_fb, _, _, _, _, _, _))| {
                                        view! {
                                            <button
                                                type="button"
                                                role="menuitem"
                                                class=move || if slide_idx.get() == idx {
                                                    "exo-chip-menu-item exo-chip-menu-item-active"
                                                } else {
                                                    "exo-chip-menu-item"
                                                }
                                                on:click=move |_| {
                                                    set_slide_idx.set(idx);
                                                    set_card_menu_open.set(false);
                                                }
                                            >
                                                <span>{move || i18n.tr(tab_key, tab_fb)}</span>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </Show>
                        </div>
                        <div class="exo-carousel-nav">
                            <button
                                type="button"
                                class="exo-btn"
                                style="min-height: 36px; padding: 0 12px; font-size: 15px;"
                                aria-label=move || i18n.tr("landing_prev_slide", "Previous slide")
                                on:click=move |_| {
                                    set_card_menu_open.set(false);
                                    set_slide_idx.update(|i| *i = (*i + 3) % 4);
                                }
                            >
                                "←"
                            </button>
                            <button
                                type="button"
                                class="exo-btn"
                                style="min-height: 36px; padding: 0 12px; font-size: 15px;"
                                aria-label=move || i18n.tr("landing_next_slide", "Next slide")
                                on:click=move |_| {
                                    set_card_menu_open.set(false);
                                    set_slide_idx.update(|i| *i = (*i + 1) % 4);
                                }
                            >
                                "→"
                            </button>
                        </div>
                    </div>

                    {move || {
                        let (_, _, t_key, t_fb, p1_key, p1_fb, p2_key, p2_fb) = slides[slide_idx.get()];
                        view! {
                            <article class="exo-carousel-body">
                                <h3>{move || i18n.tr(t_key, t_fb)}</h3>
                                <p>{move || i18n.tr(p1_key, p1_fb)}</p>
                                <p>{move || i18n.tr(p2_key, p2_fb)}</p>
                            </article>
                        }
                    }}
                </section>

                <section id="formats" class="exo-card">
                    <h2 class="exo-h2">{move || i18n.tr("landing_formats_title", "Pick Your Format. Start Creating.")}</h2>
                    <p class="exo-sub">
                        {move || i18n.tr(
                            "landing_formats_sub",
                            "Whether you're writing a novel for fun, drafting a class essay, or scripting a short film, start with a clean layout built for the way you create."
                        )}
                    </p>
                    <div class="exo-format-grid">
                        <div class="exo-format-tile">
                            <h3>{move || i18n.tr("format_novel_title", "Novels")}</h3>
                            <p>{move || i18n.tr("format_novel_desc", "Build entire worlds from scratch. Organize characters, plot twists, and chapters in one place so big stories never get messy.")}</p>
                        </div>
                        <div class="exo-format-tile">
                            <h3>{move || i18n.tr("format_article_title", "Articles & Nonfiction")}</h3>
                            <p>{move || i18n.tr("format_article_desc", "Turn ideas and research into sharp essays, blog posts, or school papers with clear outlines and neat sources.")}</p>
                        </div>
                        <div class="exo-format-tile">
                            <h3>{move || i18n.tr("format_short_title", "Short Stories")}</h3>
                            <p>{move || i18n.tr("format_short_desc", "Nail the hook, the twist, and the ending—great for flash fiction, standalone stories, or your own story collection.")}</p>
                        </div>
                        <div class="exo-format-tile">
                            <h3>{move || i18n.tr("format_screenplay_title", "Screenplays & Scripts")}</h3>
                            <p>{move || i18n.tr("format_screenplay_desc", "Write scenes and dialogue that look like real movie or video scripts, formatted cleanly as you type.")}</p>
                        </div>
                        <div class="exo-format-tile">
                            <h3>{move || i18n.tr("format_poetry_title", "Poetry")}</h3>
                            <p>{move || i18n.tr("format_poetry_desc", "Shape rhythm, stanzas, and line breaks that stay right where you put them—from a single poem to a full book.")}</p>
                        </div>
                    </div>
                </section>

                <section id="sync" class="exo-card">
                    <h2 class="exo-h2">{move || i18n.tr("landing_modes_title", "On the Web, On Your Device, and Synced Together")}</h2>
                    <p class="exo-sub">
                        {move || i18n.tr(
                            "landing_modes_sub",
                            "Write for free in your web browser and in our native device apps. Collaborating with other writers syncs automatically across web and device for free—while keeping your own personal web and device shelves synced automatically is a small monthly or annual fee.",
                        )}
                    </p>
                    <div class="exo-grid-3">
                        <div class="exo-format-tile">
                            <div class="exo-eyebrow">{move || i18n.tr("mode_free_badge", "Free Forever")}</div>
                            <h3>{move || i18n.tr("mode_free_title", "On the Web & On Your Device")}</h3>
                            <p>{move || i18n.tr("mode_free_desc", "Create, invite co-writers, and collaborate across web browsers and our free Linux, macOS, Windows, Android, and iOS apps—with free automatic collaboration sync between co-writers on web and device.")}</p>
                        </div>
                        <div class="exo-format-tile">
                            <div class="exo-eyebrow">{move || i18n.tr("mode_monthly_badge", "Personal Sync")}</div>
                            <h3>{move || i18n.tr("mode_monthly_title", "Personal Sync — $9 / month")}</h3>
                            <p>{move || i18n.tr("mode_monthly_desc", "Keep your own personal drafts on the web and on your offline devices automatically synced with end-to-end encryption ($9/month local currency equivalent).")}</p>
                        </div>
                        <div class="exo-format-tile">
                            <div class="exo-eyebrow">{move || i18n.tr("mode_annual_badge", "Annual Personal Sync")}</div>
                            <h3>{move || i18n.tr("mode_annual_title", "Personal Sync — $90 / year")}</h3>
                            <p>{move || i18n.tr("mode_annual_desc", "Full automatic sync between your own web profile and your personal devices ($7.50/month effective rate in local currency equivalent—two months free compared to monthly).")}</p>
                        </div>
                    </div>
                </section>
            </div>
        </main>
    }
}

#[component]
fn SignInView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    pending_invite: ReadSignal<Option<String>>,
    loading_session: ReadSignal<bool>,
    on_back_home: impl Fn() + Copy + Send + Sync + 'static,
    on_signed_in: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let i18n = use_i18n();
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
        let auth_failed = i18n.tr_untracked("signin_auth_failed", "Authentication failed.");

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
                    set_error_msg.set(Some(resp.text().await.unwrap_or(auth_failed)));
                }
            }
            set_submitting.set(false);
        });
    };

    view! {
        <main class="page-main">
            <header class="page-head">
                <div style="margin-bottom: 14px;">
                    <button
                        type="button"
                        class="exo-btn"
                        on:click=move |_| on_back_home()
                    >
                        {move || i18n.tr("signin_back_home", "← Back")}
                    </button>
                </div>
                <h1>{move || i18n.tr("signin_title", "Sign in to ExoNovel")}</h1>
                <p>{move || i18n.tr("signin_subtitle", "Authenticate to resolve your authoritative did:peer identity and open your workspace.")}</p>
                <Show when=move || loading_session.get()>
                    <p class="exo-sub">{move || i18n.tr("signin_checking_session", "Checking existing session...")}</p>
                </Show>
            </header>

            <div class="exo-auth-wrap">
                {move || invite_preview.get().map(|inv| {
                    let av = inv.inviter_avatar_thumb_url.clone();
                    let inviter_name = inv.inviter_display_name.clone();
                    let group_name = inv.group_name.clone();
                    let init = inviter_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                    view! {
                        <div class="exo-invite-banner">
                            {if is_safe_avatar_url(&av) {
                                view! { <img src=av alt=inviter_name.clone() class="exo-avatar-sm" /> }.into_any()
                            } else {
                                view! { <div class="exo-avatar-sm">{init}</div> }.into_any()
                            }}
                            <div>
                                <div style="font-weight: 700;">{move || i18n.tr("signin_invite_badge", "Personal Invitation")}</div>
                                <div class="exo-sub" style="margin: 2px 0 0; color: var(--text);">
                                    {move || match &group_name {
                                        Some(g) => format!(
                                            "{} {} \"{g}\"",
                                            inviter_name,
                                            i18n.tr("signin_invited_group", "invited you to join this group on ExoNovel:")
                                        ),
                                        None => format!(
                                            "{} {}",
                                            inviter_name,
                                            i18n.tr("signin_invited_platform", "invited you to join ExoNovel.")
                                        ),
                                    }}
                                </div>
                            </div>
                        </div>
                    }
                })}

                <div class="exo-card">
                    <h2 class="exo-h2">
                        {move || if is_signup.get() {
                            i18n.tr("signin_create_heading", "Create account")
                        } else {
                            i18n.tr("signin_continue_heading", "Continue to ExoNovel")
                        }}
                    </h2>
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

                    <div class="exo-divider">{move || i18n.tr("signin_or_email", "or email")}</div>
                    {move || error_msg.get().map(|msg| view! { <div class="exo-alert-error">{msg}</div> })}

                    <form on:submit=submit_email>
                        <Show when=move || is_signup.get()>
                            <div class="exo-field">
                                <label class="exo-label">{move || i18n.tr("signin_name_label", "Name")}</label>
                                <input type="text" class="exo-input" prop:value=move || name.get() on:input=move |ev| set_name.set(event_target_value(&ev)) />
                            </div>
                        </Show>
                        <div class="exo-field">
                            <label class="exo-label">{move || i18n.tr("signin_email_label", "Email")}</label>
                            <input type="email" class="exo-input" required=true prop:value=move || email.get() on:input=move |ev| set_email.set(event_target_value(&ev)) />
                        </div>
                        <div class="exo-field">
                            <label class="exo-label">{move || i18n.tr("signin_password_label", "Password")}</label>
                            <input type="password" class="exo-input" required=true minlength="8" prop:value=move || password.get() on:input=move |ev| set_password.set(event_target_value(&ev)) />
                        </div>
                        <button type="submit" class="exo-btn exo-btn-primary" style="width: 100%;" disabled=move || submitting.get()>
                            {move || if submitting.get() {
                                i18n.tr("signin_authenticating", "Authenticating...")
                            } else if is_signup.get() {
                                i18n.tr("signin_create_btn", "Create account")
                            } else {
                                i18n.tr("signin_submit_btn", "Sign in")
                            }}
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
                            {move || if is_signup.get() {
                                i18n.tr("signin_have_account", "Already have an account? Sign in")
                            } else {
                                i18n.tr("signin_need_account", "Need an account? Create one")
                            }}
                        </button>
                    </div>
                </div>
            </div>
        </main>
    }
}

#[component]
fn PublicFooter() -> impl IntoView {
    let i18n = use_i18n();
    let (lang_open, set_lang_open) = signal(false);

    view! {
        <>
            <Show when=move || lang_open.get()>
                <div
                    class="exo-nav-backdrop"
                    on:click=move |_| set_lang_open.set(false)
                />
            </Show>
            <footer class="exo-footer">
                <span class="exo-footer-brand">
                    {move || i18n.tr("footer_brand_prefix", "ExoNovel, from")}
                    " "
                    <a
                        href="https://exosystems.net"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="exo-stealth-brand-btn"
                    >
                        "Exosystems"
                    </a>
                </span>

                <div class="exo-footer-links">
                    <a href="/privacy">{move || i18n.tr("nav_privacy", "Privacy Policy")}</a>
                    <a href="/terms">{move || i18n.tr("nav_terms", "Terms of Service")}</a>
                    <a href="mailto:support@exosystems.net">"support@exosystems.net"</a>
                </div>

                <div class="exo-footer-lang-wrap">
                    <button
                        type="button"
                        class="exo-lang-trigger"
                        aria-label=move || i18n.tr("ui_lang_aria", "App Output Language")
                        aria-expanded=move || if lang_open.get() { "true" } else { "false" }
                        on:click=move |_| set_lang_open.update(|v| *v = !*v)
                    >
                        {move || {
                            let cur = i18n.ui_locale.get();
                            let name = i18n.catalog
                                .get()
                                .into_iter()
                                .find(|l| l.code == cur)
                                .map(|l| l.native_name)
                                .unwrap_or_else(|| cur.clone());
                            view! {
                                <>
                                    <LocaleFlag code=cur />
                                    <span class="exo-lang-trigger-name">{name}</span>
                                </>
                            }
                        }}
                        <span style="font-size: 12px; opacity: 0.75;">"▴"</span>
                    </button>

                    <Show when=move || lang_open.get()>
                        <div class="exo-chip-menu exo-lang-menu exo-lang-menu-up" role="menu">
                            <div class="exo-chip-menu-label">{move || i18n.tr("ui_lang_label", "App Language")}</div>
                            {move || {
                                let catalog = i18n.catalog.get();
                                let cur = i18n.ui_locale.get();
                                let pinned = i18n.pinned_ui_locales.get();
                                let list = if pinned.is_empty() {
                                    catalog.clone()
                                } else {
                                    let filtered: Vec<_> = catalog
                                        .iter()
                                        .filter(|l| pinned.contains(&l.code) || l.code == cur)
                                        .cloned()
                                        .collect();
                                    if filtered.is_empty() { catalog.clone() } else { filtered }
                                };
                                list.into_iter().map(|loc| {
                                    let code_click = loc.code.clone();
                                    let code_flag = loc.code.clone();
                                    let is_active = loc.code == cur;
                                    view! {
                                        <button
                                            type="button"
                                            class=if is_active { "exo-chip-menu-item exo-chip-menu-item-active" } else { "exo-chip-menu-item" }
                                            on:click=move |_| {
                                                let next = code_click.clone();
                                                i18n.ui_locale.set(next.clone());
                                                set_lang_open.set(false);
                                                if let Some(cb) = i18n.persist_ui_locale.get_untracked() {
                                                    cb.run(next);
                                                }
                                            }
                                        >
                                            <span class="exo-lang-item-label">
                                                <LocaleFlag code=code_flag />
                                                <span>{loc.native_name}</span>
                                            </span>
                                            <span class="exo-eyebrow" style="margin: 0; font-size: 12px;">{loc.code}</span>
                                        </button>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </Show>
                </div>
            </footer>
        </>
    }
}

#[component]
fn PrivacyPolicyView() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="exo-shell">
            <Title text=move || i18n.tr("privacy_page_title", "Privacy Policy — ExoNovel") />
            <nav class="navbar-solid">
                <a href="/" title="ExoNovel" class="navbar-brand-link">
                    <img src="/exonovel-logo.png" alt="ExoNovel" class="navbar-logo navbar-logo-light" />
                    <img src="/exonovel-logo-dark.png" alt="ExoNovel" class="navbar-logo navbar-logo-dark" />
                </a>
                <div class="navbar-menu">
                    <a href="/dear-world" class="nav-link">{move || i18n.tr("nav_dear_world", "Dear World")}</a>
                    <a href="/signin" class="exo-btn exo-btn-primary">{move || i18n.tr("nav_sign_in", "Sign in")}</a>
                </div>
            </nav>

            <main class="page-main" style="max-width: 900px; margin: 0 auto;">
                <div class="exo-card">
                    <div class="exo-eyebrow">{move || i18n.tr("privacy_effective", "Effective Date & Last Updated: November 24, 2026")}</div>
                    <h1 class="exo-h1" style="margin-top: 6px;">{move || i18n.tr("privacy_heading", "ExoNovel Privacy Policy")}</h1>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "privacy_intro",
                            "ExoNovel (https://exonovel.exosystems.net) is a collaborative creative writing and local-first publishing platform operated by Exosystems (https://exosystems.net). This Privacy Policy describes how Exosystems collects, uses, stores, protects, and deletes personal data and Google user data when you use ExoNovel across the web application and companion client builds."
                        )}
                    </p>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("privacy_s1_title", "1. Information We Collect")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "privacy_s1_intro",
                            "We collect only the information needed to authenticate your account, operate your collaborative writing workspace, and deliver features you explicitly enable:"
                        )}
                    </p>
                    <ul class="exo-legal-list">
                        <li>
                            <strong>{move || i18n.tr("privacy_s1_google_label", "Google Sign-In Account Data (openid, email, profile): ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s1_google_body",
                                "When you sign in with Google, we receive your verified email address, email verification status, display name, given and family name, profile picture URL, locale, and Google provider account identifier."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("privacy_s1_drive_label", "Optional Connected Google Drive Data: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s1_drive_body",
                                "Primary sign-in never requests Google Drive permissions. Only if you explicitly connect Google Drive inside ExoNovel's Workspace or Preferences does ExoNovel request Google Drive authorization to list folders, create or rename backup folders, upload your exported Markdown documentation/manuscript archives or session transcripts, and restore or trash files you select inside the Drive Workspace."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("privacy_s1_profile_label", "Other Sign-In & Author Profile Information: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s1_profile_body",
                                "If you sign in with GitHub or email/password, or customize your workspace, we store your display name, pen name/handle, biography, website, location, preferred publication format, output and input language preferences, timezone, notification preferences, uploaded avatar images (processed server-side into 64px, 256px, and 1024px WebP/JPEG variants), and your cryptographic public key / did:peer identity."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("privacy_s1_works_label", "Manuscripts & Collaboration Records: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s1_works_body",
                                "Creative works, story bibles, personas, collaboration group memberships, referrals, and invitations that you choose to create or share inside ExoNovel."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("privacy_s1_sec_label", "Security & Technical Diagnostics: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s1_sec_body",
                                "Sign-in timestamps, IP address, browser User-Agent string, and application health/error logs used strictly to protect accounts against unauthorized access and keep the service reliable."
                            )}
                        </li>
                    </ul>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("privacy_s2_title", "2. Cookies & Browser Storage")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "privacy_s2_body",
                            "ExoNovel uses strictly necessary HttpOnly authentication session cookies to maintain your signed-in session, and temporary browser session storage strictly to preserve pending collaboration invite codes across sign-in redirects. ExoNovel does not use third-party advertising cookies, behavioral tracking pixels, or cross-site ad networks."
                        )}
                    </p>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("privacy_s3_title", "3. How We Use Your Information")}</h2>
                    <ul class="exo-legal-list">
                        <li>{move || i18n.tr("privacy_s3_li1", "Authenticate your session and resolve your authoritative did:peer identity across web, desktop, and mobile.")}</li>
                        <li>{move || i18n.tr("privacy_s3_li2", "Display your chosen author persona, pen name, avatar, and language preferences (including right-to-left layout preferences).")}</li>
                        <li>{move || i18n.tr("privacy_s3_li3", "Operate collaborative story groups, contributor attribution, and group/referral invitations you initiate.")}</li>
                        <li>{move || i18n.tr("privacy_s3_li4", "Execute user-initiated backups or exports to your connected Google Drive workspace when you explicitly trigger them.")}</li>
                        <li>{move || i18n.tr("privacy_s3_li5", "Deliver transactional, security, and notification emails you enable (sent from notes@exosystems.net, noreply@exosystems.net, or support@exosystems.net).")}</li>
                        <li>{move || i18n.tr("privacy_s3_li6", "Prevent fraud, abuse, unauthorized access, and violations of our Terms of Service.")}</li>
                    </ul>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("privacy_s4_title", "4. Google API Services User Data Policy (Limited Use) & AI / Content Restrictions")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "privacy_s4_intro",
                            "ExoNovel's use and transfer to any other app of information received from Google APIs will adhere to the Google API Services User Data Policy, including the Limited Use requirements."
                        )}
                    </p>
                    <ul class="exo-legal-list">
                        <li>
                            <strong>{move || i18n.tr("privacy_s4_li1_label", "No Advertising or Data Brokerage: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s4_li1_body",
                                "We never sell, rent, or transfer Google user data or your writing content to data brokers, advertisers, or third-party marketing platforms."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("privacy_s4_li2_label", "No Generalized AI Model Training: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s4_li2_body",
                                "ExoNovel does not use Google user data, Google Drive files, or your private manuscripts to train generalized artificial intelligence or machine learning models."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("privacy_s4_li3_label", "No Image Generation & Zero Tolerance for AI NCII: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s4_li3_body",
                                "ExoNovel is a text-based writing and publishing platform. It does not generate images, video, or audio, and strictly prohibits using ExoNovel or any connected Google APIs for AI-generated Non-Consensual Intimate Imagery (AI NCII)."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("privacy_s4_li4_label", "Human Access Restrictions: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s4_li4_body",
                                "Exosystems personnel do not read your private Google Drive files or private manuscripts unless (a) you explicitly grant consent for support debugging, (b) it is necessary for security or abuse investigation, or (c) it is required to comply with applicable law."
                            )}
                        </li>
                    </ul>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("privacy_s5_title", "5. Data Storage, Security & Cryptographic Protection")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "privacy_s5_body",
                            "All traffic to ExoNovel is encrypted in transit using HTTPS/TLS. Hosted web records and account state are stored in managed PostgreSQL databases and managed authentication services on enterprise cloud infrastructure, alongside transactional email delivery. When optional Connected External App offline tokens are stored server-side, they are envelope-encrypted via dedicated key management encryption and isolated from primary login credentials. On device-native installations using peer-to-peer Willow and iroh relay synchronization, payloads are end-to-end encrypted on your device so relay servers remain cryptographically blind to relayed content."
                        )}
                    </p>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("privacy_s6_title", "6. How to Revoke Access, Export & Delete Your Data")}</h2>
                    <ul class="exo-legal-list">
                        <li>
                            <strong>{move || i18n.tr("privacy_s6_li1_label", "Disconnect & Revoke Google Access Anytime: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s6_li1_body",
                                "You can disconnect Google Drive or other optional Connected Apps inside ExoNovel Preferences -> Connected Apps (which revokes active tokens directly through https://oauth2.googleapis.com/revoke without closing your primary ExoNovel account), or revoke access anytime in your Google Account Permissions at https://myaccount.google.com/permissions."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("privacy_s6_li2_label", "Export & Account Deletion Requests: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "privacy_s6_li2_body",
                                "You may request a complete export or permanent deletion of your ExoNovel account, Google OAuth profile metadata, personas, avatar assets, and hosted workspace records by emailing support@exosystems.net or exonomyapp@gmail.com. Verified deletion requests are completed within 30 days and remove active personal data from our hosted database and OAuth token store."
                            )}
                        </li>
                    </ul>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("privacy_s7_title", "7. Data Retention, Children's Privacy & Policy Updates")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "privacy_s7_body",
                            "We retain personal account data only as long as your account remains active or as necessary to fulfill security, billing, or legal obligations. ExoNovel is not directed to children under 13 (or under 16 where required by applicable regional data protection law), and we do not knowingly collect personal data from minors below the applicable age threshold. If we make material changes to this Privacy Policy, we will update the Effective Date above and notify active account holders where appropriate."
                        )}
                    </p>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("privacy_s8_title", "8. Contact Information")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "privacy_s8_body",
                            "For privacy questions, Google user data inquiries, or deletion requests, contact Exosystems Support at support@exosystems.net or exonomyapp@gmail.com."
                        )}
                    </p>
                </div>
            </main>

            <PublicFooter />
        </div>
    }
}

#[component]
fn TermsOfServiceView() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="exo-shell">
            <Title text=move || i18n.tr("terms_page_title", "Terms of Service — ExoNovel") />
            <nav class="navbar-solid">
                <a href="/" title="ExoNovel" class="navbar-brand-link">
                    <img src="/exonovel-logo.png" alt="ExoNovel" class="navbar-logo navbar-logo-light" />
                    <img src="/exonovel-logo-dark.png" alt="ExoNovel" class="navbar-logo navbar-logo-dark" />
                </a>
                <div class="navbar-menu">
                    <a href="/dear-world" class="nav-link">{move || i18n.tr("nav_dear_world", "Dear World")}</a>
                    <a href="/signin" class="exo-btn exo-btn-primary">{move || i18n.tr("nav_sign_in", "Sign in")}</a>
                </div>
            </nav>

            <main class="page-main" style="max-width: 900px; margin: 0 auto;">
                <div class="exo-card">
                    <div class="exo-eyebrow">{move || i18n.tr("terms_effective", "Effective Date & Last Updated: November 24, 2026")}</div>
                    <h1 class="exo-h1" style="margin-top: 6px;">{move || i18n.tr("terms_heading", "ExoNovel Terms of Service")}</h1>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "terms_intro",
                            "By accessing or using ExoNovel (https://exonovel.exosystems.net), operated by Exosystems (https://exosystems.net), you agree to be bound by these Terms of Service and our Privacy Policy (/privacy)."
                        )}
                    </p>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("terms_s1_title", "1. Ownership of Your Creative Works")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "terms_s1_body",
                            "You retain full ownership and copyright of all manuscripts, stories, characters, worldbuilding notes, scripts, poetry, annotations, and creative assets you write or publish on ExoNovel. You grant ExoNovel only the limited technical license necessary to store, render, synchronize, back up (when directed by you), and transmit your content according to your workspace and collaboration preferences."
                        )}
                    </p>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("terms_s2_title", "2. Accounts, Keys & Acceptable Use")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "terms_s2_body",
                            "You are responsible for safeguarding your login credentials and sovereign did:peer device keys. You must be at least 13 years old (or the minimum legal age in your jurisdiction) to create an account. You agree not to interfere with platform security, attempt unauthorized access to other users' workspaces, distribute malware, or send automated spam."
                        )}
                    </p>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("terms_s3_title", "3. Prohibited Content & Google APIs Terms of Service Compliance")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "terms_s3_intro",
                            "ExoNovel is a text-focused collaborative creative writing platform and does not provide image, video, or audio generation tools. ExoNovel strictly complies with the Google APIs Terms of Service and enforces zero tolerance for harmful or unlawful content:"
                        )}
                    </p>
                    <ul class="exo-legal-list">
                        <li>
                            <strong>{move || i18n.tr("terms_s3_li1_label", "Strict Prohibition on AI NCII: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "terms_s3_li1_body",
                                "You may not use ExoNovel or any connected Google APIs (including Google Sign-In or Google Drive integrations) to create, host, upload, synchronize, distribute, or facilitate AI-generated Non-Consensual Intimate Imagery (AI NCII), non-consensual sexualized depictions, or deepfakes."
                            )}
                        </li>
                        <li>
                            <strong>{move || i18n.tr("terms_s3_li2_label", "Unlawful & Abusive Material: ")}</strong>
                            " "
                            {move || i18n.tr(
                                "terms_s3_li2_body",
                                "Child sexual abuse material (CSAM), non-consensual intimate imagery, harassment, hate incitement, and copyright infringement are strictly prohibited and result in immediate account suspension and termination."
                            )}
                        </li>
                    </ul>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("terms_s4_title", "4. Connected External Apps (Google Drive & Third-Party Services)")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "terms_s4_body",
                            "Optional integrations such as Google Drive backup are activated only upon your explicit authorization and may be disconnected or revoked at any time in ExoNovel Preferences -> Connected Apps or via your third-party provider's account security page. Your use of third-party services remains subject to their respective terms."
                        )}
                    </p>

                    <h2 class="exo-h2" style="margin-top: 24px;">{move || i18n.tr("terms_s5_title", "5. Service Availability, Sync Plans & Contact")}</h2>
                    <p class="exo-legal-body">
                        {move || i18n.tr(
                            "terms_s5_body",
                            "ExoNovel provides hosted web collaboration, local-first device storage, and optional cloud-relay sync plans. Features are provided on an 'as available' basis as the platform evolves. For legal, safety, billing, or support inquiries, contact support@exosystems.net or exonomyapp@gmail.com."
                        )}
                    </p>
                </div>
            </main>

            <PublicFooter />
        </div>
    }
}
