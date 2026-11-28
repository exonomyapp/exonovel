use std::collections::{BTreeMap, BTreeSet};

use gloo_net::http::Request;
use leptos::prelude::*;

use crate::{
    i18n::{use_i18n, InputLocaleBar},
    live::use_live_sync,
    models::{
        copy_text_to_clipboard, extract_api_error_message, resolve_fresh_jwt,
        start_google_connect_flow, AdrRecord, AuthConfigResponse, CmsAdrsResponse,
        CmsTasksResponse, CmsToolingResponse, ConnectedAppsResponse, DriveBackupResponse,
        DriveSaveResponse, MaturityNoteItem, OwnerExternalTask, ProviderModelInfo, ToolCatalogItem,
        VerifiedUser,
    },
    ui::{select_opt, ExoSelect},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CmsMode {
    ReadAdr,
    CreateOrEditAdr,
    ToolingAndProposals,
    InferenceRuntimes,
    OwnerTasks,
}

#[derive(Clone, Debug)]
enum TreeRow {
    Folder {
        folder_path: String,
        name: String,
        depth: usize,
        expanded: bool,
        file_count: usize,
    },
    File {
        adr: AdrRecord,
        depth: usize,
    },
}

#[derive(Default)]
struct FolderDir {
    files: Vec<AdrRecord>,
    subdirs: BTreeMap<String, FolderDir>,
}

impl FolderDir {
    fn insert(&mut self, parts: &[&str], adr: AdrRecord) {
        if parts.len() <= 1 {
            self.files.push(adr);
        } else {
            self.subdirs
                .entry(parts[0].to_string())
                .or_default()
                .insert(&parts[1..], adr);
        }
    }

    fn total_files(&self) -> usize {
        self.files.len()
            + self
                .subdirs
                .values()
                .map(FolderDir::total_files)
                .sum::<usize>()
    }

    fn flatten(
        &mut self,
        prefix: &str,
        depth: usize,
        expanded_folders: &BTreeSet<String>,
        auto_expand: bool,
        out: &mut Vec<TreeRow>,
    ) {
        for (name, sub) in &mut self.subdirs {
            let folder_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let expanded = auto_expand || expanded_folders.contains(&folder_path);
            let file_count = sub.total_files();
            out.push(TreeRow::Folder {
                folder_path: folder_path.clone(),
                name: name.clone(),
                depth,
                expanded,
                file_count,
            });
            if expanded {
                sub.flatten(&folder_path, depth + 1, expanded_folders, auto_expand, out);
            }
        }
        self.files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        for adr in self.files.drain(..) {
            out.push(TreeRow::File { adr, depth });
        }
    }
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
    let (proposal_filter, set_proposal_filter) = signal("all".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (expanded_folders, set_expanded_folders) = signal(BTreeSet::from([
        "docs".to_string(),
        "docs/adr".to_string(),
    ]));
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

    // Google Docs & Drive Integration State
    let (google_connected, set_google_connected) = signal(false);
    let (google_email, set_google_email) = signal(String::new());
    let (drive_link, set_drive_link) = signal::<Option<(String, String)>>(None);
    let (backup_panel_open, set_backup_panel_open) = signal(false);
    let (backup_format, set_backup_format) = signal("google-doc".to_string());
    let (drive_busy, set_drive_busy) = signal(false);

    let reload_google_status = move || {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(resp) = Request::get("/api/connected-apps")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(parsed) = resp.json::<ConnectedAppsResponse>().await {
                        if let Some(app) = parsed
                            .connected_apps
                            .into_iter()
                            .find(|a| a.provider == "google_drive" && a.status == "connected")
                        {
                            set_google_connected.set(true);
                            set_google_email.set(app.provider_email);
                        } else {
                            set_google_connected.set(false);
                            set_google_email.set(String::new());
                        }
                    }
                }
            }
        });
    };

    let reload_cms_for_locale = move |loc: String| {
        reload_google_status();
        let encoded_loc = js_sys::encode_uri_component(loc.trim())
            .as_string()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "en".to_string());
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            let adrs_url = format!("/api/cms/adrs?locale={encoded_loc}");
            if let Ok(resp) = Request::get(&adrs_url)
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

    Effect::new(move |_| {
        let loc = i18n.ui_locale.get();
        reload_cms_for_locale(loc);
    });

    // Global Live UI reaction for ADR / Tooling / Maturity Notes / Owner Task / Connected Apps mutations
    Effect::new(move |_| {
        let _seq = live.pulse_seq.get();
        if live.matches_kind(&["adr", "tooling", "maturity_note", "task", "connected_app", "system"])
        {
            reload_cms_for_locale(i18n.ui_locale.get_untracked());
        }
    });

    let connect_google_from_docs = move |_| {
        let cur_doc = active_doc_path.get_untracked();
        set_busy_msg.set(Some(i18n.tr_untracked(
            "cms_msg_redirecting_google",
            "Redirecting to Google consent screen...",
        )));
        leptos::task::spawn_local(async move {
            let doc_opt = if cur_doc.trim().is_empty() {
                None
            } else {
                Some(cur_doc.as_str())
            };
            if let Err(err) =
                start_google_connect_flow(auth_config, initial_jwt, "docs", doc_opt).await
            {
                set_busy_msg.set(Some(err));
            }
        });
    };

    let save_active_doc_to_drive = move |rel_path: String, format: &'static str| {
        set_drive_busy.set(true);
        set_busy_msg.set(Some(if format == "google-doc" {
            i18n.tr_untracked(
                "cms_msg_saving_gdoc",
                "Saving active ADR to Google Docs...",
            )
        } else {
            i18n.tr_untracked(
                "cms_msg_saving_md",
                "Backing up active ADR as .md to Google Drive...",
            )
        }));
        let fail_fallback = i18n.tr_untracked("cms_msg_save_failed", "Save failed.");
        let saved_toast = i18n.tr_untracked("cms_toast_saved_drive", "Saved to Google Docs / Drive");
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                set_drive_busy.set(false);
                return;
            };
            if let Ok(req) = Request::post("/api/drive/save-doc")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "relPath": rel_path,
                    "folderName": "ExoNovel Docs",
                    "format": format,
                }))
            {
                match req.send().await {
                    Ok(resp) if resp.ok() => {
                        if let Ok(saved) = resp.json::<DriveSaveResponse>().await {
                            let link = saved.file.web_view_link.clone().unwrap_or_default();
                            if link.starts_with("https://") {
                                set_drive_link.set(Some((saved.file.name.clone(), link)));
                            }
                            set_busy_msg.set(Some(format!(
                                "{}: {} ({})",
                                saved_toast, saved.file.name, saved.action
                            )));
                            on_toast(saved_toast);
                        }
                    }
                    Ok(resp) => {
                        let txt = resp.text().await.unwrap_or_default();
                        set_busy_msg.set(Some(extract_api_error_message(&txt, &fail_fallback)));
                    }
                    Err(e) => set_busy_msg.set(Some(e.to_string())),
                }
            }
            set_drive_busy.set(false);
        });
    };

    let run_archive_backup = move |_| {
        let fmt = backup_format.get_untracked();
        set_backup_panel_open.set(false);
        set_drive_busy.set(true);
        set_busy_msg.set(Some(i18n.tr_untracked(
            "cms_msg_backing_up_archive",
            "Backing up full ExoNovel Docs & ADR archive to Google Drive (this may take a moment)...",
        )));
        let fail_fallback = i18n.tr_untracked("cms_msg_archive_failed", "Archive backup failed.");
        let ok_toast = i18n.tr_untracked(
            "cms_toast_archive_backed_up",
            "Archive backed up to Google Drive",
        );
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                set_drive_busy.set(false);
                return;
            };
            if let Ok(req) = Request::post("/api/drive/backup-archive")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "rootFolderName": "ExoNovel Docs",
                    "format": fmt,
                }))
            {
                match req.send().await {
                    Ok(resp) if resp.ok() => {
                        if let Ok(res) = resp.json::<DriveBackupResponse>().await {
                            let link = res.archive_folder.web_view_link.clone().unwrap_or_default();
                            if link.starts_with("https://") {
                                set_drive_link
                                    .set(Some((res.archive_folder.name.clone(), link)));
                            }
                            set_busy_msg.set(Some(format!(
                                "{}: {} ({}/{} · +{}/~{})",
                                ok_toast,
                                res.archive_folder.name,
                                res.total,
                                res.total,
                                res.created_count,
                                res.updated_count
                            )));
                            on_toast(ok_toast);
                        }
                    }
                    Ok(resp) => {
                        let txt = resp.text().await.unwrap_or_default();
                        set_busy_msg
                            .set(Some(extract_api_error_message(&txt, &fail_fallback)));
                    }
                    Err(e) => set_busy_msg.set(Some(e.to_string())),
                }
            }
            set_drive_busy.set(false);
        });
    };

    let toggle_owner_task = move |task_key: String, next_done: bool| {
        let done_toast = i18n.tr_untracked("cms_toast_task_done", "Task marked complete");
        let reopened_toast = i18n.tr_untracked("cms_toast_task_reopened", "Task reopened");
        let failed_toast = i18n.tr_untracked("cms_toast_task_failed", "Failed to update task");
        let loc = i18n.ui_locale.get_untracked();
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
                        reload_cms_for_locale(loc);
                        on_toast(if next_done { done_toast } else { reopened_toast });
                    } else {
                        on_toast(failed_toast);
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
        let active_locale = i18n.ui_locale.get_untracked();
        set_busy_msg.set(Some(i18n.tr_untracked(
            "cms_msg_saving_adr",
            "Saving MADR 4.0.0 record...",
        )));
        let missing_tok = i18n.tr_untracked("cms_msg_missing_token", "Missing session token.");
        let saved_toast = i18n.tr_untracked("cms_toast_adr_saved", "ADR saved to CMS");
        let save_failed = i18n.tr_untracked("cms_msg_save_failed", "Save failed.");
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
            "locale": active_locale,
        });
        let saved_path = form_rel_path.get_untracked();
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                set_busy_msg.set(Some(missing_tok));
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
                        reload_cms_for_locale(active_locale);
                        on_toast(saved_toast);
                    } else {
                        let msg = resp.text().await.unwrap_or(save_failed);
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
        let linked_toast = i18n.tr_untracked(
            "cms_toast_dep_linked",
            "Tooling dependency linked to ADR",
        );
        let loc = i18n.ui_locale.get_untracked();
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
                        reload_cms_for_locale(loc);
                        on_toast(linked_toast);
                    }
                }
            }
        });
    };

    let run_crates_scan = move |_| {
        set_busy_msg.set(Some(i18n.tr_untracked(
            "cms_msg_scanning_crates",
            "Running deterministic crates.io release scan...",
        )));
        let scan_done = i18n.tr_untracked(
            "cms_toast_scan_done",
            "crates.io release catalog updated",
        );
        let scan_failed = i18n.tr_untracked("cms_msg_scan_failed", "Scan failed.");
        let loc = i18n.ui_locale.get_untracked();
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
                    reload_cms_for_locale(loc);
                    on_toast(scan_done);
                } else {
                    set_busy_msg.set(Some(scan_failed));
                }
            }
        });
    };

    let refresh_models = move |_| {
        set_busy_msg.set(Some(i18n.tr_untracked(
            "cms_msg_refreshing_models",
            "Refreshing live /v1/models across configured inference runtimes...",
        )));
        let ok_toast = i18n.tr_untracked(
            "cms_toast_models_refreshed",
            "Live provider models refreshed",
        );
        let loc = i18n.ui_locale.get_untracked();
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
                    reload_cms_for_locale(loc);
                    on_toast(ok_toast);
                }
            }
        });
    };

    let generate_proposal = move |_| {
        let tool_key = selected_scan_tool.get_untracked();
        let gen_prefix = i18n.tr_untracked(
            "cms_msg_generating_proposal",
            "Running active inference proposal analysis...",
        );
        set_busy_msg.set(Some(format!("{gen_prefix} ({tool_key})")));
        let gen_toast = i18n.tr_untracked(
            "cms_toast_proposal_generated",
            "Maturity proposal note generated",
        );
        let loc = i18n.ui_locale.get_untracked();
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
                        reload_cms_for_locale(loc);
                        on_toast(gen_toast);
                    } else {
                        let msg = resp.text().await.unwrap_or_default();
                        set_busy_msg.set(Some(msg));
                    }
                }
            }
        });
    };

    let update_note_status = move |note_id: i32, status: &'static str| {
        let upd_toast = i18n.tr_untracked("cms_toast_proposal_updated", "Proposal status updated");
        let loc = i18n.ui_locale.get_untracked();
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
                        reload_cms_for_locale(loc);
                        on_toast(upd_toast);
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
        let added_toast = i18n.tr_untracked(
            "cms_toast_tool_added",
            "Tool added to Maturity Catalog",
        );
        let loc = i18n.ui_locale.get_untracked();
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
                        reload_cms_for_locale(loc);
                        on_toast(added_toast);
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

    let visible_tree_rows = move || {
        let list = filtered_adrs();
        let expanded = expanded_folders.get();
        let auto_expand = !search_query.get().trim().is_empty() || status_filter.get() != "all";
        let mut root = FolderDir::default();
        for adr in list {
            let rel_path_owned = adr.rel_path.clone();
            let parts: Vec<&str> = rel_path_owned
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();
            root.insert(&parts, adr);
        }
        let mut out = Vec::new();
        root.flatten("", 0, &expanded, auto_expand, &mut out);
        out
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
                            <div class="exo-label" style="margin: 0;">
                                {move || i18n.tr("cms_sidebar_title", "MADR 4.0.0 CMS")}
                            </div>
                            <div class="exo-eyebrow tabular-nums" style="margin: 2px 0 0;">
                                {move || format!(
                                    "{} {}",
                                    adrs.get().len(),
                                    i18n.tr("cms_records_count_suffix", "records")
                                )}
                            </div>
                        </div>
                        <button
                            type="button"
                            class="exo-btn"
                            style="min-height: 34px; padding: 0 10px; font-size: 14px;"
                            on:click=move |_| set_sidebar_collapsed.set(true)
                        >
                            {move || i18n.tr("cms_hide", "Hide")}
                        </button>
                    </div>

                    <div class="exo-filter-bar">
                        {["all", "on-hold", "ready-to-build", "accepted", "proposed"]
                            .into_iter()
                            .map(|st| {
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
                                        {move || match st {
                                            "all" => i18n.tr("cms_all", "All"),
                                            "on-hold" => i18n.tr("cms_on_hold", "On Hold"),
                                            "ready-to-build" => i18n.tr("cms_ready", "Ready"),
                                            "accepted" => i18n.tr("cms_accepted", "Accepted"),
                                            _ => i18n.tr("cms_proposed", "Proposed"),
                                        }}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>

                    <input
                        type="search"
                        class="exo-input"
                        placeholder=move || i18n.tr("cms_search_placeholder", "Search ADRs...")
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />

                    <nav>
                        <For
                            each=visible_tree_rows
                            key=|row| match row {
                                TreeRow::Folder {
                                    folder_path,
                                    expanded,
                                    file_count,
                                    ..
                                } => format!("dir:{folder_path}:{expanded}:{file_count}"),
                                TreeRow::File { adr, .. } => format!(
                                    "file:{}-{}-{}-{}",
                                    adr.id, adr.status, adr.translation_status, adr.updated_at
                                ),
                            }
                            children=move |row| match row {
                                TreeRow::Folder {
                                    folder_path,
                                    name,
                                    depth,
                                    expanded,
                                    file_count,
                                } => {
                                    let fp_click = folder_path.clone();
                                    let indent_style = format!("padding-inline-start: {}px;", 10 + depth * 14);
                                    let caret = if expanded { "▾" } else { "▸" };
                                    view! {
                                        <button
                                            type="button"
                                            class="exo-tree-folder-btn"
                                            style=indent_style
                                            on:click=move |_| {
                                                let target = fp_click.clone();
                                                set_expanded_folders.update(|set| {
                                                    if set.contains(&target) {
                                                        set.remove(&target);
                                                    } else {
                                                        set.insert(target);
                                                    }
                                                });
                                            }
                                        >
                                            <span class="exo-tree-caret">{caret}</span>
                                            <span class="exo-tree-folder-name">{name}</span>
                                            <span class="exo-tree-count tabular-nums">{file_count}</span>
                                        </button>
                                    }
                                    .into_any()
                                }
                                TreeRow::File { adr, depth } => {
                                    let path_cmp = adr.rel_path.clone();
                                    let path_click = adr.rel_path.clone();
                                    let id_str = adr.id.to_string();
                                    let status_str = adr.status.clone();
                                    let status_cls = if status_str == "on-hold" {
                                        "exo-tag exo-tag-warn"
                                    } else if status_str == "accepted" || status_str == "ready-to-build" {
                                        "exo-tag exo-tag-ok"
                                    } else {
                                        "exo-tag"
                                    };
                                    let indent_style = format!("padding-inline-start: {}px;", 12 + depth * 14);
                                    view! {
                                        <button
                                            type="button"
                                            style=indent_style
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
                                            <span class=status_cls>
                                                {move || match status_str.as_str() {
                                                    "on-hold" => i18n.tr("cms_on_hold", "On Hold"),
                                                    "ready-to-build" => i18n.tr("cms_ready", "Ready"),
                                                    "accepted" => i18n.tr("cms_accepted", "Accepted"),
                                                    "proposed" => i18n.tr("cms_proposed", "Proposed"),
                                                    other => other.to_string(),
                                                }}
                                            </span>
                                        </button>
                                    }
                                    .into_any()
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
                                    {move || i18n.tr("cms_show_explorer", "Show Explorer")}
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
                                {move || i18n.tr("cms_read_adr", "Read ADR")}
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
                                    {move || i18n.tr("cms_new_madr", "+ New MADR 4.0")}
                                </button>
                            </Show>
                            <button
                                type="button"
                                class=move || if cms_mode.get() == CmsMode::ToolingAndProposals {
                                    "exo-btn exo-btn-primary"
                                } else {
                                    "exo-btn"
                                }
                                style="min-height: 36px; font-size: 15px;"
                                on:click=move |_| set_cms_mode.set(CmsMode::ToolingAndProposals)
                            >
                                {move || i18n.tr("cms_tooling_proposals", "Tooling & Proposals")}
                            </button>
                            <button
                                type="button"
                                class=move || if cms_mode.get() == CmsMode::InferenceRuntimes {
                                    "exo-btn exo-btn-primary"
                                } else {
                                    "exo-btn"
                                }
                                style="min-height: 36px; font-size: 15px;"
                                on:click=move |_| set_cms_mode.set(CmsMode::InferenceRuntimes)
                            >
                                {move || i18n.tr("cms_inference_runtimes", "Inference Runtimes")}
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
                                {move || i18n.tr("cms_owner_tasks", "Owner & Ops Tasks")}
                                " ("
                                {move || tasks.get().iter().filter(|t| !t.is_done).count()}
                                ")"
                            </button>
                        </div>

                        <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                            {move || if google_connected.get() {
                                view! {
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        style="min-height: 36px; font-size: 14px;"
                                        disabled=move || drive_busy.get()
                                        on:click=move |_| set_backup_panel_open.update(|v| *v = !*v)
                                    >
                                        {move || i18n.tr("cms_backup_drive", "Backup Archive to Drive")}
                                    </button>
                                }.into_any()
                            } else {
                                view! {
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        style="min-height: 36px; font-size: 14px;"
                                        on:click=connect_google_from_docs
                                    >
                                        {move || i18n.tr("cms_connect_google_docs", "Connect Google Docs")}
                                    </button>
                                }.into_any()
                            }}
                            <button
                                type="button"
                                class="exo-btn"
                                style="min-height: 36px; font-size: 15px;"
                                on:click=move |_| reload_cms_for_locale(i18n.ui_locale.get_untracked())
                            >
                                {move || i18n.tr("refresh_label", "Refresh")}
                            </button>
                        </div>
                    </div>

                    <Show when=move || backup_panel_open.get()>
                        <div class="exo-invite-banner" style="justify-content: space-between; flex-wrap: wrap;">
                            <div>
                                <strong>{move || i18n.tr("cms_backup_title", "Back up all ExoNovel Docs to Google Drive")}</strong>
                                <div class="exo-sub" style="margin: 2px 0 0;">
                                    {move || {
                                        let acct = google_email.get();
                                        if acct.is_empty() {
                                            i18n.tr("cms_backup_dest_default", "Destination: ExoNovel Docs/")
                                        } else {
                                            format!(
                                                "{} {}",
                                                i18n.tr("cms_backup_dest_prefix", "Destination: ExoNovel Docs/ on"),
                                                acct
                                            )
                                        }
                                    }}
                                </div>
                            </div>
                            <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                <ExoSelect
                                    value=Signal::derive(move || backup_format.get())
                                    options=Signal::derive(move || vec![
                                        select_opt("google-doc", i18n.tr("cms_backup_fmt_gdoc", "Google Docs (Editable)")),
                                        select_opt("markdown", i18n.tr("cms_backup_fmt_md", "Markdown (.md)")),
                                    ])
                                    style_extra="min-width: 190px;".to_string()
                                    aria_label=Signal::derive(move || i18n.tr("drive_target_fmt_label", "Target Format"))
                                    on_change=Callback::new(move |val| set_backup_format.set(val))
                                />
                                <button
                                    type="button"
                                    class="exo-btn exo-btn-primary"
                                    disabled=move || drive_busy.get()
                                    on:click=run_archive_backup
                                >
                                    {move || i18n.tr("cms_backup_start", "Start Backup")}
                                </button>
                                <button
                                    type="button"
                                    class="exo-btn"
                                    on:click=move |_| set_backup_panel_open.set(false)
                                >
                                    {move || i18n.tr("cancel_label", "Cancel")}
                                </button>
                            </div>
                        </div>
                    </Show>

                    {move || busy_msg.get().map(|m| view! {
                        <div class="exo-invite-banner">
                            <span>{m}</span>
                        </div>
                    })}

                    {move || drive_link.get().map(|(label, url)| view! {
                        <div class="exo-invite-banner" style="justify-content: space-between; flex-wrap: wrap;">
                            <span>
                                <strong>{label}</strong>
                                " "
                                {move || i18n.tr("cms_drive_synced_suffix", "is synced in Google Drive.")}
                            </span>
                            <div style="display: flex; gap: 8px; align-items: center;">
                                <a href=url target="_blank" rel="noopener noreferrer" class="exo-btn exo-btn-primary">
                                    {move || i18n.tr("cms_open_drive_btn", "Open in Google Docs / Drive ↗")}
                                </a>
                                <button
                                    type="button"
                                    class="exo-btn"
                                    on:click=move |_| set_drive_link.set(None)
                                >
                                    {move || i18n.tr("dismiss_label", "Dismiss")}
                                </button>
                            </div>
                        </div>
                    })}

                    {move || match cms_mode.get() {
                        CmsMode::ReadAdr => {
                            let Some(doc) = active_adr() else {
                                return view! {
                                    <p class="exo-sub">
                                        {move || i18n.tr("cms_select_adr_prompt", "Select a record from the left explorer.")}
                                    </p>
                                }.into_any();
                            };
                            let doc_for_edit = StoredValue::new(doc.clone());
                            let doc_path_gdoc = StoredValue::new(doc.rel_path.clone());
                            let doc_path_md = StoredValue::new(doc.rel_path.clone());
                            let doc_id_for_dep = doc.id;
                            let doc_id_str = doc.id.to_string();
                            let dep_list = doc.dependencies.clone();
                            let trans_status = doc.translation_status.clone();
                            let trans_badge_cls = match trans_status.as_str() {
                                "canonical" => "exo-trans-badge exo-trans-canonical",
                                "translated" => "exo-trans-badge exo-trans-translated",
                                _ => "exo-trans-badge exo-trans-fallback",
                            };

                            view! {
                                <div class=move || live.anim_class("adr", &doc_id_str)>
                                    <div style="display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; margin-bottom: 14px;">
                                        <div class="exo-eyebrow tabular-nums" style="margin: 0; display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                            <span>
                                                {doc.rel_path.clone()}
                                                " · "
                                                {doc.status.clone()}
                                                " · "
                                                {doc.word_count} " " {move || i18n.tr("cms_words_suffix", "words")} " · "
                                                {doc.reading_minutes} " " {move || i18n.tr("cms_min_read_suffix", "min read")}
                                            </span>
                                            <span class=trans_badge_cls>
                                                {move || match trans_status.as_str() {
                                                    "canonical" => i18n.tr("cms_trans_canonical", "Canonical (EN)"),
                                                    "translated" => i18n.tr("cms_trans_translated", "Translated"),
                                                    _ => i18n.tr("cms_trans_fallback_en", "English Fallback"),
                                                }}
                                            </span>
                                        </div>
                                        <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                                            {move || if google_connected.get() {
                                                view! {
                                                    <>
                                                        <button
                                                            type="button"
                                                            class="exo-btn exo-btn-primary"
                                                            style="min-height: 34px; font-size: 14px;"
                                                            disabled=move || drive_busy.get()
                                                            on:click=move |_| save_active_doc_to_drive(doc_path_gdoc.get_value(), "google-doc")
                                                        >
                                                            {move || i18n.tr("cms_save_gdoc_btn", "Save to Google Docs")}
                                                        </button>
                                                        <button
                                                            type="button"
                                                            class="exo-btn"
                                                            style="min-height: 34px; font-size: 14px;"
                                                            disabled=move || drive_busy.get()
                                                            on:click=move |_| save_active_doc_to_drive(doc_path_md.get_value(), "markdown")
                                                        >
                                                            {move || i18n.tr("cms_save_md_btn", "Save .md to Drive")}
                                                        </button>
                                                    </>
                                                }.into_any()
                                            } else {
                                                view! { <span /> }.into_any()
                                            }}
                                            <Show when=is_admin>
                                                <button
                                                    type="button"
                                                    class="exo-btn"
                                                    style="min-height: 34px; font-size: 14px;"
                                                    on:click=move |_| doc_for_edit.with_value(|d| load_adr_into_form(d))
                                                >
                                                    {move || i18n.tr("cms_edit_adr_btn", "Edit in MADR CMS")}
                                                </button>
                                            </Show>
                                        </div>
                                    </div>

                                    <div class="exo-card" style="margin-bottom: 22px; padding: 18px 20px;">
                                        <div class="exo-label" style="margin-bottom: 4px;">
                                            {move || i18n.tr("cms_deps_title", "Underlying Tooling Dependencies & Blockers")}
                                        </div>
                                        {if dep_list.is_empty() {
                                            view! {
                                                <p class="exo-sub" style="margin: 4px 0 0;">
                                                    {move || i18n.tr("cms_deps_empty", "No blocking tooling dependencies attached to this ADR.")}
                                                </p>
                                            }.into_any()
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
                                                                    {move || i18n.tr("cms_deps_installed", "installed")} " " {d.installed_version}
                                                                    " · "
                                                                    {move || i18n.tr("cms_deps_latest", "latest")} " " {d.latest_version}
                                                                    " · "
                                                                    {move || i18n.tr("cms_deps_req", "req")} " " {d.required_constraint}
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
                                                <ExoSelect
                                                    value=Signal::derive(move || dep_tool_key.get())
                                                    options=Signal::derive(move || {
                                                        tools.get().into_iter().map(|t| select_opt(t.tool_key, t.name)).collect()
                                                    })
                                                    style_extra="flex: 1; min-width: 150px;".to_string()
                                                    aria_label=Signal::derive(move || i18n.tr("cms_th_tool", "Tool"))
                                                    on_change=Callback::new(move |val| set_dep_tool_key.set(val))
                                                />
                                                <input
                                                    type="text"
                                                    class="exo-input"
                                                    style="flex: 1; min-width: 120px;"
                                                    placeholder=move || i18n.tr("cms_deps_constraint_ph", ">=0.4.0")
                                                    prop:value=move || dep_constraint.get()
                                                    on:input=move |ev| set_dep_constraint.set(event_target_value(&ev))
                                                />
                                                <input
                                                    type="text"
                                                    class="exo-input"
                                                    style="flex: 2; min-width: 200px;"
                                                    placeholder=move || i18n.tr("cms_deps_cap_ph", "Blocking capability...")
                                                    prop:value=move || dep_capability.get()
                                                    on:input=move |ev| set_dep_capability.set(event_target_value(&ev))
                                                />
                                                <label style="display: inline-flex; align-items: center; gap: 6px; font-size: 15px;">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || dep_satisfied.get()
                                                        on:change=move |ev| set_dep_satisfied.set(event_target_checked(&ev))
                                                    />
                                                    {move || i18n.tr("cms_deps_satisfied", "Satisfied")}
                                                </label>
                                                <button
                                                    type="button"
                                                    class="exo-btn"
                                                    style="min-height: 40px;"
                                                    on:click=move |_| link_dep_to_adr(doc_id_for_dep)
                                                >
                                                    {move || i18n.tr("cms_deps_link_btn", "Link Dependency")}
                                                </button>
                                            </div>
                                        </Show>
                                    </div>

                                    <article class="exo-markdown" inner_html=doc.html.clone() />
                                </div>
                            }.into_any()
                        }
                        CmsMode::CreateOrEditAdr => {
                            let ctx_lbl = i18n.tr_untracked("cms_editor_context_label", "MADR Editor");
                            view! {
                                <div class="exo-card">
                                    <h2 class="exo-h2">
                                        {move || if form_id.get().is_some() {
                                            i18n.tr("cms_editor_title_edit", "Edit MADR 4.0.0 Record")
                                        } else {
                                            i18n.tr("cms_editor_title_new", "Create New MADR 4.0.0 ADR")
                                        }}
                                    </h2>
                                    <InputLocaleBar
                                        context_label=ctx_lbl
                                        pinned_codes=pinned_input()
                                    />
                                    <form on:submit=submit_adr_form>
                                        <div class="exo-grid-2">
                                            <div class="exo-field">
                                                <label class="exo-label">
                                                    {move || i18n.tr("cms_field_title", "Title")}
                                                </label>
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
                                                <label class="exo-label">
                                                    {move || i18n.tr("cms_field_rel_path", "Relative Path (e.g. docs/adr/010-short.md)")}
                                                </label>
                                                <input
                                                    type="text"
                                                    class="exo-input exo-mono"
                                                    required=true
                                                    prop:value=move || form_rel_path.get()
                                                    on:input=move |ev| set_form_rel_path.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">
                                                    {move || i18n.tr("cms_field_status", "Status")}
                                                </label>
                                                <ExoSelect
                                                    value=Signal::derive(move || form_status.get())
                                                    options=Signal::derive(move || vec![
                                                        select_opt("proposed", i18n.tr("cms_status_opt_proposed", "proposed")),
                                                        select_opt("on-hold", i18n.tr("cms_status_opt_on_hold", "on-hold (awaiting tooling)")),
                                                        select_opt("ready-to-build", i18n.tr("cms_status_opt_ready", "ready-to-build")),
                                                        select_opt("accepted", i18n.tr("cms_status_opt_accepted", "accepted")),
                                                        select_opt("deprecated", i18n.tr("cms_status_opt_deprecated", "deprecated")),
                                                        select_opt("superseded", i18n.tr("cms_status_opt_superseded", "superseded")),
                                                        select_opt("rejected", i18n.tr("cms_status_opt_rejected", "rejected")),
                                                    ])
                                                    aria_label=Signal::derive(move || i18n.tr("cms_field_status", "Status"))
                                                    on_change=Callback::new(move |val| set_form_status.set(val))
                                                />
                                            </div>
                                            <div class="exo-field">
                                                <label class="exo-label">
                                                    {move || i18n.tr("cms_field_makers", "Decision Makers")}
                                                </label>
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
                                            <label class="exo-label">
                                                {move || i18n.tr("cms_field_context", "Context and Problem Statement")}
                                            </label>
                                            <textarea
                                                class="exo-input exo-textarea"
                                                dir=move || i18n.input_dir()
                                                prop:value=move || form_context.get()
                                                on:input=move |ev| set_form_context.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">
                                                {move || i18n.tr("cms_field_drivers", "Decision Drivers")}
                                            </label>
                                            <textarea
                                                class="exo-input exo-textarea"
                                                dir=move || i18n.input_dir()
                                                prop:value=move || form_drivers.get()
                                                on:input=move |ev| set_form_drivers.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">
                                                {move || i18n.tr("cms_field_outcome", "Decision Outcome")}
                                            </label>
                                            <textarea
                                                class="exo-input exo-textarea"
                                                dir=move || i18n.input_dir()
                                                prop:value=move || form_outcome.get()
                                                on:input=move |ev| set_form_outcome.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="exo-field">
                                            <label class="exo-label">
                                                {move || i18n.tr("cms_field_raw_md", "Full Markdown (optional override)")}
                                            </label>
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
                                                {move || i18n.tr("cms_save_adr_btn", "Save ADR")}
                                            </button>
                                            <button
                                                type="button"
                                                class="exo-btn"
                                                on:click=move |_| set_cms_mode.set(CmsMode::ReadAdr)
                                            >
                                                {move || i18n.tr("cancel_label", "Cancel")}
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            }.into_any()
                        }
                        CmsMode::ToolingAndProposals => {
                            let filtered_notes = move || {
                                let pf = proposal_filter.get();
                                notes
                                    .get()
                                    .into_iter()
                                    .filter(|n| pf == "all" || n.status == pf)
                                    .collect::<Vec<_>>()
                            };
                            view! {
                                <div>
                                    <div class="exo-card">
                                        <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                                            <div>
                                                <h2 class="exo-h2" style="margin: 0;">
                                                    {move || i18n.tr("cms_tooling_catalog_title", "Tooling Catalog & Release Scanner")}
                                                </h2>
                                                <p class="exo-sub" style="margin: 4px 0 0;">
                                                    {move || i18n.tr(
                                                        "cms_tooling_catalog_sub",
                                                        "Deterministic crates.io version monitoring linked to architectural decision records."
                                                    )}
                                                </p>
                                            </div>
                                            <Show when=is_admin>
                                                <button type="button" class="exo-btn exo-btn-primary" on:click=run_crates_scan>
                                                    {move || i18n.tr("cms_run_scan", "Run crates.io Scan")}
                                                </button>
                                            </Show>
                                        </div>

                                        <div style="overflow-x: auto; margin-top: 16px;">
                                            <table class="exo-table">
                                                <thead>
                                                    <tr>
                                                        <th>{move || i18n.tr("cms_th_tool", "Tool")}</th>
                                                        <th>{move || i18n.tr("cms_th_installed", "Installed")}</th>
                                                        <th>{move || i18n.tr("cms_th_latest", "Latest")}</th>
                                                        <th>{move || i18n.tr("cms_th_purpose", "Purpose")}</th>
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
                                                <div class="exo-label">
                                                    {move || i18n.tr("cms_add_tool_heading", "Track Another Underlying Crate / Tool")}
                                                </div>
                                                <div class="exo-grid-3">
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        required=true
                                                        placeholder=move || i18n.tr("cms_add_tool_key_ph", "Key (e.g. reqwest)")
                                                        prop:value=move || new_tool_key.get()
                                                        on:input=move |ev| set_new_tool_key.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        required=true
                                                        placeholder=move || i18n.tr("cms_add_tool_name_ph", "Display Name")
                                                        prop:value=move || new_tool_name.get()
                                                        on:input=move |ev| set_new_tool_name.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        required=true
                                                        placeholder=move || i18n.tr("cms_add_tool_pkg_ph", "crates.io package")
                                                        prop:value=move || new_tool_pkg.get()
                                                        on:input=move |ev| set_new_tool_pkg.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder=move || i18n.tr("cms_add_tool_ver_ph", "Installed version")
                                                        prop:value=move || new_tool_ver.get()
                                                        on:input=move |ev| set_new_tool_ver.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="exo-input"
                                                        placeholder=move || i18n.tr("cms_add_tool_purpose_ph", "Why ExoNovel depends on it")
                                                        prop:value=move || new_tool_purpose.get()
                                                        on:input=move |ev| set_new_tool_purpose.set(event_target_value(&ev))
                                                    />
                                                    <button type="submit" class="exo-btn exo-btn-primary">
                                                        {move || i18n.tr("cms_add_tool_btn", "Add Tool")}
                                                    </button>
                                                </div>
                                            </form>
                                        </Show>
                                    </div>

                                    <div class="exo-card">
                                        <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                                            <div>
                                                <h2 class="exo-h2" style="margin: 0;">
                                                    {move || i18n.tr("cms_maturity_proposals_title", "Maturity Proposals")}
                                                </h2>
                                                <p class="exo-sub" style="margin: 4px 0 0;">
                                                    {move || i18n.tr(
                                                        "cms_maturity_proposals_sub",
                                                        "Generate upgrade recommendations for tracked dependencies and accept or dismiss proposals."
                                                    )}
                                                </p>
                                            </div>
                                            <Show when=is_admin>
                                                <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                                                    <ExoSelect
                                                        value=Signal::derive(move || selected_scan_tool.get())
                                                        options=Signal::derive(move || {
                                                            tools.get().into_iter().map(|t| select_opt(t.tool_key, t.name)).collect()
                                                        })
                                                        style_extra="width: auto; min-width: 170px;".to_string()
                                                        aria_label=Signal::derive(move || i18n.tr("cms_th_tool", "Tool"))
                                                        on_change=Callback::new(move |val| set_selected_scan_tool.set(val))
                                                    />
                                                    <button type="button" class="exo-btn exo-btn-primary" on:click=generate_proposal>
                                                        {move || i18n.tr("cms_generate_proposal", "Generate Proposal Note")}
                                                    </button>
                                                </div>
                                            </Show>
                                        </div>

                                        <div class="exo-filter-bar" style="margin-top: 14px;">
                                            {["all", "open", "accepted", "dismissed"]
                                                .into_iter()
                                                .map(|pf| {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class=move || if proposal_filter.get() == pf {
                                                                "exo-filter-chip exo-filter-chip-active"
                                                            } else {
                                                                "exo-filter-chip"
                                                            }
                                                            on:click=move |_| set_proposal_filter.set(pf.to_string())
                                                        >
                                                            {move || match pf {
                                                                "all" => i18n.tr("cms_proposal_filter_all", "All"),
                                                                "open" => i18n.tr("cms_proposal_filter_open", "Open"),
                                                                "accepted" => i18n.tr("cms_proposal_filter_accepted", "Accepted"),
                                                                _ => i18n.tr("cms_proposal_filter_dismissed", "Dismissed"),
                                                            }}
                                                        </button>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>

                                        <div style="margin-top: 16px;">
                                            {move || if filtered_notes().is_empty() {
                                                view! {
                                                    <p class="exo-sub">
                                                        {move || i18n.tr(
                                                            "cms_proposals_empty",
                                                            "No proposal notes in this filter yet. Select a tool above and click Generate Proposal Note."
                                                        )}
                                                    </p>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <For
                                                        each=filtered_notes
                                                        key=|n| format!("{}-{}", n.id, n.status)
                                                        children=move |n| {
                                                            let n_id_str = n.id.to_string();
                                                            let n_id = n.id;
                                                            let cur_status = n.status.clone();
                                                            let cur_status_lbl = cur_status.clone();
                                                            let has_rationale = !n.rationale.trim().is_empty();
                                                            let rationale_text = n.rationale.clone();
                                                            view! {
                                                                <div class=move || format!("exo-proposal-card {}", live.anim_class("maturity_note", &n_id_str))>
                                                                    <div style="display: flex; justify-content: space-between; gap: 10px; flex-wrap: wrap; align-items: baseline;">
                                                                        <strong>{n.title}</strong>
                                                                        <span class="exo-eyebrow" style="margin: 0;">
                                                                            {move || i18n.tr("cms_proposal_status_prefix", "Status:")}
                                                                            " "
                                                                            {move || match cur_status_lbl.as_str() {
                                                                                "accepted" => i18n.tr("cms_proposal_filter_accepted", "Accepted"),
                                                                                "dismissed" => i18n.tr("cms_proposal_filter_dismissed", "Dismissed"),
                                                                                _ => i18n.tr("cms_proposal_filter_open", "Open"),
                                                                            }}
                                                                        </span>
                                                                    </div>
                                                                    <div class="exo-eyebrow" style="margin: 4px 0;">
                                                                        {n.tool_key} " " {n.target_version} " · " {n.provider_used} " / " {n.model_used}
                                                                    </div>
                                                                    <p class="exo-sub" style="color: var(--text); white-space: pre-wrap;">
                                                                        {n.proposed_feature_changes}
                                                                    </p>
                                                                    {has_rationale.then(|| view! {
                                                                        <div class="exo-sub" style="margin-top: 8px; padding-top: 8px; border-top: 1px solid var(--border); white-space: pre-wrap;">
                                                                            <strong>{move || i18n.tr("cms_proposal_rationale", "Diagnostics / Rationale:")}</strong>
                                                                            " "
                                                                            {rationale_text}
                                                                        </div>
                                                                    })}
                                                                    <Show when=is_admin>
                                                                        <div style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 10px;">
                                                                            {(cur_status != "accepted").then(|| view! {
                                                                                <button
                                                                                    type="button"
                                                                                    class="exo-btn"
                                                                                    style="min-height: 36px; font-size: 15px;"
                                                                                    on:click=move |_| update_note_status(n_id, "accepted")
                                                                                >
                                                                                    {move || i18n.tr("cms_proposal_accept", "Accept Proposal")}
                                                                                </button>
                                                                            })}
                                                                            {(cur_status != "dismissed").then(|| view! {
                                                                                <button
                                                                                    type="button"
                                                                                    class="exo-btn"
                                                                                    style="min-height: 36px; font-size: 15px;"
                                                                                    on:click=move |_| update_note_status(n_id, "dismissed")
                                                                                >
                                                                                    {move || i18n.tr("cms_proposal_dismiss", "Dismiss")}
                                                                                </button>
                                                                            })}
                                                                            {(cur_status != "open").then(|| view! {
                                                                                <button
                                                                                    type="button"
                                                                                    class="exo-btn"
                                                                                    style="min-height: 36px; font-size: 15px;"
                                                                                    on:click=move |_| update_note_status(n_id, "open")
                                                                                >
                                                                                    {move || i18n.tr("cms_proposal_reopen", "Reopen")}
                                                                                </button>
                                                                            })}
                                                                        </div>
                                                                    </Show>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                }.into_any()
                                            }}
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }

                        CmsMode::InferenceRuntimes => {
                            view! {
                                <div class="exo-card">
                                    <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                                        <div>
                                            <h2 class="exo-h2" style="margin: 0;">
                                                {move || i18n.tr("cms_ai_providers_title", "AI Inference Providers & Active Models")}
                                            </h2>
                                            <p class="exo-sub" style="margin: 4px 0 0;">
                                                {move || i18n.tr(
                                                    "cms_ai_providers_sub",
                                                    "Configured local CLI runtime and verified active provider models available in this environment."
                                                )}
                                            </p>
                                        </div>
                                        <Show when=is_admin>
                                            <button type="button" class="exo-btn" on:click=refresh_models>
                                                {move || i18n.tr("cms_refresh_models", "Refresh Live Models")}
                                            </button>
                                        </Show>
                                    </div>

                                    {move || {
                                        let configured: Vec<_> = models
                                            .get()
                                            .into_iter()
                                            .filter(|m| m.key_configured)
                                            .collect();
                                        if configured.is_empty() {
                                            view! {
                                                <p class="exo-sub" style="margin-top: 14px;">
                                                    {move || i18n.tr(
                                                        "cms_ai_empty",
                                                        "No active inference providers configured in this environment."
                                                    )}
                                                </p>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="exo-grid-2" style="margin-top: 16px;">
                                                    {configured.into_iter().map(|m| {
                                                        let is_dep = m.deprecated_at.is_some();
                                                        let is_cool = m.cooldown_until.is_some();
                                                        view! {
                                                            <div class="exo-dep-card">
                                                                <div style="display: flex; justify-content: space-between; gap: 8px; flex-wrap: wrap; align-items: baseline;">
                                                                    <strong>{m.display_name}</strong>
                                                                    <span class="exo-eyebrow" style="margin: 0;">
                                                                        {move || if is_dep {
                                                                            i18n.tr("cms_model_deprecated", "Deprecated")
                                                                        } else if is_cool {
                                                                            i18n.tr("cms_model_cooldown", "Cooldown")
                                                                        } else {
                                                                            i18n.tr("cms_model_active", "Active")
                                                                        }}
                                                                    </span>
                                                                </div>
                                                                <div class="exo-eyebrow exo-mono" style="margin: 4px 0 0;">
                                                                    {m.provider} " · " {m.model_id}
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        }

                        CmsMode::OwnerTasks => {
                            view! {
                                <div class="exo-card">
                                    <h2 class="exo-h2" style="margin: 0;">
                                        {move || i18n.tr("cms_owner_tasks_title", "External Owner & Ops Checklist")}
                                    </h2>
                                    <p class="exo-sub" style="margin: 6px 0 18px;">
                                        {move || i18n.tr(
                                            "cms_owner_tasks_sub",
                                            "Tasks requiring external consoles or production server configuration. Check a box when done to persist completion state in the database."
                                        )}
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
                                            let actor_badge = if t.owner_actor == "Production Server" || t.owner_actor.contains("Agent") {
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
                                                                view! { <span class="exo-tag exo-tag-ok">{move || i18n.tr("cms_task_done", "Done")}</span> }.into_any()
                                                            } else {
                                                                view! { <span class="exo-tag exo-tag-warn">{move || i18n.tr("cms_task_pending", "Pending")}</span> }.into_any()
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
                                                                        on_toast(i18n.tr_untracked("cms_toast_copied", "Copied text to clipboard"));
                                                                    }
                                                                >
                                                                    {move || i18n.tr("copy_text_label", "Copy Text")}
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
                                <div class="exo-toc-title">{move || i18n.tr("cms_on_this_page", "On this page")}</div>
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
