use gloo_net::http::Request;
use leptos::prelude::*;

use crate::{
    i18n::{use_i18n, InputLocaleBar},
    live::use_live_sync,
    models::{
        copy_text_to_clipboard, resolve_fresh_jwt, AdrRecord, AuthConfigResponse, CmsAdrsResponse,
        CmsTasksResponse, CmsToolingResponse, MaturityNoteItem, OwnerExternalTask,
        ProviderModelInfo, ToolCatalogItem, VerifiedUser,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CmsMode {
    ReadAdr,
    CreateOrEditAdr,
    MaturityAndAi,
    OwnerTasks,
}

#[component]
pub fn DocsCmsView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
    current_user: ReadSignal<Option<VerifiedUser>>,
    active_doc_path: ReadSignal<String>,
    set_active_doc_path: WriteSignal<String>,
    on_toast: impl Fn(String) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let live = use_live_sync();
    let i18n = use_i18n();

    let (adrs, set_adrs) = signal::<Vec<AdrRecord>>(Vec::new());
    let (tools, set_tools) = signal::<Vec<ToolCatalogItem>>(Vec::new());
    let (notes, set_notes) = signal::<Vec<MaturityNoteItem>>(Vec::new());
    let (models, set_models) = signal::<Vec<ProviderModelInfo>>(Vec::new());
    let (tasks, set_tasks) = signal::<Vec<OwnerExternalTask>>(Vec::new());

    let (status_filter, set_status_filter) = signal("all".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (cms_mode, set_cms_mode) = signal(CmsMode::ReadAdr);
    let (sidebar_collapsed, set_sidebar_collapsed) = signal(false);
    let (sidebar_width, set_sidebar_width) = signal(320_i32);
    let (dragging, set_dragging) = signal(false);
    let (busy_msg, set_busy_msg) = signal::<Option<String>>(None);

    // MADR 4.0.0 ADR Form State (Admin Create/Edit)
    let (form_id, set_form_id) = signal::<Option<i32>>(None);
    let (form_adr_num, set_form_adr_num) = signal(String::new());
    let (form_doc_kind, set_form_doc_kind) = signal("adr".to_string());
    let (form_title, set_form_title) = signal(String::new());
    let (form_rel_path, set_form_rel_path) = signal("docs/adr/".to_string());
    let (form_status, set_form_status) = signal("proposed".to_string());
    let (form_context, set_form_context) = signal(String::new());
    let (form_drivers, set_form_drivers) = signal(String::new());
    let (form_outcome, set_form_outcome) = signal(String::new());
    let (form_consequences, set_form_consequences) = signal(String::new());
    let (form_confirmation, set_form_confirmation) = signal(String::new());
    let (form_makers, set_form_makers) = signal(String::new());
    let (form_raw_md, set_form_raw_md) = signal(String::new());

    // Link Dependency State
    let (dep_tool_key, set_dep_tool_key) = signal("willow25".to_string());
    let (dep_constraint, set_dep_constraint) = signal(">=0.4.0".to_string());
    let (dep_capability, set_dep_capability) = signal(String::new());
    let (dep_satisfied, set_dep_satisfied) = signal(false);

    // Add Tool Form
    let (new_tool_key, set_new_tool_key) = signal(String::new());
    let (new_tool_name, set_new_tool_name) = signal(String::new());
    let (new_tool_pkg, set_new_tool_pkg) = signal(String::new());
    let (new_tool_ver, set_new_tool_ver) = signal("0.1.0".to_string());
    let (new_tool_purpose, set_new_tool_purpose) = signal(String::new());

    let (selected_scan_tool, set_selected_scan_tool) = signal("loro".to_string());

    let reload_cms = move || {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(resp) = Request::get("/api/cms/adrs")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<CmsAdrsResponse>().await {
                        let cur = active_doc_path.get_untracked();
                        if cur.is_empty() || !data.adrs.iter().any(|a| a.rel_path == cur) {
                            if let Some(first) = data
                                .adrs
                                .iter()
                                .find(|a| a.rel_path != "AGENTS.md")
                                .or_else(|| data.adrs.first())
                            {
                                set_active_doc_path.set(first.rel_path.clone());
                            }
                        }
                        set_adrs.set(data.adrs);
                    }
                }
            }
            if let Ok(resp) = Request::get("/api/cms/tooling")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<CmsToolingResponse>().await {
                        set_tools.set(data.tools);
                        set_notes.set(data.notes);
                        set_models.set(data.provider_models);
                    }
                }
            }
            if let Ok(resp) = Request::get("/api/cms/tasks")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<CmsTasksResponse>().await {
                        set_tasks.set(data.tasks);
                    }
                }
            }
        });
    };

    Effect::new(move |_| reload_cms());

    // Global Live UI reaction for ADR / Tooling / Maturity Notes / Owner Task mutations
    Effect::new(move |_| {
        let _seq = live.pulse_seq.get();
        if live.matches_kind(&["adr", "tooling", "maturity_note", "task", "system"]) {
            reload_cms();
        }
    });

    let toggle_owner_task = move |task_key: String, next_done: bool| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            let url = format!("/api/cms/tasks/{task_key}/toggle");
            if let Ok(req) = Request::post(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({ "isDone": next_done }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        reload_cms();
                        on_toast(if next_done {
                            "Task marked complete in Neon".to_string()
                        } else {
                            "Task reopened".to_string()
                        });
                    } else {
                        on_toast("Failed to update task".to_string());
                    }
                }
            }
        });
    };

    let load_adr_into_form = move |adr: &AdrRecord| {
        set_form_id.set(Some(adr.id));
        set_form_adr_num.set(adr.adr_number.clone().unwrap_or_default());
        set_form_doc_kind.set(adr.doc_kind.clone());
        set_form_title.set(adr.title.clone());
        set_form_rel_path.set(adr.rel_path.clone());
        set_form_status.set(adr.status.clone());
        set_form_context.set(adr.context_problem.clone());
        set_form_drivers.set(adr.decision_drivers.clone());
        set_form_outcome.set(adr.decision_outcome.clone());
        set_form_consequences.set(adr.consequences_pros_cons.clone());
        set_form_confirmation.set(adr.confirmation_criteria.clone());
        set_form_makers.set(adr.decision_makers.clone());
        set_form_raw_md.set(adr.raw_markdown.clone());
        set_cms_mode.set(CmsMode::CreateOrEditAdr);
    };

    let start_blank_adr = move || {
        set_form_id.set(None);
        set_form_adr_num.set(String::new());
        set_form_doc_kind.set("adr".to_string());
        set_form_title.set(String::new());
        set_form_rel_path.set("docs/adr/010-new-decision.md".to_string());
        set_form_status.set("proposed".to_string());
        set_form_context.set(String::new());
        set_form_drivers.set(String::new());
        set_form_outcome.set(String::new());
        set_form_consequences.set(String::new());
        set_form_confirmation.set(String::new());
        set_form_makers.set("ExoNovel Product Owner".to_string());
        set_form_raw_md.set(String::new());
        set_cms_mode.set(CmsMode::CreateOrEditAdr);
    };

    let submit_adr_form = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_busy_msg.set(Some("Saving MADR 4.0.0 record to Neon...".to_string()));
        let payload = serde_json::json!({
            "id": form_id.get_untracked(),
            "adrNumber": if form_adr_num.get_untracked().trim().is_empty() { None } else { Some(form_adr_num.get_untracked()) },
            "docKind": form_doc_kind.get_untracked(),
            "title": form_title.get_untracked(),
            "relPath": form_rel_path.get_untracked(),
            "status": form_status.get_untracked(),
            "contextProblem": form_context.get_untracked(),
            "decisionDrivers": form_drivers.get_untracked(),
            "decisionOutcome": form_outcome.get_untracked(),
            "consequencesProsCons": form_consequences.get_untracked(),
            "confirmationCriteria": form_confirmation.get_untracked(),
            "decisionMakers": form_makers.get_untracked(),
            "rawMarkdown": if form_raw_md.get_untracked().trim().is_empty() { None } else { Some(form_raw_md.get_untracked()) },
        });
        let saved_path = form_rel_path.get_untracked();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                set_busy_msg.set(Some("Missing session token.".to_string()));
                return;
            };
            if let Ok(req) = Request::post("/api/cms/adrs")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&payload)
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_active_doc_path.set(saved_path);
                        set_cms_mode.set(CmsMode::ReadAdr);
                        set_busy_msg.set(None);
                        reload_cms();
                        on_toast("ADR saved to Neon CMS".to_string());
                    } else {
                        let msg = resp.text().await.unwrap_or_else(|_| "Save failed.".to_string());
                        set_busy_msg.set(Some(msg));
                    }
                }
            }
        });
    };

    let link_dep_to_adr = move |adr_id: i32| {
        let tool_key = dep_tool_key.get_untracked();
        let constraint = dep_constraint.get_untracked();
        let capability = dep_capability.get_untracked();
        let satisfied = dep_satisfied.get_untracked();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/cms/dependencies")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "adrId": adr_id,
                    "toolKey": tool_key,
                    "requiredConstraint": constraint,
                    "blockingCapability": capability,
                    "isSatisfied": satisfied,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_dep_capability.set(String::new());
                        reload_cms();
                        on_toast("Tooling dependency linked to ADR".to_string());
                    }
                }
            }
        });
    };

    let run_crates_scan = move |_| {
        set_busy_msg.set(Some("Running deterministic crates.io release scan...".to_string()));
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(resp) = Request::post("/api/cms/scan")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                set_busy_msg.set(None);
                if resp.ok() {
                    reload_cms();
                    on_toast("crates.io release catalog updated".to_string());
                } else {
                    set_busy_msg.set(Some("Scan failed.".to_string()));
                }
            }
        });
    };

    let refresh_models = move |_| {
        set_busy_msg.set(Some("Refreshing live /v1/models across AI Studio, NVIDIA NIM & OpenRouter...".to_string()));
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(resp) = Request::post("/api/cms/models/refresh")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                set_busy_msg.set(None);
                if resp.ok() {
                    reload_cms();
                    on_toast("Live provider models refreshed".to_string());
                }
            }
        });
    };

    let generate_proposal = move |_| {
        let tool_key = selected_scan_tool.get_untracked();
        set_busy_msg.set(Some(format!(
            "Running OpenCode cross-provider proposal for {tool_key}..."
        )));
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/cms/proposals")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({ "toolKey": tool_key, "adrId": serde_json::Value::Null }))
            {
                if let Ok(resp) = req.send().await {
                    set_busy_msg.set(None);
                    if resp.ok() {
                        reload_cms();
                        on_toast("Maturity proposal note generated".to_string());
                    } else {
                        let msg = resp.text().await.unwrap_or_default();
                        set_busy_msg.set(Some(msg));
                    }
                }
            }
        });
    };

    let update_note_status = move |note_id: i32, status: &'static str| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            let url = format!("/api/cms/proposals/{note_id}");
            if let Ok(req) = Request::put(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({ "status": status }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        reload_cms();
                        on_toast(format!("Proposal marked {status}"));
                    }
                }
            }
        });
    };

    let add_tool_catalog = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let key = new_tool_key.get_untracked();
        let name = new_tool_name.get_untracked();
        let pkg = new_tool_pkg.get_untracked();
        let ver = new_tool_ver.get_untracked();
        let purpose = new_tool_purpose.get_untracked();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/cms/tooling")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "toolKey": key,
                    "name": name,
                    "ecosystem": "crates.io",
                    "packageName": pkg,
                    "installedVersion": ver,
                    "purpose": purpose,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_new_tool_key.set(String::new());
                        set_new_tool_name.set(String::new());
                        set_new_tool_pkg.set(String::new());
                        set_new_tool_purpose.set(String::new());
                        reload_cms();
                        on_toast("Tool added to Maturity Catalog".to_string());
                    }
                }
            }
        });
    };

    let filtered_adrs = move || {
        let sf = status_filter.get();
        let q = search_query.get().trim().to_lowercase();
        adrs.get()
            .into_iter()
            .filter(|a| {
                let status_ok = sf == "all" || a.status == sf;
                let query_ok = q.is_empty()
                    || a.title.to_lowercase().contains(&q)
                    || a.rel_path.to_lowercase().contains(&q);
                status_ok && query_ok
            })
            .collect::<Vec<_>>()
    };

    let active_adr = move || {
        let cur = active_doc_path.get();
        adrs.get()
            .into_iter()
            .find(|a| a.rel_path == cur)
            .or_else(|| adrs.get().into_iter().next())
    };

    let is_admin = move || current_user.get().is_some_and(|u| u.is_admin);
    let pinned_input = move || {
        current_user
            .get()
            .and_then(|u| u.profile)
            .map(|p| p.preferred_input_locales)
            .unwrap_or_default()
    };

    view! {
        <div
            class="exo-docs-layout"
            on:pointermove=move |ev: web_sys::PointerEvent| {
                if dragging.get_untracked() {
                    let next = ev.client_x().max(240).min(520);
                    set_sidebar_width.set(next);
                }
            }
            on:pointerup=move |_| set_dragging.set(false)
        >
            <Show when=move || !sidebar_collapsed.get()>
                <aside
                    class="exo-sidebar"
                    style=move || format!("width: {}px;", sidebar_width.get())
                >
                    <div style="display: flex; align-items: center; justify-content: space-between; gap: 8px;">
                        <div>
                            <div class="exo-label" style="margin: 0;">"MADR 4.0.0 CMS"</div>
                            <div class="exo-eyebrow tabular-nums" style="margin: 2px 0 0;">
                                {move || format!("{} records in Neon", adrs.get().len())}
                            </div>
                        </div>
                        <button
                            type="button"
                            class="exo-btn"
                            style="min-height: 34px; padding: 0 10px; font-size: 14px;"
                            on:click=move |_| set_sidebar_collapsed.set(true)
                        >
                            "Hide"
                        </button>
                    </div>

                    <div class="exo-filter-bar">
                        {["all", "on-hold", "ready-to-build", "accepted", "proposed"]
                            .into_iter()
                            .map(|st| {
                                let label = match st {
                                    "all" => "All",
                                    "on-hold" => "On Hold",
                                    "ready-to-build" => "Ready",
                                    "accepted" => "Accepted",
                                    _ => "Proposed",
                                };
                                view! {
                                    <button
                                        type="button"
                                        class=move || if status_filter.get() == st {
                                            "exo-filter-chip exo-filter-chip-active"
                                        } else {
                                            "exo-filter-chip"
                                        }
                                        on:click=move |_| set_status_filter.set(st.to_string())
                                    >
                                        {label}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>

                    <input
                        type="search"
                        class="exo-input"
                        placeholder="Search ADRs..."
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />

                    <nav>
                        <For
                            each=filtered_adrs
                            key=|a| format!("{}-{}-{}", a.id, a.status, a.updated_at)
                            children=move |adr| {
                                let path_cmp = adr.rel_path.clone();
                                let path_click = adr.rel_path.clone();
                                let id_str = adr.id.to_string();
                                let status_str = adr.status.clone();
                                view! {
                                    <button
                                        type="button"
                                        class=move || {
                                            let base = if active_doc_path.get() == path_cmp {
                                                "exo-tree-file-btn exo-tree-file-active"
                                            } else {
                                                "exo-tree-file-btn"
                                            };
                                            format!("{base} {}", live.anim_class("adr", &id_str))
                                        }
                                        on:click=move |_| {
                                            set_active_doc_path.set(path_click.clone());
                                            set_cms_mode.set(CmsMode::ReadAdr);
                                        }
                                    >
                                        <span style="overflow: hidden; text-overflow: ellipsis;">
                                            {adr.title.clone()}
                                        </span>
                                        <span class=if status_str == "on-hold" {
                                            "exo-tag exo-tag-warn"
                                        } else if status_str == "accepted" || status_str == "ready-to-build" {
                                            "exo-tag exo-tag-ok"
                                        } else {
                                            "exo-tag"
                                        }>
                                            {status_str}
                                        </span>
                                    </button>
                                }
                            }
                        />
                    </nav>
                </aside>
                <div
                    class="exo-resizer"
                    on:pointerdown=move |_| set_dragging.set(true)
                />
            </Show>

            <main class="exo-docs-main">
                <div style="flex: 1; min-width: 0;">
                    <div class="exo-docs-toolbar">
                        <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                            <Show when=move || sidebar_collapsed.get()>
                                <button
                                    type="button"
                                    class="exo-btn"
                                    style="min-height: 36px; padding: 0 12px; font-size: 15px;"
                                    on:click=move |_| set_sidebar_collapsed.set(false)
                                >
                                    "Show Explorer"
                                </button>
                            </Show>
                            <button
                                type="button"
                                class=move || if cms_mode.get() == CmsMode::ReadAdr {
                                    "exo-btn exo-btn-primary"
                                } else {
                                    "exo-btn"
                                }
                                style="min-height: 36px; font-size: 15px;"
                                on:click=move |_| set_cms_mode.set(CmsMode::ReadAdr)
                            >
                                "Read ADR"
                            </button>
                            <Show when=is_admin>
                                <button
                                    type="button"
                                    class=move || if cms_mode.get() == CmsMode::CreateOrEditAdr {
                                        "exo-btn exo-btn-primary"
                                    } else {
                                        "exo-btn"
                                    }
                                    style="min-height: 36px; font-size: 15px;"
                                    on:click=move |_| start_blank_adr()
                                >
                                    "+ New MADR 4.0"
                                </button>
                            </Show>
                            <button
                                type="button"
                                class=move || if cms_mode.get() == CmsMode::MaturityAndAi {
                                    "exo-btn exo-btn-primary"
                                } else {
                                    "exo-btn"
                                }
                                style="min-height: 36px; font-size: 15px;"
                                on:click=move |_| set_cms_mode.set(CmsMode::MaturityAndAi)
                            >
                                "Tooling Maturity & OpenCode AI"
                            </button>
                            <button
                                type="button"
                                class=move || if cms_mode.get() == CmsMode::OwnerTasks {
                                    "exo-btn exo-btn-primary"
                                } else {
                                    "exo-btn"
                                }
                                style="min-height: 36px; font-size: 15px;"
                                on:click=move |_| set_cms_mode.set(CmsMode::OwnerTasks)
                            >
                                "Owner & Ops Tasks ("
                                {move || tasks.get().iter().filter(|t| !t.is_done).count()}
                                ")"
                            </button>
                        </div>

                        <button
                            type="button"
                            class="exo-btn"
                            style="min-height: 36px; font-size: 15px;"
                            on:click=move |_| reload_cms()
                        >
                            "Refresh"
                        </button>
                    </div>

                    {move || busy_msg.get().map(|m| view! {
                        <div class="exo-invite-banner">
                            <span>{m}</span>
                        </div>
                    })}

                    {move || match cms_mode.get() {
                        CmsMode::ReadAdr => {
                            let Some(doc) = active_adr() else {
                                return view! { <p class="exo-sub">"Select a record from the left explorer."</p> }.into_any();
                            };
                            let doc_for_edit = StoredValue::new(doc.clone());
                            let doc_id_for_dep = doc.id;
                            let doc_id_str = doc.id.to_string();
                            let dep_list = doc.dependencies.clone();

                            view! {
                                <div class=move || live.anim_class("adr", &doc_id_str)>
                                    <div style="display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; margin-bottom: 14px;">
                                        <div class="exo-eyebrow tabular-nums" style="margin: 0;">
                                            {doc.rel_path.clone()}
                                            " · "
                                            {doc.status.clone()}
                                            " · "
                                            {doc.word_count} " words · "
                                            {doc.reading_minutes} " min read"
                                        </div>
                                        <Show when=is_admin>
                                            <button
                                                type="button"
                                                class="exo-btn"
                                                style="min-height: 34px; font-size: 14px;"
                                                on:click=move |_| doc_for_edit.with_value(|d| load_adr_into_form(d))
                                            >
                                                "Edit in MADR CMS"
                                            </button>
                                        </Show>
                                    </div>

                                    <div class="exo-card" style="margin-bottom: 22px; padding: 18px 20px;">
                                        <div class="exo-label" style="margin-bottom: 4px;">
                                            "Underlying Tooling Dependencies & Blockers"
                                        </div>
                                        {if dep_list.is_empty() {
                                            view! { <p class="exo-sub" style="margin: 4px 0 0;">"No blocking tooling dependencies attached to this ADR."</p> }.into_any()
                                        } else {
                                            dep_list
                                                .into_iter()
                                                .map(|d| {
                                                    let cls = if d.is_satisfied {
                                                        "exo-dep-card exo-dep-satisfied"
                                                    } else {
                                                        "exo-dep-card exo-dep-blocked"
                                                    };
                                                    view! {
                                                        <div class=cls>
                                                            <div style="display: flex; justify-content: space-between; gap: 10px; flex-wrap: wrap;">
                                                                <strong>{d.tool_name} " (" {d.tool_key} ")"</strong>
                                                                <span class="exo-mono tabular-nums">
                                                                    "installed " {d.installed_version} " · latest " {d.latest_version} " · req " {d.required_constraint}
                                                                </span>
                                                            </div>
                                                            <div class="exo-sub" style="margin-top: 4px;">
                                                                {d.blocking_capability}
                                                            </div>
                                                        </div>
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }}

                                        <Show when=is_admin>
                                            <div style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border);">
                                                <select
                                                    class="exo-input"
                                                    style="flex: 1; min-width: 150px;"
                                                    prop:value=move || dep_tool_key.get()
                                                    on:change=move |ev| set_dep_tool_key.set(event_target_value(&ev))
                                                >
                                                    {move || tools.get().into_iter().map(|t| {
                                                        view! { <option value=t.tool_key>{t.name}</option> }
                                                    }).collect_view()}
                                                </select>
                                                <input
                                                    type="text"
                                                    class="exo-input"
                                                    style="flex: 1; min-width: 120px;"
                                                    placeholder=">=0.4.0"
                                                    prop:value=move || dep_constraint.get()
                                                    on:input=move |ev| set_dep_constraint.set(event_target_value(&ev))
                                                />
                                                <input
                                                    type="text"
                                                    class="exo-input"
                                                    style="flex: 2; min-width: 200px;"
                                                    placeholder="Blocking capability..."
                                                    prop:value=move || dep_capability.get()
                                                    on:input=move |ev| set_dep_capability.set(event_target_value(&ev))
                                                />
                                                <label style="display: inline-flex; align-items: center; gap: 6px; font-size: 15px;">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || dep_satisfied.get()
                                                        on:change=move |ev| set_dep_satisfied.set(event_target_checked(&ev))
                                                    />
                                                    "Satisfied"
                                                </label>
                                                <button
                                                    type="button"
                                                    class="exo-btn"
                                                    style="min-height: 40px;"
                                                    on:click=move |_| link_dep_to_adr(doc_id_for_dep)
                                                >
                                                    "Link Dependency"
                                                </button>
                                            </div>
                                        </Show>
                                    </div>

                                    <article class="exo-markdown" inner_html=doc.html.clone() />
                                </div>
                            }.into_any()
                        }
                        CmsMode::CreateOrEditAdr => {
                            view! {
                                <div class="exo-card">
                                    <h2 class="exo-h2">
                                        {move || if form_id.get().is_some() { "Edit MADR 4.0.0 Record" } else { "Create New MADR 4.0.0 ADR" }}
                                    </h2>
                                    <InputLocaleBar
                                        context_label="MADR Editor"
                                        pinned_codes=pinned_input()
                                    />
                                    <form on:submit=submit_adr_form>
                                        <div class="exo-grid-2">
                                            <div class="exo-field">
                                                <label class="exo-label">"Title"</label>
                                                <input
                                                    type="text"
                                                    class="exo-input"
                                                    dir=move || i18n.input_dir()
                                                    required=true
                                                    prop:value=move || form_title.get()
                                                    on:input=move |ev| set_form_title.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">"Relative Path (e.g. docs/adr/010-short.md)"</label>
                                                <input
                                                    type="text"
                                                    class="exo-input exo-mono"
                                                    required=true
                                                    prop:value=move || form_rel_path.get()
                                                    on:input=move |ev| set_form_rel_path.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">"Status"</label>
                                                <select
                                                    class="exo-input"
                                                    prop:value=move || form_status.get()
                                                    on:change=move |ev| set_form_status.set(event_target_value(&ev))
                                                >
                                                    <option value="proposed">"proposed"</option>
                                                    <option value="on-hold">"on-hold (awaiting tooling)"</option>
                                                    <option value="ready-to-build">"ready-to-build"</option>
                                                    <option value="accepted">"accepted"</option>
                                                    <option value="deprecated">"deprecated"</option>
                                                    <option value="superseded">"superseded"</option>
                                                    <option value="rejected">"rejected"</option>
                                                </select>
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">"Decision Makers"</label>
                                                <input
                                                    type="text"
                                                    class="exo-input"
                                                    dir=move || i18n.input_dir()
                                                    prop:value=move || form_makers.get()
                                                    on:input=move |ev| set_form_makers.set(event_target_value(&ev))
                                                />
                                            </div>
                                        </div>

                                        <div class="exo-field">
                                            <label class="exo-label">"Context and Problem Statement"</label>
                                            <textarea
                                                class="exo-input exo-textarea"
                                                dir=move || i18n.input_dir()
                                                prop:value=move || form_context.get()
                                                on:input=move |ev| set_form_context.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">"Decision Drivers"</label>
                                            <textarea
                                                class="exo-input exo-textarea"
                                                dir=move || i18n.input_dir()
                                                prop:value=move || form_drivers.get()
                                                on:input=move |ev| set_form_drivers.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">"Decision Outcome"</label>
                                            <textarea
                                                class="exo-input exo-textarea"
                                                dir=move || i18n.input_dir()
                                                prop:value=move || form_outcome.get()
                                                on:input=move |ev| set_form_outcome.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">"Full Markdown (optional override)"</label>
                                            <textarea
                                                class="exo-input exo-textarea exo-mono"
                                                style="min-height: 180px;"
                                                dir=move || i18n.input_dir()
                                                prop:value=move || form_raw_md.get()
                                                on:input=move |ev| set_form_raw_md.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                            <button type="submit" class="exo-btn exo-btn-primary">
                                                "Save ADR to Neon"
                                            </button>
                                            <button
                                                type="button"
                                                class="exo-btn"
                                                on:click=move |_| set_cms_mode.set(CmsMode::ReadAdr)
                                            >
                                                "Cancel"
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            }.into_any()
                        }
                        CmsMode::MaturityAndAi => {
                            view! {
                                <div>
                                    <div class="exo-card">
                                        <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                                            <div>
                                                <h2 class="exo-h2" style="margin: 0;">"Tooling Catalog & Release Scanner"</h2>
                                                <p class="exo-sub" style="margin: 4px 0 0;">
                                                    "Deterministic crates.io version monitoring paired with OpenCode cross-provider opportunistic notes."
                                                </p>
                                            </div>
                                            <Show when=is_admin>
                                                <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                                                    <button type="button" class="exo-btn exo-btn-primary" on:click=run_crates_scan>
                                                        "Run crates.io Scan"
                                                    </button>
                                                    <button type="button" class="exo-btn" on:click=refresh_models>
                                                        "Refresh Live /v1/models"
                                                    </button>
                                                </div>
                                            </Show>
                                        </div>

                                        <div style="overflow-x: auto; margin-top: 16px;">
                                            <table class="exo-table">
                                                <thead>
                                                    <tr>
                                                        <th>"Tool"</th>
                                                        <th>"Installed"</th>
                                                        <th>"Latest"</th>
                                                        <th>"Purpose"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <For
                                                        each=move || tools.get()
                                                        key=|t| format!("{}-{}-{}", t.tool_key, t.latest_version, t.last_checked_at)
                                                        children=move |t| {
                                                            let t_key = t.tool_key.clone();
                                                            view! {
                                                                <tr class=move || live.anim_class("tooling", &t_key)>
                                                                    <td><strong>{t.name}</strong> " (" {t.package_name} ")"</td>
                                                                    <td class="exo-mono tabular-nums">{t.installed_version}</td>
                                                                    <td class="exo-mono tabular-nums">{t.latest_version}</td>
                                                                    <td>{t.purpose}</td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </div>

                                        <Show when=is_admin>
                                            <form on:submit=add_tool_catalog style="margin-top: 18px; padding-top: 16px; border-top: 1px solid var(--border);">
                                                <div class="exo-label">"Track Another Underlying Crate / Tool"</div>
                                                <div class="exo-grid-3">
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        required=true
                                                        placeholder="Key (e.g. reqwest)"
                                                        prop:value=move || new_tool_key.get()
                                                        on:input=move |ev| set_new_tool_key.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        required=true
                                                        placeholder="Display Name"
                                                        prop:value=move || new_tool_name.get()
                                                        on:input=move |ev| set_new_tool_name.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        required=true
                                                        placeholder="crates.io package"
                                                        prop:value=move || new_tool_pkg.get()
                                                        on:input=move |ev| set_new_tool_pkg.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder="Installed version"
                                                        prop:value=move || new_tool_ver.get()
                                                        on:input=move |ev| set_new_tool_ver.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder="Why ExoNovel depends on it"
                                                        prop:value=move || new_tool_purpose.get()
                                                        on:input=move |ev| set_new_tool_purpose.set(event_target_value(&ev))
                                                    />
                                                    <button type="submit" class="exo-btn exo-btn-primary">
                                                        "Add Tool"
                                                    </button>
                                                </div>
                                            </form>
                                        </Show>
                                    </div>

                                    <div class="exo-card">
                                        <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                                            <div>
                                                <h2 class="exo-h2" style="margin: 0;">"OpenCode Multi-Provider Failover & Maturity Proposals"</h2>
                                                <p class="exo-sub" style="margin: 4px 0 0;">
                                                    "Automatic 404/429 failover across Google AI Studio, NVIDIA NIM, OpenRouter (:free), and Groq (zero Alibaba)."
                                                </p>
                                            </div>
                                            <Show when=is_admin>
                                                <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                                                    <select
                                                        class="exo-input"
                                                        style="width: auto; min-width: 170px;"
                                                        prop:value=move || selected_scan_tool.get()
                                                        on:change=move |ev| set_selected_scan_tool.set(event_target_value(&ev))
                                                    >
                                                        {move || tools.get().into_iter().map(|t| {
                                                            view! { <option value=t.tool_key>{t.name}</option> }
                                                        }).collect_view()}
                                                    </select>
                                                    <button type="button" class="exo-btn exo-btn-primary" on:click=generate_proposal>
                                                        "Generate Proposal Note"
                                                    </button>
                                                </div>
                                            </Show>
                                        </div>

                                        <div class="exo-grid-2">
                                            {move || models.get().into_iter().map(|m| {
                                                let st_tag = if m.deprecated_at.is_some() {
                                                    view! { <span class="exo-tag">"Deprecated"</span> }.into_any()
                                                } else if !m.key_configured {
                                                    view! { <span class="exo-tag">"Key Unset"</span> }.into_any()
                                                } else if m.cooldown_until.is_some() {
                                                    view! { <span class="exo-tag exo-tag-warn">"Cooldown"</span> }.into_any()
                                                } else {
                                                    view! { <span class="exo-tag exo-tag-ok">"Healthy"</span> }.into_any()
                                                };
                                                view! {
                                                    <div class="exo-dep-card">
                                                        <div style="display: flex; justify-content: space-between; gap: 8px; flex-wrap: wrap;">
                                                            <strong>{m.display_name}</strong>
                                                            {st_tag}
                                                        </div>
                                                        <div class="exo-eyebrow exo-mono" style="margin: 4px 0 0;">
                                                            {m.provider} " · " {m.model_id}
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>

                                        <div style="margin-top: 20px;">
                                            <For
                                                each=move || notes.get()
                                                key=|n| format!("{}-{}", n.id, n.status)
                                                children=move |n| {
                                                    let n_id_str = n.id.to_string();
                                                    let n_id = n.id;
                                                    view! {
                                                        <div class=move || format!("exo-proposal-card {}", live.anim_class("maturity_note", &n_id_str))>
                                                            <div style="display: flex; justify-content: space-between; gap: 10px; flex-wrap: wrap;">
                                                                <strong>{n.title}</strong>
                                                                <span class="exo-tag">{n.status.clone()}</span>
                                                            </div>
                                                            <div class="exo-eyebrow" style="margin: 4px 0;">
                                                                {n.tool_key} " " {n.target_version} " · " {n.provider_used} " / " {n.model_used}
                                                            </div>
                                                            <p class="exo-sub" style="color: var(--text); white-space: pre-wrap;">
                                                                {n.proposed_feature_changes}
                                                            </p>
                                                            <Show when=is_admin>
                                                                <div style="display: flex; gap: 8px; margin-top: 10px;">
                                                                    <button
                                                                        type="button"
                                                                        class="exo-btn"
                                                                        style="min-height: 36px; font-size: 15px;"
                                                                        on:click=move |_| update_note_status(n_id, "accepted")
                                                                    >
                                                                        "Accept Proposal"
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        class="exo-btn"
                                                                        style="min-height: 36px; font-size: 15px;"
                                                                        on:click=move |_| update_note_status(n_id, "dismissed")
                                                                    >
                                                                        "Dismiss"
                                                                    </button>
                                                                </div>
                                                            </Show>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }

                        CmsMode::OwnerTasks => {
                            view! {
                                <div class="exo-card">
                                    <h2 class="exo-h2" style="margin: 0;">"External Owner & Ops Checklist"</h2>
                                    <p class="exo-sub" style="margin: 6px 0 18px;">
                                        "Tasks that require external consoles or Tencent server access outside AI Studio. Check a box when done—state persists directly in Neon PostgreSQL so ExoNovel always tracks what remains."
                                    </p>

                                    <For
                                        each=move || tasks.get()
                                        key=|t| format!("{}-{}", t.task_key, t.is_done)
                                        children=move |t| {
                                            let tk_anim = t.task_key.clone();
                                            let tk_toggle = t.task_key.clone();
                                            let has_snippet = !t.copy_snippet.trim().is_empty();
                                            let snippet_sv = StoredValue::new(t.copy_snippet.clone());
                                            let done_now = t.is_done;
                                            let actor_badge = if t.owner_actor == "Tencent Agent" {
                                                "exo-tag exo-tag-warn"
                                            } else {
                                                "exo-tag"
                                            };

                                            view! {
                                                <div
                                                    class=move || format!("exo-proposal-card {}", live.anim_class("task", &tk_anim))
                                                    style=if done_now { "opacity: 0.72;" } else { "" }
                                                >
                                                    <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
                                                        <label class="exo-check-row" style="margin: 0; flex: 1; min-width: 240px;">
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=done_now
                                                                prop:disabled=move || !is_admin()
                                                                on:change=move |ev| {
                                                                    let checked = event_target_checked(&ev);
                                                                    toggle_owner_task(tk_toggle.clone(), checked);
                                                                }
                                                            />
                                                            <span style=if done_now { "text-decoration: line-through; font-weight: 600;" } else { "font-weight: 600;" }>
                                                                {t.title.clone()}
                                                            </span>
                                                        </label>
                                                        <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                                                            <span class=actor_badge>{t.owner_actor.clone()}</span>
                                                            <span class="exo-tag">{t.category.clone()}</span>
                                                            {if done_now {
                                                                view! { <span class="exo-tag exo-tag-ok">"Done"</span> }.into_any()
                                                            } else {
                                                                view! { <span class="exo-tag exo-tag-warn">"Pending"</span> }.into_any()
                                                            }}
                                                        </div>
                                                    </div>

                                                    <p class="exo-sub" style="margin: 8px 0 0; color: var(--text); line-height: 1.55;">
                                                        {t.step_by_step.clone()}
                                                    </p>

                                                    {if has_snippet {
                                                        view! {
                                                            <div style="margin-top: 10px;">
                                                                <pre style="white-space: pre-wrap; margin: 0 0 8px; padding: 10px 12px; border-radius: 8px; background: var(--surface-2, rgba(127,127,127,0.08)); font-size: 13px;">
                                                                    {t.copy_snippet.clone()}
                                                                </pre>
                                                                <button
                                                                    type="button"
                                                                    class="exo-btn"
                                                                    style="min-height: 32px; font-size: 13px; padding: 0 12px;"
                                                                    on:click=move |_| {
                                                                        snippet_sv.with_value(|s| copy_text_to_clipboard(s));
                                                                        on_toast("Copied text to clipboard".to_string());
                                                                    }
                                                                >
                                                                    "Copy Text"
                                                                </button>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span /> }.into_any()
                                                    }}
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                {move || active_adr().filter(|_| cms_mode.get() == CmsMode::ReadAdr).map(|doc| {
                    if doc.headings.is_empty() {
                        view! { <div /> }.into_any()
                    } else {
                        view! {
                            <aside class="exo-toc">
                                <div class="exo-toc-title">"On this page"</div>
                                {doc.headings.into_iter().map(|h| {
                                    let cls = if h.level >= 3 {
                                        "exo-toc-link exo-toc-link-h3"
                                    } else {
                                        "exo-toc-link"
                                    };
                                    view! { <a href=format!("#{}", h.id) class=cls>{h.text}</a> }
                                }).collect_view()}
                            </aside>
                        }.into_any()
                    }
                })}
            </main>
        </div>
    }
}
