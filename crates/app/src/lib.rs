use std::collections::HashSet;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityRecord {
    pub did_peer: String,
    pub email: Option<String>,
    pub auth_user_id: String,
    pub provider: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct IdentitiesResponse {
    identities: Vec<IdentityRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveTab {
    Workspace,
    Docs,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppearanceMode {
    System,
    Light,
    Dark,
}

impl AppearanceMode {
    fn as_str(self) -> &'static str {
        match self {
            AppearanceMode::System => "system",
            AppearanceMode::Light => "light",
            AppearanceMode::Dark => "dark",
        }
    }

    fn label(self) -> &'static str {
        match self {
            AppearanceMode::System => "System theme",
            AppearanceMode::Light => "Light theme",
            AppearanceMode::Dark => "Dark theme",
        }
    }

    fn next(self) -> Self {
        match self {
            AppearanceMode::System => AppearanceMode::Light,
            AppearanceMode::Light => AppearanceMode::Dark,
            AppearanceMode::Dark => AppearanceMode::System,
        }
    }

    fn from_str(raw: &str) -> Self {
        match raw {
            "light" => AppearanceMode::Light,
            "dark" => AppearanceMode::Dark,
            _ => AppearanceMode::System,
        }
    }
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

async fn load_authenticated_user(
    auth_url: &str,
    verifier: Option<&str>,
) -> Option<(VerifiedUser, String)> {
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

fn flatten_files(nodes: &[DocNode]) -> Vec<DocNode> {
    let mut result = Vec::new();
    for node in nodes {
        if node.node_type == "file" {
            result.push(node.clone());
        } else {
            result.extend(flatten_files(&node.children));
        }
    }
    result
}

fn count_files(nodes: &[DocNode]) -> usize {
    nodes
        .iter()
        .map(|n| {
            if n.node_type == "file" {
                1
            } else {
                count_files(&n.children)
            }
        })
        .sum()
}

fn first_doc_path(nodes: &[DocNode]) -> Option<String> {
    let files = flatten_files(nodes);
    files
        .iter()
        .find(|n| n.path != "AGENTS.md")
        .or_else(|| files.first())
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

fn read_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

fn write_storage(key: &str, value: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(key, value);
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
    let (appearance, set_appearance) = signal(AppearanceMode::System);
    let (wide_mode, set_wide_mode) = signal(false);

    Effect::new(move |_| {
        if let Some(saved_app) = read_storage("mu_appearance") {
            set_appearance.set(AppearanceMode::from_str(&saved_app));
        }
        if let Some(saved_w) = read_storage("mu_width") {
            set_wide_mode.set(saved_w == "wide");
        }
        if web_sys::window()
            .and_then(|w| w.location().search().ok())
            .and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
            .and_then(|p| p.get("doc"))
            .is_some_and(|d| !d.trim().is_empty())
        {
            set_active_tab.set(ActiveTab::Docs);
        }
    });

    Effect::new(move |_| {
        let mode = appearance.get();
        let width_val = if wide_mode.get() { "wide" } else { "normal" };
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let _ = root.set_attribute("data-appearance", mode.as_str());
            let _ = root.set_attribute("data-width", width_val);
        }
        write_storage("mu_appearance", mode.as_str());
        write_storage("mu_width", width_val);
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

    let sign_out = move || {
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
        <div class="exo-shell">
            <nav class="navbar-solid">
                <a
                    href="https://exosystems.net/"
                    title="ExoSystems.net"
                    class="navbar-brand-link"
                >
                    <img src="/exosys.png" alt="ExoSystems" class="navbar-logo" />
                    <span class="navbar-subbrand">"ExoNovel"</span>
                </a>

                <div class="navbar-menu">
                    <a href="https://exosystems.net/" class="nav-link">"Home"</a>

                    <button
                        type="button"
                        class=move || if active_tab.get() == ActiveTab::Workspace {
                            "nav-link nav-link-active"
                        } else {
                            "nav-link"
                        }
                        on:click=move |_| set_active_tab.set(ActiveTab::Workspace)
                    >
                        "ExoNovel"
                    </button>

                    {move || match current_user.get() {
                        Some(user) => {
                            let can_read_docs = user.authorized;
                            let is_admin = user.is_admin;
                            view! {
                                <>
                                    {if can_read_docs {
                                        view! {
                                            <button
                                                type="button"
                                                class=move || if active_tab.get() == ActiveTab::Docs {
                                                    "nav-link nav-link-active"
                                                } else {
                                                    "nav-link"
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
                                                title="Settings"
                                                aria-label="Settings"
                                                class=move || if active_tab.get() == ActiveTab::Settings {
                                                    "ctl active"
                                                } else {
                                                    "ctl"
                                                }
                                                on:click=move |_| set_active_tab.set(ActiveTab::Settings)
                                            >
                                                <SettingsIcon />
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span /> }.into_any()
                                    }}

                                    <button
                                        type="button"
                                        class="nav-link"
                                        on:click=move |_| sign_out()
                                    >
                                        "Sign out"
                                    </button>
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
                            on:click=move |_| set_appearance.update(|m| *m = m.next())
                        >
                            {move || match appearance.get() {
                                AppearanceMode::Light => view! { <SunIcon /> }.into_any(),
                                AppearanceMode::Dark => view! { <MoonIcon /> }.into_any(),
                                AppearanceMode::System => view! { <MonitorIcon /> }.into_any(),
                            }}
                        </button>

                        <button
                            type="button"
                            class=move || if wide_mode.get() { "ctl active" } else { "ctl" }
                            title=move || if wide_mode.get() { "Normal width" } else { "Full width" }
                            aria-label=move || if wide_mode.get() { "Normal width" } else { "Full width" }
                            on:click=move |_| set_wide_mode.update(|w| *w = !*w)
                        >
                            <WidthIcon wide=wide_mode />
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
                {move || match current_user.get() {
                    None => view! {
                        <SignInView
                            auth_config=auth_config
                            on_signed_in=refresh_session
                        />
                    }.into_any(),
                    Some(user) if !user.authorized => view! {
                        <AccessDeniedView
                            email=user.email.unwrap_or_else(|| "Signed-in account".to_string())
                            did_peer=user.did_peer
                            on_sign_out=sign_out
                        />
                    }.into_any(),
                    Some(user) => {
                        let is_admin = user.is_admin;
                        let email_label = user.email.clone().unwrap_or_else(|| "Signed-in writer".to_string());
                        let did_label = user.did_peer.clone().unwrap_or_else(|| "Pending did:peer".to_string());

                        match active_tab.get() {
                            ActiveTab::Docs => view! {
                                <DocsReaderView
                                    auth_config=auth_config
                                    initial_jwt=session_jwt
                                />
                            }.into_any(),
                            ActiveTab::Settings if is_admin => view! {
                                <AdminSettingsView
                                    auth_config=auth_config
                                    initial_jwt=session_jwt
                                    current_user_email=email_label
                                    current_user_did=did_label
                                    on_back_to_docs=move || set_active_tab.set(ActiveTab::Docs)
                                />
                            }.into_any(),
                            _ => view! {
                                <main class="page-main">
                                    <header class="page-head">
                                        <h1>"ExoNovel."</h1>
                                        <p>
                                            "Signed in as " <strong>{email_label}</strong>
                                        </p>
                                    </header>

                                    <div class="exo-card">
                                        <div class="exo-label">"Authoritative Ownership Key (did:peer)"</div>
                                        <div class="exo-mono">{did_label}</div>
                                    </div>

                                    <div class="exo-card">
                                        <h2 class="exo-h2">"Documentation"</h2>
                                        <p class="exo-sub">
                                            "Architecture Decision Records and product specifications live directly inside ExoNovel's built-in Rust + Leptos Docs Reader."
                                        </p>
                                        <div style="margin-top: 16px; display: flex; gap: 10px; flex-wrap: wrap;">
                                            <button
                                                type="button"
                                                class="exo-btn exo-btn-primary"
                                                on:click=move |_| set_active_tab.set(ActiveTab::Docs)
                                            >
                                                "Open Docs Reader"
                                            </button>
                                            {if is_admin {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="exo-btn"
                                                        on:click=move |_| set_active_tab.set(ActiveTab::Settings)
                                                    >
                                                        "Admin Settings"
                                                    </button>
                                                }.into_any()
                                            } else {
                                                view! { <span /> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                </main>
                            }.into_any(),
                        }
                    }
                }}
            </Show>
        </div>
    }
}

#[component]
fn AccessDeniedView(
    email: String,
    did_peer: Option<String>,
    on_sign_out: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <main class="page-main">
            <div class="exo-auth-wrap">
                <div class="exo-card">
                    <div class="exo-eyebrow">"ExoNovel · Private Documentation"</div>
                    <h1 class="exo-h1">"Access not granted"</h1>
                    <p class="exo-sub">
                        "Signed in as " <strong>{email}</strong>
                        ". ExoNovel documentation is private and requires administrator approval."
                    </p>

                    {did_peer.map(|did| view! {
                        <div style="margin-top: 16px; padding: 12px; border-radius: 8px; border: 1px solid var(--border); background: var(--surface-muted);">
                            <div class="exo-label">"Authoritative did:peer"</div>
                            <div class="exo-mono">{did}</div>
                        </div>
                    })}

                    <div style="margin-top: 20px;">
                        <button
                            type="button"
                            class="exo-btn"
                            style="width: 100%;"
                            on:click=move |_| on_sign_out()
                        >
                            "Sign out"
                        </button>
                    </div>
                </div>
            </div>
        </main>
    }
}

#[component]
fn SignInView(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    on_signed_in: impl Fn() + Copy + Send + Sync + 'static,
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
        <main class="page-main">
            <header class="page-head">
                <h1>"Sign in."</h1>
                <p>
                    "Sign in to ExoNovel to resolve your authoritative "
                    <code>"did:peer"</code>
                    " identity and access documentation."
                </p>
            </header>

            <div class="exo-auth-wrap" style="margin-inline: 0;">
                <div class="exo-card">
                    <div class="exo-eyebrow">"ExoNovel · ExoSystems"</div>
                    <h2 class="exo-h2">"Sign in to continue"</h2>

                    <div class="exo-social-grid">
                        <button
                            type="button"
                            class="exo-btn exo-btn-google"
                            disabled=move || submitting.get()
                            on:click=move |_| start_social("google")
                        >
                            <GoogleLogoIcon />
                            <span>"Google"</span>
                        </button>
                        <button
                            type="button"
                            class="exo-btn exo-btn-github"
                            disabled=move || submitting.get()
                            on:click=move |_| start_social("github")
                        >
                            <GitHubLogoIcon />
                            <span>"GitHub"</span>
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
        </main>
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
    let (sidebar_width, set_sidebar_width) = signal(280_i32);
    let (dragging, set_dragging) = signal(false);
    let (collapsed_folders, set_collapsed_folders) = signal::<HashSet<String>>(HashSet::new());

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
        if let Some(win) = web_sys::window() {
            if let Ok(search) = win.location().search() {
                if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
                    params.set("doc", &doc_path);
                    let query = params.to_string().as_string().unwrap_or_default();
                    let pathname = win.location().pathname().unwrap_or_else(|_| "/".to_string());
                    let next_url = if query.is_empty() {
                        pathname
                    } else {
                        format!("{pathname}?{query}")
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

    let reload_all = move || {
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
                        let current = active_path.get_untracked();
                        let from_url = if current.is_empty() {
                            web_sys::window()
                                .and_then(|w| w.location().search().ok())
                                .and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
                                .and_then(|p| p.get("doc"))
                                .filter(|d| flatten_files(&data.tree).iter().any(|n| n.path == *d))
                        } else {
                            None
                        };
                        let initial = if !current.is_empty() {
                            Some(current)
                        } else if from_url.is_some() {
                            from_url
                        } else {
                            first_doc_path(&data.tree)
                        };
                        set_tree.set(data.tree);
                        if let Some(p) = initial {
                            load_doc(p);
                        }
                    }
                }
            }
        });
    };

    Effect::new(move |_| reload_all());

    let filtered_tree = move || filter_tree(&tree.get(), &search.get());
    let total_files = move || count_files(&tree.get());

    let toggle_folder = move |path: String| {
        set_collapsed_folders.update(|set| {
            if !set.remove(&path) {
                set.insert(path);
            }
        });
    };

    let flat_files = move || flatten_files(&tree.get());

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
                        <div>
                            <div class="exo-label" style="margin: 0;">"Documentation"</div>
                            <div class="exo-eyebrow" style="margin: 2px 0 0;">
                                {move || format!("{} files", total_files())}
                            </div>
                        </div>
                        <button
                            type="button"
                            class="exo-btn"
                            style="min-height: 30px; padding: 0 9px; font-size: 12px;"
                            on:click=move |_| set_sidebar_collapsed.set(true)
                        >
                            "Hide"
                        </button>
                    </div>

                    <div style="display: flex; gap: 6px; align-items: center;">
                        <input
                            type="search"
                            class="exo-input"
                            placeholder="Filter docs..."
                            prop:value=move || search.get()
                            on:input=move |ev| set_search.set(event_target_value(&ev))
                        />
                        <Show when=move || !search.get().is_empty()>
                            <button
                                type="button"
                                class="exo-btn"
                                style="min-height: 34px; padding: 0 9px;"
                                on:click=move |_| set_search.set(String::new())
                            >
                                "Clear"
                            </button>
                        </Show>
                    </div>

                    <nav>
                        {move || render_tree_nodes(
                            &filtered_tree(),
                            active_path,
                            collapsed_folders,
                            !search.get().trim().is_empty(),
                            toggle_folder,
                            load_doc,
                        )}
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
                        <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                            <Show when=move || sidebar_collapsed.get()>
                                <button
                                    type="button"
                                    class="exo-btn"
                                    style="min-height: 32px; padding: 0 10px; font-size: 12px;"
                                    on:click=move |_| set_sidebar_collapsed.set(false)
                                >
                                    "Show Sidebar"
                                </button>
                            </Show>
                            <span class="exo-eyebrow" style="margin: 0;">
                                {move || active_path.get()}
                            </span>
                        </div>
                        <button
                            type="button"
                            class="exo-btn"
                            style="min-height: 32px; padding: 0 11px; font-size: 12px;"
                            on:click=move |_| reload_all()
                        >
                            "Refresh"
                        </button>
                    </div>

                    {move || match active_doc.get() {
                        Some(doc) => {
                            let files = flat_files();
                            let idx = files.iter().position(|f| f.path == doc.path);
                            let prev_doc = idx.and_then(|i| if i > 0 { files.get(i - 1).cloned() } else { None });
                            let next_doc = idx.and_then(|i| files.get(i + 1).cloned());

                            view! {
                                <div>
                                    <div class="exo-eyebrow">
                                        {doc.path.clone()} " · " {doc.word_count} " words · " {doc.reading_minutes} " min read"
                                    </div>
                                    <article class="exo-markdown" inner_html=doc.html.clone() />

                                    {(prev_doc.is_some() || next_doc.is_some()).then(|| {
                                        view! {
                                            <div class="exo-prev-next">
                                                {match prev_doc {
                                                    Some(p) => {
                                                        let path_click = p.path.clone();
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="exo-btn"
                                                                style="flex-direction: column; align-items: flex-start; padding: 12px 14px; min-height: 58px;"
                                                                on:click=move |_| load_doc(path_click.clone())
                                                            >
                                                                <span class="exo-eyebrow" style="margin: 0;">"Previous"</span>
                                                                <span>{p.title}</span>
                                                            </button>
                                                        }.into_any()
                                                    }
                                                    None => view! { <div /> }.into_any(),
                                                }}
                                                {match next_doc {
                                                    Some(n) => {
                                                        let path_click = n.path.clone();
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="exo-btn"
                                                                style="flex-direction: column; align-items: flex-end; padding: 12px 14px; min-height: 58px;"
                                                                on:click=move |_| load_doc(path_click.clone())
                                                            >
                                                                <span class="exo-eyebrow" style="margin: 0;">"Next"</span>
                                                                <span>{n.title}</span>
                                                            </button>
                                                        }.into_any()
                                                    }
                                                    None => view! { <div /> }.into_any(),
                                                }}
                                            </div>
                                        }
                                    })}
                                </div>
                            }.into_any()
                        }
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
    collapsed_folders: ReadSignal<HashSet<String>>,
    is_searching: bool,
    on_toggle_folder: impl Fn(String) + Copy + Send + Sync + 'static,
    on_select: impl Fn(String) + Copy + Send + Sync + 'static,
) -> AnyView {
    nodes
        .iter()
        .map(|node| {
            if node.node_type == "folder" {
                let children = node.children.clone();
                let title = node.title.clone();
                let folder_path = node.path.clone();
                let folder_path_arrow = node.path.clone();
                let folder_path_show = node.path.clone();
                let file_count = count_files(&children);

                view! {
                    <div class="exo-tree-folder">
                        <button
                            type="button"
                            class="exo-tree-folder-btn"
                            on:click=move |_| on_toggle_folder(folder_path.clone())
                        >
                            <span>
                                {move || if is_searching || !collapsed_folders.get().contains(&folder_path_arrow) {
                                    "▾ "
                                } else {
                                    "▸ "
                                }}
                                {title.clone()}
                            </span>
                            <span class="exo-eyebrow" style="margin: 0;">{file_count}</span>
                        </button>
                        <Show when=move || is_searching || !collapsed_folders.get().contains(&folder_path_show)>
                            <div style="padding-left: 10px;">
                                {render_tree_nodes(
                                    &children,
                                    active_path,
                                    collapsed_folders,
                                    is_searching,
                                    on_toggle_folder,
                                    on_select,
                                )}
                            </div>
                        </Show>
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
    current_user_email: String,
    current_user_did: String,
    on_back_to_docs: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let (entries, set_entries) = signal::<Vec<AllowlistEntry>>(Vec::new());
    let (identities, set_identities) = signal::<Vec<IdentityRecord>>(Vec::new());
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
                    } else {
                        let msg = resp.text().await.unwrap_or_else(|_| "Failed to save reader access.".to_string());
                        set_status_msg.set(Some(msg));
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
                    } else {
                        let msg = resp.text().await.unwrap_or_else(|_| "Failed to update allowlist.".to_string());
                        set_status_msg.set(Some(msg));
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
                } else {
                    let msg = resp.text().await.unwrap_or_else(|_| "Failed to remove allowlist entry.".to_string());
                    set_status_msg.set(Some(msg));
                }
            }
        });
    };

    view! {
        <main class="page-main">
            <header class="page-head" style="display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; flex-wrap: wrap;">
                <div>
                    <h1>"Admin Settings."</h1>
                    <p>
                        "Manage documentation access and inspect resolved "
                        <code>"did:peer"</code>
                        " identities."
                    </p>
                </div>
                <button type="button" class="exo-btn" on:click=move |_| on_back_to_docs()>
                    "Back to Docs"
                </button>
            </header>

            <div class="exo-card">
                <div class="exo-label">
                    "Signed in as " {current_user_email} " · Authoritative Owner ID"
                </div>
                <div class="exo-mono">{current_user_did}</div>
            </div>

            <div class="exo-card">
                <h2 class="exo-h2">"Documentation Access Allowlist"</h2>
                <form on:submit=add_reader style="display: flex; gap: 10px; flex-wrap: wrap; margin-top: 12px;">
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

                <table class="exo-table" style="margin-top: 16px;">
                    <thead>
                        <tr>
                            <th>"Email"</th>
                            <th>"Role"</th>
                            <th>"Status"</th>
                            <th>"Note"</th>
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
                                        <td>{if entry.note.is_empty() { "—".to_string() } else { entry.note }}</td>
                                        <td style="display: flex; gap: 8px;">
                                            {if is_admin_row {
                                                view! { <span class="exo-sub">"Primary Admin"</span> }.into_any()
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

            <div class="exo-card">
                <h2 class="exo-h2">"Resolved did:peer Identities"</h2>
                <table class="exo-table" style="margin-top: 12px;">
                    <thead>
                        <tr>
                            <th>"Email"</th>
                            <th>"did:peer"</th>
                            <th>"Provider"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || identities.get()
                            key=|id| id.did_peer.clone()
                            children=move |id| {
                                view! {
                                    <tr>
                                        <td>{id.email.unwrap_or_else(|| "—".to_string())}</td>
                                        <td class="exo-mono">{id.did_peer}</td>
                                        <td>{id.provider}</td>
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

#[component]
fn GoogleLogoIcon() -> impl IntoView {
    view! {
        <svg width="16" height="16" viewBox="0 0 48 48" aria-hidden="true">
            <path
                fill="#EA4335"
                d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"
            />
            <path
                fill="#4285F4"
                d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"
            />
            <path
                fill="#FBBC05"
                d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"
            />
            <path
                fill="#34A853"
                d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"
            />
        </svg>
    }
}

#[component]
fn GitHubLogoIcon() -> impl IntoView {
    view! {
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
            />
        </svg>
    }
}

#[component]
fn SunIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="4"/>
            <path d="M12 2v2"/><path d="M12 20v2"/>
            <path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/>
            <path d="M2 12h2"/><path d="M20 12h2"/>
            <path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>
        </svg>
    }
}

#[component]
fn MoonIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"/>
        </svg>
    }
}

#[component]
fn MonitorIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect width="20" height="14" x="2" y="3" rx="2"/>
            <line x1="8" x2="16" y1="21" y2="21"/>
            <line x1="12" x2="12" y1="17" y2="21"/>
        </svg>
    }
}

#[component]
fn WidthIcon(wide: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            {move || if wide.get() {
                view! {
                    <>
                        <path d="M8 3v3a2 2 0 0 1-2 2H3"/>
                        <path d="M21 8h-3a2 2 0 0 1-2-2V3"/>
                        <path d="M3 16h3a2 2 0 0 1 2 2v3"/>
                        <path d="M16 21v-3a2 2 0 0 1 2-2h3"/>
                    </>
                }.into_any()
            } else {
                view! {
                    <>
                        <path d="M8 3H5a2 2 0 0 0-2 2v3"/>
                        <path d="M21 8V5a2 2 0 0 0-2-2h-3"/>
                        <path d="M3 16v3a2 2 0 0 0 2 2h3"/>
                        <path d="M16 21h3a2 2 0 0 0 2-2v-3"/>
                    </>
                }.into_any()
            }}
        </svg>
    }
}

#[component]
fn SettingsIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"/>
            <circle cx="12" cy="12" r="3"/>
        </svg>
    }
}
