use gloo_net::http::Request;
use leptos::prelude::*;

use crate::{
    connected_apps_ui::ConnectedAppsPreferencesPanel,
    i18n::use_i18n,
    live::use_live_sync,
    models::{
        account_role_key, account_status_key, allowlist_status_key, detect_browser_timezone,
        display_name_for_user, is_safe_avatar_url, locale_flag_emoji, resolve_fresh_jwt,
        save_profile_patch, upload_avatar_file, ActiveTab, AllowlistEntry, AllowlistResponse,
        AuthConfigResponse, CopyIcon, IdentitiesResponse, IdentityRecord, NotificationDevice,
        NotificationPreferences, NotificationPrefsResponse, PreferencesSection,
        ProfilePatchResponse, VerifiedUser,
    },
    ui::{
        select_opt, ExoFileButton, ExoSelect, ExoSettingsPage, ExoSettingsSplit, SettingsNavGroup,
        SettingsNavIcon, SettingsNavItem, use_settings_resize,
    },
};

#[cfg(not(feature = "ssr"))]
fn request_browser_notification_permission(on_result: impl Fn(String) + 'static) {
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
fn request_browser_notification_permission(_on_result: impl Fn(String) + 'static) {}

#[component]
pub fn PreferencesView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
    user: VerifiedUser,
    return_tab: ReadSignal<ActiveTab>,
    on_copy_did: impl Fn(String) + Copy + Send + Sync + 'static,
    on_toast: impl Fn(String) + Copy + Send + Sync + 'static,
    on_open_profile_plans: impl Fn() + Copy + Send + Sync + 'static,
    on_profile_updated: impl Fn(ProfilePatchResponse) + Copy + Send + Sync + 'static,
    on_back: impl Fn() + Copy + Send + Sync + 'static,
    #[prop(optional)] initial_section: Option<RwSignal<Option<PreferencesSection>>>,
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

    let (active_section, set_active_section) = signal(
        initial_section
            .and_then(|s| s.get_untracked())
            .unwrap_or(PreferencesSection::Account),
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
    let (profile_msg, set_profile_msg) = signal::<Option<String>>(None);
    let (saving_profile, set_saving_profile) = signal(false);

    // Notification Channels & Devices
    let (notif_prefs, set_notif_prefs) = signal(NotificationPreferences::default());
    let (devices, set_devices) = signal::<Vec<NotificationDevice>>(Vec::new());
    let (email_configured, set_email_configured) = signal(false);
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
                        set_email_configured.set(data.email_configured);
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

    let handle_avatar_data_url = move |data_url: String| {
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        let fallback_jwt = initial_jwt.get_untracked();
        let target_did = user_did.get_value();
        set_profile_msg.set(Some(i18n.tr_untracked(
            "preferences_msg_avatar_processing",
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
                        "preferences_msg_avatar_saved",
                        "Avatar saved.",
                    )));
                }
                Err(e) => set_profile_msg.set(Some(e)),
            }
        });
    };

    let save_preferences_profile = move || {
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
            "uiLocale": next_ui,
            "defaultInputLocale": next_in,
        });
        i18n.ui_locale.set(next_ui);
        i18n.input_locale.set(next_in);

        leptos::task::spawn_local(async move {
            match save_profile_patch(auth_url.as_deref(), fallback_jwt.as_deref(), patch).await {
                Ok(updated) => {
                    on_profile_updated(updated);
                    set_profile_msg.set(Some(i18n.tr_untracked(
                        "preferences_msg_saved",
                        "Preferences saved.",
                    )));
                    on_toast(i18n.tr_untracked("preferences_toast_saved", "Preferences saved"));
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
                            "preferences_toast_notif_saved",
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
                            "preferences_toast_device_reg",
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
                        "preferences_toast_test_sent",
                        "Test notification & email dispatched",
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
                            "preferences_msg_allowlist_updated",
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
                            "preferences_toast_tier_updated",
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
                        "preferences_toast_entry_removed",
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
                            "preferences_msg_account_updated",
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
        ActiveTab::Guides => format!("← {}", i18n.tr("nav_dear_world", "Dear World")),
        _ => i18n.tr("back_to_desk", "← Back to Desk"),
    };

    let format_options = Signal::derive(move || {
        vec![
            select_opt("novel", i18n.tr("format_opt_novel", "Novels")),
            select_opt("article", i18n.tr("format_opt_article", "Articles & Nonfiction")),
            select_opt("short-story", i18n.tr("format_opt_short", "Short Stories")),
            select_opt("screenplay", i18n.tr("format_opt_screenplay", "Screenplays")),
            select_opt("poetry", i18n.tr("format_opt_poetry", "Poetry")),
        ]
    });

    let ui_lang_options = Signal::derive(move || {
        i18n.catalog
            .get()
            .into_iter()
            .map(|loc| {
                let lbl = format!(
                    "{} {} — {} [{}]",
                    locale_flag_emoji(&loc.code),
                    loc.native_name,
                    loc.english_name,
                    loc.dir.to_uppercase()
                );
                select_opt(loc.code, lbl)
            })
            .collect()
    });

    let in_lang_options = Signal::derive(move || {
        i18n.catalog
            .get()
            .into_iter()
            .map(|loc| {
                let lbl = format!(
                    "{} {} — {}",
                    locale_flag_emoji(&loc.code),
                    loc.native_name,
                    loc.english_name
                );
                select_opt(loc.code, lbl)
            })
            .collect()
    });

    let digest_options = Signal::derive(move || {
        vec![
            select_opt("instant", i18n.tr("notif_digest_instant", "Instant Email")),
            select_opt("daily", i18n.tr("notif_digest_daily", "Daily Digest")),
            select_opt("off", i18n.tr("notif_digest_off", "Email Disabled")),
        ]
    });

    let platform_options = Signal::derive(move || {
        vec![
            select_opt("android", i18n.tr("device_plat_android", "Android")),
            select_opt("ios", i18n.tr("device_plat_ios", "iOS")),
            select_opt("linux", i18n.tr("device_plat_linux", "Linux")),
            select_opt("macos", i18n.tr("device_plat_macos", "macOS")),
            select_opt("windows", i18n.tr("device_plat_windows", "Windows")),
            select_opt("web", i18n.tr("device_plat_web", "Web Browser")),
        ]
    });

    let transport_options = Signal::derive(move || {
        vec![
            select_opt("unifiedpush", i18n.tr("device_tr_unifiedpush", "UnifiedPush (Open Standard)")),
            select_opt("fcm", i18n.tr("device_tr_fcm", "FCM (Firebase Cloud Messaging)")),
            select_opt("apns", i18n.tr("device_tr_apns", "APNs (Apple Push Notification service)")),
            select_opt("wns", i18n.tr("device_tr_wns", "WNS (Windows Push Notification Services)")),
            select_opt("web_push", i18n.tr("device_tr_webpush", "Web Push (VAPID)")),
        ]
    });

    let allowlist_status_options = Signal::derive(move || {
        vec![
            select_opt("reader", i18n.tr("status_reader", "Reader")),
            select_opt("moderator", i18n.tr("role_moderator", "Moderator")),
            select_opt("pending", i18n.tr("status_pending", "Pending")),
            select_opt("suspended", i18n.tr("status_suspended", "Suspended")),
        ]
    });

    let account_role_options = Signal::derive(move || {
        vec![
            select_opt("user", i18n.tr("role_user", "User")),
            select_opt("moderator", i18n.tr("role_moderator", "Moderator")),
            select_opt("admin", i18n.tr("role_admin", "Admin")),
        ]
    });

    let account_status_options = Signal::derive(move || {
        vec![
            select_opt("active", i18n.tr("active_badge", "Active")),
            select_opt("restricted", i18n.tr("status_restricted", "Restricted")),
            select_opt("suspended", i18n.tr("status_suspended", "Suspended")),
        ]
    });

    let resize = use_settings_resize(
        prof.preferences_nav_width,
        Callback::new(move |new_w: i32| {
            let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
            let fallback_jwt = initial_jwt.get_untracked();
            leptos::task::spawn_local(async move {
                if let Ok(updated) = save_profile_patch(
                    auth_url.as_deref(),
                    fallback_jwt.as_deref(),
                    serde_json::json!({ "preferencesNavWidth": new_w }),
                )
                .await
                {
                    on_profile_updated(updated);
                }
            });
        }),
    );

    let nav_groups = vec![
        SettingsNavGroup {
            divider_before: false,
            heading: Some(("nav_group_account", "Account")),
            visible: true,
            items: vec![
                SettingsNavItem {
                    icon: SettingsNavIcon::User,
                    label_key: "sec_account",
                    label_fallback: "Account & Sovereign Identity",
                    is_active: Signal::derive(move || {
                        active_section.get() == PreferencesSection::Account
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_section.set(PreferencesSection::Account)
                    }),
                },
                SettingsNavItem {
                    icon: SettingsNavIcon::Apps,
                    label_key: "sec_connected_apps",
                    label_fallback: "Connected Apps & Google Docs",
                    is_active: Signal::derive(move || {
                        active_section.get() == PreferencesSection::ConnectedApps
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_section.set(PreferencesSection::ConnectedApps)
                    }),
                },
            ],
        },
        SettingsNavGroup {
            divider_before: true,
            heading: Some(("nav_group_workspace", "Workspace & Delivery")),
            visible: true,
            items: vec![
                SettingsNavItem {
                    icon: SettingsNavIcon::Globe,
                    label_key: "sec_appearance_langs",
                    label_fallback: "Languages (29)",
                    is_active: Signal::derive(move || {
                        active_section.get() == PreferencesSection::AppearanceAndLanguages
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_section.set(PreferencesSection::AppearanceAndLanguages)
                    }),
                },
                SettingsNavItem {
                    icon: SettingsNavIcon::Sync,
                    label_key: "sec_sync_storage",
                    label_fallback: "Web, Device & Collaboration Sync",
                    is_active: Signal::derive(move || {
                        active_section.get() == PreferencesSection::SyncAndStorage
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_section.set(PreferencesSection::SyncAndStorage)
                    }),
                },
                SettingsNavItem {
                    icon: SettingsNavIcon::Bell,
                    label_key: "sec_notifications",
                    label_fallback: "Notification Channels",
                    is_active: Signal::derive(move || {
                        active_section.get() == PreferencesSection::NotificationChannels
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_section.set(PreferencesSection::NotificationChannels)
                    }),
                },
            ],
        },
        SettingsNavGroup {
            divider_before: true,
            heading: Some(("nav_group_admin", "Administration")),
            visible: is_admin,
            items: vec![SettingsNavItem {
                icon: SettingsNavIcon::Shield,
                label_key: "sec_admin_allowlist",
                label_fallback: "Docs Allowlist & Accounts",
                is_active: Signal::derive(move || {
                    active_section.get() == PreferencesSection::AdminDocsAndMaturity
                }),
                on_click: Callback::new(move |_| {
                    set_active_section.set(PreferencesSection::AdminDocsAndMaturity)
                }),
            }],
        },
    ];

    view! {
        <ExoSettingsPage resize=resize>
            <div class="exo-gh-header">
                <div>
                    <h1 class="exo-h1">{move || i18n.tr("preferences_title", "Preferences")}</h1>
                    <p class="exo-sub" style="margin: 4px 0 0;">
                        {move || i18n.tr("preferences_subtitle", "Account, language, sync overview, and notification delivery preferences (purchases, paywalls, and plans live in Profile).")}
                    </p>
                </div>
                <button type="button" class="exo-btn" on:click=move |_| on_back()>
                    {back_label}
                </button>
            </div>

            <ExoSettingsSplit
                resize=resize
                nav_aria_label=Signal::derive(move || i18n.tr("preferences_nav_aria", "Preferences sections"))
                groups=nav_groups
            >
                    {move || match active_section.get() {
                        PreferencesSection::Account => {
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
                                            <ExoFileButton
                                                label=Signal::derive(move || i18n.tr("choose_image_btn", "Choose Profile Photo"))
                                                on_data_url=Callback::new(handle_avatar_data_url)
                                                on_error=Callback::new(move |err| set_profile_msg.set(Some(err)))
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
                                            <ExoSelect
                                                value=Signal::derive(move || preferred_format.get())
                                                options=format_options
                                                aria_label=Signal::derive(move || i18n.tr("preferred_format_label", "Preferred Writing Format"))
                                                on_change=Callback::new(move |val| set_preferred_format.set(val))
                                            />
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
                                        on:click=move |_| save_preferences_profile()
                                    >
                                        {move || i18n.tr("save_account_btn", "Save Account Preferences")}
                                    </button>
                                </div>
                            }.into_any()
                        }

                        PreferencesSection::ConnectedApps => view! {
                            <ConnectedAppsPreferencesPanel
                                auth_config=auth_config
                                initial_jwt=initial_jwt
                                on_toast=on_toast
                            />
                        }.into_any(),

                        PreferencesSection::AppearanceAndLanguages => view! {
                            <div class="exo-card">
                                <h2 class="exo-h2">{move || i18n.tr("appearance_title", "Dual Language Architecture (29 Languages)")}</h2>
                                <p class="exo-sub">
                                    {move || i18n.tr("appearance_desc", "App Language controls menus, navigation, and layout direction (automatic right-to-left for Arabic and Hebrew). Writing Input controls composers independently and saves directly to your account.")}
                                </p>

                                <div class="exo-grid-2" style="margin-top: 16px;">
                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("ui_lang_label", "App Language")}</label>
                                        <ExoSelect
                                            value=Signal::derive(move || ui_loc.get())
                                            options=ui_lang_options
                                            aria_label=Signal::derive(move || i18n.tr("ui_lang_label", "App Language"))
                                            on_change=Callback::new(move |val| set_ui_loc.set(val))
                                        />
                                    </div>

                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("input_lang_label", "Writing Input")}</label>
                                        <ExoSelect
                                            value=Signal::derive(move || in_loc.get())
                                            options=in_lang_options
                                            aria_label=Signal::derive(move || i18n.tr("input_lang_label", "Writing Input"))
                                            on_change=Callback::new(move |val| set_in_loc.set(val))
                                        />
                                    </div>
                                </div>

                                <div class="exo-format-tile" style="margin-top: 14px;">
                                    <h3>{move || i18n.tr("offline_packs_title", "Decentralized Offline Language Packs")}</h3>
                                    <p>
                                        {move || i18n.tr("offline_packs_desc", "ExoNovel device installs download any selected language pack directly from ExoNovel's public read-only Willow namespace: ")}
                                        " "
                                        <span class="exo-mono">{move || i18n.willow_namespace.get()}</span>
                                    </p>
                                </div>

                                <button
                                    type="button"
                                    class="exo-btn exo-btn-primary"
                                    style="margin-top: 18px;"
                                    on:click=move |_| save_preferences_profile()
                                >
                                    {move || i18n.tr("save_theme_lang_btn", "Save Language Preferences")}
                                </button>
                            </div>
                        }.into_any(),

                        PreferencesSection::SyncAndStorage => view! {
                            <div class="exo-card">
                                <h2 class="exo-h2">{move || i18n.tr("sync_title", "Writing on Web, on Device & Collaboration Sync")}</h2>
                                <p class="exo-sub">
                                    {move || i18n.tr("sync_sub", "Use ExoNovel for free in your browser and free offline on your devices. Team collaboration sync between writers is free across both.")}
                                </p>

                                <div class="exo-grid-3" style="margin-top: 16px;">
                                    <div class="exo-format-tile">
                                        <h3>{move || i18n.tr("sync_m1_title", "On the Web (Free)")}</h3>
                                        <p>{move || i18n.tr("sync_m1_desc", "Cloud workspace accessible from any browser with live collaboration across writers and groups.")}</p>
                                    </div>
                                    <div class="exo-format-tile">
                                        <h3>{move || i18n.tr("sync_m2_title", "On Your Device (Free)")}</h3>
                                        <p>{move || i18n.tr("sync_m2_desc", "Offline-first storage on Windows, macOS, Linux, Android, and iOS—write anywhere and keep full local control of your library.")}</p>
                                    </div>
                                    <div class="exo-format-tile" style="border-color: var(--accent);">
                                        <h3>{move || i18n.tr("sync_m3_title", "Sync Your Own Devices ($9/mo · $90/yr)")}</h3>
                                        <p>{move || i18n.tr("sync_m3_desc", "End-to-end encrypted sync keeping your own offline device drafts continuously aligned with your own web profile.")}</p>
                                    </div>
                                </div>

                                <div class="exo-format-tile" style="margin-top: 16px; border-color: var(--accent);">
                                    <h3>{move || i18n.tr("sync_collab_title", "Collaboration Between Writers Is Always Free (Web ↔ Device)")}</h3>
                                    <p>
                                        {move || i18n.tr("sync_collab_desc", "Whenever you add collaborators, sync between collaborators happens automatically so you can share your work and write together—even if one writer works on the web while another writes offline on a device. Collaboration exchange between users is always free; only syncing your own device content with your own web profile carries a fee in Profile -> Plans.")}
                                    </p>
                                </div>

                                <div class="exo-format-tile" style="margin-top: 16px;">
                                    <h3>{move || i18n.tr("sync_zk_title", "Zero-Knowledge Cryptographic Privacy")}</h3>
                                    <p>
                                        {move || i18n.tr("sync_zk_desc", "Device-synced drafts and shared payloads are end-to-end encrypted by the sender before passing through ExoNovel's relay infrastructure.")}
                                    </p>
                                </div>

                                <div style="margin-top: 18px; display: flex; justify-content: flex-end;">
                                    <button
                                        type="button"
                                        class="exo-btn exo-btn-primary"
                                        on:click=move |_| on_open_profile_plans()
                                    >
                                        {move || i18n.tr("sync_manage_in_profile", "View Sync Plans in Profile →")}
                                    </button>
                                </div>
                            </div>
                        }.into_any(),

                        PreferencesSection::NotificationChannels => view! {
                            <div>
                                <div class="exo-card">
                                    <div class="exo-gh-section-head-row">
                                        <div>
                                            <h2 class="exo-h2">{move || i18n.tr("notif_channels_title", "Multi-Platform Delivery Channels")}</h2>
                                            <p class="exo-sub" style="margin: 4px 0 0;">
                                                {move || i18n.tr("notif_email_prefix", "Transactional Email Service (notes@exosystems.net)")}
                                                " "
                                                {move || if email_configured.get() {
                                                    i18n.tr("notif_email_active", "· Email Delivery Active")
                                                } else {
                                                    i18n.tr("notif_email_unset", "· Transactional email credentials not configured")
                                                }}
                                            </p>
                                        </div>
                                        <button type="button" class="exo-btn" on:click=send_test_notification>
                                            {move || i18n.tr("notif_send_test", "Send Test Notification & Email")}
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
                                                    <span><strong>{move || i18n.tr("notif_in_app_title", "In-App Live Feed & Bell")}</strong> " — " {move || i18n.tr("notif_in_app_desc", "Instant delivery over the persistent real-time stream.")}</span>
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
                                                                    on_toast(format!("{} {st}", i18n.tr_untracked("preferences_toast_perm", "Browser notification permission:")));
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
                                                    <span><strong>{move || i18n.tr("notif_email_title", "Transactional Email (Notes from ExoNovel)")}</strong></span>
                                                </label>
                                                <div class="exo-field" style="margin-top: 14px;">
                                                    <label class="exo-label">{move || i18n.tr("notif_digest_label", "Email Digest Frequency")}</label>
                                                    <ExoSelect
                                                        value=Signal::derive(move || notif_prefs.get().email_digest_mode)
                                                        options=digest_options
                                                        style_extra="max-width: 280px;"
                                                        aria_label=Signal::derive(move || i18n.tr("notif_digest_label", "Email Digest Frequency"))
                                                        on_change=Callback::new(move |val| {
                                                            set_notif_prefs.update(|np| np.email_digest_mode = val);
                                                        })
                                                    />
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
                                                <ExoSelect
                                                    value=Signal::derive(move || dev_platform.get())
                                                    options=platform_options
                                                    aria_label=Signal::derive(move || i18n.tr("device_platform_label", "Platform"))
                                                    on_change=Callback::new(move |val| set_dev_platform.set(val))
                                                />
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">{move || i18n.tr("device_transport_label", "Push Transport")}</label>
                                                <ExoSelect
                                                    value=Signal::derive(move || dev_transport.get())
                                                    options=transport_options
                                                    aria_label=Signal::derive(move || i18n.tr("device_transport_label", "Push Transport"))
                                                    on_change=Callback::new(move |val| set_dev_transport.set(val))
                                                />
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
                                                    placeholder=move || i18n.tr("device_token_ph", "https://push.example.org/... or FCM/APNs token")
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

                        PreferencesSection::AdminDocsAndMaturity => view! {
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
                                        <ExoSelect
                                            value=Signal::derive(move || grant_status.get())
                                            options=allowlist_status_options
                                            style_extra="flex: 1; min-width: 140px;"
                                            aria_label=Signal::derive(move || i18n.tr("admin_th_status", "Status"))
                                            on_change=Callback::new(move |val| set_grant_status.set(val))
                                        />
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
                                                                        let cur_status_sv = cur_status.clone();
                                                                        view! {
                                                                            <ExoSelect
                                                                                value=Signal::derive(move || cur_status_sv.clone())
                                                                                options=allowlist_status_options
                                                                                style_extra="min-width: 140px;"
                                                                                aria_label=Signal::derive(move || i18n.tr("admin_th_status", "Status"))
                                                                                on_change=Callback::new(move |val| update_entry_status(did_toggle.clone(), email_toggle.clone(), val))
                                                                            />
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
                                                                                <ExoSelect
                                                                                    value=Signal::derive(move || edit_account_role.get())
                                                                                    options=account_role_options
                                                                                    aria_label=Signal::derive(move || i18n.tr("admin_th_role", "Role"))
                                                                                    on_change=Callback::new(move |val| set_edit_account_role.set(val))
                                                                                />
                                                                                <ExoSelect
                                                                                    value=Signal::derive(move || edit_account_status.get())
                                                                                    options=account_status_options
                                                                                    aria_label=Signal::derive(move || i18n.tr("admin_th_status", "Status"))
                                                                                    on_change=Callback::new(move |val| set_edit_account_status.set(val))
                                                                                />
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
            </ExoSettingsSplit>
        </ExoSettingsPage>
    }
}
