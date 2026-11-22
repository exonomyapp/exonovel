# 002-004. Database-First Preferences (Eliminating Redundant Browser Storage)

Date: 2026-11-21
Status: Proposed (for owner review)
Parent: [002-data](../002-data.md)

## Context & Question Addressed

ExoNovel already authenticates users via Neon Managed Better Auth and stores preferences in **Neon PostgreSQL (`user_profiles`)** (`appearance_mode`, `preferred_format`, `locale`, `timezone`, `active_persona_did`, `last_active_tab`, `last_doc_path`, and upcoming notification checkboxes). Yet the frontend currently still writes `mu_appearance` (and docs sidebar collapse/width/folder state) to browser `localStorage`.

> **Design question**: *"If we're already storing preferences and settings in the Neon PostgreSQL user profile, do we really need browser storage if it is so efficient to just make a DB call for them?"*

## Answer & Architectural Decision

**No — once a user is authenticated, browser `localStorage` is redundant and harmful for persistent user preferences.** Keeping preferences in both `localStorage` and Neon PostgreSQL causes cross-device drift (changing theme, language, active persona, or doc workspace on Linux/Desktop does not follow the user to Android/iOS or another browser).

### 1. Single Source of Truth: Neon PostgreSQL (`user_profiles`)
- Store **all durable user preferences** exclusively in **Neon PostgreSQL** (`user_profiles`), fetched in the single initial `/api/me` session payload on load and persisted asynchronously via `PUT /api/profile`:
  - `appearance_mode` (`system`, `light`, `dark`)
  - `locale` (selected UI language, e.g., `en`, `de`, `pl`, `ar`, `zh-Hans`, `hi`, ...)
  - `preferred_format`, `timezone`, `active_persona_did`
  - `last_active_tab`, `last_doc_path`
  - Sidebar UI workspace state (`docs_sidebar_collapsed`, `docs_sidebar_width`, collapsed folders if kept)
  - Notification checkboxes (`notify_prefs_json`)
- Server-side rendering (`shell()` in `crates/server/src/main.rs`) injects `<html lang="..." dir="..." data-appearance="...">` cleanly without requiring `localStorage` reads.

### 2. Only Strict Pre-Authentication Exceptions
Do **not** store durable preferences in `localStorage`. The only transient browser state permitted before a session exists is:
1. **Pending Invite Referral Code (`sessionStorage`)**: Temporary handshake value (`exo_pending_invite`) held only across the OAuth redirect from Sign-In back to ExoNovel, then immediately claimed to PostgreSQL (`POST /api/invites/claim`) and deleted.
2. **Pre-Auth Sign-In Language / Theme Choice (Query/Cookie or Session-Free Memory)**: While viewing the Sign-In screen before authentication, theme/language selection works in memory (or a short cookie for SSR) and immediately syncs to `user_profiles` once signed in.
3. **Offline Mobile/Desktop Local Store (`willow25`)**: When native mobile/desktop clients gain offline editing (`002-iroh`), preferences replicate as regular **Willow Protocol** entries under `/personas/...` and `/profile`, replacing browser `localStorage` entirely across all 5 platforms.

## Consequences

- **Zero cross-device divergence**: Switching language, theme, persona, or notification settings in Chrome on Linux takes effect immediately on iOS, Android, macOS, and Windows.
- **Cleaner privacy & simpler codebase**: Removes redundant `localStorage` read/write synchronization helpers (`mu_appearance`, sidebar `localStorage` keys) from the signed-in lifecycle.
