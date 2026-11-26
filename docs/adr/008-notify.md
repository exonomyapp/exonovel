# 008. Cross-Platform Notification System (5 Target Platforms, Web & Active Brevo Email)

Date: 2026-11-21
Status: Accepted (Brevo active now; GitHub-style Profile vs. Preferences split & Android FCM+UnifiedPush confirmed)

## Context

ExoNovel runs on **Web** plus **5 native platforms: Linux, macOS, Windows, Android, and iOS**. Users need timely alerts for social collaboration, **author commercial/paywall revenue events**, security/billing events, and **Maturity Process** updates (when watched libraries announce or release new versions or unblock `on-hold` ADRs). Preferences live in **Neon PostgreSQL** (and relocate identically with `willow25`, zero `localStorage`), and all UI controls follow a clean **GitHub-style Profile vs. Preferences split**.

## Decision

### 1. Multi-Platform Delivery Architecture
Notifications are generated once as structured events in Neon PostgreSQL / local Willow stores (`notifications` + `notification_devices` + `notification_preferences`) and routed concurrently across all active channels:

- **In-App Live Feed (All 6 Environments)**:
  - Delivered immediately via **Axum Server-Sent Events (SSE)** (plus unread badge & notification drawer synced to `notifications`), or direct local `iroh` events in Local-Only mode.
- **Web Browsers (`Web Push API`)**:
  - Server signs VAPID requests using Rust **`web-push` (`0.11.0`)** targeting registered Service Worker endpoints (`notification_devices.platform = 'web_push'`).
- **Desktop Native (`Linux`, `macOS`, `Windows` via Tauri v2)**:
  - **Foreground / active**: Native OS system notifications via **`tauri-plugin-notification` (`2.7.0`)**.
  - **Background / closed-app**:
    - **Linux**: **UnifiedPush** HTTP distributor + D-Bus `org.freedesktop.Notifications` bridge (distro-agnostic, Google-free).
    - **macOS**: **APNs** via Rust **`a2`** crate + `UserNotifications`.
    - **Windows**: **WNS** / WinRT Toast notifications.
- **Mobile Native (`Android`, `iOS` via Tauri v2)**:
  - Device token registration via **`tauri-plugin-push-notifications` (`0.1.0`)** or **`tauri-plugin-remote-push` (`1.0.10`)**.
  - **Android (FCM + UnifiedPush, F-Droid First)**:
    - **Why keep FCM alongside UnifiedPush**: Dropping FCM entirely would force mainstream Play Store / Samsung / Xiaomi Android users to manually install and maintain a third-party UnifiedPush distributor app (e.g., *ntfy*, *NextPush*) and fight aggressive OEM battery managers—or fall back to battery-draining background polling. Therefore, ExoNovel uses **Firebase Cloud Messaging HTTP v1** (`fcm_v1` `0.3.0` / `fcm-service` `0.2.3`) as the zero-friction default on Play Services devices **AND** first-class **UnifiedPush** for de-Googled / GrapheneOS / F-Droid builds. For **F-Droid** first-wave live testing, ExoNovel ships a pure **`foss` build flavor** stripping all proprietary Firebase/Google Play Services dependencies (reproducible builds, UnifiedPush-only) before adding the `play` flavor for Google Play.
  - **iOS**: **APNs** via Rust **`a2` (`0.3.x`)** HTTP/2 client using `.p8` Token-Based Authentication.
- **Transactional Email (`Brevo` — Active Now on `exosystems.net`)**:
  - Calls **Brevo Transactional Email API v3** (`POST https://api.brevo.com/v3/smtp/email`) using `BREVO_API_KEY` (subdomain sending path `exonovel.exosystems.net` dropped in favor of verified root `exosystems.net` DNS with DKIM `brevo1`/`brevo2`, DMARC, and branded tracking domain `exomail.exosystems.net`).
  - Three functional non-secret sender identities (hardcoded as defaults in `crates/server/src/notify.rs` `BrevoSenderRole::resolve`, auto-seeded into the production server environment on every deploy by `.github/workflows/deploy.yml`, and left commented in `.env.example` as optional overrides only — never required in deployment secrets):
    - **Notifications (`BREVO_SENDER_NOTES_EMAIL` / `BREVO_SENDER_NOTES_NAME`)**: **`notes@exosystems.net`** (`"Notes from ExoNovel"`) — notification-domain alerts, digests, and maturity notices.
    - **General / Non-Notification (`BREVO_SENDER_NOREPLY_EMAIL` / `BREVO_SENDER_NOREPLY_NAME`)**: **`noreply@exosystems.net`** (`"Exosystems"`) — general automated/auth emails outside the notification domain.
    - **User Support (`BREVO_SENDER_SUPPORT_EMAIL` / `BREVO_SENDER_SUPPORT_NAME`)**: **`support@exosystems.net`** (`"ExoSupport"`) — user support inquiries and issue replies.

### 2. GitHub-Style Profile vs. Preferences Split (Strict Ownership Boundary)
ExoNovel adopts a **modern GitHub UI format** for both **Profile** (tabbed header workspace) and **Preferences** (left-rail category sidebar), with a strict boundary: **100% of commercial, wallet, purchase, and billing activity lives in Profile — zero commercial items live in Preferences**:

- **Profile Page (Identity, Collaboration & ALL Commercial / Wallet Activity)**:
  - *Tabs*: `Overview`, `Works`, `Groups & Invites`, **`Wallet, Paywalls & Purchases`**, `Personas`, `Notifications`.
  - **Everything financial belongs here**, including:
    - **Author Commercials**: Paywall pricing, connected payout accounts (`Stripe Connect`, `dLocal`, `Razorpay`, `Flutterwave`, `Airwallex`, `TON`, `Lightning`, `Reown AppKit` / `USDC`), and author earnings/payouts.
    - **Reader Purchases & ExoNovel Sync Subscription**: History/receipts of works unlocked from other authors **plus** purchasing/renewing **ExoNovel Cloud + Device Sync** (`$9/month` or `$90/year` local-currency equivalent, paid directly to ExoNovel's own `did:peer` wallets—which in turn unlocks the Sync toggle in Preferences).
    - **Profile Notification Checkboxes**:
      - `notify_author_earnings` — **Author Paywall Earnings & Payouts** (when readers pay for your works or payouts settle)
      - `notify_reader_purchases` — **Reader Purchases & Sync Plan Billing** (receipts when you unlock another author's work or renew ExoNovel Sync)
      - `notify_group_invites` — Invitations to join a creative group
      - `notify_referral_signups` — New user signed up via your invite link
      - `notify_moderator_comments` — Moderator feedback on your creative objects
      - `notify_collaborator_comments` — Collaborator & reader comments/replies
      - `notify_mentions_and_forks` — `@mentions`, persona messages, and story architecture forks
- **Preferences Page (Strictly Technical Preferences, Languages, Sync Mode & Admin Maturity)**:
  - *Sidebar sections*: `Account & Security`, `Appearance & Languages` (Output UI locale, Default Input locale, pinned quick-switchers, offline `.ftl` pack downloads), `Sync & Storage` (enables Cloud + Device Meadowcap-delegated Willow sync once subscribed via Profile), `Notification Channels` (Web Push, Desktop/Mobile Push, transactional email & instant/daily/weekly digest cadence, paired devices), and *(Admin only)* `Docs Allowlist & Maturity Process`.
  - **Preferences Notification Checkboxes**:
    - `notify_security_signins` — New device login, `did:peer` key linkage, or password/session events
    - `notify_maturity_releases` *(Admin / Owner)* — Watched dependency announces an upcoming feature or ships a new release
    - `notify_maturity_unblocked` *(Admin / Owner)* — An `on-hold` ADR has its blocking tool requirement satisfied (`ready-to-build`) or a new capability proposal (`maturity_proposal_notes`) is ready for review

### 3. Neon + Willow Storage Parity
`notification_preferences`, `notification_devices`, and `notifications` follow the same `willow_*` coordinate conventions (`willow_namespace`, `willow_subspace`, `willow_path`, `willow_timestamp_us`) so device-local native installs without Neon maintain full notification preferences and history cleanly.
