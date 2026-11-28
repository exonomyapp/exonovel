use std::collections::HashMap;

use gloo_net::http::Request;
use leptos::prelude::*;

use crate::{
    models::{locale_flag_emoji, I18nCatalogResponse, LocaleDescriptor},
    ui::{select_opt, ExoSelect},
};

#[derive(Clone, Copy)]
pub struct I18nState {
    pub catalog: RwSignal<Vec<LocaleDescriptor>>,
    pub willow_namespace: RwSignal<String>,
    pub ui_locale: RwSignal<String>,
    pub input_locale: RwSignal<String>,
    pub dict: RwSignal<HashMap<String, String>>,
    pub pinned_ui_locales: RwSignal<Vec<String>>,
    pub persist_ui_locale: RwSignal<Option<Callback<String>>>,
}

impl I18nState {
    pub fn new() -> Self {
        Self {
            catalog: RwSignal::new(Vec::new()),
            willow_namespace: RwSignal::new(
                "willow25://exonovel.exosystems.net/language-packs/v1".to_string(),
            ),
            ui_locale: RwSignal::new("en".to_string()),
            input_locale: RwSignal::new("en".to_string()),
            dict: RwSignal::new(HashMap::new()),
            pinned_ui_locales: RwSignal::new(Vec::new()),
            persist_ui_locale: RwSignal::new(None),
        }
    }

    pub fn tr(&self, key: &str, fallback: &str) -> String {
        let dict = self.dict.get();
        if let Some(val) = dict.get(key) {
            return val.clone();
        }
        let normalized = key.replace('-', "_");
        if let Some(val) = dict.get(&normalized) {
            return val.clone();
        }
        fallback.to_string()
    }

    pub fn tr_untracked(&self, key: &str, fallback: &str) -> String {
        let dict = self.dict.get_untracked();
        if let Some(val) = dict.get(key) {
            return val.clone();
        }
        let normalized = key.replace('-', "_");
        if let Some(val) = dict.get(&normalized) {
            return val.clone();
        }
        fallback.to_string()
    }

    pub fn ui_dir(&self) -> &'static str {
        let code = self.ui_locale.get();
        if code == "ar" || code == "he" {
            "rtl"
        } else {
            "ltr"
        }
    }

    pub fn input_dir(&self) -> &'static str {
        let code = self.input_locale.get();
        if code == "ar" || code == "he" {
            "rtl"
        } else {
            "ltr"
        }
    }

    pub fn locale_label(&self, code: &str) -> String {
        let list = self.catalog.get();
        if let Some(found) = list.iter().find(|l| l.code == code) {
            format!("{} ({})", found.native_name, found.code)
        } else {
            code.to_string()
        }
    }
}

pub fn use_i18n() -> I18nState {
    expect_context::<I18nState>()
}

pub fn parse_ftl(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim();
            let val = v.trim().to_string();
            map.insert(key.to_string(), val.clone());
            map.insert(key.replace('-', "_"), val);
        }
    }
    map
}

pub async fn fetch_locale_catalog(i18n: I18nState) {
    if let Ok(resp) = Request::get("/api/i18n/catalog").send().await {
        if resp.ok() {
            if let Ok(data) = resp.json::<I18nCatalogResponse>().await {
                i18n.catalog.set(data.locales);
                i18n.willow_namespace.set(data.public_willow_namespace);
            }
        }
    }
}

pub async fn fetch_locale_pack(i18n: I18nState, code: &str) {
    let clean = if code.trim().is_empty() { "en" } else { code.trim() };
    let url = format!("/api/i18n/{clean}.ftl");
    if let Ok(resp) = Request::get(&url).send().await {
        if resp.ok() {
            if let Ok(body) = resp.text().await {
                i18n.dict.set(parse_ftl(&body));
            }
        }
    }
}

/// Reusable composer bar allowing instant switching of Input Language independently of Output UI Language (ADR 007).
#[component]
pub fn InputLocaleBar(
    #[prop(into)] context_label: String,
    pinned_codes: Vec<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let pinned_sv = StoredValue::new(pinned_codes);

    let options = Signal::derive(move || {
        let catalog = i18n.catalog.get();
        let filtered: Vec<LocaleDescriptor> = pinned_sv.with_value(|pinned| {
            if pinned.is_empty() {
                catalog.clone()
            } else {
                let subset: Vec<LocaleDescriptor> = catalog
                    .iter()
                    .filter(|l| {
                        pinned.contains(&l.code) || l.code == i18n.input_locale.get()
                    })
                    .cloned()
                    .collect();
                if subset.is_empty() {
                    catalog.clone()
                } else {
                    subset
                }
            }
        });
        filtered
            .into_iter()
            .map(|loc| {
                let label = format!(
                    "{} {} — {}",
                    locale_flag_emoji(&loc.code),
                    loc.native_name,
                    loc.english_name
                );
                select_opt(loc.code, label)
            })
            .collect()
    });

    view! {
        <div class="exo-input-lang-bar">
            <span class="exo-eyebrow" style="margin: 0;">
                {context_label}
                " · "
                {move || i18n.tr("input_lang_label", "Writing Input")}
                ": "
                <strong>{move || i18n.locale_label(&i18n.input_locale.get())}</strong>
                " ("
                {move || i18n.input_dir().to_uppercase()}
                ")"
            </span>
            <ExoSelect
                value=Signal::derive(move || i18n.input_locale.get())
                options=options
                aria_label=Signal::derive(move || i18n.tr("input_lang_aria", "Writing Input Language"))
                style_extra="min-width: 210px;"
                on_change=Callback::new(move |next| i18n.input_locale.set(next))
            />
        </div>
    }
}
