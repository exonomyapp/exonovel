use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfigResponse {
    pub auth_configured: bool,
    pub auth_url: Option<String>,
    pub admin_email: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub did_peer: String,
    pub active_persona_did: String,
    pub display_name: String,
    pub pen_name: String,
    pub bio: String,
    pub avatar_asset_id: String,
    pub avatar_url: String,
    pub avatar_thumb_url: String,
    pub website: String,
    pub location: String,
    pub locale: String,
    pub timezone: String,
    pub preferred_format: String,
    pub appearance_mode: String,
    pub ui_locale: String,
    pub default_input_locale: String,
    pub preferred_ui_locales: Vec<String>,
    pub preferred_input_locales: Vec<String>,
    pub sync_plan_tier: String,
    pub willow_sync_enabled: bool,
    pub last_active_tab: String,
    pub last_doc_path: String,
    pub provider_given_name: String,
    pub provider_family_name: String,
    pub provider_account_id: String,
    pub provider_name: String,
    pub provider_scopes: String,
    pub email_verified: bool,
    pub auth_created_at: Option<String>,
    pub last_sign_in_at: String,
    pub last_sign_in_ip: String,
    pub last_user_agent: String,
    pub provider_raw_json: serde_json::Value,
    pub willow_namespace: String,
    pub willow_subspace: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPersona {
    pub persona_did: String,
    pub owner_did: String,
    pub public_key: String,
    pub is_default: bool,
    pub display_name: String,
    pub handle: String,
    pub bio: String,
    pub avatar_asset_id: String,
    pub avatar_url: String,
    pub avatar_thumb_url: String,
    pub willow_namespace: String,
    pub willow_subspace: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifiedUser {
    pub auth_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub did_peer: Option<String>,
    pub account_role: String,
    pub account_status: String,
    pub authorized: bool,
    pub is_admin: bool,
    pub profile: Option<UserProfile>,
    pub personas: Vec<UserPersona>,
    pub active_persona: Option<UserPersona>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeResponse {
    pub authenticated: bool,
    pub user: VerifiedUser,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePatchResponse {
    pub profile: UserProfile,
    pub personas: Vec<UserPersona>,
    pub active_persona: Option<UserPersona>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonasResponse {
    pub personas: Vec<UserPersona>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvatarUploadResponse {
    pub asset_id: String,
    pub thumb_url: String,
    pub medium_url: String,
    pub large_url: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberView {
    pub member_owner_did: String,
    pub member_persona_did: String,
    pub display_name: String,
    pub avatar_thumb_url: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupView {
    pub group_id: String,
    pub owner_did: String,
    pub created_by_persona_did: String,
    pub name: String,
    pub description: String,
    pub my_role: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
    pub created_at: String,
    pub members: Vec<GroupMemberView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupsResponse {
    pub groups: Vec<GroupView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteView {
    pub code: String,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub target_group_role: String,
    pub note: String,
    pub use_count: i32,
    pub active: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralView {
    pub referred_owner_did: String,
    pub referred_persona_did: String,
    pub display_name: String,
    pub avatar_thumb_url: String,
    pub invite_code: String,
    pub group_name: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitesResponse {
    pub invites: Vec<InviteView>,
    pub referrals: Vec<ReferralView>,
    pub referred_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteResponse {
    pub invite: InviteView,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitePreview {
    pub code: String,
    pub valid: bool,
    pub inviter_display_name: String,
    pub inviter_avatar_thumb_url: String,
    pub group_name: Option<String>,
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
pub struct AdrToolDependencyView {
    pub tool_key: String,
    pub tool_name: String,
    pub installed_version: String,
    pub latest_version: String,
    pub required_constraint: String,
    pub blocking_capability: String,
    pub is_satisfied: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdrRecord {
    pub id: i32,
    pub adr_number: Option<String>,
    pub parent_adr_id: Option<i32>,
    pub doc_kind: String,
    pub title: String,
    pub slug: String,
    pub rel_path: String,
    pub status: String,
    pub superseded_by_id: Option<i32>,
    pub context_problem: String,
    pub decision_drivers: String,
    pub considered_options: serde_json::Value,
    pub decision_outcome: String,
    pub consequences_pros_cons: String,
    pub confirmation_criteria: String,
    pub decision_makers: String,
    pub consulted: String,
    pub informed: String,
    pub raw_markdown: String,
    pub html: String,
    pub word_count: usize,
    pub reading_minutes: usize,
    pub headings: Vec<DocHeading>,
    pub author_did: String,
    pub dependencies: Vec<AdrToolDependencyView>,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub translation_status: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmsAdrsResponse {
    pub adrs: Vec<AdrRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogItem {
    pub tool_key: String,
    pub name: String,
    pub ecosystem: String,
    pub package_name: String,
    pub repo_url: String,
    pub installed_version: String,
    pub latest_version: String,
    pub release_notes_url: String,
    pub purpose: String,
    pub last_checked_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaturityNoteItem {
    pub id: i32,
    pub adr_id: Option<i32>,
    pub adr_title: Option<String>,
    pub tool_key: String,
    pub target_version: String,
    pub note_kind: String,
    pub title: String,
    pub rationale: String,
    pub proposed_feature_changes: String,
    pub provider_used: String,
    pub model_used: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelInfo {
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    pub is_free: bool,
    pub supports_tools: bool,
    pub is_active: bool,
    pub key_configured: bool,
    pub cooldown_until: Option<String>,
    pub deprecated_at: Option<String>,
    pub last_verified_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmsToolingResponse {
    pub tools: Vec<ToolCatalogItem>,
    pub notes: Vec<MaturityNoteItem>,
    pub provider_models: Vec<ProviderModelInfo>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerExternalTask {
    pub task_key: String,
    pub category: String,
    pub owner_actor: String,
    pub title: String,
    pub step_by_step: String,
    pub copy_snippet: String,
    pub is_done: bool,
    pub completed_by: String,
    pub completed_at: Option<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CmsTasksResponse {
    pub tasks: Vec<OwnerExternalTask>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaleDescriptor {
    pub code: String,
    pub native_name: String,
    pub english_name: String,
    pub dir: String,
    pub region: String,
    pub pack_url: String,
    pub willow_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nCatalogResponse {
    pub count: usize,
    pub public_willow_namespace: String,
    pub locales: Vec<LocaleDescriptor>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferences {
    pub did_peer: String,
    pub in_app_enabled: bool,
    pub web_push_enabled: bool,
    pub native_push_enabled: bool,
    pub email_enabled: bool,
    pub email_digest_mode: String,
    pub notify_author_earnings: bool,
    pub notify_reader_purchases: bool,
    pub notify_group_invites: bool,
    pub notify_referral_signups: bool,
    pub notify_moderator_comments: bool,
    pub notify_collaborator_comments: bool,
    pub notify_mentions_and_forks: bool,
    pub notify_security_signins: bool,
    pub notify_maturity_releases: bool,
    pub notify_maturity_unblocked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDevice {
    pub device_id: String,
    pub platform: String,
    pub transport: String,
    pub endpoint_or_token: String,
    pub device_label: String,
    pub last_seen_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    pub id: String,
    pub recipient_did: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub action_url: String,
    pub is_read: bool,
    pub email_sent: bool,
    pub willow_namespace: String,
    pub willow_subspace: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsListResponse {
    pub notifications: Vec<NotificationItem>,
    pub unread_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPrefsResponse {
    pub preferences: NotificationPreferences,
    pub devices: Vec<NotificationDevice>,
    pub brevo_configured: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingLedgerItem {
    pub id: String,
    pub kind: String,
    pub currency: String,
    pub amount_cents: i64,
    pub billing_cycle: String,
    pub status: String,
    pub counterparty_did: String,
    pub note: String,
    pub willow_path: String,
    pub willow_timestamp_us: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingResponse {
    pub entries: Vec<BillingLedgerItem>,
    pub exonovel_sync_peer_did: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePatchEvent {
    pub seq: u64,
    pub scope_did: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub op: String,
    pub changed_fields: Vec<String>,
    pub payload: serde_json::Value,
    pub origin: String,
    pub emitted_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowlistEntry {
    pub did_peer: Option<String>,
    pub display_name: String,
    pub email: String,
    pub status: String,
    pub enabled: bool,
    pub role: String,
    pub note: String,
    pub added_by: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AllowlistResponse {
    pub entries: Vec<AllowlistEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityRecord {
    pub did_peer: String,
    pub public_key: String,
    pub email: Option<String>,
    pub auth_user_id: String,
    pub provider: String,
    pub account_role: String,
    pub account_status: String,
    pub created_at: String,
    pub profile: UserProfile,
    pub docs_status: String,
    pub docs_allowed: bool,
    pub docs_role: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdentitiesResponse {
    pub identities: Vec<IdentityRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveTab {
    Workspace,
    Docs,
    Profile,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileTab {
    Overview,
    Works,
    Groups,
    Wallet,
    Personas,
    LanguagesAndAlerts,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoAccountSummary {
    pub provider_id: String,
    pub account_id: String,
    pub scopes: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppItem {
    pub provider: String,
    pub status: String,
    pub provider_email: String,
    pub provider_account_id: String,
    pub scopes: String,
    pub connected_at: String,
    pub revoked_at: Option<String>,
    pub willow_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAppsResponse {
    pub sso_accounts: Vec<SsoAccountSummary>,
    pub connected_apps: Vec<ConnectedAppItem>,
    pub oauth_configured: bool,
    pub redirect_uri: String,
    pub recommended_scopes: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFileItem {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub modified_time: Option<String>,
    pub size: Option<String>,
    pub web_view_link: Option<String>,
    pub trashed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFilesResponse {
    pub files: Vec<DriveFileItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFolderCreateResponse {
    pub folder: DriveFileItem,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveContentResponse {
    pub id: String,
    pub content: String,
    pub exported_mime_type: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveSaveResponse {
    pub file: DriveFileItem,
    pub action: String,
    pub folder: Option<DriveFileItem>,
    pub format: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveBackupResponse {
    pub archive_folder: DriveFileItem,
    pub created_count: usize,
    pub updated_count: usize,
    pub total: usize,
    pub format: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSection {
    Account,
    ConnectedApps,
    AppearanceAndLanguages,
    SyncAndStorage,
    NotificationChannels,
    AdminDocsAndMaturity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppearanceMode {
    System,
    Light,
    Dark,
}

impl AppearanceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            AppearanceMode::System => "system",
            AppearanceMode::Light => "light",
            AppearanceMode::Dark => "dark",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AppearanceMode::System => "System theme",
            AppearanceMode::Light => "Light theme",
            AppearanceMode::Dark => "Dark theme",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            AppearanceMode::System => "theme_system_label",
            AppearanceMode::Light => "theme_light_label",
            AppearanceMode::Dark => "theme_dark_label",
        }
    }

    pub fn next(self) -> Self {
        match self {
            AppearanceMode::System => AppearanceMode::Light,
            AppearanceMode::Light => AppearanceMode::Dark,
            AppearanceMode::Dark => AppearanceMode::System,
        }
    }

    pub fn from_str(raw: &str) -> Self {
        match raw {
            "light" => AppearanceMode::Light,
            "dark" => AppearanceMode::Dark,
            _ => AppearanceMode::System,
        }
    }
}

pub async fn fetch_neon_jwt(auth_url: &str, verifier: Option<&str>) -> Option<String> {
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

pub async fn resolve_fresh_jwt(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
) -> Option<String> {
    let auth_url = auth_config.get_untracked().and_then(|c| c.auth_url);
    let fallback_jwt = initial_jwt.get_untracked();
    if let Some(url) = auth_url {
        if let Some(fresh) = fetch_neon_jwt(&url, None).await {
            return Some(fresh);
        }
    }
    fallback_jwt
}

pub async fn load_authenticated_user(
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

pub async fn save_profile_patch(
    auth_url: Option<&str>,
    fallback_jwt: Option<&str>,
    patch: serde_json::Value,
) -> Result<ProfilePatchResponse, String> {
    let fresh = match auth_url {
        Some(url) => fetch_neon_jwt(url, None).await,
        None => None,
    };
    let Some(jwt) = fresh.or_else(|| fallback_jwt.map(str::to_string)) else {
        return Err("Missing session token.".to_string());
    };
    let req = Request::put("/api/profile")
        .header("Authorization", &format!("Bearer {jwt}"))
        .json(&patch)
        .map_err(|e| e.to_string())?;
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        let msg = resp
            .text()
            .await
            .unwrap_or_else(|_| "Failed to save profile.".to_string());
        return Err(msg);
    }
    resp.json::<ProfilePatchResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn upload_avatar_file(
    auth_url: Option<&str>,
    fallback_jwt: Option<&str>,
    data_url: String,
    persona_did: Option<String>,
) -> Result<AvatarUploadResponse, String> {
    let fresh = match auth_url {
        Some(url) => fetch_neon_jwt(url, None).await,
        None => None,
    };
    let Some(jwt) = fresh.or_else(|| fallback_jwt.map(str::to_string)) else {
        return Err("Missing session token.".to_string());
    };
    let req = Request::post("/api/media/avatar")
        .header("Authorization", &format!("Bearer {jwt}"))
        .json(&serde_json::json!({
            "dataUrl": data_url,
            "personaDid": persona_did,
        }))
        .map_err(|e| e.to_string())?;
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        let msg = resp
            .text()
            .await
            .unwrap_or_else(|_| "Avatar upload failed.".to_string());
        return Err(msg);
    }
    resp.json::<AvatarUploadResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub fn extract_api_error_message(raw: &str, fallback: &str) -> String {
    #[derive(Deserialize)]
    struct ErrObj {
        error: Option<String>,
    }
    if let Ok(parsed) = serde_json::from_str::<ErrObj>(raw) {
        if let Some(msg) = parsed.error {
            if !msg.trim().is_empty() {
                return msg;
            }
        }
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

pub async fn start_google_connect_flow(
    auth_config: ReadSignal<Option<AuthConfigResponse>>,
    initial_jwt: ReadSignal<Option<String>>,
    return_tab: &str,
    return_doc_path: Option<&str>,
) -> Result<(), String> {
    let Some(jwt) = resolve_fresh_jwt(auth_config, initial_jwt).await else {
        return Err("Missing active session token. Please sign in again.".to_string());
    };
    let req = Request::post("/api/oauth/google/start")
        .header("Authorization", &format!("Bearer {jwt}"))
        .json(&serde_json::json!({
            "returnTab": return_tab,
            "returnDocPath": return_doc_path,
        }))
        .map_err(|e| e.to_string())?;
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(extract_api_error_message(
            &txt,
            "Failed to start Google authorization.",
        ));
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct StartRes {
        auth_url: String,
    }
    let parsed = resp.json::<StartRes>().await.map_err(|e| e.to_string())?;
    if let Some(win) = web_sys::window() {
        let _ = win.location().assign(&parsed.auth_url);
    }
    Ok(())
}

// Pre-auth temporary invite persistence uses sessionStorage only (zero localStorage across the app)
pub fn read_session_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.session_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

pub fn write_session_storage(key: &str, value: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.session_storage().ok().flatten()) {
        let _ = storage.set_item(key, value);
    }
}

pub fn remove_session_storage(key: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.session_storage().ok().flatten()) {
        let _ = storage.remove_item(key);
    }
}

pub fn copy_text_to_clipboard(text: &str) {
    if let Some(win) = web_sys::window() {
        let nav = win.navigator();
        if let Ok(clip) = js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("clipboard")) {
            if let Ok(write_fn) =
                js_sys::Reflect::get(&clip, &wasm_bindgen::JsValue::from_str("writeText"))
            {
                if let Ok(func) = write_fn.dyn_into::<js_sys::Function>() {
                    let _ = func.call1(&clip, &wasm_bindgen::JsValue::from_str(text));
                }
            }
        }
    }
}

pub async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

pub fn detect_browser_timezone() -> String {
    if let Some(win) = web_sys::window() {
        if let Ok(intl) = js_sys::Reflect::get(&win, &wasm_bindgen::JsValue::from_str("Intl")) {
            if let Ok(dtf_ctor) =
                js_sys::Reflect::get(&intl, &wasm_bindgen::JsValue::from_str("DateTimeFormat"))
            {
                if let Ok(func) = dtf_ctor.dyn_into::<js_sys::Function>() {
                    if let Ok(dtf) = func.call0(&intl) {
                        if let Ok(ro_fn) = js_sys::Reflect::get(
                            &dtf,
                            &wasm_bindgen::JsValue::from_str("resolvedOptions"),
                        ) {
                            if let Ok(ro) = ro_fn.dyn_into::<js_sys::Function>() {
                                if let Ok(opts) = ro.call0(&dtf) {
                                    if let Ok(tz) = js_sys::Reflect::get(
                                        &opts,
                                        &wasm_bindgen::JsValue::from_str("timeZone"),
                                    ) {
                                        if let Some(s) = tz.as_string() {
                                            return s;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    String::new()
}

pub fn is_safe_avatar_url(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("/api/media/")
        || lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("data:image/png;base64,")
        || lower.starts_with("data:image/jpeg;base64,")
        || lower.starts_with("data:image/jpg;base64,")
        || lower.starts_with("data:image/webp;base64,")
        || lower.starts_with("data:image/gif;base64,")
}

pub fn active_persona_for_user(user: &VerifiedUser) -> UserPersona {
    if let Some(ap) = &user.active_persona {
        return ap.clone();
    }
    if let Some(prof) = &user.profile {
        if let Some(found) = user
            .personas
            .iter()
            .find(|p| p.persona_did == prof.active_persona_did)
        {
            return found.clone();
        }
        return UserPersona {
            persona_did: prof.did_peer.clone(),
            owner_did: prof.did_peer.clone(),
            is_default: true,
            display_name: prof.display_name.clone(),
            handle: prof.pen_name.clone(),
            bio: prof.bio.clone(),
            avatar_asset_id: prof.avatar_asset_id.clone(),
            avatar_url: prof.avatar_url.clone(),
            avatar_thumb_url: prof.avatar_thumb_url.clone(),
            ..Default::default()
        };
    }
    UserPersona::default()
}

pub fn display_name_for_user(user: &VerifiedUser) -> String {
    let ap = active_persona_for_user(user);
    if !ap.display_name.trim().is_empty() {
        return ap.display_name;
    }
    if let Some(p) = &user.profile {
        if !p.display_name.trim().is_empty() {
            return p.display_name.clone();
        }
    }
    if let Some(name) = &user.name {
        if !name.trim().is_empty() {
            return name.clone();
        }
    }
    if let Some(email) = &user.email {
        if let Some(local) = email.split('@').next() {
            if !local.is_empty() {
                return local.to_string();
            }
        }
        return email.clone();
    }
    "Writer".to_string()
}

pub fn invite_url_for_code(code: &str) -> String {
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "https://exonovel.exosystems.net".to_string());
    format!("{origin}/?invite={code}")
}

pub fn format_allowlist_status(status: &str) -> &'static str {
    match status {
        "admin" => "Admin",
        "moderator" => "Moderator",
        "reader" => "Reader",
        "pending" => "Pending",
        "suspended" => "Suspended",
        _ => "Reader",
    }
}

pub fn allowlist_status_key(status: &str) -> &'static str {
    match status {
        "admin" => "role_admin",
        "moderator" => "role_moderator",
        "reader" => "status_reader",
        "pending" => "status_pending",
        "suspended" => "status_suspended",
        _ => "status_reader",
    }
}

pub fn format_account_role(role: &str) -> &'static str {
    match role {
        "admin" => "Admin",
        "moderator" => "Moderator",
        _ => "User",
    }
}

pub fn account_role_key(role: &str) -> &'static str {
    match role {
        "admin" => "role_admin",
        "moderator" => "role_moderator",
        _ => "role_user",
    }
}

pub fn account_status_key(status: &str) -> &'static str {
    match status {
        "restricted" => "status_restricted",
        "suspended" => "status_suspended",
        _ => "active_badge",
    }
}

#[component]
pub fn CopyIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect width="14" height="14" x="8" y="8" rx="2" ry="2"/>
            <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>
        </svg>
    }
}

#[component]
pub fn BellIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/>
            <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>
        </svg>
    }
}

#[component]
pub fn GoogleLogoIcon() -> impl IntoView {
    view! {
        <svg width="20" height="20" viewBox="0 0 48 48" aria-hidden="true">
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
pub fn GitHubLogoIcon() -> impl IntoView {
    view! {
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
            />
        </svg>
    }
}

#[component]
pub fn SunIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="4"/>
            <path d="M12 2v2"/><path d="M12 20v2"/>
            <path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/>
            <path d="M2 12h2"/><path d="M20 12h2"/>
            <path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>
        </svg>
    }
}

#[component]
pub fn MoonIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"/>
        </svg>
    }
}

#[component]
pub fn MonitorIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect width="20" height="14" x="2" y="3" rx="2"/>
            <line x1="8" x2="16" y1="21" y2="21"/>
            <line x1="12" x2="12" y1="17" y2="21"/>
        </svg>
    }
}

#[component]
pub fn MenuIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <line x1="4" x2="20" y1="6" y2="6"/>
            <line x1="4" x2="20" y1="12" y2="12"/>
            <line x1="4" x2="20" y1="18" y2="18"/>
        </svg>
    }
}

#[component]
pub fn CloseIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <line x1="18" x2="6" y1="6" y2="18"/>
            <line x1="6" x2="18" y1="6" y2="18"/>
        </svg>
    }
}

#[component]
pub fn GlobeIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/>
            <path d="M2 12h20"/>
        </svg>
    }
}

pub fn locale_flag_emoji(code: &str) -> &'static str {
    match code.trim() {
        "en" => "🇬🇧",
        "de" => "🇩🇪",
        "pl" => "🇵🇱",
        "fr" => "🇫🇷",
        "es" => "🇪🇸",
        "it" => "🇮🇹",
        "nl" => "🇳🇱",
        "pt" => "🇵🇹",
        "hu" => "🇭🇺",
        "ro" => "🇷🇴",
        "uk" => "🇺🇦",
        "ru" => "🇷🇺",
        "sr" => "🇷🇸",
        "el" => "🇬🇷",
        "ar" => "🇸🇦",
        "he" => "🇮🇱",
        "zh-Hans" => "🇨🇳",
        "zh-Hant" => "🇹🇼",
        "hi" | "mr" | "te" | "ta" => "🇮🇳",
        "bn" => "🇧🇩",
        "sw" => "🇹🇿",
        "ha" | "yo" | "ig" => "🇳🇬",
        "am" => "🇪🇹",
        "zu" => "🇿🇦",
        _ => "🌐",
    }
}

#[component]
pub fn LocaleFlag(#[prop(into)] code: String) -> impl IntoView {
    match code.trim() {
        "en" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="14" fill="#012169"/>
                <path d="M0 0 L20 14 M20 0 L0 14" stroke="#ffffff" stroke-width="2.6"/>
                <path d="M0 0 L20 14 M20 0 L0 14" stroke="#c8102e" stroke-width="1.3"/>
                <path d="M10 0 V14 M0 7 H20" stroke="#ffffff" stroke-width="4"/>
                <path d="M10 0 V14 M0 7 H20" stroke="#c8102e" stroke-width="2.4"/>
            </svg>
        }.into_any(),
        "de" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="4.67" y="0" fill="#111111"/>
                <rect width="20" height="4.67" y="4.67" fill="#dd0000"/>
                <rect width="20" height="4.67" y="9.33" fill="#ffce00"/>
            </svg>
        }.into_any(),
        "pl" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="7" y="0" fill="#ffffff"/>
                <rect width="20" height="7" y="7" fill="#dc143c"/>
            </svg>
        }.into_any(),
        "fr" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="6.67" height="14" x="0" fill="#0055a4"/>
                <rect width="6.67" height="14" x="6.67" fill="#ffffff"/>
                <rect width="6.67" height="14" x="13.33" fill="#ef4135"/>
            </svg>
        }.into_any(),
        "es" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="3.5" y="0" fill="#aa151b"/>
                <rect width="20" height="7" y="3.5" fill="#f1bf00"/>
                <rect width="20" height="3.5" y="10.5" fill="#aa151b"/>
                <rect x="4.2" y="5.1" width="2.4" height="3.8" rx="0.5" fill="#aa151b"/>
            </svg>
        }.into_any(),
        "it" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="6.67" height="14" x="0" fill="#009246"/>
                <rect width="6.67" height="14" x="6.67" fill="#ffffff"/>
                <rect width="6.67" height="14" x="13.33" fill="#ce2b37"/>
            </svg>
        }.into_any(),
        "nl" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="4.67" y="0" fill="#ae1c28"/>
                <rect width="20" height="4.67" y="4.67" fill="#ffffff"/>
                <rect width="20" height="4.67" y="9.33" fill="#21468b"/>
            </svg>
        }.into_any(),
        "pt" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="8" height="14" x="0" fill="#006600"/>
                <rect width="12" height="14" x="8" fill="#ff0000"/>
                <circle cx="8" cy="7" r="2.6" fill="#ffcc00"/>
                <circle cx="8" cy="7" r="1.4" fill="#003399"/>
            </svg>
        }.into_any(),
        "hu" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="4.67" y="0" fill="#cd2a3e"/>
                <rect width="20" height="4.67" y="4.67" fill="#ffffff"/>
                <rect width="20" height="4.67" y="9.33" fill="#436f4d"/>
            </svg>
        }.into_any(),
        "ro" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="6.67" height="14" x="0" fill="#002b7f"/>
                <rect width="6.67" height="14" x="6.67" fill="#fcd116"/>
                <rect width="6.67" height="14" x="13.33" fill="#ce1126"/>
            </svg>
        }.into_any(),
        "uk" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="7" y="0" fill="#0057b7"/>
                <rect width="20" height="7" y="7" fill="#ffd700"/>
            </svg>
        }.into_any(),
        "ru" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="4.67" y="0" fill="#ffffff"/>
                <rect width="20" height="4.67" y="4.67" fill="#0039a6"/>
                <rect width="20" height="4.67" y="9.33" fill="#d52b1e"/>
            </svg>
        }.into_any(),
        "sr" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="4.67" y="0" fill="#c6363c"/>
                <rect width="20" height="4.67" y="4.67" fill="#0c4076"/>
                <rect width="20" height="4.67" y="9.33" fill="#ffffff"/>
                <rect x="4.8" y="4" width="2.8" height="4.4" rx="0.6" fill="#edb92e"/>
            </svg>
        }.into_any(),
        "el" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="14" fill="#0d5eaf"/>
                <rect width="20" height="1.55" y="1.55" fill="#ffffff"/>
                <rect width="20" height="1.55" y="4.66" fill="#ffffff"/>
                <rect width="20" height="1.55" y="7.77" fill="#ffffff"/>
                <rect width="20" height="1.55" y="10.88" fill="#ffffff"/>
                <rect width="8" height="7.77" x="0" y="0" fill="#0d5eaf"/>
                <rect width="1.6" height="7.77" x="3.2" y="0" fill="#ffffff"/>
                <rect width="8" height="1.6" x="0" y="3.08" fill="#ffffff"/>
            </svg>
        }.into_any(),
        "ar" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="14" fill="#006c35"/>
                <path d="M4.5 5.4 H15.5 M5.5 7.4 H14.5" stroke="#ffffff" stroke-width="1.2" stroke-linecap="round"/>
                <path d="M5 10 H14.5" stroke="#ffffff" stroke-width="1.1" stroke-linecap="round"/>
            </svg>
        }.into_any(),
        "he" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="14" fill="#ffffff"/>
                <rect width="20" height="2.2" y="1.4" fill="#0038b8"/>
                <rect width="20" height="2.2" y="10.4" fill="#0038b8"/>
                <polygon points="10,4.2 12.4,8.4 7.6,8.4" fill="none" stroke="#0038b8" stroke-width="0.9"/>
                <polygon points="10,9.8 12.4,5.6 7.6,5.6" fill="none" stroke="#0038b8" stroke-width="0.9"/>
            </svg>
        }.into_any(),
        "zh-Hans" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="14" fill="#de2910"/>
                <polygon points="4.2,2.2 4.8,4 6.6,4 5.1,5 5.7,6.8 4.2,5.7 2.7,6.8 3.3,5 1.8,4 3.6,4" fill="#ffde00"/>
                <circle cx="8" cy="2.5" r="0.65" fill="#ffde00"/>
                <circle cx="9.2" cy="4" r="0.65" fill="#ffde00"/>
                <circle cx="9.1" cy="5.8" r="0.65" fill="#ffde00"/>
                <circle cx="7.8" cy="7.2" r="0.65" fill="#ffde00"/>
            </svg>
        }.into_any(),
        "zh-Hant" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="14" fill="#fe0000"/>
                <rect width="10" height="7" x="0" y="0" fill="#000095"/>
                <circle cx="5" cy="3.5" r="1.9" fill="#ffffff"/>
            </svg>
        }.into_any(),
        "hi" | "mr" | "te" | "ta" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="4.67" y="0" fill="#ff9933"/>
                <rect width="20" height="4.67" y="4.67" fill="#ffffff"/>
                <rect width="20" height="4.67" y="9.33" fill="#138808"/>
                <circle cx="10" cy="7" r="1.7" fill="none" stroke="#000080" stroke-width="0.8"/>
            </svg>
        }.into_any(),
        "bn" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="14" fill="#006a4e"/>
                <circle cx="9" cy="7" r="3.8" fill="#f42a41"/>
            </svg>
        }.into_any(),
        "sw" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <polygon points="0,0 20,0 0,14" fill="#1eb53a"/>
                <polygon points="20,0 20,14 0,14" fill="#00a3dd"/>
                <path d="M-2 15 L22 -1" stroke="#fcd116" stroke-width="4.6"/>
                <path d="M-2 15 L22 -1" stroke="#111111" stroke-width="3"/>
            </svg>
        }.into_any(),
        "ha" | "yo" | "ig" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="6.67" height="14" x="0" fill="#008751"/>
                <rect width="6.67" height="14" x="6.67" fill="#ffffff"/>
                <rect width="6.67" height="14" x="13.33" fill="#008751"/>
            </svg>
        }.into_any(),
        "am" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="4.67" y="0" fill="#078930"/>
                <rect width="20" height="4.67" y="4.67" fill="#fcdd09"/>
                <rect width="20" height="4.67" y="9.33" fill="#da121a"/>
                <circle cx="10" cy="7" r="2.7" fill="#0f47af"/>
                <circle cx="10" cy="7" r="1.2" fill="#fcdd09"/>
            </svg>
        }.into_any(),
        "zu" => view! {
            <svg viewBox="0 0 20 14" width="20" height="14" class="exo-locale-flag" aria-hidden="true">
                <rect width="20" height="7" y="0" fill="#e03c31"/>
                <rect width="20" height="7" y="7" fill="#001489"/>
                <path d="M0 0 L9 7 L0 14 Z" fill="#ffb81c"/>
                <path d="M0 1.4 L7.2 7 L0 12.6 Z" fill="#111111"/>
                <path d="M0 0 L9.5 7 L20 7 M0 14 L9.5 7" stroke="#ffffff" stroke-width="3.2" fill="none"/>
                <path d="M0 0 L9.5 7 L20 7 M0 14 L9.5 7" stroke="#007749" stroke-width="2" fill="none"/>
            </svg>
        }.into_any(),
        _ => view! {
            <GlobeIcon />
        }.into_any(),
    }
}


