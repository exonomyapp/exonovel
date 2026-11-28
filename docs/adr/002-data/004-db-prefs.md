# 002-004. Database-First Preferences (Eliminating Redundant Browser Storage)

Date: 2026-11-27
Status: Accepted (PostgreSQL `user_profiles` + invisible UI preferences pattern)
Parent: [002-data](../002-data.md)

## Context & Question Addressed

ExoNovel already authenticates users via Managed Better Auth and stores preferences in **PostgreSQL (`user_profiles`)** (`appearance_mode`, `preferred_format`, `locale`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, and notification flags). Writing UI state to browser `localStorage` causes cross-device drift and hides layout adjustments inside browser-local silos.

> **Design decisions**:
> 1. *"If we're already storing preferences in the PostgreSQL user profile, do we really need browser storage if it is so efficient to just make a DB call for them?"*
> 2. *"Both Preferences and Profile share the same non-floating, persistently resizable left navigation format; instead of relying on browser storage, keep each page's distinct resize width (`profile_nav_width` for Profile, `preferences_nav_width` for Preferences) and similar 'invisible' UI settings in PostgreSQL user preferences without adding extra form controls beyond the resize handle/slider itself."*

## Answer & Architectural Decision

**No — once a user is authenticated, browser `localStorage` is banned for persistent user preferences and UI layout state.**

### 1. Single Source of Truth: PostgreSQL (`user_profiles`)
- Store **all durable user preferences and invisible UI state** exclusively in **PostgreSQL** (`user_profiles`), loaded in the initial `/api/me` session payload and updated via `PUT /api/profile`:
  - `appearance_mode` (`system`, `light`, `dark`)
  - `locale`, `preferred_ui_locales`, `default_input_locale`, `preferred_input_locales`
  - `preferred_format`, `timezone`, `active_persona_did`
  - `last_active_tab`, `last_doc_path`
  - **Invisible UI preferences** (`docs_sidebar_collapsed`, `docs_sidebar_width`, `profile_nav_width` for **Profile**, `preferences_nav_width` for **Preferences**, and similar direct-manipulation layout state). Because **Profile** and **Preferences** hold completely different navigation lists and contents, their sidebar widths are stored independently per page rather than shared.
  - Notification flags (`notify_prefs_json`)
- Server-side rendering (`shell()` in `crates/server/src/main.rs`) injects `<html lang="..." dir="..." data-appearance="...">` directly without reading `localStorage`.

### 2. "Invisible Preferences" Pattern (Direct Manipulation, Zero Clutter)
- UI ergonomics adjusted through direct on-page gestures (dragging or keyboard-adjusting the flush, non-floating left navigation resizer independently on **Profile** (`profile_nav_width`) and **Preferences** (`preferences_nav_width`) via WAI-ARIA `role="separator"`, collapsing/resizing the **Docs & ADRs** sidebar, or remembering the last section/doc path) **MUST persist to PostgreSQL `user_profiles`**, never browser `localStorage`.
- **No redundant form UI**: Do **not** expose extra text inputs, dropdowns, or sliders inside **Preferences** forms for settings whose natural control is an on-page handle, toggle, or navigation selection. The resize handle/splitter itself is the sole user control.
- **Unified Settings Layout**: Both **Profile** (replacing horizontal tabs) and **Preferences** use the shared flush, integrated (non-floating), persistently resizable left navigation shell (`ExoSettingsPage` + `ExoSettingsSplit`), while saving each page's width to its own column.

### 3. Strict Pre-Authentication Exceptions
Do **not** store durable preferences in `localStorage`. The only browser state permitted before a session exists is:
1. **Pending Invite Referral Code (`sessionStorage`)**: Temporary handshake value (`exo_pending_invite`) held only across the OAuth redirect from Sign-In back to ExoNovel, then immediately claimed to PostgreSQL (`POST /api/invites/claim`) and deleted.
2. **Pre-Auth Sign-In Language / Theme Choice (Session-Free Memory)**: While unauthenticated, theme/language selection works in memory and syncs to `user_profiles` upon sign-in.
3. **Offline Mobile/Desktop Local Store (`willow25`)**: When native mobile/desktop clients gain offline editing (`002-iroh`), durable preferences and invisible UI settings replicate as **Willow Protocol** entries under `/personas/...` and `/profile`, replacing browser `localStorage` across all platforms.

## Consequences

- **Zero cross-device divergence**: Switching language, theme, persona, notification preferences, or sidebar widths on Linux/Desktop persists seamlessly to iOS, Android, macOS, and Windows.
- **Cleaner UI**: Direct-manipulation workspace adjustments stay persistent across sessions and devices without bloating **Preferences** with technical width/state form fields.
