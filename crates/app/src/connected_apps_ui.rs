use gloo_net::http::Request;
use leptos::prelude::*;

use crate::{
    i18n::use_i18n,
    live::use_live_sync,
    models::{
        copy_text_to_clipboard, extract_api_error_message, resolve_fresh_jwt,
        start_google_connect_flow, AuthConfigResponse, ConnectedAppItem, ConnectedAppsResponse,
        CopyIcon, DriveContentResponse, DriveFileItem, DriveFilesResponse, DriveSaveResponse,
        GoogleLogoIcon,
    },
};

fn friendly_mime_key(mime: &str) -> (&'static str, &'static str) {
    match mime {
        "application/vnd.google-apps.folder" => ("drive_mime_folder", "Folder"),
        "application/vnd.google-apps.document" => ("drive_mime_gdoc", "Google Doc"),
        "application/vnd.google-apps.spreadsheet" => ("drive_mime_gsheet", "Google Sheet"),
        "application/vnd.google-apps.presentation" => ("drive_mime_gslides", "Google Slides"),
        "text/markdown" => ("drive_mime_md", "Markdown (.md)"),
        "text/plain" => ("drive_mime_txt", "Text"),
        _ => ("drive_mime_file", "File"),
    }
}

fn is_https_url(raw: &str) -> bool {
    raw.trim().starts_with("https://")
}

#[component]
pub fn ConnectedAppsPreferencesPanel(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
    on_toast: impl Fn(String) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let live = use_live_sync();
    let i18n = use_i18n();

    let (apps_data, set_apps_data) = signal::<Option<ConnectedAppsResponse>>(None);
    let (loading_apps, set_loading_apps) = signal(true);
    let (connecting, set_connecting) = signal(false);
    let (revoke_confirm_open, set_revoke_confirm_open) = signal(false);
    let (status_msg, set_status_msg) = signal::<Option<String>>(None);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    // Google Docs & Drive Workspace State
    let (drive_files, set_drive_files) = signal::<Vec<DriveFileItem>>(Vec::new());
    let (loading_files, set_loading_files) = signal(false);
    let (folder_stack, set_folder_stack) = signal::<Vec<(String, String)>>(vec![(
        "root".to_string(),
        i18n.tr_untracked("drive_my_drive_root", "My Drive"),
    )]);
    let (drive_search, set_drive_search) = signal(String::new());
    let (show_trashed, set_show_trashed) = signal(false);

    let (show_new_folder, set_show_new_folder) = signal(false);
    let (new_folder_name, set_new_folder_name) = signal(String::new());

    let (show_new_doc, set_show_new_doc) = signal(false);
    let (new_doc_title, set_new_doc_title) = signal(String::new());
    let (new_doc_format, set_new_doc_format) = signal("google-doc".to_string());
    let (new_doc_markdown, set_new_doc_markdown) = signal(String::new());
    let (last_saved_link, set_last_saved_link) = signal::<Option<(String, String)>>(None);

    let (renaming_id, set_renaming_id) = signal::<Option<String>>(None);
    let (rename_draft, set_rename_draft) = signal(String::new());
    let (delete_confirm_id, set_delete_confirm_id) = signal::<Option<String>>(None);
    let (preview_doc, set_preview_doc) = signal::<Option<(String, String, String)>>(None);

    let load_drive_files = move || {
        let folder_id = folder_stack
            .get_untracked()
            .last()
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| "root".to_string());
        let search = drive_search.get_untracked().trim().to_string();
        let trashed = show_trashed.get_untracked();

        set_loading_files.set(true);
        set_error_msg.set(None);

        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                set_loading_files.set(false);
                return;
            };
            let enc_folder = js_sys::encode_uri_component(&folder_id)
                .as_string()
                .unwrap_or(folder_id);
            let mut url = format!("/api/drive/files?folderId={enc_folder}");
            if trashed {
                url.push_str("&trashed=true");
            }
            if !search.is_empty() {
                let enc_q = js_sys::encode_uri_component(&search)
                    .as_string()
                    .unwrap_or(search);
                url.push_str(&format!("&q={enc_q}"));
            }

            match Request::get(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.ok() {
                        if let Ok(parsed) = resp.json::<DriveFilesResponse>().await {
                            set_drive_files.set(parsed.files);
                        }
                    } else {
                        let txt = resp.text().await.unwrap_or_default();
                        let fallback = i18n.tr_untracked(
                            "conn_msg_list_err",
                            "Unable to list Google Drive & Docs files.",
                        );
                        set_error_msg.set(Some(extract_api_error_message(&txt, &fallback)));
                    }
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
            set_loading_files.set(false);
        });
    };

    let reload_connected_apps = move || {
        set_loading_apps.set(true);
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                set_loading_apps.set(false);
                return;
            };
            match Request::get("/api/connected-apps")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.ok() {
                        if let Ok(parsed) = resp.json::<ConnectedAppsResponse>().await {
                            let is_google_connected = parsed
                                .connected_apps
                                .iter()
                                .any(|a| a.provider == "google_drive" && a.status == "connected");
                            set_apps_data.set(Some(parsed));
                            if is_google_connected {
                                load_drive_files();
                            } else {
                                set_drive_files.set(Vec::new());
                            }
                        }
                    } else {
                        let txt = resp.text().await.unwrap_or_default();
                        let fallback = i18n.tr_untracked(
                            "conn_msg_load_err",
                            "Unable to load connected apps.",
                        );
                        set_error_msg.set(Some(extract_api_error_message(&txt, &fallback)));
                    }
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
            set_loading_apps.set(false);
        });
    };

    Effect::new(move |_| reload_connected_apps());

    Effect::new(move |_| {
        let _seq = live.pulse_seq.get();
        if live.matches_kind(&["connected_app", "system"]) {
            reload_connected_apps();
        }
    });

    let connect_google = move |_| {
        set_connecting.set(true);
        set_error_msg.set(None);
        set_status_msg.set(Some(i18n.tr_untracked(
            "cms_msg_redirecting_google",
            "Redirecting to Google consent screen...",
        )));
        leptos::task::spawn_local(async move {
            if let Err(err) =
                start_google_connect_flow(auth_config, initial_jwt, "preferences", None).await
            {
                set_connecting.set(false);
                set_status_msg.set(None);
                set_error_msg.set(Some(err));
            }
        });
    };

    let revoke_google = move |_| {
        set_revoke_confirm_open.set(false);
        set_status_msg.set(Some(i18n.tr_untracked(
            "conn_msg_revoking",
            "Disconnecting and revoking Google tokens...",
        )));
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                set_status_msg.set(None);
                return;
            };
            match Request::post("/api/oauth/google/revoke")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                Ok(resp) if resp.ok() => {
                    set_drive_files.set(Vec::new());
                    set_preview_doc.set(None);
                    set_status_msg.set(Some(i18n.tr_untracked(
                        "conn_msg_revoked",
                        "Google Drive & Docs disconnected and OAuth tokens revoked.",
                    )));
                    on_toast(i18n.tr_untracked(
                        "conn_toast_revoked",
                        "Google Drive & Docs revoked",
                    ));
                    reload_connected_apps();
                }
                Ok(resp) => {
                    let txt = resp.text().await.unwrap_or_default();
                    set_status_msg.set(None);
                    let fallback = i18n.tr_untracked(
                        "conn_msg_revoke_err",
                        "Failed to revoke Google authorization.",
                    );
                    set_error_msg.set(Some(extract_api_error_message(&txt, &fallback)));
                }
                Err(e) => {
                    set_status_msg.set(None);
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        });
    };

    let create_folder = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = new_folder_name.get_untracked().trim().to_string();
        if name.is_empty() {
            return;
        }
        let parent_id = folder_stack
            .get_untracked()
            .last()
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| "root".to_string());

        set_status_msg.set(Some(format!(
            "{} \"{name}\"...",
            i18n.tr_untracked("conn_msg_creating_folder", "Creating folder...")
        )));
        set_error_msg.set(None);

        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            if let Ok(req) = Request::post("/api/drive/folders")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "name": name,
                    "parentId": parent_id,
                }))
            {
                match req.send().await {
                    Ok(resp) if resp.ok() => {
                        set_new_folder_name.set(String::new());
                        set_show_new_folder.set(false);
                        set_status_msg.set(None);
                        on_toast(i18n.tr_untracked(
                            "conn_toast_folder_created",
                            "Folder created in Google Drive",
                        ));
                        load_drive_files();
                    }
                    Ok(resp) => {
                        let txt = resp.text().await.unwrap_or_default();
                        set_status_msg.set(None);
                        let fallback = i18n.tr_untracked(
                            "conn_msg_folder_err",
                            "Folder create failed.",
                        );
                        set_error_msg.set(Some(extract_api_error_message(&txt, &fallback)));
                    }
                    Err(e) => {
                        set_status_msg.set(None);
                        set_error_msg.set(Some(e.to_string()));
                    }
                }
            }
        });
    };

    let create_or_save_doc = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let title = new_doc_title.get_untracked().trim().to_string();
        let markdown = new_doc_markdown.get_untracked();
        let format = new_doc_format.get_untracked();
        if title.is_empty() || markdown.trim().is_empty() {
            set_error_msg.set(Some(i18n.tr_untracked(
                "conn_msg_doc_required",
                "Enter both a title and Markdown content.",
            )));
            return;
        }
        let cur_folder = folder_stack
            .get_untracked()
            .last()
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| "root".to_string());

        set_status_msg.set(Some(if format == "google-doc" {
            i18n.tr_untracked(
                "conn_msg_converting_gdoc",
                "Converting Markdown to native Google Doc...",
            )
        } else {
            i18n.tr_untracked(
                "conn_msg_uploading_md",
                "Uploading Markdown file to Google Drive...",
            )
        }));
        set_error_msg.set(None);

        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            let mut payload = serde_json::json!({
                "title": title,
                "markdown": markdown,
                "format": format,
            });
            if cur_folder == "root" {
                payload["folderName"] = serde_json::json!("ExoNovel Docs");
            } else {
                payload["parentId"] = serde_json::json!(cur_folder);
            }

            if let Ok(req) = Request::post("/api/drive/save-doc")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&payload)
            {
                match req.send().await {
                    Ok(resp) if resp.ok() => {
                        if let Ok(saved) = resp.json::<DriveSaveResponse>().await {
                            let link = saved.file.web_view_link.clone().unwrap_or_default();
                            if is_https_url(&link) {
                                set_last_saved_link.set(Some((saved.file.name.clone(), link)));
                            }
                            set_new_doc_title.set(String::new());
                            set_new_doc_markdown.set(String::new());
                            set_show_new_doc.set(false);
                            set_status_msg.set(Some(format!(
                                "{}: \"{}\" ({})",
                                i18n.tr_untracked("conn_toast_doc_saved", "Document saved"),
                                saved.file.name,
                                saved.action
                            )));
                            on_toast(i18n.tr_untracked(
                                "conn_toast_doc_saved",
                                "Document saved to Google Docs / Drive",
                            ));
                            load_drive_files();
                        }
                    }
                    Ok(resp) => {
                        let txt = resp.text().await.unwrap_or_default();
                        set_status_msg.set(None);
                        let fallback = i18n.tr_untracked("cms_msg_save_failed", "Save failed.");
                        set_error_msg.set(Some(extract_api_error_message(&txt, &fallback)));
                    }
                    Err(e) => {
                        set_status_msg.set(None);
                        set_error_msg.set(Some(e.to_string()));
                    }
                }
            }
        });
    };

    let preview_drive_file = move |file: DriveFileItem| {
        set_status_msg.set(Some(format!(
            "{} \"{}\"...",
            i18n.tr_untracked("drive_loading_files", "Loading"),
            file.name
        )));
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            let enc_id = js_sys::encode_uri_component(&file.id)
                .as_string()
                .unwrap_or_else(|| file.id.clone());
            let enc_mime = js_sys::encode_uri_component(&file.mime_type)
                .as_string()
                .unwrap_or_else(|| file.mime_type.clone());
            let url = format!("/api/drive/files/{enc_id}/content?mimeType={enc_mime}");
            match Request::get(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                Ok(resp) if resp.ok() => {
                    if let Ok(parsed) = resp.json::<DriveContentResponse>().await {
                        set_preview_doc.set(Some((
                            file.name,
                            parsed.exported_mime_type,
                            parsed.content,
                        )));
                        set_status_msg.set(None);
                    }
                }
                Ok(resp) => {
                    let txt = resp.text().await.unwrap_or_default();
                    set_status_msg.set(None);
                    let fallback = i18n.tr_untracked("conn_msg_preview_err", "Preview failed.");
                    set_error_msg.set(Some(extract_api_error_message(&txt, &fallback)));
                }
                Err(e) => {
                    set_status_msg.set(None);
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        });
    };

    let run_file_action = move |file_id: String, action: &'static str, name: Option<String>| {
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
                return;
            };
            let enc_id = js_sys::encode_uri_component(&file_id)
                .as_string()
                .unwrap_or(file_id);
            let url = format!("/api/drive/files/{enc_id}/action");
            if let Ok(req) = Request::post(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "action": action,
                    "name": name,
                }))
            {
                match req.send().await {
                    Ok(resp) if resp.ok() => {
                        set_renaming_id.set(None);
                        set_delete_confirm_id.set(None);
                        on_toast(format!(
                            "{} ({action})",
                            i18n.tr_untracked("conn_toast_item_updated", "Drive item updated")
                        ));
                        load_drive_files();
                    }
                    Ok(resp) => {
                        let txt = resp.text().await.unwrap_or_default();
                        let fallback = i18n.tr_untracked("conn_msg_action_err", "Action failed.");
                        set_error_msg.set(Some(extract_api_error_message(&txt, &fallback)));
                    }
                    Err(e) => set_error_msg.set(Some(e.to_string())),
                }
            }
        });
    };

    let google_app = move || -> Option<ConnectedAppItem> {
        apps_data.get().and_then(|d| {
            d.connected_apps
                .into_iter()
                .find(|a| a.provider == "google_drive")
        })
    };

    let is_google_connected = move || {
        google_app().is_some_and(|a| a.status == "connected")
    };

    view! {
        <div>
            <div class="exo-card">
                <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
                    <div>
                        <h2 class="exo-h2" style="margin: 0;">{move || i18n.tr("connected_apps_title", "Connected Apps & Google Docs")}</h2>
                        <p class="exo-sub" style="margin: 6px 0 0;">
                            {move || i18n.tr("connected_apps_desc", "Primary sign-in uses minimal identity scopes only (openid, email, profile). Connect Google Drive & Google Docs separately here to create, preview, and back up your Markdown manuscripts and ADRs into Google Docs or .md files.")}
                        </p>
                    </div>
                    <button
                        type="button"
                        class="exo-btn"
                        style="min-height: 36px; font-size: 14px;"
                        on:click=move |_| reload_connected_apps()
                    >
                        {move || i18n.tr("refresh_label", "Refresh")}
                    </button>
                </div>

                {move || status_msg.get().map(|m| view! {
                    <div class="exo-invite-banner" style="margin-top: 14px;">
                        <span>{m}</span>
                    </div>
                })}
                {move || error_msg.get().map(|m| view! {
                    <div class="exo-alert-error" style="margin-top: 14px;">
                        {m}
                    </div>
                })}

                <div style="margin-top: 18px; padding-top: 16px; border-top: 1px solid var(--border);">
                    <div class="exo-label">{move || i18n.tr("conn_primary_signin_title", "Primary Sign-In Methods (Identity Only)")}</div>
                    <Show when=move || loading_apps.get()>
                        <p class="exo-sub">{move || i18n.tr("conn_loading_accounts", "Loading linked accounts...")}</p>
                    </Show>
                    {move || apps_data.get().map(|data| {
                        if data.sso_accounts.is_empty() {
                            view! { <p class="exo-sub">{move || i18n.tr("conn_session_auth", "Authenticated with session credentials.")}</p> }.into_any()
                        } else {
                            view! {
                                <div style="display: flex; gap: 10px; flex-wrap: wrap; margin-top: 8px;">
                                    {data.sso_accounts.into_iter().map(|acc| {
                                        let sc = if acc.scopes.trim().is_empty() { "openid email profile".to_string() } else { acc.scopes };
                                        view! {
                                            <div class="exo-dep-card" style="margin: 0; flex: 1; min-width: 220px;">
                                                <div style="display: flex; justify-content: space-between; gap: 8px; align-items: center;">
                                                    <strong>{acc.provider_id}</strong>
                                                    <span class="exo-tag exo-tag-ok">{move || i18n.tr("conn_signin_linked", "Sign-in Linked")}</span>
                                                </div>
                                                <div class="exo-sub exo-mono" style="font-size: 13px; margin-top: 4px;">
                                                    {move || i18n.tr("conn_scopes_prefix", "Scopes:")} " " {sc}
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    })}
                </div>
            </div>

            <div class="exo-card">
                <div style="display: flex; align-items: center; justify-content: space-between; gap: 14px; flex-wrap: wrap;">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <GoogleLogoIcon />
                        <div>
                            <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                <h2 class="exo-h2" style="margin: 0;">{move || i18n.tr("google_drive_docs_title", "Google Drive & Google Docs")}</h2>
                                {move || match google_app() {
                                    Some(app) if app.status == "connected" => view! {
                                        <span class="exo-tag exo-tag-ok">{move || i18n.tr("status_connected", "Connected")}</span>
                                    }.into_any(),
                                    Some(_) => view! {
                                        <span class="exo-tag exo-tag-warn">{move || i18n.tr("status_revoked", "Revoked")}</span>
                                    }.into_any(),
                                    None => view! {
                                        <span class="exo-tag">{move || i18n.tr("status_not_connected", "Not Connected")}</span>
                                    }.into_any(),
                                }}
                            </div>
                            <p class="exo-sub" style="margin: 4px 0 0;">
                                {move || match google_app() {
                                    Some(app) if app.status == "connected" => {
                                        let email = if app.provider_email.is_empty() {
                                            i18n.tr("conn_auth_acct_fallback", "Authorized Google account")
                                        } else {
                                            app.provider_email
                                        };
                                        let scopes_lbl = i18n.tr("conn_scopes_prefix", "Scopes:");
                                        format!("{email} · {scopes_lbl} {}", app.scopes)
                                    }
                                    _ => i18n.tr("conn_drive_optional_desc", "Optional per-file authorization (drive.file) to create Google Docs, save .md backups, and browse ExoNovel Docs."),
                                }}
                            </p>
                        </div>
                    </div>

                    <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                        <button
                            type="button"
                            class="exo-btn exo-btn-primary"
                            disabled=move || connecting.get()
                            on:click=connect_google
                        >
                            {move || if is_google_connected() {
                                i18n.tr("reconnect_google_btn", "Reconnect Google")
                            } else {
                                i18n.tr("connect_google_btn", "Connect Google Drive & Docs")
                            }}
                        </button>
                        <Show when=is_google_connected>
                            <button
                                type="button"
                                class="exo-btn"
                                on:click=move |_| set_revoke_confirm_open.set(true)
                            >
                                {move || i18n.tr("disconnect_revoke_btn", "Disconnect & Revoke")}
                            </button>
                        </Show>
                    </div>
                </div>

                <Show when=move || revoke_confirm_open.get()>
                    <div class="exo-invite-banner" style="margin-top: 14px; justify-content: space-between; flex-wrap: wrap;">
                        <span>
                            {move || i18n.tr("conn_revoke_confirm_msg", "Revoke ExoNovel's Google Drive & Docs OAuth tokens? This disconnects Drive backups without affecting your main ExoNovel sign-in.")}
                        </span>
                        <div style="display: flex; gap: 8px;">
                            <button type="button" class="exo-btn exo-btn-primary" on:click=revoke_google>
                                {move || i18n.tr("conn_revoke_confirm_btn", "Confirm Revoke")}
                            </button>
                            <button type="button" class="exo-btn" on:click=move |_| set_revoke_confirm_open.set(false)>
                                {move || i18n.tr("cancel_label", "Cancel")}
                            </button>
                        </div>
                    </div>
                </Show>

                {move || apps_data.get().map(|data| {
                    let redir = data.redirect_uri.clone();
                    let redir_copy = data.redirect_uri.clone();
                    let configured = data.oauth_configured;
                    view! {
                        <div class="exo-format-tile" style="margin-top: 16px;">
                            <div style="display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap;">
                                <span class="exo-label" style="margin: 0;">
                                    {move || if configured {
                                        i18n.tr("conn_redirect_configured", "OAuth Redirect URI (Configured with Provider)")
                                    } else {
                                        i18n.tr("conn_redirect_missing", "Server Google OAuth Credentials Required")
                                    }}
                                </span>
                                <button
                                    type="button"
                                    class="exo-btn"
                                    style="min-height: 30px; font-size: 13px; padding: 0 10px;"
                                    on:click=move |_| {
                                        copy_text_to_clipboard(&redir_copy);
                                        on_toast(i18n.tr_untracked("conn_toast_uri_copied", "Redirect URI copied"));
                                    }
                                >
                                    <CopyIcon />
                                    <span style="margin-left: 6px;">{move || i18n.tr("conn_copy_uri_btn", "Copy URI")}</span>
                                </button>
                            </div>
                            <div class="exo-mono" style="font-size: 13px; margin-top: 6px; word-break: break-all;">
                                {redir}
                            </div>
                        </div>
                    }
                })}
            </div>

            <Show when=is_google_connected>
                <div class="exo-card">
                    <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                        <div>
                            <h2 class="exo-h2" style="margin: 0;">{move || i18n.tr("drive_ws_title", "Google Docs & Drive Workspace")}</h2>
                            <p class="exo-sub" style="margin: 4px 0 0;">
                                {move || i18n.tr("drive_ws_sub", "Create native editable Google Docs from Markdown, browse ExoNovel folders, preview docs as Markdown, or manage Trash.")}
                            </p>
                        </div>
                        <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                            <button
                                type="button"
                                class="exo-btn exo-btn-primary"
                                on:click=move |_| {
                                    set_show_new_folder.set(false);
                                    set_show_new_doc.update(|v| *v = !*v);
                                }
                            >
                                {move || i18n.tr("drive_new_doc_btn", "+ New Google Doc")}
                            </button>
                            <button
                                type="button"
                                class="exo-btn"
                                on:click=move |_| {
                                    set_show_new_doc.set(false);
                                    set_show_new_folder.update(|v| *v = !*v);
                                }
                            >
                                {move || i18n.tr("drive_new_folder_btn", "+ New Folder")}
                            </button>
                            <button
                                type="button"
                                class=move || if show_trashed.get() { "exo-btn exo-btn-primary" } else { "exo-btn" }
                                on:click=move |_| {
                                    set_show_trashed.update(|v| *v = !*v);
                                    load_drive_files();
                                }
                            >
                                {move || if show_trashed.get() {
                                    i18n.tr("drive_viewing_trash", "Viewing Trash")
                                } else {
                                    i18n.tr("drive_trash_btn", "Trash")
                                }}
                            </button>
                            <button type="button" class="exo-btn" on:click=move |_| load_drive_files()>
                                {move || i18n.tr("drive_refresh_files", "Refresh Files")}
                            </button>
                        </div>
                    </div>

                    {move || last_saved_link.get().map(|(doc_name, link)| view! {
                        <div class="exo-invite-banner" style="margin-top: 14px; justify-content: space-between; flex-wrap: wrap;">
                            <span><strong>{doc_name}</strong> " " {move || i18n.tr("drive_ready_suffix", "is ready in Google Docs.")}</span>
                            <a href=link target="_blank" rel="noopener noreferrer" class="exo-btn exo-btn-primary">
                                {move || i18n.tr("drive_open_gdocs", "Open in Google Docs ↗")}
                            </a>
                        </div>
                    })}

                    <Show when=move || show_new_folder.get()>
                        <form on:submit=create_folder style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 14px;">
                            <input
                                type="text"
                                class="exo-input"
                                style="flex: 1; min-width: 220px;"
                                required=true
                                placeholder=move || i18n.tr("drive_new_folder_ph", "New folder name...")
                                prop:value=move || new_folder_name.get()
                                on:input=move |ev| set_new_folder_name.set(event_target_value(&ev))
                            />
                            <button type="submit" class="exo-btn exo-btn-primary">{move || i18n.tr("drive_create_folder_btn", "Create Folder")}</button>
                            <button type="button" class="exo-btn" on:click=move |_| set_show_new_folder.set(false)>{move || i18n.tr("cancel_label", "Cancel")}</button>
                        </form>
                    </Show>

                    <Show when=move || show_new_doc.get()>
                        <form on:submit=create_or_save_doc style="margin-top: 14px; padding: 16px; border: 1px solid var(--border); border-radius: 10px;">
                            <div class="exo-grid-2">
                                <div class="exo-field">
                                    <label class="exo-label">{move || i18n.tr("drive_doc_title_label", "Document Title")}</label>
                                    <input
                                        type="text"
                                        class="exo-input"
                                        required=true
                                        placeholder=move || i18n.tr("drive_doc_title_ph", "Chapter 1 — Outline")
                                        prop:value=move || new_doc_title.get()
                                        on:input=move |ev| set_new_doc_title.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="exo-field">
                                    <label class="exo-label">{move || i18n.tr("drive_target_fmt_label", "Target Format")}</label>
                                    <select
                                        class="exo-input"
                                        prop:value=move || new_doc_format.get()
                                        on:change=move |ev| set_new_doc_format.set(event_target_value(&ev))
                                    >
                                        <option value="google-doc">{move || i18n.tr("drive_fmt_gdoc", "Google Doc (Native Editable)")}</option>
                                        <option value="markdown">{move || i18n.tr("drive_fmt_md", "Markdown (.md)")}</option>
                                    </select>
                                </div>
                            </div>
                            <div class="exo-field">
                                <label class="exo-label">{move || i18n.tr("drive_md_content_label", "Markdown Content")}</label>
                                <textarea
                                    class="exo-input exo-textarea exo-mono"
                                    style="min-height: 140px;"
                                    required=true
                                    placeholder=move || i18n.tr("drive_md_content_ph", "# Title\n\nWrite your scene or notes in Markdown...")
                                    prop:value=move || new_doc_markdown.get()
                                    on:input=move |ev| set_new_doc_markdown.set(event_target_value(&ev))
                                />
                            </div>
                            <div style="display: flex; gap: 8px;">
                                <button type="submit" class="exo-btn exo-btn-primary">{move || i18n.tr("drive_save_btn", "Save to Google Docs / Drive")}</button>
                                <button type="button" class="exo-btn" on:click=move |_| set_show_new_doc.set(false)>{move || i18n.tr("cancel_label", "Cancel")}</button>
                            </div>
                        </form>
                    </Show>

                    <div style="display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap; margin-top: 16px;">
                        <div style="display: flex; gap: 6px; flex-wrap: wrap; align-items: center;">
                            {move || folder_stack.get().into_iter().enumerate().map(|(idx, (_id, label))| {
                                view! {
                                    <button
                                        type="button"
                                        class="exo-filter-chip"
                                        on:click=move |_| {
                                            set_show_trashed.set(false);
                                            set_folder_stack.update(|s| s.truncate(idx + 1));
                                            load_drive_files();
                                        }
                                    >
                                        {move || if idx == 0 {
                                            i18n.tr("drive_my_drive_root", "My Drive")
                                        } else {
                                            format!("› {label}")
                                        }}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <form
                            style="display: flex; gap: 8px; min-width: 240px;"
                            on:submit=move |ev: web_sys::SubmitEvent| {
                                ev.prevent_default();
                                load_drive_files();
                            }
                        >
                            <input
                                type="search"
                                class="exo-input"
                                placeholder=move || i18n.tr("drive_search_ph", "Search Drive files...")
                                prop:value=move || drive_search.get()
                                on:input=move |ev| set_drive_search.set(event_target_value(&ev))
                            />
                            <button type="submit" class="exo-btn">{move || i18n.tr("drive_search_btn", "Search")}</button>
                        </form>
                    </div>

                    <Show when=move || loading_files.get()>
                        <p class="exo-sub" style="margin-top: 14px;">{move || i18n.tr("drive_loading_files", "Loading files from Google Drive...")}</p>
                    </Show>

                    <div style="overflow-x: auto; margin-top: 14px;">
                        <table class="exo-table">
                            <thead>
                                <tr>
                                    <th>{move || i18n.tr("drive_th_name", "Name")}</th>
                                    <th>{move || i18n.tr("drive_th_type", "Type")}</th>
                                    <th>{move || i18n.tr("drive_th_modified", "Modified")}</th>
                                    <th>{move || i18n.tr("drive_th_actions", "Actions")}</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || drive_files.get()
                                    key=|f| format!("{}-{}-{}", f.id, f.name, f.modified_time.clone().unwrap_or_default())
                                    children=move |file| {
                                        let is_folder = file.mime_type == "application/vnd.google-apps.folder";
                                        let is_gdoc = file.mime_type == "application/vnd.google-apps.document";
                                        let is_trashed = file.trashed;
                                        let file_id = StoredValue::new(file.id.clone());
                                        let file_name = StoredValue::new(file.name.clone());
                                        let file_for_preview = StoredValue::new(file.clone());
                                        let link_opt = file.web_view_link.clone().filter(|u| is_https_url(u));
                                        let (mime_key, mime_fallback) = friendly_mime_key(&file.mime_type);
                                        let mod_time = file.modified_time.clone().unwrap_or_else(|| "—".to_string());

                                        view! {
                                            <tr>
                                                <td>
                                                    {if is_folder && !is_trashed {
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="exo-email-trigger"
                                                                on:click=move |_| {
                                                                    set_folder_stack.update(|s| {
                                                                        s.push((file_id.get_value(), file_name.get_value()));
                                                                    });
                                                                    set_drive_search.set(String::new());
                                                                    load_drive_files();
                                                                }
                                                            >
                                                                <span>{file.name.clone()}</span>
                                                            </button>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span>{file.name.clone()}</span> }.into_any()
                                                    }}
                                                    <Show when=move || renaming_id.get().as_deref() == Some(file_id.get_value().as_str())>
                                                        <div style="display: flex; gap: 6px; margin-top: 6px;">
                                                            <input
                                                                type="text"
                                                                class="exo-input"
                                                                style="min-width: 160px;"
                                                                prop:value=move || rename_draft.get()
                                                                on:input=move |ev| set_rename_draft.set(event_target_value(&ev))
                                                            />
                                                            <button
                                                                type="button"
                                                                class="exo-btn exo-btn-primary"
                                                                style="min-height: 32px; font-size: 13px;"
                                                                on:click=move |_| run_file_action(file_id.get_value(), "rename", Some(rename_draft.get_untracked()))
                                                            >
                                                                {move || i18n.tr("drive_save_inline", "Save")}
                                                            </button>
                                                            <button
                                                                type="button"
                                                                class="exo-btn"
                                                                style="min-height: 32px; font-size: 13px;"
                                                                on:click=move |_| set_renaming_id.set(None)
                                                            >
                                                                {move || i18n.tr("cancel_label", "Cancel")}
                                                            </button>
                                                        </div>
                                                    </Show>
                                                </td>
                                                <td><span class="exo-tag">{move || i18n.tr(mime_key, mime_fallback)}</span></td>
                                                <td class="exo-mono" style="font-size: 13px;">{mod_time}</td>
                                                <td>
                                                    <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                                                        {link_opt.map(|href| {
                                                            view! {
                                                                <a href=href target="_blank" rel="noopener noreferrer" class="exo-btn" style="min-height: 30px; font-size: 13px; padding: 0 10px;">
                                                                    {move || if is_gdoc {
                                                                        i18n.tr("drive_open_doc_btn", "Open Doc ↗")
                                                                    } else {
                                                                        i18n.tr("drive_open_btn", "Open ↗")
                                                                    }}
                                                                </a>
                                                            }
                                                        })}
                                                        {(!is_folder).then(|| view! {
                                                            <button
                                                                type="button"
                                                                class="exo-btn"
                                                                style="min-height: 30px; font-size: 13px; padding: 0 10px;"
                                                                on:click=move |_| preview_drive_file(file_for_preview.get_value())
                                                            >
                                                                {move || i18n.tr("drive_preview_btn", "Preview")}
                                                            </button>
                                                        })}
                                                        <button
                                                            type="button"
                                                            class="exo-btn"
                                                            style="min-height: 30px; font-size: 13px; padding: 0 10px;"
                                                            on:click=move |_| {
                                                                set_rename_draft.set(file_name.get_value());
                                                                set_renaming_id.set(Some(file_id.get_value()));
                                                            }
                                                        >
                                                            {move || i18n.tr("drive_rename_btn", "Rename")}
                                                        </button>
                                                        {if is_trashed {
                                                            view! {
                                                                <>
                                                                    <button
                                                                        type="button"
                                                                        class="exo-btn"
                                                                        style="min-height: 30px; font-size: 13px; padding: 0 10px;"
                                                                        on:click=move |_| run_file_action(file_id.get_value(), "restore", None)
                                                                    >
                                                                        {move || i18n.tr("drive_restore_btn", "Restore")}
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        class="exo-btn"
                                                                        style="min-height: 30px; font-size: 13px; padding: 0 10px;"
                                                                        on:click=move |_| set_delete_confirm_id.set(Some(file_id.get_value()))
                                                                    >
                                                                        {move || i18n.tr("drive_delete_forever_btn", "Delete Forever")}
                                                                    </button>
                                                                </>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <button
                                                                    type="button"
                                                                    class="exo-btn"
                                                                    style="min-height: 30px; font-size: 13px; padding: 0 10px;"
                                                                    on:click=move |_| run_file_action(file_id.get_value(), "trash", None)
                                                                >
                                                                    {move || i18n.tr("drive_trash_item_btn", "Trash")}
                                                                </button>
                                                            }.into_any()
                                                        }}
                                                    </div>
                                                    <Show when=move || delete_confirm_id.get().as_deref() == Some(file_id.get_value().as_str())>
                                                        <div style="display: flex; gap: 6px; align-items: center; margin-top: 6px;">
                                                            <span class="exo-sub">{move || i18n.tr("drive_confirm_perm_delete", "Confirm permanent delete?")}</span>
                                                            <button
                                                                type="button"
                                                                class="exo-btn exo-btn-primary"
                                                                style="min-height: 28px; font-size: 12px;"
                                                                on:click=move |_| run_file_action(file_id.get_value(), "delete", None)
                                                            >
                                                                {move || i18n.tr("confirm_label", "Confirm")}
                                                            </button>
                                                            <button
                                                                type="button"
                                                                class="exo-btn"
                                                                style="min-height: 28px; font-size: 12px;"
                                                                on:click=move |_| set_delete_confirm_id.set(None)
                                                            >
                                                                {move || i18n.tr("cancel_label", "Cancel")}
                                                            </button>
                                                        </div>
                                                    </Show>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                        {move || drive_files.get().is_empty().then(|| view! {
                            <p class="exo-sub" style="margin-top: 12px;">
                                {move || i18n.tr("drive_empty_files", "No files found in this view. Click '+ New Google Doc' or save ADRs from Docs & ADRs.")}
                            </p>
                        })}
                    </div>

                    {move || preview_doc.get().map(|(name, mime, content)| {
                        let copy_body = StoredValue::new(content.clone());
                        view! {
                            <div style="margin-top: 18px; padding-top: 16px; border-top: 1px solid var(--border);">
                                <div style="display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap;">
                                    <div>
                                        <strong>{name}</strong>
                                        <span class="exo-tag" style="margin-left: 8px;">{mime}</span>
                                    </div>
                                    <div style="display: flex; gap: 8px;">
                                        <button
                                            type="button"
                                            class="exo-btn"
                                            style="min-height: 32px; font-size: 13px;"
                                            on:click=move |_| {
                                                copy_body.with_value(|c| copy_text_to_clipboard(c));
                                                on_toast(i18n.tr_untracked("drive_toast_doc_copied", "Document content copied"));
                                            }
                                        >
                                            {move || i18n.tr("copy_text_label", "Copy Text")}
                                        </button>
                                        <button
                                            type="button"
                                            class="exo-btn"
                                            style="min-height: 32px; font-size: 13px;"
                                            on:click=move |_| set_preview_doc.set(None)
                                        >
                                            {move || i18n.tr("drive_close_preview", "Close Preview")}
                                        </button>
                                    </div>
                                </div>
                                <pre style="white-space: pre-wrap; margin-top: 10px; padding: 14px; border-radius: 8px; background: var(--surface-2, rgba(127,127,127,0.08)); max-height: 380px; overflow-y: auto; font-size: 14px;">
                                    {content}
                                </pre>
                            </div>
                        }
                    })}
                </div>
            </Show>
        </div>
    }
}
