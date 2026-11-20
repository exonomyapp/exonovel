use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use serde::{Deserialize, Serialize};
use web_sys::RequestCredentials;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfigResponse {
    pub auth_configured: bool,
    pub auth_url: Option<String>,
    pub admin_email: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifiedUser {
    pub auth_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub did_peer: Option<String>,
    pub authorized: bool,
    pub is_admin: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct MeResponse {
    authenticated: bool,
    user: VerifiedUser,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocHeading {
    pub id: String,
    pub text: String,
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocNode {
    pub id: String,
    pub name: String,
    pub title: String,
    pub path: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub children: Vec<DocNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct DocsTreeResponse {
    tree: Vec<DocNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocPayload {
    pub path: String,
    pub name: String,
    pub title: String,
    pub html: String,
    pub word_count: usize,
    pub reading_minutes: usize,
    pub headings: Vec<DocHeading>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowlistEntry {
    pub email: String,
    pub enabled: bool,
    pub role: String,
    pub note: String,
    pub added_by: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AllowlistResponse {
    entries: Vec<AllowlistEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveTab {
    Workspace,
    Docs,
    Settings,
}

async fn fetch_neon_jwt(auth_url: &str, verifier: Option<&str>) -> Option<String> {
    let base = auth_url.trim_end_matches('/');
    if let Some(v) = verifier {
        let session_url = format!("{base}/get-session?neon_auth_session_verifier={v}");
        let _ = Request::get(&session_url)
            .credentials(RequestCredentials::Include)
            .send()
            .await;
    }

    let token_url = format!("{base}/token");
    let resp = Request::get(&token_url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .ok()?;
    if !resp.ok() {
        return None;
    }
    #[derive(Deserialize)]
    struct TokenBody {
        token: Option<String>,
    }
    let body = resp.json::<TokenBody>().await.ok()?;
    body.token
}

async fn load_authenticated_user(auth_url: &str, verifier: Option<&str>) -> Option<(VerifiedUser, String)> {
    let jwt = fetch_neon_jwt(auth_url, verifier).await?;
    let resp = Request::get("/api/me")
        .header("Authorization", &format!("Bearer {jwt}"))
        .send()
        .await
        .ok()?;
    if !resp.ok() {
        return None;
    }
    let me = resp.json::<MeResponse>().await.ok()?;
    Some((me.user, jwt))
}

fn first_doc_path(nodes: &[DocNode]) -> Option<String> {
    for node in nodes {
        if node.node_type == "file" {
            if node.path != "AGENTS.md" {
                return Some(node.path.clone());
            }
        } else if let Some(child_path) = first_doc_path(&node.children) {
            return Some(child_path);
        }
    }
    nodes
        .iter()
        .find(|n| n.node_type == "file")
        .map(|n| n.path.clone())
}

fn filter_tree(nodes: &[DocNode], query: &str) -> Vec<DocNode> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return nodes.to_vec();
    }
    let mut out = Vec::new();
    for node in nodes {
        if node.node_type == "file" {
            if node.title.to_lowercase().contains(&q) || node.path.to_lowercase().contains(&q) {
                out.push(node.clone());
            }
        } else {
            let children = filter_tree(&node.children, &q);
            if !children.is_empty() {
                let mut cloned = node.clone();
                cloned.children = children;
                out.push(cloned);
            }
        }
    }
    out
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="ExoNovel" />
        <Router>
            <Routes fallback=|| view! { <HomeView /> }>
                <Route path=path!("/") view=HomeView />
            </Routes>
        </Router>
    }
}

#[component]
fn HomeView() -> impl IntoView {

    let (auth_config, set_auth_config) = signal::<Option<AuthConfigResponse>>(None);
    let (loading_session, set_loading_session) = signal(true);
    let (current_user, set_current_user) = signal::<Option<VerifiedUser>>(None);
    let (session_jwt, set_session_jwt) = signal::<Option<String>>(None);
    let (active_tab, set_active_tab) = signal(ActiveTab::Workspace);
    let (dark_mode, set_dark_mode) = signal(false);

    Effect::new(move |_| {
        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                if let Some(root) = doc.document_element() {
                    let _ = if dark_mode.get() {
                        root.class_list().add_1("dark")
                    } else {
                        root.class_list().remove_1("dark")
                    };
                }
            }
        }
    });

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
                    set_current_user.set(Some(user));
                    set_session_jwt.set(Some(jwt));
                }
            }
            set_loading_session.set(false);
        });
    });

    let sign_out = move |_| {
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
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
            set_active_tab.set(ActiveTab::Workspace);
        });
    };

    view! {
        <Show
            when=move || !loading_session.get()
            fallback=|| view! {
                <div class="exo-auth-page">
                    <p class="exo-sub">"Loading ExoNovel session..."</p>
                </div>
            }
        >
            <Show
                when=move || current_user.get().is_some()
                fallback=move || view! {
                    <SignInView
                        auth_config=auth_config
                        on_signed_in=refresh_session
                    />
                }
            >
                {move || {
                    let Some(user) = current_user.get() else {
                        return view! { <span /> }.into_any();
                    };
                    let can_read_docs = user.authorized;
                    let is_admin = user.is_admin;
                    let email_label = user.email.clone().unwrap_or_else(|| "Signed-in writer".to_string());
                    let did_label = user.did_peer.clone().unwrap_or_else(|| "Pending did:peer".to_string());

                    view! {
                        <div class="exo-shell">
                            <header class="exo-header">
                                <div class="exo-header-left">
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        on:click=move |_| set_active_tab.set(ActiveTab::Workspace)
                                    >
                                        <span class="exo-brand">"ExoNovel"</span>
                                    </button>

                                    {if can_read_docs {
                                        view! {
                                            <button
                                                type="button"
                                                class=move || if active_tab.get() == ActiveTab::Docs {
                                                    "exo-btn exo-btn-active"
                                                } else {
                                                    "exo-btn"
                                                }
                                                on:click=move |_| set_active_tab.set(ActiveTab::Docs)
                                            >
                                                "Docs Reader"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span /> }.into_any()
                                    }}

                                    {if is_admin {
                                        view! {
                                            <button
                                                type="button"
                                                class=move || if active_tab.get() == ActiveTab::Settings {
                                                    "exo-btn exo-btn-active"
                                                } else {
                                                    "exo-btn"
                                                }
                                                on:click=move |_| set_active_tab.set(ActiveTab::Settings)
                                            >
                                                "Settings"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span /> }.into_any()
                                    }}
                                </div>

                                <div class="exo-header-actions">
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        on:click=move |_| set_dark_mode.update(|d| *d = !*d)
                                    >
                                        {move || if dark_mode.get() { "Light" } else { "Dark" }}
                                    </button>
                                    <button
                                        type="button"
                                        class="exo-btn"
                                        on:click=sign_out
                                    >
                                        "Sign out"
                                    </button>
                                </div>
                            </header>

                            {move || match active_tab.get() {
                                ActiveTab::Docs if can_read_docs => view! {
                                    <DocsReaderView
                                        auth_config=auth_config
                                        initial_jwt=session_jwt
                                    />
                                }.into_any(),
                                ActiveTab::Settings if is_admin => view! {
                                    <AdminSettingsView
                                        auth_config=auth_config
                                        initial_jwt=session_jwt
                                    />
                                }.into_any(),
                                _ => view! {
                                    <main class="exo-workspace-body">
                                        <h1 class="exo-h1">"ExoNovel Workspace"</h1>
                                        <p class="exo-sub">
                                            "Signed in as " <strong>{email_label.clone()}</strong>
                                        </p>

                                        <div class="exo-card">
                                            <div class="exo-label">"Authoritative Ownership Key (did:peer)"</div>
                                            <div class="exo-mono">{did_label.clone()}</div>
                                        </div>

                                        {if can_read_docs {
                                            view! {
                                                <div class="exo-card">
                                                    <div class="exo-label">"Project Documentation"</div>
                                                    <p class="exo-sub">
                                                        "Architecture Decision Records and product scope are available in the built-in Leptos Docs Reader."
                                                    </p>
                                                    <div style="margin-top: 14px;">
                                                        <button
                                                            type="button"
                                                            class="exo-btn exo-btn-primary"
                                                            on:click=move |_| set_active_tab.set(ActiveTab::Docs)
                                                        >
                                                            "Open Docs Reader"
                                                        </button>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span /> }.into_any()
                                        }}
                                    </main>
                                }.into_any(),
                            }}
                        </div>
                    }.into_any()
                }}
            </Show>
        </Show>
    }
}

#[component]
fn SignInView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    on_signed_in: impl Fn() + Copy + 'static,
) -> impl IntoView {
    let (is_signup, set_is_signup) = signal(false);
    let (name, set_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    let start_social = move |provider: &'static str| {
        let Some(auth_url) = auth_config.get_untracked().and_then(|c| c.auth_url) else {
            return;
        };
        set_error_msg.set(None);
        set_submitting.set(true);
        leptos::task::spawn_local(async move {
            let callback_url = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .map(|o| format!("{o}/"))
                .unwrap_or_else(|| "/".to_string());
            let endpoint = format!("{}/sign-in/social", auth_url.trim_end_matches('/'));
            let req = Request::post(&endpoint)
                .credentials(RequestCredentials::Include)
                .json(&serde_json::json!({
                    "provider": provider,
                    "callbackURL": callback_url,
                }));

            match req {
                Ok(builder) => match builder.send().await {
                    Ok(resp) if resp.ok() => {
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
                        set_error_msg.set(Some("Missing OAuth redirect URL.".to_string()));
                    }
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_default();
                        set_error_msg.set(Some(format!("Sign-in error: {text}")));
                    }
                    Err(e) => {
                        set_error_msg.set(Some(format!("Network error: {e}")));
                    }
                },
                Err(e) => {
                    set_error_msg.set(Some(format!("Request error: {e}")));
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
                let fallback_name = user_email
                    .split('@')
                    .next()
                    .unwrap_or("Author")
                    .to_string();
                (
                    format!("{base}/sign-up/email"),
                    serde_json::json!({
                        "email": user_email,
                        "password": user_pass,
                        "name": if user_name.is_empty() { fallback_name } else { user_name },
                    }),
                )
            } else {
                (
                    format!("{base}/sign-in/email"),
                    serde_json::json!({
                        "email": user_email,
                        "password": user_pass,
                    }),
                )
            };

            match Request::post(&endpoint)
                .credentials(RequestCredentials::Include)
                .json(&payload)
            {
                Ok(req) => match req.send().await {
                    Ok(resp) if resp.ok() => {
                        on_signed_in();
                        return;
                    }
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_default();
                        set_error_msg.set(Some(if text.is_empty() {
                            "Authentication failed.".to_string()
                        } else {
                            text
                        }));
                    }
                    Err(e) => set_error_msg.set(Some(format!("Network error: {e}"))),
                },
                Err(e) => set_error_msg.set(Some(format!("Request error: {e}"))),
            }
            set_submitting.set(false);
        });
    };

    view! {
        <div class="exo-auth-page">
            <div class="exo-auth-card">
                <div class="exo-eyebrow">"ExoNovel · Rust + Leptos"</div>
                <h1 class="exo-h1">"Sign in to ExoNovel"</h1>
                <p class="exo-sub">
                    "Sign in or create an account to resolve your authoritative "
                    <code>"did:peer"</code>
                    " identity."
                </p>

                <div class="exo-social-grid">
                    <button
                        type="button"
                        class="exo-btn exo-btn-google"
                        disabled=move || submitting.get()
                        on:click=move |_| start_social("google")
                    >
                        "Google"
                    </button>
                    <button
                        type="button"
                        class="exo-btn exo-btn-github"
                        disabled=move || submitting.get()
                        on:click=move |_| start_social("github")
                    >
                        "GitHub"
                    </button>
                </div>

                <div class="exo-divider">"or email"</div>

                {move || error_msg.get().map(|msg| view! {
                    <div class="exo-alert-error">{msg}</div>
                })}

                <form on:submit=submit_email>
                    <Show when=move || is_signup.get()>
                        <div class="exo-field">
                            <label class="exo-label">"Name"</label>
                            <input
                                type="text"
                                class="exo-input"
                                placeholder="Author name"
                                prop:value=move || name.get()
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                            />
                        </div>
                    </Show>

                    <div class="exo-field">
                        <label class="exo-label">"Email"</label>
                        <input
                            type="email"
                            class="exo-input"
                            required=true
                            placeholder="you@example.com"
                            prop:value=move || email.get()
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="exo-field">
                        <label class="exo-label">"Password"</label>
                        <input
                            type="password"
                            class="exo-input"
                            required=true
                            minlength="8"
                            placeholder="Minimum 8 characters"
                            prop:value=move || password.get()
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>

                    <button
                        type="submit"
                        class="exo-btn exo-btn-primary"
                        style="width: 100%; min-height: 42px;"
                        disabled=move || submitting.get()
                    >
                        {move || if submitting.get() {
                            "Authenticating..."
                        } else if is_signup.get() {
                            "Create account"
                        } else {
                            "Sign in"
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
                            "Already have an account? Sign in"
                        } else {
                            "Need an account? Create one"
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DocsReaderView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
) -> impl IntoView {
    let (tree, set_tree) = signal::<Vec<DocNode>>(Vec::new());
    let (active_path, set_active_path) = signal::<String>(String::new());
    let (active_doc, set_active_doc) = signal::<Option<DocPayload>>(None);
    let (search, set_search) = signal(String::new());
    let (sidebar_collapsed, set_sidebar_collapsed) = signal(false);
    let (sidebar_width, set_sidebar_width) = signal(290_i32);
    let (dragging, set_dragging) = signal(false);

    let resolve_token = move || {
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        let fallback_jwt = initial_jwt.get_untracked();
        async move {
            if let Some(url) = auth_url {
                if let Some(fresh) = fetch_neon_jwt(&url, None).await {
                    return Some(fresh);
                }
            }
            fallback_jwt
        }
    };

    let load_doc = move |doc_path: String| {
        if doc_path.is_empty() {
            return;
        }
        set_active_path.set(doc_path.clone());
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_token().await else {
                return;
            };
            let encoded = js_sys::encode_uri_component(&doc_path)
                .as_string()
                .unwrap_or(doc_path);
            let url = format!("/api/doc?path={encoded}");
            if let Ok(resp) = Request::get(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(payload) = resp.json::<DocPayload>().await {
                        set_active_doc.set(Some(payload));
                    }
                }
            }
        });
    };

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_token().await else {
                return;
            };
            if let Ok(resp) = Request::get("/api/docs")
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<DocsTreeResponse>().await {
                        let initial = first_doc_path(&data.tree);
                        set_tree.set(data.tree);
                        if let Some(p) = initial {
                            load_doc(p);
                        }
                    }
                }
            }
        });
    });

    let filtered_tree = move || filter_tree(&tree.get(), &search.get());

    view! {
        <div
            class="exo-docs-layout"
            on:pointermove=move |ev: web_sys::PointerEvent| {
                if dragging.get_untracked() {
                    let next = ev.client_x().max(210);
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
                        <span class="exo-label" style="margin: 0;">"Documentation"</span>
                        <button
                            type="button"
                            class="exo-btn"
                            style="min-height: 30px; padding: 0 9px; font-size: 12px;"
                            on:click=move |_| set_sidebar_collapsed.set(true)
                        >
                            "Hide"
                        </button>
                    </div>
                    <input
                        type="search"
                        class="exo-input"
                        placeholder="Filter docs..."
                        prop:value=move || search.get()
                        on:input=move |ev| set_search.set(event_target_value(&ev))
                    />
                    <nav>
                        {move || render_tree_nodes(&filtered_tree(), active_path, load_doc)}
                    </nav>
                </aside>
                <div
                    class="exo-resizer"
                    on:pointerdown=move |_| set_dragging.set(true)
                />
            </Show>

            <main class="exo-docs-main">
                <div style="flex: 1; min-width: 0;">
                    <Show when=move || sidebar_collapsed.get()>
                        <div style="margin-bottom: 14px;">
                            <button
                                type="button"
                                class="exo-btn"
                                on:click=move |_| set_sidebar_collapsed.set(false)
                            >
                                "Show Sidebar"
                            </button>
                        </div>
                    </Show>

                    {move || match active_doc.get() {
                        Some(doc) => view! {
                            <div>
                                <div class="exo-eyebrow">
                                    {doc.path.clone()} " · " {doc.reading_minutes} " min read"
                                </div>
                                <article class="exo-markdown" inner_html=doc.html.clone() />
                            </div>
                        }.into_any(),
                        None => view! {
                            <p class="exo-sub">"Select a document from the left explorer."</p>
                        }.into_any(),
                    }}
                </div>

                {move || active_doc.get().map(|doc| {
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
                                    view! {
                                        <a href=format!("#{}", h.id) class=cls>{h.text}</a>
                                    }
                                }).collect_view()}
                            </aside>
                        }.into_any()
                    }
                })}
            </main>
        </div>
    }
}

fn render_tree_nodes(
    nodes: &[DocNode],
    active_path: ReadSignal<String>,
    on_select: impl Fn(String) + Copy + 'static,
) -> AnyView {
    nodes
        .iter()
        .map(|node| {
            if node.node_type == "folder" {
                let children = node.children.clone();
                let title = node.title.clone();
                view! {
                    <div class="exo-tree-folder">
                        <div class="exo-tree-folder-btn">{title}</div>
                        <div style="padding-left: 10px;">
                            {render_tree_nodes(&children, active_path, on_select)}
                        </div>
                    </div>
                }
                .into_any()
            } else {
                let path = node.path.clone();
                let path_for_click = path.clone();
                let title = node.title.clone();
                view! {
                    <button
                        type="button"
                        class=move || if active_path.get() == path {
                            "exo-tree-file-btn exo-tree-file-active"
                        } else {
                            "exo-tree-file-btn"
                        }
                        on:click=move |_| on_select(path_for_click.clone())
                    >
                        {title}
                    </button>
                }
                .into_any()
            }
        })
        .collect_view()
        .into_any()
}

#[component]
fn AdminSettingsView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
) -> impl IntoView {
    let (entries, set_entries) = signal::<Vec<AllowlistEntry>>(Vec::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_note, set_new_note) = signal(String::new());
    let (status_msg, set_status_msg) = signal::<Option<String>>(None);

    let resolve_token = move || {
        let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
        let fallback_jwt = initial_jwt.get_untracked();
        async move {
            if let Some(url) = auth_url {
                if let Some(fresh) = fetch_neon_jwt(&url, None).await {
                    return Some(fresh);
                }
            }
            fallback_jwt
        }
    };

    let reload = move || {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_token().await else {
                return;
            };
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
        });
    };

    Effect::new(move |_| reload());

    let add_reader = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let email_val = new_email.get_untracked().trim().to_string();
        let note_val = new_note.get_untracked().trim().to_string();
        if email_val.is_empty() {
            return;
        }
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_token().await else {
                return;
            };
            if let Ok(req) = Request::post("/api/admin/allowlist")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "email": email_val,
                    "enabled": true,
                    "note": note_val,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_new_email.set(String::new());
                        set_new_note.set(String::new());
                        set_status_msg.set(Some("Reader access saved.".to_string()));
                        reload();
                    }
                }
            }
        });
    };

    let toggle_entry = move |email: String, enabled: bool| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_token().await else {
                return;
            };
            if let Ok(req) = Request::post("/api/admin/allowlist")
                .header("Authorization", &format!("Bearer {jwt}"))
                .json(&serde_json::json!({
                    "email": email,
                    "enabled": enabled,
                }))
            {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        reload();
                    }
                }
            }
        });
    };

    let remove_entry = move |email: String| {
        leptos::task::spawn_local(async move {
            let Some(jwt) = resolve_token().await else {
                return;
            };
            let encoded = js_sys::encode_uri_component(&email)
                .as_string()
                .unwrap_or(email);
            let url = format!("/api/admin/allowlist?email={encoded}");
            if let Ok(resp) = Request::delete(&url)
                .header("Authorization", &format!("Bearer {jwt}"))
                .send()
                .await
            {
                if resp.ok() {
                    reload();
                }
            }
        });
    };

    view! {
        <main class="exo-workspace-body">
            <h1 class="exo-h1">"Documentation Access Settings"</h1>
            <p class="exo-sub">
                "Control which signed-in accounts see the Docs Reader option inside ExoNovel."
            </p>

            <div class="exo-card">
                <form on:submit=add_reader style="display: flex; gap: 10px; flex-wrap: wrap;">
                    <input
                        type="email"
                        class="exo-input"
                        style="flex: 2; min-width: 220px;"
                        required=true
                        placeholder="reviewer@example.com"
                        prop:value=move || new_email.get()
                        on:input=move |ev| set_new_email.set(event_target_value(&ev))
                    />
                    <input
                        type="text"
                        class="exo-input"
                        style="flex: 2; min-width: 180px;"
                        placeholder="Optional note"
                        prop:value=move || new_note.get()
                        on:input=move |ev| set_new_note.set(event_target_value(&ev))
                    />
                    <button type="submit" class="exo-btn exo-btn-primary">
                        "Grant Access"
                    </button>
                </form>

                {move || status_msg.get().map(|m| view! {
                    <p class="exo-sub" style="margin-top: 10px;">{m}</p>
                })}
            </div>

            <div class="exo-card">
                <table class="exo-table">
                    <thead>
                        <tr>
                            <th>"Email"</th>
                            <th>"Role"</th>
                            <th>"Status"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || entries.get()
                            key=|e| format!("{}-{}", e.email, e.enabled)
                            children=move |entry| {
                                let is_admin_row = entry.role == "admin";
                                let email_toggle = entry.email.clone();
                                let email_del = entry.email.clone();
                                let next_enabled = !entry.enabled;
                                view! {
                                    <tr>
                                        <td>{entry.email}</td>
                                        <td>{entry.role}</td>
                                        <td>{if entry.enabled { "Enabled" } else { "Disabled" }}</td>
                                        <td style="display: flex; gap: 8px;">
                                            {if is_admin_row {
                                                view! { <span class="exo-sub">"Owner"</span> }.into_any()
                                            } else {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="exo-btn"
                                                        on:click=move |_| toggle_entry(email_toggle.clone(), next_enabled)
                                                    >
                                                        {if next_enabled { "Enable" } else { "Disable" }}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="exo-btn"
                                                        on:click=move |_| remove_entry(email_del.clone())
                                                    >
                                                        "Remove"
                                                    </button>
                                                }.into_any()
                                            }}
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </main>
    }
}
