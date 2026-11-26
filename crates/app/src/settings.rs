use gloo_net::http::Request;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::{
    connected_apps_ui::ConnectedAppsSettingsPanel,
    i18n::use_i18n,
    live::use_live_sync,
    models::{
        account_role_key, account_status_key, allowlist_status_key, detect_browser_timezone,
        display_name_for_user, is_safe_avatar_url, locale_flag_emoji, resolve_fresh_jwt,
        save_profile_patch, upload_avatar_file, ActiveTab, AllowlistEntry, AllowlistResponse,
        AppearanceMode, AuthConfigResponse, CopyIcon, IdentitiesResponse, IdentityRecord,
        NotificationDevice, NotificationPreferences, NotificationPrefsResponse, ProfilePatchResponse,
        SettingsSection, VerifiedUser,
    },
};

#[cfg(not(feature = "ssr"))]
fn request_browser_notification_permission(
    on_result: impl Fn(String) + 'static,
) {
    if let Ok(promise) = web_sys::Notification::request_permission() {
        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(val) => {
                    let st = val.as_string().unwrap_or_else(|| "default".to_string());
                    on_result(st);
                }
                Err(_) => on_result("denied".to_string()),
            }
        });
    }
}

#[cfg(feature = "ssr")]
fn request_browser_notification_permission(
    _on_result: impl Fn(String) + 'static,
) {
}

#[component]
pub fn SettingsView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
    user: VerifiedUser,
    return_tab: ReadSignal<ActiveTab>,
    appearance: ReadSignal<AppearanceMode>,
    set_appearance: WriteSignal<AppearanceMode>,
    on_copy_did: impl Fn(String) + Copy + Send + Sync + 'static,
    on_toast: impl Fn(String) + Copy + Send + Sync + 'static,
    on_open_profile_billing: impl Fn() + Copy + Send + Sync + 'static,
    on_profile_updated: impl Fn(ProfilePatchResponse) + Copy + Send + Sync + 'static,
    on_back: impl Fn() + Copy + Send + Sync + 'static,
    #[prop(optional)] initial_section: Option<RwSignal<Option<SettingsSection>>>,
) -> impl IntoView {
    let live = use_live_sync();
    let i18n = use_i18n();

    let prof = user.profile.clone().unwrap_or_default();
    let user_did_str = user
        .did_peer
        .clone()
        .unwrap_or_else(|| prof.did_peer.clone());
    let user_did = StoredValue::new(user_did_str);
    let is_admin = user.is_admin;
    let sync_plan_tier_str = if prof.sync_plan_tier.is_empty() {
        "free".to_string()
    } else {
        prof.sync_plan_tier.clone()
    };
    let can_enable_mode3 = sync_plan_tier_str != "free";
    let sync_plan_tier = StoredValue::new(sync_plan_tier_str);

    let (active_section, set_active_section) = signal(
        initial_section
            .and_then(|s| s.get_untracked())
            .unwrap_or(SettingsSection::Account),
    );
    if let Some(ext_sig) = initial_section {
        Effect::new(move |_| {
            if let Some(sec) = ext_sig.get() {
                set_active_section.set(sec);
                ext_sig.set(None);
            }
        });
    }

    // Account & Appearance Signals
    let (display_name, set_display_name) = signal(if prof.display_name.is_empty() {
        display_name_for_user(&user)
    } else {
        prof.display_name.clone()
    });
    let (pen_name, set_pen_name) = signal(prof.pen_name.clone());
    let (bio, set_bio) = signal(prof.bio.clone());
    let (avatar_url, set_avatar_url) = signal(prof.avatar_url.clone());
    let (avatar_thumb_url, set_avatar_thumb_url) = signal(prof.avatar_thumb_url.clone());
    let (avatar_asset_id, set_avatar_asset_id) = signal(prof.avatar_asset_id.clone());
    let (website, set_website) = signal(prof.website.clone());
    let (location, set_location) = signal(prof.location.clone());
    let (timezone, set_timezone) = signal(if prof.timezone.is_empty() {
        detect_browser_timezone()
    } else {
        prof.timezone.clone()
    });
    let (preferred_format, set_preferred_format) = signal(if prof.preferred_format.is_empty() {
        "novel".to_string()
    } else {
        prof.preferred_format.clone()
    });
    let (ui_loc, set_ui_loc) = signal(if prof.ui_locale.is_empty() {
        i18n.ui_locale.get_untracked()
    } else {
        prof.ui_locale.clone()
    });
    let (in_loc, set_in_loc) = signal(if prof.default_input_locale.is_empty() {
        i18n.input_locale.get_untracked()
    } else {
        prof.default_input_locale.clone()
    });
    let (willow_sync_enabled, set_willow_sync_enabled) = signal(prof.willow_sync_enabled);
    let (profile_msg, set_profile_msg) = signal::<Option<String>>(None);
    let (saving_profile, set_saving_profile) = signal(false);
    let avatar_input_ref = NodeRef::<leptos::html::Input>::new();

    // Notification Channels & Devices
    let (notif_prefs, set_notif_prefs) = signal(NotificationPreferences::default());
    let (devices, set_devices) = signal::<Vec<NotificationDevice>>(Vec::new());
    let (brevo_configured, set_brevo_configured) = signal(false);
    let (dev_platform, set_dev_platform) = signal("android".to_string());
    let (dev_transport, set_dev_transport) = signal("unifiedpush".to_string());
    let (dev_token, set_dev_token) = signal(String::new());
    let (dev_label, set_dev_label) = signal("My Primary Device".to_string());

    // Admin Allowlist & Identities Directory
    let (entries, set_entries) = signal::<Vec<AllowlistEntry>>(Vec::new());
    let (identities, set_identities) = signal::<Vec<IdentityRecord>>(Vec::new());
    let (grant_did, set_grant_did) = signal(String::new());
    let (grant_status, set_grant_status) = signal("reader".to_string());
    let (new_note, set_new_note) = signal(String::new());
    let (admin_msg, set_admin_msg) = signal::<Option<String>>(None);

    let (expanded_did, set_expanded_did) = signal::<Option<String>>(None);
    let (edit_display_name, set_edit_display_name) = signal(String::new());
    let (edit_pen_name, set_edit_pen_name) = signal(String::new());
    let (edit_website, set_edit_website) = signal(String::new());
    let (edit_location, set_edit_location) = signal(String::new());
    let (edit_format, set_edit_format) = signal("novel".to_string());
    let (edit_account_role, set_edit_account_role) = signal("user".to_string());
    let (edit_account_status, set_edit_account_status) = signal("active".to_string());
    let (edit_bio, set_edit_bio) = signal(String::new());
    let (edit_msg, set_edit_msg) = signal::<Option<String>>(None);

    let reload_channels_and_admin = move || {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(resp) = Request::get("/api/notifications/preferences")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<NotificationPrefsResponse>().await {
                        set_notif_prefs.set(data.preferences);
                        set_devices.set(data.devices);
                        set_brevo_configured.set(data.brevo_configured);
                    }
                }
            }
            if is_admin {
                if let Ok(resp) = Request::get("/api/admin/allowlist")
                    .header("Authorization", &format!("Bearer {jwt}"))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<AllowlistResponse>().await {
                            set_entries.set(data.entries);
                        }
                    }
                }
                if let Ok(resp) = Request::get("/api/admin/identities")
                    .header("Authorization", &format!("Bearer {jwt}"))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<IdentitiesResponse>().await {
                            set_identities.set(data.identities);
                        }
                    }
                }
            }
        });
    };

    Effect::new(move |_| reload_channels_and_admin());

    Effect::new(move |_| {
        let _seq = live.pulse_seq.get();
        if live.matches_kind(&[
            "notification_prefs",
            "notification_device",
            "allowlist",
            "identity",
            "system",
        ]) {
            reload_channels_and_admin();
        }
    });

    let on_avatar_file = move |_| {
        if let Some(input) = avatar_input_ref.get() {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    if file.size() > 2_097_152.0 {
                        set_profile_msg.set(Some(i18n.tr_untracked(
                            "settings_msg_avatar_size",
                            "Avatar file must be 2 MB or smaller.",
                        )));
                        return;
                    }
                    let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
                    let fallback_jwt = initial_jwt.get_untracked();
                    let target_did = user_did.get_value();
                    if let Ok(reader) = web_sys::FileReader::new() {
                        let reader_clone = reader.clone();
                        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                            if let Ok(res) = reader_clone.result() {
                                if let Some(data_url) = res.as_string() {
                                    set_profile_msg.set(Some(i18n.tr_untracked(
                                        "settings_msg_avatar_processing",
                                        "Processing avatar derivatives...",
                                    )));
                                    leptos::task::spawn_local(async move {
                                        match upload_avatar_file(
                                            auth_url.as_deref(),
                                            fallback_jwt.as_deref(),
                                            data_url,
                                            Some(target_did),
                                        )
                                        .await
                                        {
                                            Ok(uploaded) => {
                                                set_avatar_asset_id.set(uploaded.asset_id);
                                                set_avatar_url.set(uploaded.medium_url);
                                                set_avatar_thumb_url.set(uploaded.thumb_url);
                                                if let Ok(updated) = save_profile_patch(
                                                    auth_url.as_deref(),
                                                    fallback_jwt.as_deref(),
                                                    serde_json::json!({}),
                                                )
                                                .await
                                                {
                                                    on_profile_updated(updated);
                                                }
                                                set_profile_msg.set(Some(i18n.tr_untracked(
                                                    "settings_msg_avatar_saved",
                                                    "Avatar saved.",
                                                )));
                                            }
                                            Err(e) => set_profile_msg.set(Some(e)),
                                        }
                                    });
                                }
                            }
                        });
                        reader.set_onload(Some(cb.unchecked_ref()));
                        let _ = reader.read_as_data_url(&file);
                    }
                }
            }
        }
    };

    let save_settings_profile = move || {
        set_saving_profile.set(true);
        set_profile_msg.set(None);
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        let fallback_jwt = initial_jwt.get_untracked();
        let next_ui = ui_loc.get_untracked();
        let next_in = in_loc.get_untracked();
        let patch = serde_json::json!({
            "displayName": display_name.get_untracked(),
            "penName": pen_name.get_untracked(),
            "bio": bio.get_untracked(),
            "avatarAssetId": avatar_asset_id.get_untracked(),
            "avatarUrl": avatar_url.get_untracked(),
            "avatarThumbUrl": avatar_thumb_url.get_untracked(),
            "website": website.get_untracked(),
            "location": location.get_untracked(),
            "timezone": timezone.get_untracked(),
            "preferredFormat": preferred_format.get_untracked(),
            "appearanceMode": appearance.get_untracked().as_str(),
            "uiLocale": next_ui,
            "defaultInputLocale": next_in,
            "willowSyncEnabled": willow_sync_enabled.get_untracked(),
        });
        i18n.ui_locale.set(next_ui);
        i18n.input_locale.set(next_in);

        leptos::task::spawn_local(async move {
            match save_profile_patch(auth_url.as_deref(), fallback_jwt.as_deref(), patch).await {
                Ok(updated) => {
                    on_profile_updated(updated);
                    set_profile_msg.set(Some(i18n.tr_untracked(
                        "settings_msg_saved",
                        "Settings saved to Neon.",
                    )));
                    on_toast(i18n.tr_untracked("settings_toast_saved", "Settings saved"));
                }
                Err(e) => set_profile_msg.set(Some(e)),
            }
            set_saving_profile.set(false);
        });
    };

    let save_notif_channels = move || {
        let p = notif_prefs.get_untracked();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::put("/api/notifications/preferences")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "inAppEnabled": p.in_app_enabled,
                    "webPushEnabled": p.web_push_enabled,
                    "nativePushEnabled": p.native_push_enabled,
                    "emailEnabled": p.email_enabled,
                    "emailDigestMode": p.email_digest_mode,
                    "notifySecuritySignins": p.notify_security_signins,
                    "notifyMaturityReleases": p.notify_maturity_releases,
                    "notifyMaturityUnblocked": p.notify_maturity_unblocked,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        on_toast(i18n.tr_untracked(
                            "settings_toast_notif_saved",
                            "Notification channels updated",
                        ));
                    }
                }
            }
        });
    };

    let register_device = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let platform = dev_platform.get_untracked();
        let transport = dev_transport.get_untracked();
        let tok = dev_token.get_untracked();
        let label = dev_label.get_untracked();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/notifications/devices")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "platform": platform,
                    "transport": transport,
                    "endpointOrToken": tok,
                    "deviceLabel": label,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_dev_token.set(String::new());
                        reload_channels_and_admin();
                        on_toast(i18n.tr_untracked(
                            "settings_toast_device_reg",
                            "Push device registered",
                        ));
                    }
                }
            }
        });
    };

    let send_test_notification = move |_| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(resp) = Request::post("/api/notifications/test")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    on_toast(i18n.tr_untracked(
                        "settings_toast_test_sent",
                        "Test notification & Brevo email dispatched",
                    ));
                } else {
                    let msg = resp.text().await.unwrap_or_default();
                    set_profile_msg.set(Some(msg));
                }
            }
        });
    };

    // Admin Allowlist CRUD handlers
    let add_reader = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let val = grant_did.get_untracked().trim().to_string();
        if val.is_empty() {
            return;
        }
        let status = grant_status.get_untracked();
        let note = new_note.get_untracked().trim().to_string();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            let payload = if val.starts_with("did:peer:") {
                serde_json::json!({ "didPeer": val, "status": status, "note": note })
            } else {
                serde_json::json!({ "email": val, "status": status, "note": note })
            };
            if let Ok(req) = Request::post("/api/admin/allowlist")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&payload)
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_grant_did.set(String::new());
                        set_new_note.set(String::new());
                        set_admin_msg.set(Some(i18n.tr_untracked(
                            "settings_msg_allowlist_updated",
                            "Allowlist updated.",
                        )));
                        reload_channels_and_admin();
                    }
                }
            }
        });
    };

    let update_entry_status = move |did_opt: Option<String>, email: String, next_status: String| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/admin/allowlist")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "didPeer": did_opt,
                    "email": email,
                    "status": next_status,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        reload_channels_and_admin();
                        on_toast(i18n.tr_untracked(
                            "settings_toast_tier_updated",
                            "Allowlist tier updated",
                        ));
                    }
                }
            }
        });
    };

    let remove_entry = move |did_opt: Option<String>, email: String| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            let query = if let Some(d) = did_opt.filter(|s| !s.is_empty()) {
                let enc = js_sys::encode_uri_component(&d).as_string().unwrap_or(d);
                format!("didPeer={enc}")
            } else {
                let enc = js_sys::encode_uri_component(&email).as_string().unwrap_or(email);
                format!("email={enc}")
            };
            let url = format!("/api/admin/allowlist?{query}");
            if let Ok(resp) = Request::delete(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    reload_channels_and_admin();
                    on_toast(i18n.tr_untracked(
                        "settings_toast_entry_removed",
                        "Allowlist entry removed",
                    ));
                }
            }
        });
    };

    let save_identity_accordion = move |target_did: String| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::put("/api/admin/identities/profile")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "didPeer": target_did,
                    "displayName": edit_display_name.get_untracked(),
                    "penName": edit_pen_name.get_untracked(),
                    "website": edit_website.get_untracked(),
                    "location": edit_location.get_untracked(),
                    "preferredFormat": edit_format.get_untracked(),
                    "accountRole": edit_account_role.get_untracked(),
                    "accountStatus": edit_account_status.get_untracked(),
                    "bio": edit_bio.get_untracked(),
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_edit_msg.set(Some(i18n.tr_untracked(
                            "settings_msg_account_updated",
                            "Account updated.",
                        )));
                        reload_channels_and_admin();
                    }
                }
            }
        });
    };

    let back_label = move || match return_tab.get() {
        ActiveTab::Docs => format!("← {}", i18n.tr("nav_docs", "Docs & ADRs")),
        _ => i18n.tr("back_to_workspace", "← Back to Workspace"),
    };

    view! {
        <main class="page-main">
            <div style="display: flex; align-items: center; justify-content: space-between; gap: 16px; flex-wrap: wrap; margin-bottom: 22px;">
                <div>
                    <h1 class="exo-h1">{move || i18n.tr("settings_title", "Settings")}</h1>
                    <p class="exo-sub" style="margin: 4px 0 0;">
                        {move || i18n.tr("settings_subtitle", "Technical preferences, languages, operating modes, and delivery channels (all commercial purchases & subscriptions stay in Profile).")}
                    </p>
                </div>
                <button type="button" class="exo-btn" on:click=move |_| on_back()>
                    {back_label}
                </button>
            </div>

            <div class="exo-gh-settings">
                <nav class="exo-gh-nav" aria-label=move || i18n.tr("settings_nav_aria", "Settings sections")>
                    <button
                        type="button"
                        class=move || if active_section.get() == SettingsSection::Account { "exo-gh-nav-item exo-gh-nav-item-active" } else { "exo-gh-nav-item" }
                        on:click=move |_| set_active_section.set(SettingsSection::Account)
                    >
                        <span>{move || i18n.tr("sec_account", "Account & Security")}</span>
                    </button>
                    <button
                        type="button"
                        class=move || if active_section.get() == SettingsSection::ConnectedApps { "exo-gh-nav-item exo-gh-nav-item-active" } else { "exo-gh-nav-item" }
                        on:click=move |_| set_active_section.set(SettingsSection::ConnectedApps)
                    >
                        <span>{move || i18n.tr("sec_connected_apps", "Connected Apps & Google Docs")}</span>
                    </button>
                    <button
                        type="button"
                        class=move || if active_section.get() == SettingsSection::AppearanceAndLanguages { "exo-gh-nav-item exo-gh-nav-item-active" } else { "exo-gh-nav-item" }
                        on:click=move |_| set_active_section.set(SettingsSection::AppearanceAndLanguages)
                    >
                        <span>{move || i18n.tr("sec_appearance_langs", "Appearance & Languages (29)")}</span>
                    </button>
                    <button
                        type="button"
                        class=move || if active_section.get() == SettingsSection::SyncAndStorage { "exo-gh-nav-item exo-gh-nav-item-active" } else { "exo-gh-nav-item" }
                        on:click=move |_| set_active_section.set(SettingsSection::SyncAndStorage)
                    >
                        <span>{move || i18n.tr("sec_sync_storage", "Sync & Storage (3 Modes)")}</span>
                    </button>
                    <button
                        type="button"
                        class=move || if active_section.get() == SettingsSection::NotificationChannels { "exo-gh-nav-item exo-gh-nav-item-active" } else { "exo-gh-nav-item" }
                        on:click=move |_| set_active_section.set(SettingsSection::NotificationChannels)
                    >
                        <span>{move || i18n.tr("sec_notifications", "Notification Channels")}</span>
                    </button>
                    <Show when=move || is_admin>
                        <button
                            type="button"
                            class=move || if active_section.get() == SettingsSection::AdminDocsAndMaturity { "exo-gh-nav-item exo-gh-nav-item-active" } else { "exo-gh-nav-item" }
                            on:click=move |_| set_active_section.set(SettingsSection::AdminDocsAndMaturity)
                        >
                            <span>{move || i18n.tr("sec_admin_allowlist", "Docs Allowlist & Accounts")}</span>
                        </button>
                    </Show>
                </nav>

                <div class="exo-gh-content">
                    {move || match active_section.get() {
                        SettingsSection::Account => {
                            let avatar_preview = avatar_url.get();
                            let did_display = user_did.get_value();
                            let av_alt = display_name.get();
                            let av_init = av_alt.chars().next().unwrap_or('W').to_uppercase().to_string();
                            view! {
                                <div class="exo-card">
                                    <h2 class="exo-h2">{move || i18n.tr("account_title", "Account & Sovereign Identity")}</h2>
                                    <div class="exo-home-hero" style="margin: 14px 0 18px;">
                                        {if is_safe_avatar_url(&avatar_preview) {
                                            view! { <img src=avatar_preview alt=av_alt class="exo-avatar" /> }.into_any()
                                        } else {
                                            view! { <div class="exo-avatar">{av_init}</div> }.into_any()
                                        }}
                                        <div>
                                            <label class="exo-label">{move || i18n.tr("upload_avatar_label", "Upload Profile Photo (64px / 256px / 1024px WebP)")}</label>
                                            <input
                                                type="file"
                                                accept="image/png,image/jpeg,image/webp,image/gif"
                                                node_ref=avatar_input_ref
                                                on:change=on_avatar_file
                                            />
                                        </div>
                                    </div>

                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("root_identity_label", "Root Identity (did:peer)")}</label>
                                        <div class="exo-id-cell">
                                            <button type="button" class="exo-copy-btn" on:click=move |_| on_copy_did(user_did.get_value())>
                                                <CopyIcon />
                                            </button>
                                            <span class="exo-mono">{did_display}</span>
                                        </div>
                                    </div>

                                    <div class="exo-grid-2">
                                        <div class="exo-field">
                                            <label class="exo-label">{move || i18n.tr("display_name_label", "Display Name")}</label>
                                            <input
                                                type="text"
                                                class="exo-input"
                                                prop:value=move || display_name.get()
                                                on:input=move |ev| set_display_name.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">{move || i18n.tr("pen_name_field_label", "Default Pen Name")}</label>
                                            <input
                                                type="text"
                                                class="exo-input"
                                                prop:value=move || pen_name.get()
                                                on:input=move |ev| set_pen_name.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">{move || i18n.tr("website_label", "Website")}</label>
                                            <input
                                                type="text"
                                                class="exo-input"
                                                prop:value=move || website.get()
                                                on:input=move |ev| set_website.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">{move || i18n.tr("location_label", "Location & Timezone")}</label>
                                            <input
                                                type="text"
                                                class="exo-input"
                                                prop:value=move || location.get()
                                                on:input=move |ev| set_location.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">{move || i18n.tr("preferred_format_label", "Preferred Writing Format")}</label>
                                            <select
                                                class="exo-input"
                                                prop:value=move || preferred_format.get()
                                                on:change=move |ev| set_preferred_format.set(event_target_value(&ev))
                                            >
                                                <option value="novel">{move || i18n.tr("format_opt_novel", "Novels")}</option>
                                                <option value="article">{move || i18n.tr("format_opt_article", "Articles & Nonfiction")}</option>
                                                <option value="short-story">{move || i18n.tr("format_opt_short", "Short Stories")}</option>
                                                <option value="screenplay">{move || i18n.tr("format_opt_screenplay", "Screenplays")}</option>
                                                <option value="poetry">{move || i18n.tr("format_opt_poetry", "Poetry")}</option>
                                            </select>
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">{move || i18n.tr("timezone_label", "Timezone")}</label>
                                            <input
                                                type="text"
                                                class="exo-input"
                                                prop:value=move || timezone.get()
                                                on:input=move |ev| set_timezone.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>

                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("author_bio_label", "Author Bio")}</label>
                                        <textarea
                                            class="exo-input exo-textarea"
                                            prop:value=move || bio.get()
                                            on:input=move |ev| set_bio.set(event_target_value(&ev))
                                        />
                                    </div>

                                    {move || profile_msg.get().map(|m| view! { <p class="exo-sub">{m}</p> })}
                                    <button
                                        type="button"
                                        class="exo-btn exo-btn-primary"
                                        disabled=move || saving_profile.get()
                                        on:click=move |_| save_settings_profile()
                                    >
                                        {move || i18n.tr("save_account_btn", "Save Account Settings")}
                                    </button>
                                </div>
                            }.into_any()
                        }

                        SettingsSection::ConnectedApps => view! {
                            <ConnectedAppsSettingsPanel
                                auth_config=auth_config
                                initial_jwt=initial_jwt
                                on_toast=on_toast
                            />
                        }.into_any(),

                        SettingsSection::AppearanceAndLanguages => view! {
                            <div class="exo-card">
                                <h2 class="exo-h2">{move || i18n.tr("appearance_title", "Theme & Dual Language Architecture")}</h2>
                                <p class="exo-sub">
                                    {move || i18n.tr("appearance_desc", "Output UI Language controls menus and layout direction (auto RTL for Arabic and Hebrew). Default Input Language controls composers independently. All preferences save directly to Neon (zero browser localStorage).")}
                                </p>

                                <div class="exo-grid-2" style="margin-top: 16px;">
                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("theme_label", "Theme Appearance")}</label>
                                        <select
                                            class="exo-input"
                                            prop:value=move || appearance.get().as_str().to_string()
                                            on:change=move |ev| set_appearance.set(AppearanceMode::from_str(&event_target_value(&ev)))
                                        >
                                            <option value="system">{move || i18n.tr("theme_opt_system", "System")}</option>
                                            <option value="light">{move || i18n.tr("theme_opt_light", "Light")}</option>
                                            <option value="dark">{move || i18n.tr("theme_opt_dark", "Dark")}</option>
                                        </select>
                                    </div>

                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("ui_lang_label", "App Language")}</label>
                                        <select
                                            class="exo-input"
                                            prop:value=move || ui_loc.get()
                                            on:change=move |ev| set_ui_loc.set(event_target_value(&ev))
                                        >
                                            {move || i18n.catalog.get().into_iter().map(|loc| {
                                                let lbl = format!("{} {} — {} [{}]", locale_flag_emoji(&loc.code), loc.native_name, loc.english_name, loc.dir.to_uppercase());
                                                view! { <option value=loc.code>{lbl}</option> }
                                            }).collect_view()}
                                        </select>
                                    </div>

                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("input_lang_label", "Writing Input")}</label>
                                        <select
                                            class="exo-input"
                                            prop:value=move || in_loc.get()
                                            on:change=move |ev| set_in_loc.set(event_target_value(&ev))
                                        >
                                            {move || i18n.catalog.get().into_iter().map(|loc| {
                                                let lbl = format!("{} {} — {}", locale_flag_emoji(&loc.code), loc.native_name, loc.english_name);
                                                view! { <option value=loc.code>{lbl}</option> }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                </div>

                                <div class="exo-format-tile" style="margin-top: 14px;">
                                    <h3>{move || i18n.tr("offline_packs_title", "Decentralized Offline Language Packs")}</h3>
                                    <p>
                                        {move || i18n.tr("offline_packs_desc", "Decentralized Mode 2 installs download any selected language pack directly from ExoNovel's public read-only Willow namespace: ")}
                                        " "
                                        <span class="exo-mono">{move || i18n.willow_namespace.get()}</span>
                                    </p>
                                </div>

                                <button
                                    type="button"
                                    class="exo-btn exo-btn-primary"
                                    style="margin-top: 18px;"
                                    on:click=move |_| save_settings_profile()
                                >
                                    {move || i18n.tr("save_theme_lang_btn", "Save Theme & Language Defaults")}
                                </button>
                            </div>
                        }.into_any(),

                        SettingsSection::SyncAndStorage => view! {
                            <div class="exo-card">
                                <h2 class="exo-h2">{move || i18n.tr("sync_title", "Three Operating Modes & Cryptographic Blindness")}</h2>
                                <p class="exo-sub">
                                    {move || i18n.tr("sync_sub", "Choose how ExoNovel stores and synchronizes your writing across Web and Device installs.")}
                                </p>

                                <div class="exo-grid-3" style="margin-top: 16px;">
                                    <div class="exo-format-tile">
                                        <h3>{move || i18n.tr("sync_m1_title", "Mode 1 · ExoNovel on Web (Free)")}</h3>
                                        <p>{move || i18n.tr("sync_m1_desc", "Cloud workspace backed by Neon Postgres with instant real-time updates across tabs.")}</p>
                                    </div>
                                    <div class="exo-format-tile">
                                        <h3>{move || i18n.tr("sync_m2_title", "Mode 2 · Sovereign Device (Free)")}</h3>
                                        <p>{move || i18n.tr("sync_m2_desc", "100% local Willow store on Windows, macOS, Linux, Android, and iOS. Includes free ExoNovel iroh discovery & blind relay transit.")}</p>
                                    </div>
                                    <div class="exo-format-tile" style="border-color: var(--accent);">
                                        <h3>{move || i18n.tr("sync_m3_title", "Mode 3 · Web + Device Managed Sync")}</h3>
                                        <p>{move || i18n.tr("sync_m3_desc", "Paid ($9/mo or $90/yr in Profile). Meadowcap-authorized blind backup and bidirectional Web-to-Device Willow sync.")}</p>
                                    </div>
                                </div>

                                <div class="exo-format-tile" style="margin-top: 16px;">
                                    <h3>{move || i18n.tr("sync_zk_title", "Zero-Knowledge Cryptographic Blindness")}</h3>
                                    <p>
                                        {move || i18n.tr("sync_zk_desc", "In Modes 2 and 3, drafts and private synced Willow entries are end-to-end encrypted by the sender before touching ExoNovel's relay or storage. Only routing frames and Meadowcap capability tokens are visible.")}
                                    </p>
                                </div>

                                <div style="margin-top: 18px; padding-top: 16px; border-top: 1px solid var(--border);">
                                    <label class="exo-check-row">
                                        <input
                                            type="checkbox"
                                            prop:disabled=move || !can_enable_mode3
                                            prop:checked=move || willow_sync_enabled.get()
                                            on:change=move |ev| set_willow_sync_enabled.set(event_target_checked(&ev))
                                        />
                                        <span>
                                            <strong>{move || i18n.tr("sync_enable_m3", "Enable Mode 3 Managed Willow Sync on this account")}</strong>
                                            " — " {move || i18n.tr("sync_current_tier", "Current tier:")} " "
                                            <span class="exo-mono">{sync_plan_tier.get_value()}</span>
                                        </span>
                                    </label>

                                    {if !can_enable_mode3 {
                                        view! {
                                            <div class="exo-invite-banner" style="margin-top: 14px; justify-content: space-between; flex-wrap: wrap;">
                                                <span>{move || i18n.tr("sync_requires_sub", "Mode 3 Managed Sync requires an active subscription managed in Profile.")}</span>
                                                <button
                                                    type="button"
                                                    class="exo-btn exo-btn-primary"
                                                    on:click=move |_| on_open_profile_billing()
                                                >
                                                    {move || i18n.tr("sync_manage_in_profile", "Manage Subscription in Profile →")}
                                                </button>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <button
                                                type="button"
                                                class="exo-btn exo-btn-primary"
                                                style="margin-top: 14px;"
                                                on:click=move |_| save_settings_profile()
                                            >
                                                {move || i18n.tr("sync_save_toggle", "Save Sync Toggle")}
                                            </button>
                                        }.into_any()
                                    }}
                                </div>
                            </div>
                        }.into_any(),

                        SettingsSection::NotificationChannels => view! {
                            <div>
                                <div class="exo-card">
                                    <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                                        <div>
                                            <h2 class="exo-h2" style="margin: 0;">{move || i18n.tr("notif_channels_title", "Multi-Platform Delivery Channels")}</h2>
                                            <p class="exo-sub" style="margin: 4px 0 0;">
                                                {move || i18n.tr("notif_brevo_prefix", "Brevo Transactional Email (notes@exosystems.net)")}
                                                " "
                                                {move || if brevo_configured.get() {
                                                    i18n.tr("notif_brevo_active", "· Brevo API Key Active")
                                                } else {
                                                    i18n.tr("notif_brevo_unset", "· Configure BREVO_API_KEY in server env")
                                                }}
                                            </p>
                                        </div>
                                        <button type="button" class="exo-btn" on:click=send_test_notification>
                                            {move || i18n.tr("notif_send_test", "Send Test Notification & Brevo Email")}
                                        </button>
                                    </div>

                                    {move || {
                                        let p = notif_prefs.get();
                                        view! {
                                            <div style="margin-top: 14px;">
                                                <label class="exo-check-row">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=p.in_app_enabled
                                                        on:change=move |ev| {
                                                            let v = event_target_checked(&ev);
                                                            set_notif_prefs.update(|np| np.in_app_enabled = v);
                                                        }
                                                    />
                                                    <span><strong>{move || i18n.tr("notif_in_app_title", "In-App Live Feed & Bell")}</strong> " — " {move || i18n.tr("notif_in_app_desc", "Instant delivery over the persistent SSE stream.")}</span>
                                                </label>
                                                <label class="exo-check-row">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=p.web_push_enabled
                                                        on:change=move |ev| {
                                                            let v = event_target_checked(&ev);
                                                            set_notif_prefs.update(|np| np.web_push_enabled = v);
                                                            if v {
                                                                request_browser_notification_permission(move |st| {
                                                                    on_toast(format!("{} {st}", i18n.tr_untracked("settings_toast_perm", "Browser notification permission:")));
                                                                });
                                                            }
                                                        }
                                                    />
                                                    <span><strong>{move || i18n.tr("notif_web_push_title", "Web Browser Notifications")}</strong> " — " {move || i18n.tr("notif_web_push_desc", "Native OS banner via browser Notification API.")}</span>
                                                </label>
                                                <label class="exo-check-row">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=p.native_push_enabled
                                                        on:change=move |ev| {
                                                            let v = event_target_checked(&ev);
                                                            set_notif_prefs.update(|np| np.native_push_enabled = v);
                                                        }
                                                    />
                                                    <span><strong>{move || i18n.tr("notif_native_push_title", "Native Push Devices (Android UnifiedPush + FCM, iOS/macOS APNs, Windows WNS, Linux)")}</strong></span>
                                                </label>
                                                <label class="exo-check-row">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=p.email_enabled
                                                        on:change=move |ev| {
                                                            let v = event_target_checked(&ev);
                                                            set_notif_prefs.update(|np| np.email_enabled = v);
                                                        }
                                                    />
                                                    <span><strong>{move || i18n.tr("notif_email_title", "Brevo Transactional Email (Notes from ExoNovel)")}</strong></span>
                                                </label>
                                                <div class="exo-field" style="margin-top: 14px;">
                                                    <label class="exo-label">{move || i18n.tr("notif_digest_label", "Email Digest Mode")}</label>
                                                    <select
                                                        class="exo-input"
                                                        style="max-width: 280px;"
                                                        prop:value=p.email_digest_mode
                                                        on:change=move |ev| {
                                                            let v = event_target_value(&ev);
                                                            set_notif_prefs.update(|np| np.email_digest_mode = v);
                                                        }
                                                    >
                                                        <option value="instant">{move || i18n.tr("notif_digest_instant", "Instant Email")}</option>
                                                        <option value="daily">{move || i18n.tr("notif_digest_daily", "Daily Digest")}</option>
                                                        <option value="off">{move || i18n.tr("notif_digest_off", "Email Disabled")}</option>
                                                    </select>
                                                </div>
                                                <label class="exo-check-row">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=p.notify_security_signins
                                                        on:change=move |ev| {
                                                            let v = event_target_checked(&ev);
                                                            set_notif_prefs.update(|np| np.notify_security_signins = v);
                                                        }
                                                    />
                                                    <span><strong>{move || i18n.tr("notif_security_title", "Security & Sign-In Alerts")}</strong> " — " {move || i18n.tr("notif_security_desc", "Alert when your sovereign identity authenticates on a new device.")}</span>
                                                </label>
                                                <label class="exo-check-row">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=p.notify_maturity_releases
                                                        on:change=move |ev| {
                                                            let v = event_target_checked(&ev);
                                                            set_notif_prefs.update(|np| np.notify_maturity_releases = v);
                                                        }
                                                    />
                                                    <span><strong>{move || i18n.tr("notif_maturity_title", "Maturity Scanner Release Alerts")}</strong> " — " {move || i18n.tr("notif_maturity_desc", "Notify when tracked dependencies ship new versions.")}</span>
                                                </label>
                                            </div>
                                        }
                                    }}

                                    <button
                                        type="button"
                                        class="exo-btn exo-btn-primary"
                                        style="margin-top: 16px;"
                                        on:click=move |_| save_notif_channels()
                                    >
                                        {move || i18n.tr("notif_save_btn", "Save Channel Preferences")}
                                    </button>
                                </div>

                                <div class="exo-card">
                                    <h2 class="exo-h2">{move || i18n.tr("devices_title", "Registered Device Push Endpoints (AOTA)")}</h2>
                                    <p class="exo-sub">
                                        {move || i18n.tr("devices_sub", "Register any native or web push transport including UnifiedPush (de-Googled Android/Linux), FCM (Google Play Android), APNs (iOS/macOS), or WNS (Windows).")}
                                    </p>
                                    <form on:submit=register_device style="margin-top: 14px;">
                                        <div class="exo-grid-2">
                                            <div class="exo-field">
                                                <label class="exo-label">{move || i18n.tr("device_platform_label", "Platform")}</label>
                                                <select
                                                    class="exo-input"
                                                    prop:value=move || dev_platform.get()
                                                    on:change=move |ev| set_dev_platform.set(event_target_value(&ev))
                                                >
                                                    <option value="android">{move || i18n.tr("device_plat_android", "Android")}</option>
                                                    <option value="ios">{move || i18n.tr("device_plat_ios", "iOS")}</option>
                                                    <option value="linux">{move || i18n.tr("device_plat_linux", "Linux")}</option>
                                                    <option value="macos">{move || i18n.tr("device_plat_macos", "macOS")}</option>
                                                    <option value="windows">{move || i18n.tr("device_plat_windows", "Windows")}</option>
                                                    <option value="web">{move || i18n.tr("device_plat_web", "Web Browser")}</option>
                                                </select>
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">{move || i18n.tr("device_transport_label", "Push Transport")}</label>
                                                <select
                                                    class="exo-input"
                                                    prop:value=move || dev_transport.get()
                                                    on:change=move |ev| set_dev_transport.set(event_target_value(&ev))
                                                >
                                                    <option value="unifiedpush">{move || i18n.tr("device_tr_unifiedpush", "UnifiedPush (Open Standard / ntfy)")}</option>
                                                    <option value="fcm">{move || i18n.tr("device_tr_fcm", "FCM (Firebase Cloud Messaging)")}</option>
                                                    <option value="apns">{move || i18n.tr("device_tr_apns", "APNs (Apple Push Notification service)")}</option>
                                                    <option value="wns">{move || i18n.tr("device_tr_wns", "WNS (Windows Push Notification Services)")}</option>
                                                    <option value="web_push">{move || i18n.tr("device_tr_webpush", "Web Push (VAPID)")}</option>
                                                </select>
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">{move || i18n.tr("device_label_field", "Device Label")}</label>
                                                <input
                                                    type="text"
                                                    class="exo-input"
                                                    prop:value=move || dev_label.get()
                                                    on:input=move |ev| set_dev_label.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">{move || i18n.tr("device_token_label", "Endpoint URL or Device Token")}</label>
                                                <input
                                                    type="text"
                                                    class="exo-input exo-mono"
                                                    required=true
                                                    placeholder=move || i18n.tr("device_token_ph", "https://ntfy.sh/... or FCM/APNs token")
                                                    prop:value=move || dev_token.get()
                                                    on:input=move |ev| set_dev_token.set(event_target_value(&ev))
                                                />
                                            </div>
                                        </div>
                                        <button type="submit" class="exo-btn exo-btn-primary">
                                            {move || i18n.tr("device_register_btn", "Register Push Endpoint")}
                                        </button>
                                    </form>

                                    <div style="overflow-x: auto; margin-top: 16px;">
                                        <table class="exo-table">
                                            <thead>
                                                <tr>
                                                    <th>{move || i18n.tr("device_th_label", "Label")}</th>
                                                    <th>{move || i18n.tr("device_th_platform", "Platform")}</th>
                                                    <th>{move || i18n.tr("device_th_transport", "Transport")}</th>
                                                    <th>{move || i18n.tr("device_th_last_seen", "Last Seen")}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || devices.get()
                                                    key=|d| d.device_id.clone()
                                                    children=move |d| {
                                                        let did = d.device_id.clone();
                                                        view! {
                                                            <tr class=move || live.anim_class("notification_device", &did)>
                                                                <td>{d.device_label}</td>
                                                                <td>{d.platform}</td>
                                                                <td><span class="exo-tag">{d.transport}</span></td>
                                                                <td>{d.last_seen_at}</td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        SettingsSection::AdminDocsAndMaturity => view! {
                            <div>
                                <div class="exo-card">
                                    <h2 class="exo-h2">{move || i18n.tr("admin_allowlist_title", "Documentation Access Allowlist")}</h2>
                                    <p class="exo-sub">
                                        {move || i18n.tr("admin_allowlist_sub", "Controls who can view ExoNovel's MADR 4.0.0 CMS and at which tier. Click any did:peer below to populate.")}
                                    </p>
                                    <form on:submit=add_reader style="display: flex; gap: 10px; flex-wrap: wrap; margin-top: 14px;">
                                        <input
                                            type="text"
                                            class="exo-input"
                                            style="flex: 2; min-width: 240px;"
                                            required=true
                                            placeholder=move || i18n.tr("admin_grant_ph", "did:peer:0z... or email")
                                            prop:value=move || grant_did.get()
                                            on:input=move |ev| set_grant_did.set(event_target_value(&ev))
                                        />
                                        <select
                                            class="exo-input"
                                            style="flex: 1; min-width: 140px;"
                                            prop:value=move || grant_status.get()
                                            on:change=move |ev| set_grant_status.set(event_target_value(&ev))
                                        >
                                            <option value="reader">{move || i18n.tr("status_reader", "Reader")}</option>
                                            <option value="moderator">{move || i18n.tr("role_moderator", "Moderator")}</option>
                                            <option value="pending">{move || i18n.tr("status_pending", "Pending")}</option>
                                            <option value="suspended">{move || i18n.tr("status_suspended", "Suspended")}</option>
                                        </select>
                                        <input
                                            type="text"
                                            class="exo-input"
                                            style="flex: 1; min-width: 160px;"
                                            placeholder=move || i18n.tr("admin_note_ph", "Note")
                                            prop:value=move || new_note.get()
                                            on:input=move |ev| set_new_note.set(event_target_value(&ev))
                                        />
                                        <button type="submit" class="exo-btn exo-btn-primary">{move || i18n.tr("admin_save_access_btn", "Save Access")}</button>
                                    </form>
                                    {move || admin_msg.get().map(|m| view! { <p class="exo-sub">{m}</p> })}

                                    <div style="overflow-x: auto; margin-top: 16px;">
                                        <table class="exo-table">
                                            <thead>
                                                <tr>
                                                    <th>{move || i18n.tr("admin_th_user", "User")}</th>
                                                    <th>{move || i18n.tr("admin_th_email", "Email")}</th>
                                                    <th>{move || i18n.tr("admin_th_status", "Status")}</th>
                                                    <th>{move || i18n.tr("admin_th_actions", "Actions")}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || entries.get()
                                                    key=|e| format!("{}-{}-{}", e.email, e.status, e.updated_at)
                                                    children=move |entry| {
                                                        let is_admin_row = entry.status == "admin" || entry.role == "admin";
                                                        let did_toggle = entry.did_peer.clone();
                                                        let did_del = entry.did_peer.clone();
                                                        let email_toggle = entry.email.clone();
                                                        let email_del = entry.email.clone();
                                                        let did_copy = entry.did_peer.clone().unwrap_or_else(|| entry.email.clone());
                                                        let cur_status = entry.status.clone();
                                                        let cur_status_lbl = cur_status.clone();
                                                        let anim_key = entry.email.clone();
                                                        view! {
                                                            <tr class=move || live.anim_class("allowlist", &anim_key)>
                                                                <td>
                                                                    <div class="exo-id-cell">
                                                                        <button type="button" class="exo-copy-btn" on:click=move |_| on_copy_did(did_copy.clone())>
                                                                            <CopyIcon />
                                                                        </button>
                                                                        <span>{entry.display_name}</span>
                                                                    </div>
                                                                </td>
                                                                <td>{entry.email}</td>
                                                                <td>
                                                                    {if is_admin_row {
                                                                        view! { <span class="exo-tag exo-tag-accent">{move || i18n.tr(allowlist_status_key(&cur_status_lbl), &cur_status_lbl)}</span> }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <select
                                                                                class="exo-input"
                                                                                style="min-width: 140px; padding: 6px 10px;"
                                                                                prop:value=cur_status
                                                                                on:change=move |ev| update_entry_status(did_toggle.clone(), email_toggle.clone(), event_target_value(&ev))
                                                                            >
                                                                                <option value="reader">{move || i18n.tr("status_reader", "Reader")}</option>
                                                                                <option value="moderator">{move || i18n.tr("role_moderator", "Moderator")}</option>
                                                                                <option value="pending">{move || i18n.tr("status_pending", "Pending")}</option>
                                                                                <option value="suspended">{move || i18n.tr("status_suspended", "Suspended")}</option>
                                                                            </select>
                                                                        }.into_any()
                                                                    }}
                                                                </td>
                                                                <td>
                                                                    {if !is_admin_row {
                                                                        view! {
                                                                            <button type="button" class="exo-btn" on:click=move |_| remove_entry(did_del.clone(), email_del.clone())>
                                                                                {move || i18n.tr("admin_remove_btn", "Remove")}
                                                                            </button>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! { <span class="exo-sub">{move || i18n.tr("admin_owner_badge", "Owner")}</span> }.into_any()
                                                                    }}
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                </div>

                                <div class="exo-card">
                                    <h2 class="exo-h2">{move || i18n.tr("admin_accounts_title", "Registered Accounts & Roles")}</h2>
                                    <div style="overflow-x: auto; margin-top: 14px;">
                                        <table class="exo-table">
                                            <thead>
                                                <tr>
                                                    <th>{move || i18n.tr("admin_th_email", "Email")}</th>
                                                    <th>{move || i18n.tr("admin_th_role", "Role")}</th>
                                                    <th>{move || i18n.tr("admin_th_did_populate", "did:peer (click to populate)")}</th>
                                                    <th>{move || i18n.tr("admin_th_provider", "Provider")}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || identities.get()
                                                    key=|id| format!("{}:{}:{}", id.did_peer, id.account_role, id.account_status)
                                                    children=move |id| {
                                                        let did_row = id.did_peer.clone();
                                                        let did_toggle = did_row.clone();
                                                        let did_show = did_row.clone();
                                                        let did_copy = did_row.clone();
                                                        let did_populate = did_row.clone();
                                                        let did_save = StoredValue::new(did_row.clone());
                                                        let did_anim = did_row.clone();
                                                        let email_label = id.email.clone().unwrap_or_else(|| "—".to_string());
                                                        let prof_pop = id.profile.clone();
                                                        let role_pop = id.account_role.clone();
                                                        let st_pop = id.account_status.clone();
                                                        let role_lbl = role_pop.clone();
                                                        let st_lbl = st_pop.clone();
                                                        view! {
                                                            <>
                                                                <tr class=move || live.anim_class("identity", &did_anim)>
                                                                    <td>
                                                                        <button
                                                                            type="button"
                                                                            class="exo-email-trigger"
                                                                            on:click=move |_| {
                                                                                if expanded_did.get_untracked().as_deref() == Some(did_toggle.as_str()) {
                                                                                    set_expanded_did.set(None);
                                                                                } else {
                                                                                    set_edit_display_name.set(prof_pop.display_name.clone());
                                                                                    set_edit_pen_name.set(prof_pop.pen_name.clone());
                                                                                    set_edit_website.set(prof_pop.website.clone());
                                                                                    set_edit_location.set(prof_pop.location.clone());
                                                                                    set_edit_format.set(if prof_pop.preferred_format.is_empty() { "novel".to_string() } else { prof_pop.preferred_format.clone() });
                                                                                    set_edit_account_role.set(role_pop.clone());
                                                                                    set_edit_account_status.set(st_pop.clone());
                                                                                    set_edit_bio.set(prof_pop.bio.clone());
                                                                                    set_edit_msg.set(None);
                                                                                    set_expanded_did.set(Some(did_toggle.clone()));
                                                                                }
                                                                            }
                                                                        >
                                                                            <span>{email_label}</span>
                                                                        </button>
                                                                    </td>
                                                                    <td><span class="exo-tag">{move || format!("{} ({})", i18n.tr(account_role_key(&role_lbl), &role_lbl), i18n.tr(account_status_key(&st_lbl), &st_lbl))}</span></td>
                                                                    <td>
                                                                        <div class="exo-id-cell">
                                                                            <button type="button" class="exo-copy-btn" on:click=move |_| on_copy_did(did_copy.clone())>
                                                                                <CopyIcon />
                                                                            </button>
                                                                            <span class="exo-mono exo-did-clickable" on:click=move |_| set_grant_did.set(did_populate.clone())>
                                                                                {did_row}
                                                                            </span>
                                                                        </div>
                                                                    </td>
                                                                    <td>{id.provider}</td>
                                                                </tr>
                                                                <Show when=move || expanded_did.get().as_deref() == Some(did_show.as_str())>
                                                                    <tr class="exo-accordion-row">
                                                                        <td colspan="4">
                                                                            <div class="exo-grid-2">
                                                                                <select class="exo-input" prop:value=move || edit_account_role.get() on:change=move |ev| set_edit_account_role.set(event_target_value(&ev))>
                                                                                    <option value="user">{move || i18n.tr("role_user", "User")}</option>
                                                                                    <option value="moderator">{move || i18n.tr("role_moderator", "Moderator")}</option>
                                                                                    <option value="admin">{move || i18n.tr("role_admin", "Admin")}</option>
                                                                                </select>
                                                                                <select class="exo-input" prop:value=move || edit_account_status.get() on:change=move |ev| set_edit_account_status.set(event_target_value(&ev))>
                                                                                    <option value="active">{move || i18n.tr("active_badge", "Active")}</option>
                                                                                    <option value="restricted">{move || i18n.tr("status_restricted", "Restricted")}</option>
                                                                                    <option value="suspended">{move || i18n.tr("status_suspended", "Suspended")}</option>
                                                                                </select>
                                                                                <input type="text" class="exo-input" placeholder=move || i18n.tr("admin_display_name_ph", "Display Name") prop:value=move || edit_display_name.get() on:input=move |ev| set_edit_display_name.set(event_target_value(&ev)) />
                                                                                <input type="text" class="exo-input" placeholder=move || i18n.tr("admin_pen_name_ph", "Pen Name") prop:value=move || edit_pen_name.get() on:input=move |ev| set_edit_pen_name.set(event_target_value(&ev)) />
                                                                            </div>
                                                                            <div style="margin-top: 12px; display: flex; gap: 10px; align-items: center;">
                                                                                <button type="button" class="exo-btn exo-btn-primary" on:click=move |_| save_identity_accordion(did_save.get_value())>
                                                                                    {move || i18n.tr("admin_save_account_btn", "Save Account Changes")}
                                                                                </button>
                                                                                {move || edit_msg.get().map(|m| view! { <span class="exo-sub">{m}</span> })}
                                                                            </div>
                                                                        </td>
                                                                    </tr>
                                                                </Show>
                                                            </>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                    }}
                </div>
            </div>
        </main>
    }
}
