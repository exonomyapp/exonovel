use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::i18n::use_i18n;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

pub fn select_opt(value: impl Into<String>, label: impl Into<String>) -> SelectOption {
    SelectOption {
        value: value.into(),
        label: label.into(),
    }
}

/// Custom in-app select dropdown (WAI-ARIA select-only combobox without OS native popups).
#[component]
pub fn ExoSelect(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] options: Signal<Vec<SelectOption>>,
    on_change: Callback<String>,
    #[prop(optional, into)] aria_label: Option<Signal<String>>,
    #[prop(optional, into)] class_extra: String,
    #[prop(optional, into)] style_extra: String,
    #[prop(optional, into)] menu_direction: String,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    let (open, set_open) = signal(false);
    let drop_up = menu_direction == "up";
    let menu_cls = if drop_up {
        "exo-select-menu exo-select-menu-up"
    } else {
        "exo-select-menu"
    };
    let wrap_cls = if class_extra.trim().is_empty() {
        "exo-select-wrap".to_string()
    } else {
        format!("exo-select-wrap {class_extra}")
    };

    view! {
        <div class=wrap_cls style=style_extra>
            <Show when=move || open.get()>
                <div
                    class="exo-nav-backdrop"
                    on:click=move |_| set_open.set(false)
                />
            </Show>
            <button
                type="button"
                class="exo-input exo-select-trigger"
                role="combobox"
                aria-haspopup="listbox"
                aria-expanded=move || if open.get() { "true" } else { "false" }
                aria-label=move || aria_label.map(|a| a.get()).unwrap_or_default()
                disabled=move || disabled.map(|d| d.get()).unwrap_or(false)
                on:click=move |_| {
                    if !disabled.map(|d| d.get()).unwrap_or(false) {
                        set_open.update(|v| *v = !*v);
                    }
                }
            >
                <span class="exo-select-value">
                    {move || {
                        let cur = value.get();
                        let opts = options.get();
                        opts.into_iter()
                            .find(|o| o.value == cur)
                            .map(|o| o.label)
                            .unwrap_or(cur)
                    }}
                </span>
                <span class="exo-select-caret" aria-hidden="true">"▾"</span>
            </button>

            <Show when=move || open.get()>
                <div class=menu_cls role="listbox">
                    {move || {
                        let cur = value.get();
                        options
                            .get()
                            .into_iter()
                            .map(|opt| {
                                let val_click = opt.value.clone();
                                let is_sel = opt.value == cur;
                                view! {
                                    <button
                                        type="button"
                                        role="option"
                                        aria-selected=if is_sel { "true" } else { "false" }
                                        class=if is_sel {
                                            "exo-select-option exo-select-option-active"
                                        } else {
                                            "exo-select-option"
                                        }
                                        on:click=move |_| {
                                            set_open.set(false);
                                            on_change.run(val_click.clone());
                                        }
                                    >
                                        <span>{opt.label}</span>
                                        {is_sel.then(|| view! {
                                            <span class="exo-select-check" aria-hidden="true">"✓"</span>
                                        })}
                                    </button>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </Show>
        </div>
    }
}

/// Custom in-app themed image/file picker button wrapping a hidden file input.
#[component]
pub fn ExoFileButton(
    #[prop(into)] label: Signal<String>,
    #[prop(optional, into)] accept: String,
    #[prop(default = 2_097_152.0)] max_bytes: f64,
    #[prop(optional, into)] size_error_msg: Option<Signal<String>>,
    on_data_url: Callback<String>,
    #[prop(optional)] on_error: Option<Callback<String>>,
    #[prop(optional, into)] class_extra: String,
) -> impl IntoView {
    let i18n = use_i18n();
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let accept_val = if accept.trim().is_empty() {
        "image/png,image/jpeg,image/webp,image/gif".to_string()
    } else {
        accept
    };
    let btn_cls = if class_extra.trim().is_empty() {
        "exo-btn".to_string()
    } else {
        format!("exo-btn {class_extra}")
    };

    let handle_change = move |_| {
        if let Some(input) = input_ref.get() {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    if file.size() > max_bytes {
                        input.set_value("");
                        if let Some(err_cb) = on_error {
                            let msg = size_error_msg
                                .map(|s| s.get_untracked())
                                .unwrap_or_else(|| {
                                    i18n.tr_untracked(
                                        "preferences_msg_avatar_size",
                                        "Avatar file must be 2 MB or smaller.",
                                    )
                                });
                            err_cb.run(msg);
                        }
                        return;
                    }
                    if let Ok(reader) = web_sys::FileReader::new() {
                        let reader_clone = reader.clone();
                        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                            if let Ok(res) = reader_clone.result() {
                                if let Some(data_url) = res.as_string() {
                                    on_data_url.run(data_url);
                                }
                            }
                        });
                        reader.set_onload(Some(cb.unchecked_ref()));
                        let _ = reader.read_as_data_url(&file);
                    }
                }
            }
            input.set_value("");
        }
    };

    view! {
        <div class="exo-file-btn-wrap">
            <input
                type="file"
                class="exo-sr-only"
                tabindex="-1"
                accept=accept_val
                node_ref=input_ref
                on:change=handle_change
            />
            <button
                type="button"
                class=btn_cls
                on:click=move |_| {
                    if let Some(el) = input_ref.get() {
                        el.click();
                    }
                }
            >
                <span>{move || label.get()}</span>
            </button>
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsNavIcon {
    User,
    Apps,
    Globe,
    Sync,
    Bell,
    Shield,
    Overview,
    Book,
    Users,
    Personas,
    Wallet,
    Lock,
    Layers,
}

#[component]
fn SettingsNavSvg(icon: SettingsNavIcon) -> impl IntoView {
    match icon {
        SettingsNavIcon::User => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="8" cy="5.25" r="2.75"/>
                <path d="M2.75 13.75c0-2.6 2.35-4.25 5.25-4.25s5.25 1.65 5.25 4.25"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Apps => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="2" y="2" width="4.75" height="4.75" rx="1"/>
                <rect x="9.25" y="2" width="4.75" height="4.75" rx="1"/>
                <rect x="2" y="9.25" width="4.75" height="4.75" rx="1"/>
                <path d="M11.6 9.25V14M9.25 11.6H14"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Globe => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="8" cy="8" r="6"/>
                <path d="M2.4 8h11.2M8 2a9.2 9.2 0 0 1 2.5 6A9.2 9.2 0 0 1 8 14a9.2 9.2 0 0 1-2.5-6A9.2 9.2 0 0 1 8 2z"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Sync => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M13.25 4.75A5.75 5.75 0 0 0 3.5 4.5M2.75 2.25v2.75h2.75"/>
                <path d="M2.75 11.25A5.75 5.75 0 0 0 12.5 11.5M13.25 13.75V11h-2.75"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Bell => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M4.25 6.5a3.75 3.75 0 0 1 7.5 0c0 4 1.75 5 1.75 5H2.5s1.75-1 1.75-5"/>
                <path d="M6.75 13.5a1.35 1.35 0 0 0 2.5 0"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Shield => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M8 1.75 13.25 4v3.8c0 3.25-2.2 5.45-5.25 6.45-3.05-1-5.25-3.2-5.25-6.45V4L8 1.75z"/>
                <path d="m6 8.2 1.35 1.35 2.75-2.8"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Overview => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="8" cy="8" r="2.25"/>
                <path d="M8 1.75v1.5M8 12.75v1.5M1.75 8h1.5M12.75 8h1.5M3.6 3.6l1.05 1.05M11.35 11.35l1.05 1.05M12.4 3.6l-1.05 1.05M4.65 11.35 3.6 12.4"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Book => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M3 2.5h8.25a1.5 1.5 0 0 1 1.5 1.5v9.5a1.5 1.5 0 0 1-1.5 1.5H3a1.25 1.25 0 0 1-1.25-1.25V3.75A1.25 1.25 0 0 1 3 2.5z"/>
                <path d="M5.25 2.5v12.5M7.75 6h3M7.75 8.75h2.25"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Users => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="6" cy="5.5" r="2.25"/>
                <path d="M1.75 13.5c0-2.1 1.9-3.5 4.25-3.5s4.25 1.4 4.25 3.5"/>
                <circle cx="11.5" cy="6.25" r="1.75"/>
                <path d="M11.25 10.25c1.75.2 3 1.3 3 3.25"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Personas => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="2" y="3" width="12" height="10" rx="1.75"/>
                <circle cx="6" cy="7.25" r="1.5"/>
                <path d="M3.75 11.25c.4-1.15 1.3-1.75 2.25-1.75s1.85.6 2.25 1.75"/>
                <path d="M9.75 6.5h2.5M9.75 9.25h2"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Wallet => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M2.25 4.25A1.5 1.5 0 0 1 3.75 2.75h8.5a1 1 0 0 1 1 1V5.5"/>
                <rect x="2.25" y="5" width="11.5" height="8.25" rx="1.5"/>
                <circle cx="10.75" cy="9.1" r="0.9" fill="currentColor" stroke="none"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Lock => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="3" y="7" width="10" height="7" rx="1.5"/>
                <path d="M5.25 7V5a2.75 2.75 0 1 1 5.5 0v2"/>
            </svg>
        }.into_any(),
        SettingsNavIcon::Layers => view! {
            <svg class="exo-gh-nav-icon" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="m8 2 6 3.25L8 8.5 2 5.25 8 2z"/>
                <path d="m2 8.25 6 3.25 6-3.25"/>
                <path d="m2 11.25 6 3.25 6-3.25"/>
            </svg>
        }.into_any(),
    }
}

#[derive(Clone)]
pub struct SettingsNavItem {
    pub icon: SettingsNavIcon,
    pub label_key: &'static str,
    pub label_fallback: &'static str,
    pub is_active: Signal<bool>,
    pub on_click: Callback<()>,
}

#[derive(Clone)]
pub struct SettingsNavGroup {
    pub divider_before: bool,
    pub heading: Option<(&'static str, &'static str)>,
    pub visible: bool,
    pub items: Vec<SettingsNavItem>,
}

#[derive(Clone, Copy)]
pub struct SettingsResizeCtl {
    pub width: RwSignal<i32>,
    pub dragging: RwSignal<bool>,
    pub start_x: RwSignal<i32>,
    pub start_width: RwSignal<i32>,
    pub on_commit: Callback<i32>,
}

pub fn normalize_settings_nav_width(raw: i32) -> i32 {
    if raw <= 0 {
        260
    } else {
        raw.clamp(200, 420)
    }
}

pub fn use_settings_resize(initial_width: i32, on_commit: Callback<i32>) -> SettingsResizeCtl {
    let clamped = normalize_settings_nav_width(initial_width);
    SettingsResizeCtl {
        width: RwSignal::new(clamped),
        dragging: RwSignal::new(false),
        start_x: RwSignal::new(0),
        start_width: RwSignal::new(clamped),
        on_commit,
    }
}

#[component]
pub fn ExoSettingsPage(resize: SettingsResizeCtl, children: Children) -> impl IntoView {
    let i18n = use_i18n();
    let finish_drag = move || {
        if resize.dragging.get_untracked() {
            resize.dragging.set(false);
            let committed = resize.width.get_untracked().clamp(200, 420);
            resize.on_commit.run(committed);
        }
    };

    view! {
        <main
            class=move || {
                if resize.dragging.get() {
                    "exo-gh-page exo-gh-page-resizing"
                } else {
                    "exo-gh-page"
                }
            }
            on:pointermove=move |ev: web_sys::PointerEvent| {
                if resize.dragging.get_untracked() {
                    let delta_raw = ev.client_x() - resize.start_x.get_untracked();
                    let delta = if i18n.ui_dir() == "rtl" {
                        -delta_raw
                    } else {
                        delta_raw
                    };
                    let next = (resize.start_width.get_untracked() + delta).clamp(200, 420);
                    resize.width.set(next);
                }
            }
            on:pointerup=move |_| finish_drag()
            on:pointercancel=move |_| finish_drag()
            on:pointerleave=move |_| finish_drag()
        >
            <div class="exo-gh-container">
                {children()}
            </div>
        </main>
    }
}

#[component]
pub fn ExoSettingsSplit(
    resize: SettingsResizeCtl,
    #[prop(into)] nav_aria_label: Signal<String>,
    groups: Vec<SettingsNavGroup>,
    children: Children,
) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="exo-gh-layout">
            <aside
                class="exo-gh-nav-col"
                style=move || format!("--exo-gh-nav-w: {}px;", resize.width.get())
            >
                <nav class="exo-gh-nav" aria-label=move || nav_aria_label.get()>
                    {groups
                        .into_iter()
                        .filter(|g| g.visible)
                        .map(|group| {
                            let heading = group.heading;
                            let divider_before = group.divider_before;
                            view! {
                                <div class="exo-gh-nav-group">
                                    {divider_before.then(|| view! {
                                        <div class="exo-gh-nav-divider" role="separator" />
                                    })}
                                    {heading.map(|(h_key, h_fallback)| view! {
                                        <div class="exo-gh-nav-group-title">
                                            {move || i18n.tr(h_key, h_fallback)}
                                        </div>
                                    })}
                                    {group
                                        .items
                                        .into_iter()
                                        .map(|item| {
                                            let active_sig = item.is_active;
                                            let on_clk = item.on_click;
                                            let key = item.label_key;
                                            let fallback = item.label_fallback;
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || if active_sig.get() {
                                                        "exo-gh-nav-item exo-gh-nav-item-active"
                                                    } else {
                                                        "exo-gh-nav-item"
                                                    }
                                                    aria-current=move || if active_sig.get() { Some("page") } else { None }
                                                    on:click=move |_| on_clk.run(())
                                                >
                                                    <SettingsNavSvg icon=item.icon />
                                                    <span class="exo-gh-nav-label">
                                                        {move || i18n.tr(key, fallback)}
                                                    </span>
                                                </button>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                        })
                        .collect_view()}
                </nav>
            </aside>

            <div
                class=move || {
                    if resize.dragging.get() {
                        "exo-gh-resizer exo-gh-resizer-dragging"
                    } else {
                        "exo-gh-resizer"
                    }
                }
                role="separator"
                tabindex="0"
                aria-orientation="vertical"
                aria-valuemin="200"
                aria-valuemax="420"
                aria-valuenow=move || resize.width.get().to_string()
                aria-label=move || i18n.tr("settings_resize_aria", "Resize settings navigation sidebar")
                on:pointerdown=move |ev: web_sys::PointerEvent| {
                    if ev.button() != 0 {
                        return;
                    }
                    ev.prevent_default();
                    resize.start_x.set(ev.client_x());
                    resize.start_width.set(resize.width.get_untracked());
                    resize.dragging.set(true);
                }
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    let cur = resize.width.get_untracked();
                    let is_rtl = i18n.ui_dir() == "rtl";
                    let next_opt = match ev.key().as_str() {
                        "ArrowLeft" => Some(if is_rtl { cur + 16 } else { cur - 16 }),
                        "ArrowRight" => Some(if is_rtl { cur - 16 } else { cur + 16 }),
                        "Home" => Some(200),
                        "End" => Some(420),
                        _ => None,
                    };
                    if let Some(raw_next) = next_opt {
                        ev.prevent_default();
                        let clamped = raw_next.clamp(200, 420);
                        if clamped != cur {
                            resize.width.set(clamped);
                            resize.on_commit.run(clamped);
                        }
                    }
                }
            />

            <div class="exo-gh-content">
                {children()}
            </div>
        </div>
    }
}

