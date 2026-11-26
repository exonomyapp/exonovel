# 005-002. Private Docs Auth Gate & Admin Preferences

Date: 2026-11-19
Status: Accepted (in implementation)
Parent: [005-docs-ui](../005-docs-ui.md)

## Decision

1. **Public Unauthenticated Landing Page, Sign-In Route & Auth Gate (Neon Better Auth)**:
   - Docs are strictly private and belong exclusively to the independent **ExoNovel** app (`https://exonovel.exosystems.net`), never `exosystems.net`; docs endpoints reject unauthenticated or unauthorized requests with `401`/`403`.
   - Unauthenticated visitors and crawlers arriving at `/` immediately receive a server-side-rendered public **ExoNovel** homepage (never behind a login wall or session-loading blocker) explaining the platform's creative writing purpose, five authoring formats, Google Sign-In / optional Drive usage, and explicit Google APIs Terms of Service & anti-AI-NCII compliance, with direct links to `/privacy`, `/terms`, and `/signin`.
   - Clicking **Sign in** (or visiting `/signin` or an `?invite=` URL) opens the Sign-In view supporting Neon Better Auth with branded **Google** and **GitHub** buttons plus Email/Password sign-in/sign-up.
   - Non-allowlisted signed-in users land cleanly on a standard **ExoNovel** Workspace Home card without exposing private documentation controls or unsolicited greeting banners.
   - As soon as `exonomyapp@gmail.com` or an enabled `did:peer` allowlist entry signs in, **Docs Reader** is visible in ExoNovel's navbar.
2. **Navbar User Avatar Menu, Profile Hub & Preferences Split**:
   - On desktop, signed-in users see a navbar user chip showing their 64px circular avatar thumbnail and active persona display name; on mobile viewports (`<= 768px`), primary links move inside the top-right **Avatar** menu (no hamburger menu toggle) and navbar dropdown panels open as viewport-wide scrollable sheets (`position: fixed`) above a dismiss backdrop.
   - Dropping down the Avatar menu lets users open their **Profile** page (managing Personas, collaboration Groups, group Managers, Invites, and Referrals), switch among *inactive* personas (the active persona is excluded; when no other personas exist, the menu displays `"Create Personas in your Profile"` paired with a `"Personas"` button navigating to **Profile -> Personas**), open **Preferences**, or Sign out.
   - **Preferences** handles personal account preferences, **Connected External Apps** (inspecting SSO login providers vs. optional app permissions like Google Drive, connecting apps, and revoking access cleanly), appearance theme (`system`/`light`/`dark`), and 2 MB avatar uploads (processed server-side into `thumb` 64×64, `medium` 256×256, and `large` <= 1024px WebP/JPEG variants). Page-width toggles are intentionally omitted.
   - On sign-in, the backend captures maximum available metadata from `neon_auth."user"`, `neon_auth.account`, verified JWT claims, and HTTP headers into `user_profiles`, plus profile-backed return navigation (`last_active_tab`, `last_doc_path`), and automatically claims any pending `?invite=` referral/group invitation.
3. **Admin Allowlist & Multi-Level Account Roles**:
   - Only the primary admin (`ADMIN_EMAIL`, defaulting to `exonomyapp@gmail.com`) sees the site-admin panels inside **Preferences**: **Documentation Access Allowlist** (multi-level status: `reader`, `moderator`, `pending`, `suspended`, `admin`) and **Registered Accounts & Moderation Roles** (account authorization tier `user` < `moderator` < `admin` and `active`/`restricted`/`suspended` status).
   - Tables identify users primarily by `did:peer` (with toolbar copy icon + hover reveal + bottom toast confirmation), while Registered Accounts supports email-click accordion inspection/editing and click-to-populate `did:peer` selection.
4. **Connected Apps Lifecycle in Preferences (`001-auth` Section 7)**:
   - Every signed-in user sees their **Connected Apps** section in **Preferences** displaying linked SSO identities (`listAccounts()`), active external app grants (starting with **Google Drive**), a **Connect** button (pre-selecting matching Google SSO accounts via `login_hint`), and a confirmed **Disconnect & Revoke** action that revokes the provider grant without terminating their primary ExoNovel login.
