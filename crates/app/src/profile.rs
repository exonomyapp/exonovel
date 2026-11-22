use gloo_net::http::Request;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::{
    i18n::{use_i18n, InputLocaleBar},
    live::use_live_sync,
    models::{
        active_persona_for_user, copy_text_to_clipboard, display_name_for_user,
        invite_url_for_code, is_safe_avatar_url, load_authenticated_user, resolve_fresh_jwt,
        save_profile_patch, upload_avatar_file, ActiveTab, AuthConfigResponse, BillingLedgerItem,
        BillingResponse, CopyIcon, CreateInviteResponse, GroupView, GroupsResponse, InviteView,
        InvitesResponse, NotificationPreferences, NotificationPrefsResponse, PersonasResponse,
        ProfileTab, ReferralView, VerifiedUser,
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
) -> impl IntoView {
    let live = use_live_sync();
    let i18n = use_i18n();

    let (active_profile_tab, set_active_profile_tab) = signal(ProfileTab::Overview);
    let (groups, set_groups) = signal::<Vec<GroupView>>(Vec::new());
    let (invites, set_invites) = signal::<Vec<InviteView>>(Vec::new());
    let (referrals, set_referrals) = signal::<Vec<ReferralView>>(Vec::new());
    let (referred_by, set_referred_by) = signal::<Option<String>>(None);
    let (billing_entries, set_billing_entries) = signal::<Vec<BillingLedgerItem>>(Vec::new());
    let (exo_sync_peer_did, set_exo_sync_peer_did) =
        signal("did:peer:0zExoNovelRelaySyncService".to_string());
    let (notif_prefs, set_notif_prefs) = signal(NotificationPreferences::default());
    let (profile_notice, set_profile_notice) = signal::<Option<String>>(None);

    // Mode 3 Managed Willow Sync Subscription form (100% Commercial on Profile per ADR 004 / 008)
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
    let persona_avatar_ref = NodeRef::<leptos::html::Input>::new();

    // Create Group form
    let (new_group_name, set_new_group_name) = signal(String::new());
    let (new_group_desc, set_new_group_desc) = signal(String::new());

    // Pinned languages (managed via Neon user_profiles)
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
                    on_toast(format!("Switched to {label}"));
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
                            on_toast("Persona created".to_string());
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
                        on_toast("Persona updated".to_string());
                    }
                }
            }
        });
    };

    let on_persona_avatar_file = move |target_did: String| {
        if let Some(input) = persona_avatar_ref.get() {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    if file.size() > 2_097_152.0 {
                        set_profile_notice.set(Some("Image must be 2 MB or smaller.".to_string()));
                        return;
                    }
                    let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
                    let fallback_jwt = initial_jwt.get_untracked();
                    if let Ok(reader) = web_sys::FileReader::new() {
                        let reader_clone = reader.clone();
                        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                            if let Ok(res) = reader_clone.result() {
                                if let Some(data_url) = res.as_string() {
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
                                                if let Some((fresh_user, _)) =
                                                    load_authenticated_user(url, None).await
                                                {
                                                    set_current_user.set(Some(fresh_user));
                                                }
                                            }
                                            on_toast("Persona avatar processed".to_string());
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
                        on_toast("Group created".to_string());
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
                        on_toast(format!("Member role set to {next_role}"));
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
                            on_toast("Invite link copied".to_string());
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
                        on_toast(format!("Sync plan updated: {plan_tier}"));
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
                    on_toast("Language quick-switchers saved to Neon".to_string());
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
                        on_toast("Activity notification preferences saved".to_string());
                    }
                }
            }
        });
    };

    let toggle_pin_code = move |list_sig: RwSignal<Vec<String>>, code: String| {
        list_sig.update(|vec| {
            if let Some(pos) = vec.iter().position(|x| *x == code) {
                vec.remove(pos);
            } else {
                vec.push(code);
            }
        });
    };

    let back_label = move || match return_tab.get() {
        ActiveTab::Docs => "← Back to Docs & ADRs",
        _ => "← Back to Workspace",
    };

    let plan_badge = move || {
        current_user
            .get()
            .and_then(|u| u.profile)
            .map(|p| p.sync_plan_tier)
            .unwrap_or_else(|| "free".to_string())
    };

    view! {
        <main class="page-main">
            <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; flex-wrap: wrap; margin-bottom: 18px;">
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
                                        "Active persona "
                                        <strong>{ap.handle}</strong>
                                        " · Sync tier: "
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

            <nav class="exo-gh-tabs" aria-label="Profile sections">
                {[
                    (ProfileTab::Overview, "Overview"),
                    (ProfileTab::Works, "Works"),
                    (ProfileTab::Groups, "Groups & Invites"),
                    (ProfileTab::Wallet, "Wallet, Paywalls & Purchases"),
                    (ProfileTab::Personas, "Personas"),
                    (ProfileTab::LanguagesAndAlerts, "Languages & Notifications"),
                ]
                .into_iter()
                .map(|(tab, label)| {
                    view! {
                        <button
                            type="button"
                            class=move || if active_profile_tab.get() == tab {
                                "exo-gh-tab exo-gh-tab-active"
                            } else {
                                "exo-gh-tab"
                            }
                            on:click=move |_| set_active_profile_tab.set(tab)
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
            </nav>

            {move || match active_profile_tab.get() {
                ProfileTab::Overview => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">"Public Creator Identity Overview"</h2>
                            <p class="exo-sub">
                                "Your sovereign root identity anchors your works, personas, group invitations, paywall payouts, and reader purchases."
                            </p>
                            {referred_by.get().map(|inviter| view! {
                                <div class="exo-invite-banner" style="margin-top: 16px;">
                                    <span>"Referred to ExoNovel by: " <strong>{inviter}</strong></span>
                                </div>
                            })}
                            <div class="exo-grid-3" style="margin-top: 18px;">
                                <div class="exo-format-tile">
                                    <h3>"Personas"</h3>
                                    <p class="tabular-nums">{move || current_user.get().map(|u| u.personas.len()).unwrap_or(0)} " workspace keys"</p>
                                </div>
                                <div class="exo-format-tile">
                                    <h3>"Groups"</h3>
                                    <p class="tabular-nums">{move || groups.get().len()} " collaborative groups"</p>
                                </div>
                                <div class="exo-format-tile">
                                    <h3>"Referrals"</h3>
                                    <p class="tabular-nums">{move || referrals.get().len()} " joined writers"</p>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any(),

                ProfileTab::Works => view! {
                    <div class="exo-card">
                        <h2 class="exo-h2">"Authored & Forked Works"</h2>
                        <p class="exo-sub">
                            "Every manuscript stores collaborative Loro CRDT edits alongside Willow Meadowcap capability paths anchored to your active persona."
                        </p>
                        <div class="exo-format-grid">
                            <div class="exo-format-tile">
                                <h3>"Novels & Serials"</h3>
                                <p>"LoroText chapters + paywalled or free public reader tiers configured in Wallet, Paywalls & Purchases."</p>
                            </div>
                            <div class="exo-format-tile">
                                <h3>"Screenplays & Poetry"</h3>
                                <p>"Multi-language input composer (29 languages + RTL) with live collaborative cursors."</p>
                            </div>
                        </div>
                    </div>
                }.into_any(),

                ProfileTab::Groups => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">"Groups & Managers"</h2>
                            <InputLocaleBar
                                context_label="Group Composer"
                                pinned_codes=pinned_input.get()
                            />
                            <form on:submit=create_new_group style="margin-top: 12px;">
                                <div class="exo-grid-2">
                                    <div class="exo-field">
                                        <label class="exo-label">"Group Name"</label>
                                        <input
                                            type="text"
                                            class="exo-input"
                                            dir=move || i18n.input_dir()
                                            required=true
                                            placeholder="e.g. Anthology Editors"
                                            prop:value=move || new_group_name.get()
                                            on:input=move |ev| set_new_group_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="exo-field">
                                        <label class="exo-label">"Description"</label>
                                        <input
                                            type="text"
                                            class="exo-input"
                                            dir=move || i18n.input_dir()
                                            placeholder="What this group collaborates on"
                                            prop:value=move || new_group_desc.get()
                                            on:input=move |ev| set_new_group_desc.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                                <button type="submit" class="exo-btn exo-btn-primary">"Create Group"</button>
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
                                                        {if is_mgr { "Manager" } else { "Member" }}
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
                                                "Invite to Group"
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
                                                                {if m_is_mgr { "Manager" } else { "Member" }}
                                                            </span>
                                                        </div>
                                                        {(is_mgr && !is_creator).then(|| view! {
                                                            <button
                                                                type="button"
                                                                class="exo-btn"
                                                                style="min-height: 36px; font-size: 15px;"
                                                                on:click=move |_| set_member_role(gid_role.clone(), m_did_action.clone(), next_role)
                                                            >
                                                                {if m_is_mgr { "Set as Member" } else { "Assign Manager" }}
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
                            <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                                <div>
                                    <h2 class="exo-h2" style="margin-bottom: 4px;">"Invites & Referrals"</h2>
                                    <p class="exo-sub" style="margin: 0;">"Invite writers and track joined referrals."</p>
                                </div>
                                <button
                                    type="button"
                                    class="exo-btn exo-btn-primary"
                                    on:click=move |_| create_and_copy_invite(None)
                                >
                                    "Create Invite Link"
                                </button>
                            </div>
                            <div style="overflow-x: auto; margin-top: 14px;">
                                <table class="exo-table">
                                    <thead>
                                        <tr>
                                            <th>"Invite Scope"</th>
                                            <th>"Invite Link"</th>
                                            <th>"Uses"</th>
                                            <th>"Created"</th>
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
                                                let scope = inv.group_name.unwrap_or_else(|| "General ExoNovel Invite".to_string());
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
                                                                        on_toast("Invite link copied".to_string());
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
                            <h2 class="exo-h2">"Mode 3 Managed Willow Sync Subscription"</h2>
                            <p class="exo-sub">
                                "Mode 1 (Web) and Mode 2 (Local Device Willow + Free ExoNovel Relay Discovery) are completely free. Subscribe here to register your did:peer IDs for blind encrypted Mode 3 server-to-device Meadowcap sync ($9/month or $90/year local currency equivalent)."
                            </p>
                            <div class="exo-eyebrow" style="margin-top: 12px;">
                                "ExoNovel Service Recipient Peer: "
                                <span class="exo-mono">{move || exo_sync_peer_did.get()}</span>
                            </div>

                            <div class="exo-grid-2" style="margin-top: 16px;">
                                <div class="exo-field">
                                    <label class="exo-label">"Local Currency"</label>
                                    <select
                                        class="exo-input"
                                        prop:value=move || billing_currency.get()
                                        on:change=move |ev| set_billing_currency.set(event_target_value(&ev))
                                    >
                                        <option value="USD">"USD ($9/mo · $90/yr)"</option>
                                        <option value="EUR">"EUR (€9/mo · €90/yr)"</option>
                                        <option value="GBP">"GBP (£7/mo · £70/yr)"</option>
                                        <option value="PLN">"PLN (36 zł/mo · 360 zł/yr)"</option>
                                        <option value="NGN">"NGN (₦13,500/mo · ₦135,000/yr)"</option>
                                        <option value="KES">"KES (KSh 1,200/mo · KSh 12,000/yr)"</option>
                                        <option value="INR">"INR (₹750/mo · ₹7,500/yr)"</option>
                                        <option value="BRL">"BRL (R$45/mo · R$450/yr)"</option>
                                    </select>
                                </div>
                                <div class="exo-field">
                                    <label class="exo-label">"Payment Rail (AOTA)"</label>
                                    <select
                                        class="exo-input"
                                        prop:value=move || billing_rail.get()
                                        on:change=move |ev| set_billing_rail.set(event_target_value(&ev))
                                    >
                                        <option value="card-stripe">"Card / Local Currency Gateway"</option>
                                        <option value="solana-usdc">"Solana Pay / USDC"</option>
                                        <option value="bitcoin-lightning">"Bitcoin Lightning (L402)"</option>
                                        <option value="ethereum-base">"Base / Arbitrum / Ethereum"</option>
                                        <option value="mobile-money">"Mobile Money / PIX / UPI"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="exo-grid-3" style="margin-top: 14px;">
                                <div class="exo-format-tile">
                                    <h3>"Free Modes (1 & 2)"</h3>
                                    <p>"Web Neon App or Device Local Willow Store + Free Discovery Relay"</p>
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        style="margin-top: 12px; width: 100%;"
                                        on:click=move |_| activate_sync_plan("free", 0)
                                    >
                                        {move || if plan_badge() == "free" { "Current Plan (Free)" } else { "Switch to Free" }}
                                    </button>
                                </div>
                                <div class="exo-format-tile" style="border-color: var(--accent);">
                                    <h3>"Sync Monthly ($9/mo eq.)"</h3>
                                    <p>"Authorizes Mode 3 Settings toggle & blind Meadowcap replication."</p>
                                    <button
                                        type="button"
                                        class="exo-btn exo-btn-primary"
                                        style="margin-top: 12px; width: 100%;"
                                        on:click=move |_| activate_sync_plan("sync-monthly", 900)
                                    >
                                        {move || if plan_badge() == "sync-monthly" { "Active Monthly" } else { "Activate $9/month" }}
                                    </button>
                                </div>
                                <div class="exo-format-tile" style="border-color: var(--accent);">
                                    <h3>"Sync Annual ($90/yr eq.)"</h3>
                                    <p>"2 months free · full blind Willow server-to-device sync."</p>
                                    <button
                                        type="button"
                                        class="exo-btn exo-btn-primary"
                                        style="margin-top: 12px; width: 100%;"
                                        on:click=move |_| activate_sync_plan("sync-annual", 9000)
                                    >
                                        {move || if plan_badge() == "sync-annual" { "Active Annual" } else { "Activate $90/year" }}
                                    </button>
                                </div>
                            </div>
                        </div>

                        <div class="exo-card">
                            <h2 class="exo-h2">"Author Paywalls, Reader Purchases & Sync Ledger"</h2>
                            <p class="exo-sub">
                                "All commercial records—connected wallets, paywall earnings, reader purchases of works you buy, and Sync receipts—live exclusively here in Profile."
                            </p>
                            <div style="overflow-x: auto; margin-top: 14px;">
                                <table class="exo-table">
                                    <thead>
                                        <tr>
                                            <th>"Type"</th>
                                            <th>"Amount"</th>
                                            <th>"Cycle"</th>
                                            <th>"Details"</th>
                                            <th>"Date"</th>
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

                ProfileTab::Personas => view! {
                    <div class="exo-card">
                        <h2 class="exo-h2">"Pen Names & Personas"</h2>
                        <p class="exo-sub">
                            "Each persona carries its own cryptographic subspace key and avatar while rolling up to your root identity."
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
                                    let p_name = if p.display_name.is_empty() { "Writer".to_string() } else { p.display_name.clone() };
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
                                                    view! { <span class="exo-tag exo-tag-accent">"Active"</span> }.into_any()
                                                } else {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="exo-btn"
                                                            style="min-height: 36px; font-size: 15px;"
                                                            on:click=move |_| switch_active_persona(p_did_switch.clone(), p_name_switch.clone())
                                                        >
                                                            "Use"
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
                                                "Edit Persona & Avatar"
                                            </button>

                                            <Show when=move || editing_persona_did.get().as_deref() == Some(p_did_val.get_value().as_str())>
                                                <div style="display: flex; flex-direction: column; gap: 10px; padding-top: 10px; border-top: 1px solid var(--border);">
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder="Display name"
                                                        prop:value=move || edit_p_name.get()
                                                        on:input=move |ev| set_edit_p_name.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder="Handle"
                                                        prop:value=move || edit_p_handle.get()
                                                        on:input=move |ev| set_edit_p_handle.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder="Bio"
                                                        prop:value=move || edit_p_bio.get()
                                                        on:input=move |ev| set_edit_p_bio.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="file"
                                                        accept="image/png,image/jpeg,image/webp,image/gif"
                                                        node_ref=persona_avatar_ref
                                                        on:change=move |_| on_persona_avatar_file(p_did_val.get_value())
                                                    />
                                                    <button
                                                        type="button"
                                                        class="exo-btn exo-btn-primary"
                                                        on:click=move |_| save_persona_edits(p_did_val.get_value())
                                                    >
                                                        "Save Persona"
                                                    </button>
                                                </div>
                                            </Show>
                                        </div>
                                    }
                                }).collect_view()
                            })}
                        </div>

                        <form on:submit=create_new_persona style="margin-top: 22px; padding-top: 18px; border-top: 1px solid var(--border);">
                            <div class="exo-label" style="font-size: 18px;">"Create New Persona"</div>
                            <InputLocaleBar
                                context_label="Persona Composer"
                                pinned_codes=pinned_input.get()
                            />
                            <div class="exo-grid-2">
                                <div class="exo-field">
                                    <label class="exo-label">"Persona Display Name"</label>
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
                                    <label class="exo-label">"Handle"</label>
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
                                <label class="exo-label">"Bio"</label>
                                <input
                                    type="text"
                                    class="exo-input"
                                    dir=move || i18n.input_dir()
                                    prop:value=move || new_persona_bio.get()
                                    on:input=move |ev| set_new_persona_bio.set(event_target_value(&ev))
                                />
                            </div>
                            <button type="submit" class="exo-btn exo-btn-primary">"Create Persona"</button>
                        </form>
                    </div>
                }.into_any(),

                ProfileTab::LanguagesAndAlerts => view! {
                    <div>
                        <div class="exo-card">
                            <h2 class="exo-h2">"Personal Quick-Switch Languages"</h2>
                            <p class="exo-sub">
                                "Choose which of the 29 supported languages appear in your compact top-bar UI switcher and your writing composer Input switcher. All 29 remain available in Settings."
                            </p>

                            <div class="exo-grid-2" style="margin-top: 16px;">
                                <div>
                                    <div class="exo-label">"Pinned Output UI Languages"</div>
                                    <div class="exo-filter-bar">
                                        {move || i18n.catalog.get().into_iter().map(|loc| {
                                            let code = loc.code.clone();
                                            let code_cmp = code.clone();
                                            let code_clk = code.clone();
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
                                                    {loc.native_name} " (" {code} ")"
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                <div>
                                    <div class="exo-label">"Pinned Composer Input Languages"</div>
                                    <div class="exo-filter-bar">
                                        {move || i18n.catalog.get().into_iter().map(|loc| {
                                            let code = loc.code.clone();
                                            let code_cmp = code.clone();
                                            let code_clk = code.clone();
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
                                                    {loc.native_name} " (" {code} ")"
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
                                "Save Quick-Switch Language Lists"
                            </button>
                        </div>

                        <div class="exo-card">
                            <h2 class="exo-h2">"Activity, Collaboration & Commercial Notifications"</h2>
                            <p class="exo-sub">
                                "Choose which persona activity, group invitations, paywall earnings, and reader purchases generate notifications."
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
                                            <span><strong>"Author Paywall Earnings"</strong> " — Notify when a reader unlocks your paid works."</span>
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
                                            <span><strong>"Reader Purchases & Receipts"</strong> " — Receipts when you purchase works or unlock paywalls from other authors."</span>
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
                                            <span><strong>"Group Invitations & Role Changes"</strong> " — Invites and Manager assignments."</span>
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
                                            <span><strong>"Referral Signups"</strong> " — Alerts when a writer joins via your personal invite link."</span>
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
                                            <span><strong>"Co-Author & Collaborator Notes"</strong> " — Inline edits and replies on shared works."</span>
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
                                            <span><strong>"Mentions & Forks"</strong> " — Alerts when your building blocks are forked with attribution."</span>
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
                                "Save Activity Notification Rules"
                            </button>
                        </div>
                    </div>
                }.into_any(),
            }}
        </main>
    }
}
