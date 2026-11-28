use gloo_net::http::Request;
use leptos::prelude::*;

use crate::{
    i18n::{use_i18n, InputLocaleBar},
    live::use_live_sync,
    models::{
        active_persona_for_user, copy_text_to_clipboard, display_name_for_user,
        invite_url_for_code, is_safe_avatar_url, load_authenticated_user, resolve_fresh_jwt,
        save_profile_patch, upload_avatar_file, ActiveTab, AuthConfigResponse, BillingLedgerItem,
        BillingResponse, CopyIcon, CreateInviteResponse, GroupView, GroupsResponse, InviteView,
        InvitesResponse, LocaleFlag, NotificationPreferences, NotificationPrefsResponse,
        PersonasResponse, ProfileTab, ReferralView, VerifiedUser,
    },
    ui::{
        select_opt, ExoFileButton, ExoSelect, ExoSettingsPage, ExoSettingsSplit, SettingsNavGroup,
        SettingsNavIcon, SettingsNavItem, normalize_settings_nav_width, use_settings_resize,
    },
};

#[component]
pub fn ProfileView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
    current_user: ReadSignal<Option<VerifiedUser>>,
    set_current_user: WriteSignal<Option<VerifiedUser>>,
    return_tab: ReadSignal<ActiveTab>,
    on_copy_did: impl Fn(String) + Copy + Send + Sync + 'static,
    on_toast: impl Fn(String) + Copy + Send + Sync + 'static,
    on_back: impl Fn() + Copy + Send + Sync + 'static,
    #[prop(optional)] initial_tab: Option<RwSignal<Option<ProfileTab>>>,
) -> impl IntoView {
    let live = use_live_sync();
    let i18n = use_i18n();

    let (active_profile_tab, set_active_profile_tab) = signal(
        initial_tab
            .and_then(|s| s.get_untracked())
            .unwrap_or(ProfileTab::Overview),
    );
    if let Some(ext_sig) = initial_tab {
        Effect::new(move |_| {
            if let Some(tab) = ext_sig.get() {
                set_active_profile_tab.set(tab);
                ext_sig.set(None);
            }
        });
    }
    let (groups, set_groups) = signal::<Vec<GroupView>>(Vec::new());
    let (invites, set_invites) = signal::<Vec<InviteView>>(Vec::new());
    let (referrals, set_referrals) = signal::<Vec<ReferralView>>(Vec::new());
    let (referred_by, set_referred_by) = signal::<Option<String>>(None);
    let (billing_entries, set_billing_entries) = signal::<Vec<BillingLedgerItem>>(Vec::new());
    let (exo_sync_peer_did, set_exo_sync_peer_did) =
        signal("did:peer:0zExoNovelRelaySyncService".to_string());
    let (notif_prefs, set_notif_prefs) = signal(NotificationPreferences::default());
    let (profile_notice, set_profile_notice) = signal::<Option<String>>(None);

    // Personal Web <-> Device Sync Plan form (100% Commercial on Profile -> Plans per ADR 004 / 008)
    let (billing_currency, set_billing_currency) = signal("USD".to_string());
    let (billing_rail, set_billing_rail) = signal("card-stripe".to_string());

    // Create Persona form
    let (new_persona_name, set_new_persona_name) = signal(String::new());
    let (new_persona_handle, set_new_persona_handle) = signal(String::new());
    let (new_persona_bio, set_new_persona_bio) = signal(String::new());

    // Inline Edit Persona
    let (editing_persona_did, set_editing_persona_did) = signal::<Option<String>>(None);
    let (edit_p_name, set_edit_p_name) = signal(String::new());
    let (edit_p_handle, set_edit_p_handle) = signal(String::new());
    let (edit_p_bio, set_edit_p_bio) = signal(String::new());

    // Create Group form
    let (new_group_name, set_new_group_name) = signal(String::new());
    let (new_group_desc, set_new_group_desc) = signal(String::new());

    // Pinned languages
    let (pinned_ui, set_pinned_ui) = signal::<Vec<String>>(Vec::new());
    let (pinned_input, set_pinned_input) = signal::<Vec<String>>(Vec::new());

    Effect::new(move |_| {
        if let Some(u) = current_user.get() {
            if let Some(p) = u.profile {
                set_pinned_ui.set(p.preferred_ui_locales);
                set_pinned_input.set(p.preferred_input_locales);
            }
        }
    });

    let reload_profile_hub = move || {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(resp) = Request::get("/api/groups")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<GroupsResponse>().await {
                        set_groups.set(data.groups);
                    }
                }
            }
            if let Ok(resp) = Request::get("/api/invites")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<InvitesResponse>().await {
                        set_invites.set(data.invites);
                        set_referrals.set(data.referrals);
                        set_referred_by.set(data.referred_by);
                    }
                }
            }
            if let Ok(resp) = Request::get("/api/billing")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<BillingResponse>().await {
                        set_billing_entries.set(data.entries);
                        set_exo_sync_peer_did.set(data.exonovel_sync_peer_did);
                    }
                }
            }
            if let Ok(resp) = Request::get("/api/notifications/preferences")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<NotificationPrefsResponse>().await {
                        set_notif_prefs.set(data.preferences);
                    }
                }
            }
        });
    };

    Effect::new(move |_| reload_profile_hub());

    Effect::new(move |_| {
        let _seq = live.pulse_seq.get();
        if live.matches_kind(&[
            "group",
            "invite",
            "persona",
            "billing",
            "notification_prefs",
            "profile",
            "system",
        ]) {
            reload_profile_hub();
        }
    });

    let switch_active_persona = move |persona_did: String, label: String| {
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        let fallback_jwt = initial_jwt.get_untracked();
        leptos::task::spawn_local(async move {
            match save_profile_patch(
                auth_url.as_deref(),
                fallback_jwt.as_deref(),
                serde_json::json!({ "activePersonaDid": persona_did }),
            )
            .await
            {
                Ok(updated) => {
                    set_current_user.update(|u_opt| {
                        if let Some(u) = u_opt {
                            u.profile = Some(updated.profile);
                            u.personas = updated.personas;
                            u.active_persona = updated.active_persona;
                        }
                    });
                    let toast_prefix = i18n.tr_untracked("toast_switched_persona", "Switched persona");
                    on_toast(format!("{toast_prefix}: {label}"));
                }
                Err(e) => set_profile_notice.set(Some(e)),
            }
        });
    };

    let create_new_persona = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name_val = new_persona_name.get_untracked().trim().to_string();
        if name_val.is_empty() {
            return;
        }
        let handle_val = new_persona_handle.get_untracked().trim().to_string();
        let bio_val = new_persona_bio.get_untracked().trim().to_string();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/personas")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "displayName": name_val,
                    "handle": handle_val,
                    "bio": bio_val,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<PersonasResponse>().await {
                            set_current_user.update(|u_opt| {
                                if let Some(u) = u_opt {
                                    u.personas = data.personas;
                                }
                            });
                            set_new_persona_name.set(String::new());
                            set_new_persona_handle.set(String::new());
                            set_new_persona_bio.set(String::new());
                            on_toast(i18n.tr_untracked(
                                "profile_toast_persona_created",
                                "Persona created",
                            ));
                        }
                    } else {
                        let msg = resp.text().await.unwrap_or_default();
                        set_profile_notice.set(Some(msg));
                    }
                }
            }
        });
    };

    let save_persona_edits = move |target_did: String| {
        let name_val = edit_p_name.get_untracked().trim().to_string();
        let handle_val = edit_p_handle.get_untracked().trim().to_string();
        let bio_val = edit_p_bio.get_untracked().trim().to_string();
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::put("/api/personas")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "personaDid": target_did,
                    "displayName": name_val,
                    "handle": handle_val,
                    "bio": bio_val,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        if let Some(url) = auth_url.as_deref() {
                            if let Some((fresh_user, _)) = load_authenticated_user(url, None).await {
                                set_current_user.set(Some(fresh_user));
                            }
                        }
                        set_editing_persona_did.set(None);
                        on_toast(i18n.tr_untracked(
                            "profile_toast_persona_updated",
                            "Persona updated",
                        ));
                    }
                }
            }
        });
    };

    let upload_persona_avatar_data_url = move |target_did: String, data_url: String| {
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        let fallback_jwt = initial_jwt.get_untracked();
        leptos::task::spawn_local(async move {
            if upload_avatar_file(
                auth_url.as_deref(),
                fallback_jwt.as_deref(),
                data_url,
                Some(target_did),
            )
            .await
            .is_ok()
            {
                if let Some(url) = auth_url.as_deref() {
                    if let Some((fresh_user, _)) = load_authenticated_user(url, None).await {
                        set_current_user.set(Some(fresh_user));
                    }
                }
                on_toast(i18n.tr_untracked(
                    "profile_toast_avatar_done",
                    "Persona avatar processed",
                ));
            }
        });
    };

    let create_new_group = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name_val = new_group_name.get_untracked().trim().to_string();
        if name_val.is_empty() {
            return;
        }
        let desc_val = new_group_desc.get_untracked().trim().to_string();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/groups")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "name": name_val,
                    "description": desc_val,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_new_group_name.set(String::new());
                        set_new_group_desc.set(String::new());
                        reload_profile_hub();
                        on_toast(i18n.tr_untracked(
                            "profile_toast_group_created",
                            "Group created",
                        ));
                    }
                }
            }
        });
    };

    let set_member_role = move |group_id: String, member_did: String, next_role: &'static str| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::put("/api/groups/member-role")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "groupId": group_id,
                    "memberOwnerDid": member_did,
                    "role": next_role,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        reload_profile_hub();
                        let role_disp = i18n.tr_untracked(
                            if next_role == "manager" {
                                "status_manager"
                            } else {
                                "status_member"
                            },
                            next_role,
                        );
                        on_toast(format!(
                            "{}: {role_disp}",
                            i18n.tr_untracked(
                                "profile_toast_role_updated",
                                "Member role updated"
                            )
                        ));
                    }
                }
            }
        });
    };

    let create_and_copy_invite = move |group_id_opt: Option<String>| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/invites")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "groupId": group_id_opt,
                    "targetGroupRole": "member",
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        if let Ok(created) = resp.json::<CreateInviteResponse>().await {
                            let url = invite_url_for_code(&created.invite.code);
                            copy_text_to_clipboard(&url);
                            reload_profile_hub();
                            on_toast(i18n.tr_untracked(
                                "profile_toast_invite_copied",
                                "Invite link copied",
                            ));
                        }
                    }
                }
            }
        });
    };

    let activate_sync_plan = move |plan_tier: &'static str, usd_cents: i64| {
        let cur = billing_currency.get_untracked();
        let rail = billing_rail.get_untracked();
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/billing/sync-plan")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "planTier": plan_tier,
                    "currency": cur,
                    "localAmountCents": usd_cents,
                    "paymentRail": rail,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        if let Some(url) = auth_url.as_deref() {
                            if let Some((fresh_user, _)) = load_authenticated_user(url, None).await {
                                set_current_user.set(Some(fresh_user));
                            }
                        }
                        reload_profile_hub();
                        on_toast(format!(
                            "{}: {plan_tier}",
                            i18n.tr_untracked(
                                "profile_toast_plan_updated",
                                "Sync plan updated"
                            )
                        ));
                    }
                }
            }
        });
    };

    let save_pinned_languages = move |_| {
        let ui_list = pinned_ui.get_untracked();
        let in_list = pinned_input.get_untracked();
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        let fallback_jwt = initial_jwt.get_untracked();
        leptos::task::spawn_local(async move {
            match save_profile_patch(
                auth_url.as_deref(),
                fallback_jwt.as_deref(),
                serde_json::json!({
                    "preferredUiLocales": ui_list,
                    "preferredInputLocales": in_list,
                }),
            )
            .await
            {
                Ok(updated) => {
                    set_current_user.update(|u_opt| {
                        if let Some(u) = u_opt {
                            u.profile = Some(updated.profile);
                        }
                    });
                    on_toast(i18n.tr_untracked(
                        "profile_toast_langs_saved",
                        "Language quick-switchers saved",
                    ));
                }
                Err(e) => set_profile_notice.set(Some(e)),
            }
        });
    };

    let save_profile_notif_prefs = move |_| {
        let p = notif_prefs.get_untracked();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::put("/api/notifications/preferences")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "notifyAuthorEarnings": p.notify_author_earnings,
                    "notifyReaderPurchases": p.notify_reader_purchases,
                    "notifyGroupInvites": p.notify_group_invites,
                    "notifyReferralSignups": p.notify_referral_signups,
                    "notifyModeratorComments": p.notify_moderator_comments,
                    "notifyCollaboratorComments": p.notify_collaborator_comments,
                    "notifyMentionsAndForks": p.notify_mentions_and_forks,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        on_toast(i18n.tr_untracked(
                            "profile_toast_notif_saved",
                            "Activity notification preferences saved",
                        ));
                    }
                }
            }
        });
    };

    let toggle_pin_code = move |list_sig: WriteSignal<Vec<String>>, code: String| {
        list_sig.update(|vec| {
            if let Some(pos) = vec.iter().position(|x| *x == code) {
                vec.remove(pos);
            } else {
                vec.push(code);
            }
        });
    };

    let back_label = move || match return_tab.get() {
        ActiveTab::Docs => format!("← {}", i18n.tr("nav_docs", "Docs & ADRs")),
        ActiveTab::Guides => format!("← {}", i18n.tr("nav_dear_world", "Dear World")),
        _ => i18n.tr("back_to_desk", "← Back to Desk"),
    };

    let plan_badge = move || {
        current_user
            .get()
            .and_then(|u| u.profile)
            .map(|p| p.sync_plan_tier)
            .unwrap_or_else(|| "free".to_string())
    };

    let currency_options = Signal::derive(move || {
        vec![
            select_opt("USD", i18n.tr("wallet_cur_usd", "USD ($9/mo · $90/yr)")),
            select_opt("EUR", i18n.tr("wallet_cur_eur", "EUR (€9/mo · €90/yr)")),
            select_opt("GBP", i18n.tr("wallet_cur_gbp", "GBP (£7/mo · £70/yr)")),
            select_opt("PLN", i18n.tr("wallet_cur_pln", "PLN (36 zł/mo · 360 zł/yr)")),
            select_opt("NGN", i18n.tr("wallet_cur_ngn", "NGN (₦13,500/mo · ₦135,000/yr)")),
            select_opt("KES", i18n.tr("wallet_cur_kes", "KES (KSh 1,200/mo · KSh 12,000/yr)")),
            select_opt("INR", i18n.tr("wallet_cur_inr", "INR (₹750/mo · ₹7,500/yr)")),
            select_opt("BRL", i18n.tr("wallet_cur_brl", "BRL (R$45/mo · R$450/yr)")),
        ]
    });

    let rail_options = Signal::derive(move || {
        vec![
            select_opt("card-stripe", i18n.tr("wallet_rail_card", "Card / Local Currency Gateway")),
            select_opt("solana-usdc", i18n.tr("wallet_rail_solana", "Solana Pay / USDC")),
            select_opt("bitcoin-lightning", i18n.tr("wallet_rail_btc", "Bitcoin Lightning (L402)")),
            select_opt("ethereum-base", i18n.tr("wallet_rail_eth", "Base / Arbitrum / Ethereum")),
            select_opt("mobile-money", i18n.tr("wallet_rail_mobile", "Mobile Money / PIX / UPI")),
        ]
    });

    let initial_nav_w = current_user
        .get_untracked()
        .and_then(|u| u.profile)
        .map(|p| p.profile_nav_width)
        .unwrap_or(260);

    let resize = use_settings_resize(
        initial_nav_w,
        Callback::new(move |new_w: i32| {
            let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
            let fallback_jwt = initial_jwt.get_untracked();
            leptos::task::spawn_local(async move {
                if let Ok(updated) = save_profile_patch(
                    auth_url.as_deref(),
                    fallback_jwt.as_deref(),
                    serde_json::json!({ "profileNavWidth": new_w }),
                )
                .await
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
        }),
    );

    Effect::new(move |_| {
        if !resize.dragging.get_untracked() {
            if let Some(w) = current_user
                .get()
                .and_then(|u| u.profile)
                .map(|p| normalize_settings_nav_width(p.profile_nav_width))
            {
                resize.width.set(w);
            }
        }
    });

    let nav_groups = vec![
        SettingsNavGroup {
            divider_before: false,
            heading: None,
            visible: true,
            items: vec![SettingsNavItem {
                icon: SettingsNavIcon::Overview,
                label_key: "tab_overview",
                label_fallback: "Overview",
                is_active: Signal::derive(move || {
                    active_profile_tab.get() == ProfileTab::Overview
                }),
                on_click: Callback::new(move |_| set_active_profile_tab.set(ProfileTab::Overview)),
            }],
        },
        SettingsNavGroup {
            divider_before: true,
            heading: Some(("nav_group_writing", "Writing & Collaboration")),
            visible: true,
            items: vec![
                SettingsNavItem {
                    icon: SettingsNavIcon::Book,
                    label_key: "tab_works",
                    label_fallback: "Works",
                    is_active: Signal::derive(move || {
                        active_profile_tab.get() == ProfileTab::Works
                    }),
                    on_click: Callback::new(move |_| set_active_profile_tab.set(ProfileTab::Works)),
                },
                SettingsNavItem {
                    icon: SettingsNavIcon::Users,
                    label_key: "tab_groups",
                    label_fallback: "Groups & Invites",
                    is_active: Signal::derive(move || {
                        active_profile_tab.get() == ProfileTab::Groups
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_profile_tab.set(ProfileTab::Groups)
                    }),
                },
                SettingsNavItem {
                    icon: SettingsNavIcon::Personas,
                    label_key: "tab_personas",
                    label_fallback: "Personas",
                    is_active: Signal::derive(move || {
                        active_profile_tab.get() == ProfileTab::Personas
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_profile_tab.set(ProfileTab::Personas)
                    }),
                },
            ],
        },
        SettingsNavGroup {
            divider_before: true,
            heading: Some(("nav_group_commerce", "Billing & Reader Access")),
            visible: true,
            items: vec![
                SettingsNavItem {
                    icon: SettingsNavIcon::Wallet,
                    label_key: "tab_wallet",
                    label_fallback: "Wallet",
                    is_active: Signal::derive(move || {
                        active_profile_tab.get() == ProfileTab::Wallet
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_profile_tab.set(ProfileTab::Wallet)
                    }),
                },
                SettingsNavItem {
                    icon: SettingsNavIcon::Lock,
                    label_key: "tab_paywalls",
                    label_fallback: "Paywalls",
                    is_active: Signal::derive(move || {
                        active_profile_tab.get() == ProfileTab::Paywalls
                    }),
                    on_click: Callback::new(move |_| {
                        set_active_profile_tab.set(ProfileTab::Paywalls)
                    }),
                },
                SettingsNavItem {
                    icon: SettingsNavIcon::Layers,
                    label_key: "tab_plans",
                    label_fallback: "Plans",
                    is_active: Signal::derive(move || {
                        active_profile_tab.get() == ProfileTab::Plans
                    }),
                    on_click: Callback::new(move |_| set_active_profile_tab.set(ProfileTab::Plans)),
                },
            ],
        },
        SettingsNavGroup {
            divider_before: true,
            heading: Some(("nav_group_personalization", "Personalization")),
            visible: true,
            items: vec![SettingsNavItem {
                icon: SettingsNavIcon::Globe,
                label_key: "tab_langs_alerts",
                label_fallback: "Languages & Notifications",
                is_active: Signal::derive(move || {
                    active_profile_tab.get() == ProfileTab::LanguagesAndAlerts
                }),
                on_click: Callback::new(move |_| {
                    set_active_profile_tab.set(ProfileTab::LanguagesAndAlerts)
                }),
            }],
        },
    ];

    view! {
        <ExoSettingsPage resize=resize>
            <div class="exo-gh-header">
                <div class="exo-home-hero">
                    {move || {
                        let Some(user) = current_user.get() else {
                            return view! { <div /> }.into_any();
                        };
                        let ap = active_persona_for_user(&user);
                        let name = display_name_for_user(&user);
                        let init = name.chars().next().unwrap_or('W').to_uppercase().to_string();
                        let av = if !ap.avatar_url.is_empty() { ap.avatar_url } else { ap.avatar_thumb_url };
                        let root_did = user.did_peer.clone().unwrap_or_default();
                        view! {
                            <>
                                {if is_safe_avatar_url(&av) {
                                    view! { <img src=av alt=name.clone() class="exo-avatar" /> }.into_any()
                                } else {
                                    view! { <div class="exo-avatar">{init}</div> }.into_any()
                                }}
                                <div>
                                    <h1 class="exo-h1">{name}</h1>
                                    <div class="exo-sub" style="margin-top: 4px;">
                                        {move || i18n.tr("active_persona_label", "Active persona")} " "
                                        <strong>{ap.handle}</strong>
                                        " · " {move || i18n.tr("profile_sync_plan_label", "Personal sync plan:")} " "
                                        <strong class="tabular-nums">{plan_badge()}</strong>
                                    </div>
                                    <div class="exo-mono" style="font-size: 13px; color: var(--text-muted); margin-top: 4px;">
                                        {root_did}
                                    </div>
                                </div>
                            </>
                        }.into_any()
                    }}
                </div>
                <button type="button" class="exo-btn" on:click=move |_| on_back()>
                    {back_label}
                </button>
            </div>

            {move || profile_notice.get().map(|m| view! {
                <div class="exo-alert-error">{m}</div>
            })}

            <ExoSettingsSplit
                resize=resize
                nav_aria_label=Signal::derive(move || i18n.tr("profile_tabs_aria", "Profile sections"))
                groups=nav_groups
            >


            {move || match active_profile_tab.get() {
                ProfileTab::Overview => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("profile_overview_title", "Public Creator Identity Overview")}</h2>
                            <p class="exo-sub">
                                {move || i18n.tr("profile_overview_desc", "Your sovereign root identity anchors your works, personas, group invitations, paywall payouts, and reader purchases.")}
                            </p>
                            {referred_by.get().map(|inviter| view! {
                                <div class="exo-invite-banner" style="margin-top: 16px;">
                                    <span>{move || i18n.tr("profile_referred_by", "Referred to ExoNovel by:")} " " <strong>{inviter}</strong></span>
                                </div>
                            })}
                            <div class="exo-grid-3" style="margin-top: 18px;">
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("personas_label", "Personas")}</h3>
                                    <p class="tabular-nums">{move || current_user.get().map(|u| u.personas.len()).unwrap_or(0)} " " {move || i18n.tr("profile_personas_unit", "workspace keys")}</p>
                                </div>
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("groups_label", "Groups")}</h3>
                                    <p class="tabular-nums">{move || groups.get().len()} " " {move || i18n.tr("profile_groups_unit", "collaborative groups")}</p>
                                </div>
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("referrals_label", "Referrals")}</h3>
                                    <p class="tabular-nums">{move || referrals.get().len()} " " {move || i18n.tr("profile_referrals_unit", "joined writers")}</p>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any(),

                ProfileTab::Works => view! {
                    <div class="exo-card">
                        <h2 class="exo-h2">{move || i18n.tr("works_title", "Authored & Forked Works")}</h2>
                        <p class="exo-sub">
                            {move || i18n.tr("works_desc", "Every manuscript stores collaborative Loro CRDT edits alongside Willow Meadowcap capability paths anchored to your active persona.")}
                        </p>
                        <div class="exo-format-grid">
                            <div class="exo-format-tile">
                                <h3>{move || i18n.tr("works_novels_title", "Novels & Serials")}</h3>
                                <p>{move || i18n.tr("works_novels_desc", "Chapters and scenes saved with free public or paid reader tiers configured in Paywalls.")}</p>
                            </div>
                            <div class="exo-format-tile">
                                <h3>{move || i18n.tr("works_scripts_title", "Screenplays & Poetry")}</h3>
                                <p>{move || i18n.tr("works_scripts_desc", "Multi-language input composer (29 languages + RTL) with live collaborative cursors.")}</p>
                            </div>
                        </div>
                    </div>
                }.into_any(),

                ProfileTab::Groups => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("groups_title", "Groups & Managers")}</h2>
                            <InputLocaleBar
                                context_label=i18n.tr("groups_composer_label", "Group Composer")
                                pinned_codes=pinned_input.get()
                            />
                            <form on:submit=create_new_group style="margin-top: 12px;">
                                <div class="exo-grid-2">
                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("groups_name_label", "Group Name")}</label>
                                        <input
                                            type="text"
                                            class="exo-input"
                                            dir=move || i18n.input_dir()
                                            required=true
                                            placeholder=move || i18n.tr("groups_name_ph", "e.g. Anthology Editors")
                                            prop:value=move || new_group_name.get()
                                            on:input=move |ev| set_new_group_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="exo-field">
                                        <label class="exo-label">{move || i18n.tr("groups_desc_label", "Description")}</label>
                                        <input
                                            type="text"
                                            class="exo-input"
                                            dir=move || i18n.input_dir()
                                            placeholder=move || i18n.tr("groups_desc_ph", "What this group collaborates on")
                                            prop:value=move || new_group_desc.get()
                                            on:input=move |ev| set_new_group_desc.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                                <button type="submit" class="exo-btn exo-btn-primary">{move || i18n.tr("groups_create_btn", "Create Group")}</button>
                            </form>

                            {move || groups.get().into_iter().map(|g| {
                                let gid = g.group_id.clone();
                                let gid_invite = gid.clone();
                                let anim_id = gid.clone();
                                let is_mgr = g.my_role == "manager";
                                let group_owner = g.owner_did.clone();
                                view! {
                                    <div class=move || format!("exo-group-card {}", live.anim_class("group", &anim_id))>
                                        <div style="display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; flex-wrap: wrap;">
                                            <div>
                                                <div style="display: flex; align-items: center; gap: 10px;">
                                                    <h3 style="margin: 0;">{g.name}</h3>
                                                    <span class=if is_mgr { "exo-tag exo-tag-accent" } else { "exo-tag" }>
                                                        {move || if is_mgr { i18n.tr("status_manager", "Manager") } else { i18n.tr("status_member", "Member") }}
                                                    </span>
                                                </div>
                                                {(!g.description.is_empty()).then(|| view! {
                                                    <p class="exo-sub" style="margin: 6px 0 0;">{g.description}</p>
                                                })}
                                            </div>
                                            <button
                                                type="button"
                                                class="exo-btn"
                                                on:click=move |_| create_and_copy_invite(Some(gid_invite.clone()))
                                            >
                                                {move || i18n.tr("groups_invite_btn", "Invite to Group")}
                                            </button>
                                        </div>

                                        <div style="margin-top: 12px;">
                                            {g.members.into_iter().map(|m| {
                                                let m_did = m.member_owner_did.clone();
                                                let m_did_action = m_did.clone();
                                                let gid_role = gid.clone();
                                                let m_name = m.display_name.clone();
                                                let m_av = m.avatar_thumb_url.clone();
                                                let m_init = m_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                                                let m_is_mgr = m.role == "manager";
                                                let is_creator = m_did == group_owner;
                                                let next_role = if m_is_mgr { "member" } else { "manager" };
                                                view! {
                                                    <div class="exo-member-row">
                                                        <div style="display: flex; align-items: center; gap: 10px;">
                                                            {if is_safe_avatar_url(&m_av) {
                                                                view! { <img src=m_av alt=m_name.clone() class="exo-avatar-sm" /> }.into_any()
                                                            } else {
                                                                view! { <div class="exo-avatar-sm">{m_init}</div> }.into_any()
                                                            }}
                                                            <span>{m_name}</span>
                                                            <span class=if m_is_mgr { "exo-tag exo-tag-accent" } else { "exo-tag" }>
                                                                {move || if m_is_mgr { i18n.tr("status_manager", "Manager") } else { i18n.tr("status_member", "Member") }}
                                                            </span>
                                                        </div>
                                                        {(is_mgr && !is_creator).then(|| view! {
                                                            <button
                                                                type="button"
                                                                class="exo-btn"
                                                                style="min-height: 36px; font-size: 15px;"
                                                                on:click=move |_| set_member_role(gid_role.clone(), m_did_action.clone(), next_role)
                                                            >
                                                                {move || if m_is_mgr { i18n.tr("groups_set_member_btn", "Set as Member") } else { i18n.tr("groups_assign_mgr_btn", "Assign Manager") }}
                                                            </button>
                                                        })}
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>

                        <div class="exo-card">
                            <div class="exo-gh-section-head-row">
                                <div>
                                    <h2 class="exo-h2" style="margin-bottom: 4px;">{move || i18n.tr("invites_title", "Invites & Referrals")}</h2>
                                    <p class="exo-sub" style="margin: 0;">{move || i18n.tr("invites_sub", "Invite writers and track joined referrals.")}</p>
                                </div>
                                <button
                                    type="button"
                                    class="exo-btn exo-btn-primary"
                                    on:click=move |_| create_and_copy_invite(None)
                                >
                                    {move || i18n.tr("invites_create_btn", "Create Invite Link")}
                                </button>
                            </div>
                            <div style="overflow-x: auto; margin-top: 14px;">
                                <table class="exo-table">
                                    <thead>
                                        <tr>
                                            <th>{move || i18n.tr("invites_th_scope", "Invite Scope")}</th>
                                            <th>{move || i18n.tr("invites_th_link", "Invite Link")}</th>
                                            <th>{move || i18n.tr("invites_th_uses", "Uses")}</th>
                                            <th>{move || i18n.tr("invites_th_created", "Created")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || invites.get()
                                            key=|i| format!("{}-{}", i.code, i.use_count)
                                            children=move |inv| {
                                                let url = invite_url_for_code(&inv.code);
                                                let url_copy = url.clone();
                                                let inv_code = inv.code.clone();
                                                let scope = inv.group_name.unwrap_or_else(|| i18n.tr("invites_scope_general", "General ExoNovel Invite"));
                                                view! {
                                                    <tr class=move || live.anim_class("invite", &inv_code)>
                                                        <td>{scope}</td>
                                                        <td>
                                                            <div class="exo-id-cell">
                                                                <button
                                                                    type="button"
                                                                    class="exo-copy-btn"
                                                                    on:click=move |_| {
                                                                        copy_text_to_clipboard(&url_copy);
                                                                        on_toast(i18n.tr_untracked(
                                                                            "profile_toast_invite_copied",
                                                                            "Invite link copied",
                                                                        ));
                                                                    }
                                                                >
                                                                    <CopyIcon />
                                                                </button>
                                                                <span class="exo-mono" style="font-size: 13px;">{url}</span>
                                                            </div>
                                                        </td>
                                                        <td class="tabular-nums">{inv.use_count}</td>
                                                        <td>{inv.created_at}</td>
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

                ProfileTab::Wallet => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("wallet_tab_title", "Connected Wallets & Payment Rails")}</h2>
                            <p class="exo-sub">
                                {move || i18n.tr("wallet_tab_sub", "Choose how you receive reader payments and pay for personal device sync plans across all supported local currency, card, and crypto rails.")}
                            </p>
                            <div class="exo-eyebrow" style="margin-top: 12px;">
                                {move || i18n.tr("wallet_peer_prefix", "ExoNovel Service Recipient Peer:")} " "
                                <span class="exo-mono">{move || exo_sync_peer_did.get()}</span>
                            </div>

                            <div class="exo-grid-2" style="margin-top: 16px;">
                                <div class="exo-field">
                                    <label class="exo-label">{move || i18n.tr("wallet_rail_label", "Default Payment Rail (AOTA)")}</label>
                                    <ExoSelect
                                        value=Signal::derive(move || billing_rail.get())
                                        options=rail_options
                                        aria_label=Signal::derive(move || i18n.tr("wallet_rail_label", "Default Payment Rail (AOTA)"))
                                        on_change=Callback::new(move |val| set_billing_rail.set(val))
                                    />
                                </div>
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("wallet_selected_rail_title", "Active Settlement Rail")}</h3>
                                    <p class="exo-mono" style="margin-top: 6px;">{move || billing_rail.get()}</p>
                                    <p style="margin-top: 8px;">
                                        {move || i18n.tr("wallet_selected_rail_desc", "Used when activating personal sync plans or receiving writer payouts.")}
                                    </p>
                                </div>
                            </div>

                            <div class="exo-grid-3" style="margin-top: 16px;">
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("wallet_rail_card", "Card / Local Currency Gateway")}</h3>
                                    <p>{move || i18n.tr("wallet_card_desc", "Cards, Apple Pay, Google Pay, and local regional currency settlements.")}</p>
                                </div>
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("wallet_rail_solana", "Solana Pay / USDC")}</h3>
                                    <p>{move || i18n.tr("wallet_solana_desc", "Sub-second stablecoin creator payouts and instant reader unlocks.")}</p>
                                </div>
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("wallet_rail_btc", "Bitcoin Lightning (L402)")}</h3>
                                    <p>{move || i18n.tr("wallet_btc_desc", "Low-fee Lightning invoices for global micro-unlocks and subscriptions.")}</p>
                                </div>
                            </div>
                        </div>

                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("ledger_wallet_title", "Wallet Transactions & Receipts")}</h2>
                            <p class="exo-sub">
                                {move || i18n.tr("ledger_wallet_sub", "Complete record of writer earnings, reader unlock receipts, and sync plan charges.")}
                            </p>
                            <div style="overflow-x: auto; margin-top: 14px;">
                                <table class="exo-table">
                                    <thead>
                                        <tr>
                                            <th>{move || i18n.tr("ledger_th_type", "Type")}</th>
                                            <th>{move || i18n.tr("ledger_th_amount", "Amount")}</th>
                                            <th>{move || i18n.tr("ledger_th_cycle", "Cycle")}</th>
                                            <th>{move || i18n.tr("ledger_th_details", "Details")}</th>
                                            <th>{move || i18n.tr("ledger_th_date", "Date")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || billing_entries.get()
                                            key=|b| b.id.clone()
                                            children=move |b| {
                                                let bid = b.id.clone();
                                                let amt = format!("{:.2} {}", b.amount_cents as f64 / 100.0, b.currency);
                                                view! {
                                                    <tr class=move || live.anim_class("billing", &bid)>
                                                        <td><span class="exo-tag">{b.kind}</span></td>
                                                        <td class="tabular-nums"><strong>{amt}</strong></td>
                                                        <td>{b.billing_cycle}</td>
                                                        <td>{b.note}</td>
                                                        <td>{b.created_at}</td>
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

                ProfileTab::Paywalls => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("paywalls_title", "Author Paywalls & Reader Access")}</h2>
                            <p class="exo-sub">
                                {move || i18n.tr("paywalls_sub", "Decide which stories, chapters, or reusable worldbuilding blocks stay free for everyone and which unlock for paying readers—with automatic attribution splits for collaborators.")}
                            </p>

                            <div class="exo-grid-3" style="margin-top: 16px;">
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("paywalls_free_tier_title", "Free Public Chapters")}</h3>
                                    <p>{move || i18n.tr("paywalls_free_tier_desc", "Keep opening chapters or whole books open to all readers so newcomers can discover your writing.")}</p>
                                </div>
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("paywalls_unlock_title", "Per-Story & Chapter Paywalls")}</h3>
                                    <p>{move || i18n.tr("paywalls_unlock_desc", "Set direct reader unlock prices in your preferred currency across cards, mobile money, and crypto wallets.")}</p>
                                </div>
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("paywalls_splits_title", "Collaborator Attribution Splits")}</h3>
                                    <p>{move || i18n.tr("paywalls_splits_desc", "When you co-write in a group, paywall revenue traces cleanly to each credited creator.")}</p>
                                </div>
                            </div>
                        </div>

                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("paywalls_ledger_title", "Reader Unlocks & Paywall Activity")}</h2>
                            <p class="exo-sub">
                                {move || i18n.tr("paywalls_ledger_sub", "Works you unlocked as a reader and paid reader unlocks across your own paywalled stories.")}
                            </p>
                            {move || {
                                let items: Vec<_> = billing_entries
                                    .get()
                                    .into_iter()
                                    .filter(|b| b.kind != "sync_subscription")
                                    .collect();
                                if items.is_empty() {
                                    view! {
                                        <p class="exo-sub" style="margin-top: 14px;">
                                            {move || i18n.tr("paywalls_empty", "No paywall sales or reader unlock receipts recorded yet.")}
                                        </p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="overflow-x: auto; margin-top: 14px;">
                                            <table class="exo-table">
                                                <thead>
                                                    <tr>
                                                        <th>{move || i18n.tr("ledger_th_type", "Type")}</th>
                                                        <th>{move || i18n.tr("ledger_th_amount", "Amount")}</th>
                                                        <th>{move || i18n.tr("ledger_th_details", "Details")}</th>
                                                        <th>{move || i18n.tr("ledger_th_date", "Date")}</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {items.into_iter().map(|b| {
                                                        let amt = format!("{:.2} {}", b.amount_cents as f64 / 100.0, b.currency);
                                                        view! {
                                                            <tr>
                                                                <td><span class="exo-tag">{b.kind}</span></td>
                                                                <td class="tabular-nums"><strong>{amt}</strong></td>
                                                                <td>{b.note}</td>
                                                                <td>{b.created_at}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                }.into_any(),

                ProfileTab::Plans => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("plans_title", "Writing Access & Personal Device Sync Plans")}</h2>
                            <p class="exo-sub">
                                {move || i18n.tr("plans_sub", "Writing on the web is free, writing on your own devices is free, and team collaboration sync between writers is always free—even when one writer is on the web and another works offline on a device. A paid plan is only for keeping your own offline device drafts synced with your own web profile.")}
                            </p>

                            <div class="exo-format-tile" style="margin-top: 16px; border-color: var(--accent);">
                                <h3>{move || i18n.tr("sync_collab_title", "Collaborating With Other Writers Is Free Everywhere")}</h3>
                                <p>
                                    {move || i18n.tr("sync_collab_desc", "Whenever you add collaborators, sync between collaborators happens automatically so you can share your work and collaborate—no matter whether one writer is in a browser and another is on a desktop or phone app. Only syncing your own device content with your own web profile carries a fee.")}
                                </p>
                            </div>

                            <div class="exo-grid-2" style="margin-top: 16px;">
                                <div class="exo-field">
                                    <label class="exo-label">{move || i18n.tr("wallet_currency_label", "Local Currency")}</label>
                                    <ExoSelect
                                        value=Signal::derive(move || billing_currency.get())
                                        options=currency_options
                                        aria_label=Signal::derive(move || i18n.tr("wallet_currency_label", "Local Currency"))
                                        on_change=Callback::new(move |val| set_billing_currency.set(val))
                                    />
                                </div>
                                <div class="exo-format-tile" style="display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
                                    <div>
                                        <div class="exo-label" style="margin-bottom: 2px;">{move || i18n.tr("plans_payment_rail_note", "Selected Payment Rail")}</div>
                                        <span class="exo-mono" style="font-size: 14px;">{move || billing_rail.get()}</span>
                                    </div>
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        style="min-height: 36px; font-size: 14px;"
                                        on:click=move |_| set_active_profile_tab.set(ProfileTab::Wallet)
                                    >
                                        {move || i18n.tr("plans_change_wallet_btn", "Configure in Wallet →")}
                                    </button>
                                </div>
                            </div>

                            <div class="exo-grid-3" style="margin-top: 14px;">
                                <div class="exo-format-tile">
                                    <h3>{move || i18n.tr("plans_free_title", "Web, Device & Collaboration (Free)")}</h3>
                                    <p>{move || i18n.tr("plans_free_desc", "Free writing on the web, free local writing on your devices, and free collaboration sync between you and your writing team.")}</p>
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        style="margin-top: 12px; width: 100%;"
                                        on:click=move |_| activate_sync_plan("free", 0)
                                    >
                                        {move || if plan_badge() == "free" {
                                            i18n.tr("wallet_free_current", "Current Plan (Free)")
                                        } else {
                                            i18n.tr("wallet_free_switch", "Switch to Free")
                                        }}
                                    </button>
                                </div>
                                <div class="exo-format-tile" style="border-color: var(--accent);">
                                    <h3>{move || i18n.tr("wallet_mo_title", "Personal Sync Monthly ($9/mo eq.)")}</h3>
                                    <p>{move || i18n.tr("wallet_mo_desc", "Encrypted sync connecting your own offline device drafts with your own web profile.")}</p>
                                    <button
                                        type="button"
                                        class="exo-btn exo-btn-primary"
                                        style="margin-top: 12px; width: 100%;"
                                        on:click=move |_| activate_sync_plan("sync-monthly", 900)
                                    >
                                        {move || if plan_badge() == "sync-monthly" {
                                            i18n.tr("wallet_mo_active", "Active Monthly")
                                        } else {
                                            i18n.tr("wallet_mo_activate", "Activate $9/month")
                                        }}
                                    </button>
                                </div>
                                <div class="exo-format-tile" style="border-color: var(--accent);">
                                    <h3>{move || i18n.tr("wallet_yr_title", "Personal Sync Annual ($90/yr eq.)")}</h3>
                                    <p>{move || i18n.tr("wallet_yr_desc", "2 months free · full encrypted sync between your own devices and your own web profile.")}</p>
                                    <button
                                        type="button"
                                        class="exo-btn exo-btn-primary"
                                        style="margin-top: 12px; width: 100%;"
                                        on:click=move |_| activate_sync_plan("sync-annual", 9000)
                                    >
                                        {move || if plan_badge() == "sync-annual" {
                                            i18n.tr("wallet_yr_active", "Active Annual")
                                        } else {
                                            i18n.tr("wallet_yr_activate", "Activate $90/year")
                                        }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any(),

                ProfileTab::Personas => view! {
                    <div class="exo-card">
                        <h2 class="exo-h2">{move || i18n.tr("personas_title", "Pen Names & Personas")}</h2>
                        <p class="exo-sub">
                            {move || i18n.tr("personas_sub", "Each persona carries its own cryptographic subspace key and avatar while rolling up to your root identity.")}
                        </p>

                        <div class="exo-persona-list">
                            {move || current_user.get().map(|user| {
                                let active_did = active_persona_for_user(&user).persona_did;
                                user.personas.into_iter().map(|p| {
                                    let p_did = p.persona_did.clone();
                                    let p_did_val = StoredValue::new(p.persona_did.clone());
                                    let p_did_switch = p_did.clone();
                                    let p_did_edit = p_did.clone();
                                    let p_did_copy = p_did.clone();
                                    let p_did_anim = p_did.clone();
                                    let p_name = if p.display_name.is_empty() {
                                        i18n.tr("persona_default_writer", "Writer")
                                    } else {
                                        p.display_name.clone()
                                    };
                                    let p_name_switch = p_name.clone();
                                    let p_name_form = p.display_name.clone();
                                    let p_handle_form = p.handle.clone();
                                    let p_bio_form = p.bio.clone();
                                    let p_init = p_name.chars().next().unwrap_or('W').to_uppercase().to_string();
                                    let av = if !p.avatar_thumb_url.is_empty() { p.avatar_thumb_url.clone() } else { p.avatar_url.clone() };
                                    let is_act = p_did == active_did;
                                    let card_cls = if is_act { "exo-persona-card exo-persona-card-active" } else { "exo-persona-card" };
                                    view! {
                                        <div class=move || format!("{card_cls} {}", live.anim_class("persona", &p_did_anim))>
                                            <div style="display: flex; align-items: center; justify-content: space-between; gap: 10px;">
                                                <div style="display: flex; align-items: center; gap: 10px;">
                                                    {if is_safe_avatar_url(&av) {
                                                        view! { <img src=av alt=p_name.clone() class="exo-avatar-sm" /> }.into_any()
                                                    } else {
                                                        view! { <div class="exo-avatar-sm">{p_init}</div> }.into_any()
                                                    }}
                                                    <div>
                                                        <strong>{p_name}</strong>
                                                        <div class="exo-eyebrow" style="margin: 0;">{p.handle}</div>
                                                    </div>
                                                </div>
                                                {if is_act {
                                                    view! { <span class="exo-tag exo-tag-accent">{move || i18n.tr("active_badge", "Active")}</span> }.into_any()
                                                } else {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="exo-btn"
                                                            style="min-height: 36px; font-size: 15px;"
                                                            on:click=move |_| switch_active_persona(p_did_switch.clone(), p_name_switch.clone())
                                                        >
                                                            {move || i18n.tr("persona_use_btn", "Use")}
                                                        </button>
                                                    }.into_any()
                                                }}
                                            </div>

                                            <div class="exo-id-cell">
                                                <button type="button" class="exo-copy-btn" on:click=move |_| on_copy_did(p_did_copy.clone())>
                                                    <CopyIcon />
                                                </button>
                                                <span class="exo-mono" style="font-size: 12px;">{p_did}</span>
                                            </div>

                                            <button
                                                type="button"
                                                class="exo-btn"
                                                style="min-height: 36px; font-size: 15px;"
                                                on:click=move |_| {
                                                    if editing_persona_did.get_untracked().as_deref() == Some(p_did_edit.as_str()) {
                                                        set_editing_persona_did.set(None);
                                                    } else {
                                                        set_edit_p_name.set(p_name_form.clone());
                                                        set_edit_p_handle.set(p_handle_form.clone());
                                                        set_edit_p_bio.set(p_bio_form.clone());
                                                        set_editing_persona_did.set(Some(p_did_edit.clone()));
                                                    }
                                                }
                                            >
                                                {move || i18n.tr("persona_edit_btn", "Edit Persona & Avatar")}
                                            </button>

                                            <Show when=move || editing_persona_did.get().as_deref() == Some(p_did_val.get_value().as_str())>
                                                <div style="display: flex; flex-direction: column; gap: 10px; padding-top: 10px; border-top: 1px solid var(--border);">
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder=move || i18n.tr("persona_display_ph", "Display name")
                                                        prop:value=move || edit_p_name.get()
                                                        on:input=move |ev| set_edit_p_name.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder=move || i18n.tr("persona_handle_ph", "Handle")
                                                        prop:value=move || edit_p_handle.get()
                                                        on:input=move |ev| set_edit_p_handle.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder=move || i18n.tr("persona_bio_ph", "Bio")
                                                        prop:value=move || edit_p_bio.get()
                                                        on:input=move |ev| set_edit_p_bio.set(event_target_value(&ev))
                                                    />
                                                    <ExoFileButton
                                                        label=Signal::derive(move || i18n.tr("persona_choose_avatar_btn", "Choose Persona Photo"))
                                                        on_data_url=Callback::new(move |url| upload_persona_avatar_data_url(p_did_val.get_value(), url))
                                                        on_error=Callback::new(move |err| set_profile_notice.set(Some(err)))
                                                    />
                                                    <button
                                                        type="button"
                                                        class="exo-btn exo-btn-primary"
                                                        on:click=move |_| save_persona_edits(p_did_val.get_value())
                                                    >
                                                        {move || i18n.tr("persona_save_btn", "Save Persona")}
                                                    </button>
                                                </div>
                                            </Show>
                                        </div>
                                    }
                                }).collect_view()
                            })}
                        </div>

                        <form on:submit=create_new_persona style="margin-top: 22px; padding-top: 18px; border-top: 1px solid var(--border);">
                            <div class="exo-label" style="font-size: 18px;">{move || i18n.tr("persona_create_heading", "Create New Persona")}</div>
                            <InputLocaleBar
                                context_label=i18n.tr("persona_composer_label", "Persona Composer")
                                pinned_codes=pinned_input.get()
                            />
                            <div class="exo-grid-2">
                                <div class="exo-field">
                                    <label class="exo-label">{move || i18n.tr("persona_display_label", "Persona Display Name")}</label>
                                    <input
                                        type="text"
                                        class="exo-input"
                                        dir=move || i18n.input_dir()
                                        required=true
                                        prop:value=move || new_persona_name.get()
                                        on:input=move |ev| set_new_persona_name.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="exo-field">
                                    <label class="exo-label">{move || i18n.tr("persona_handle_label", "Handle")}</label>
                                    <input
                                        type="text"
                                        class="exo-input"
                                        dir=move || i18n.input_dir()
                                        prop:value=move || new_persona_handle.get()
                                        on:input=move |ev| set_new_persona_handle.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <div class="exo-field">
                                <label class="exo-label">{move || i18n.tr("persona_bio_label", "Bio")}</label>
                                <input
                                    type="text"
                                    class="exo-input"
                                    dir=move || i18n.input_dir()
                                    prop:value=move || new_persona_bio.get()
                                    on:input=move |ev| set_new_persona_bio.set(event_target_value(&ev))
                                />
                            </div>
                            <button type="submit" class="exo-btn exo-btn-primary">{move || i18n.tr("persona_create_btn", "Create Persona")}</button>
                        </form>
                    </div>
                }.into_any(),

                ProfileTab::LanguagesAndAlerts => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("langs_quick_title", "Personal Quick-Switch Languages")}</h2>
                            <p class="exo-sub">
                                {move || i18n.tr("langs_quick_sub", "Choose which of the 29 supported languages appear in your compact top-bar UI switcher and your writing composer Input switcher. All 29 remain available in Preferences.")}
                            </p>

                            <div class="exo-grid-2" style="margin-top: 16px;">
                                <div>
                                    <div class="exo-label">{move || i18n.tr("langs_pinned_ui_label", "Pinned Output UI Languages")}</div>
                                    <div class="exo-filter-bar">
                                        {move || i18n.catalog.get().into_iter().map(|loc| {
                                            let code = loc.code.clone();
                                            let code_cmp = code.clone();
                                            let code_clk = code.clone();
                                            let code_flag = code.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || if pinned_ui.get().contains(&code_cmp) {
                                                        "exo-filter-chip exo-filter-chip-active"
                                                    } else {
                                                        "exo-filter-chip"
                                                    }
                                                    on:click=move |_| toggle_pin_code(set_pinned_ui, code_clk.clone())
                                                >
                                                    <LocaleFlag code=code_flag />
                                                    <span>{loc.native_name} " (" {code} ")"</span>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                <div>
                                    <div class="exo-label">{move || i18n.tr("langs_pinned_input_label", "Pinned Composer Input Languages")}</div>
                                    <div class="exo-filter-bar">
                                        {move || i18n.catalog.get().into_iter().map(|loc| {
                                            let code = loc.code.clone();
                                            let code_cmp = code.clone();
                                            let code_clk = code.clone();
                                            let code_flag = code.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || if pinned_input.get().contains(&code_cmp) {
                                                        "exo-filter-chip exo-filter-chip-active"
                                                    } else {
                                                        "exo-filter-chip"
                                                    }
                                                    on:click=move |_| toggle_pin_code(set_pinned_input, code_clk.clone())
                                                >
                                                    <LocaleFlag code=code_flag />
                                                    <span>{loc.native_name} " (" {code} ")"</span>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            </div>

                            <button
                                type="button"
                                class="exo-btn exo-btn-primary"
                                style="margin-top: 18px;"
                                on:click=save_pinned_languages
                            >
                                {move || i18n.tr("langs_save_btn", "Save Quick-Switch Language Lists")}
                            </button>
                        </div>

                        <div class="exo-card">
                            <h2 class="exo-h2">{move || i18n.tr("activity_notif_title", "Activity, Collaboration & Commercial Notifications")}</h2>
                            <p class="exo-sub">
                                {move || i18n.tr("activity_notif_sub", "Choose which persona activity, group invitations, paywall earnings, and reader purchases generate notifications.")}
                            </p>

                            {move || {
                                let p = notif_prefs.get();
                                view! {
                                    <div>
                                        <label class="exo-check-row">
                                            <input
                                                type="checkbox"
                                                prop:checked=p.notify_author_earnings
                                                on:change=move |ev| {
                                                    let v = event_target_checked(&ev);
                                                    set_notif_prefs.update(|np| np.notify_author_earnings = v);
                                                }
                                            />
                                            <span><strong>{move || i18n.tr("notif_earn_title", "Author Paywall Earnings")}</strong> " — " {move || i18n.tr("notif_earn_desc", "Notify when a reader unlocks your paid works.")}</span>
                                        </label>
                                        <label class="exo-check-row">
                                            <input
                                                type="checkbox"
                                                prop:checked=p.notify_reader_purchases
                                                on:change=move |ev| {
                                                    let v = event_target_checked(&ev);
                                                    set_notif_prefs.update(|np| np.notify_reader_purchases = v);
                                                }
                                            />
                                            <span><strong>{move || i18n.tr("notif_purch_title", "Reader Purchases & Receipts")}</strong> " — " {move || i18n.tr("notif_purch_desc", "Receipts when you purchase works or unlock paywalls from other authors.")}</span>
                                        </label>
                                        <label class="exo-check-row">
                                            <input
                                                type="checkbox"
                                                prop:checked=p.notify_group_invites
                                                on:change=move |ev| {
                                                    let v = event_target_checked(&ev);
                                                    set_notif_prefs.update(|np| np.notify_group_invites = v);
                                                }
                                            />
                                            <span><strong>{move || i18n.tr("notif_inv_title", "Group Invitations & Role Changes")}</strong> " — " {move || i18n.tr("notif_inv_desc", "Invites and Manager assignments.")}</span>
                                        </label>
                                        <label class="exo-check-row">
                                            <input
                                                type="checkbox"
                                                prop:checked=p.notify_referral_signups
                                                on:change=move |ev| {
                                                    let v = event_target_checked(&ev);
                                                    set_notif_prefs.update(|np| np.notify_referral_signups = v);
                                                }
                                            />
                                            <span><strong>{move || i18n.tr("notif_ref_title", "Referral Signups")}</strong> " — " {move || i18n.tr("notif_ref_desc", "Alerts when a writer joins via your personal invite link.")}</span>
                                        </label>
                                        <label class="exo-check-row">
                                            <input
                                                type="checkbox"
                                                prop:checked=p.notify_collaborator_comments
                                                on:change=move |ev| {
                                                    let v = event_target_checked(&ev);
                                                    set_notif_prefs.update(|np| np.notify_collaborator_comments = v);
                                                }
                                            />
                                            <span><strong>{move || i18n.tr("notif_collab_title", "Co-Author & Collaborator Notes")}</strong> " — " {move || i18n.tr("notif_collab_desc", "Inline edits and replies on shared works.")}</span>
                                        </label>
                                        <label class="exo-check-row">
                                            <input
                                                type="checkbox"
                                                prop:checked=p.notify_mentions_and_forks
                                                on:change=move |ev| {
                                                    let v = event_target_checked(&ev);
                                                    set_notif_prefs.update(|np| np.notify_mentions_and_forks = v);
                                                }
                                            />
                                            <span><strong>{move || i18n.tr("notif_fork_title", "Mentions & Forks")}</strong> " — " {move || i18n.tr("notif_fork_desc", "Alerts when your building blocks are forked with attribution.")}</span>
                                        </label>
                                    </div>
                                }
                            }}

                            <button
                                type="button"
                                class="exo-btn exo-btn-primary"
                                style="margin-top: 16px;"
                                on:click=save_profile_notif_prefs
                            >
                                {move || i18n.tr("activity_notif_save_btn", "Save Activity Notification Rules")}
                            </button>
                        </div>
                    </div>
                }.into_any(),
            }}
            </ExoSettingsSplit>
        </ExoSettingsPage>
    }
}
